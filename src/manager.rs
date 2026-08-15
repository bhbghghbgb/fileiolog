use ferrisetw::trace::{TraceBuilder, TraceError, TraceTrait, stop_trace_by_name};
use ferrisetw::{KernelTrace, UserTrace};
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

use crate::provider_event::ProviderEvent;
use crate::providers;
use crate::rundown::request_rundown;

/// Builder for configuring an ETW trace session.
/// Consumed by `start()` which returns a `EtwTraceSession`.
pub struct EtwTraceManager {
    session_name: String,
    /// Optional PERFINFO_GROUPMASK for extended kernel flags.
    /// When set, the session is started as a kernel trace with TraceSetInformation.
    group_mask: Option<[u32; 8]>,
    /// EnableFlags for the kernel provider (only used when group_mask is None).
    enable_flags: u32,
}

impl EtwTraceManager {
    pub fn new(session_name: &str) -> Self {
        let session_name = session_name.to_string();

        // Pre-emptively stop any orphan sessions from previous crashes
        log::info!("Checking for any lingering ETW sessions named '{session_name}'...");
        match stop_trace_by_name(&session_name) {
            Ok(()) => log::debug!("No lingering session detected or clean bypass '{session_name}'"),
            Err(e) => log::warn!(
                "Found and terminated a stale trace session: '{session_name}'. Error: {e:?}"
            ),
        }
        Self {
            session_name,
            group_mask: None,
            enable_flags: 0,
        }
    }

    /// Set an extended PERFINFO_GROUPMASK for kernel traces.
    ///
    /// When set, the trace session is started as a kernel trace and
    /// `TraceSetInformation` is called after the trace is opened but
    /// before `ProcessTrace` begins.
    ///
    /// This is mutually exclusive with `enable_flags` — if both are set,
    /// the flags are OR'd into `Masks[0]` of the group mask.
    pub fn with_group_mask(mut self, mask: [u32; 8]) -> Self {
        self.group_mask = Some(mask);
        self
    }

    /// Set EnableFlags for a kernel trace.
    ///
    /// When a `group_mask` is also set, these flags are OR'd into `Masks[0]`.
    pub fn with_enable_flags(mut self, flags: u32) -> Self {
        self.enable_flags = flags;
        self
    }

    /// Starts the ETW trace. Accepts a single unified callback processing `ProviderEvent`.
    ///
    /// Always requests rundown (DCStart/DCEnd) for every enabled provider after
    /// the session starts but before `ProcessTrace` begins processing events.
    pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        log::info!("Creating new ETW session: '{}'...", self.session_name);

        if self.group_mask.is_some() {
            self.start_kernel_trace(shared_callback)
        } else {
            self.start_user_trace(shared_callback)
        }
    }

    /// Start a user-mode trace (current behavior).
    fn start_user_trace<F>(
        self,
        shared_callback: F,
    ) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let (builder, provider_guids) = self.register_user_providers(shared_callback);

        let (trace, trace_handle) = builder.named(self.session_name.clone()).start()?;

        let query_handle = request_rundown(&self.session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        #[cfg(debug_assertions)]
        verify_control_handle_user(&trace, query_handle);

        std::thread::spawn(move || {
            let _ = UserTrace::process_from_handle(trace_handle);
        });

        log::info!("ETW Trace session '{}' is now active.", self.session_name);
        log::info!("{:?}", trace);
        Ok(EtwTraceSession {
            session_name: self.session_name,
            inner: TraceInner::User(Some(trace)),
            control_handle: Some(query_handle),
        })
    }

    /// Start a kernel trace with optional PERFINFO_GROUPMASK.
    fn start_kernel_trace<F>(
        self,
        shared_callback: F,
    ) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let group_mask = self.group_mask.unwrap_or([0u32; 8]);

        // Build the kernel provider with 0 flags — the actual flags are set
        // via TraceSetInformation after the trace is opened. The macro-generated
        // build_provider bakes in compile-time flags, but for extended groupmasks
        // we need to set them at runtime.
        let cb = shared_callback.clone();
        let kernel_provider = providers::kernel_trace_fileio::build_provider_zero_flags(
            move |evt| cb(ProviderEvent::KernelTraceFile(evt)),
        );

        let provider_guid = providers::kernel_trace_fileio::PROVIDER_GUID;

        let builder = KernelTrace::new()
            .named(self.session_name.clone())
            .enable(kernel_provider)
            .stop_if_exist(true);

        // Start the trace (without processing yet)
        let (trace, trace_handle) = builder.start()?;

        // Get the control handle by querying the session
        let query_handle = query_control_handle(&self.session_name)
            .map_err(|e| TraceError::EtwNativeError(
                ferrisetw::native::EvntraceNativeError::IoError(e),
            ))?;

        // Set the extended group mask via TraceSetInformation.
        // This must happen after the trace is opened but before ProcessTrace.
        set_group_mask(query_handle, group_mask, self.enable_flags)?;

        // Request rundown for the provider
        trigger_capture_state(query_handle, provider_guid)?;

        // Spawn the blocking ProcessTrace on a background thread.
        std::thread::spawn(move || {
            let _ = KernelTrace::process_from_handle(trace_handle);
        });

        log::info!(
            "ETW Kernel Trace session '{}' is now active (group_mask set).",
            self.session_name
        );
        log::info!("{:?}", trace);
        Ok(EtwTraceSession {
            session_name: self.session_name,
            inner: TraceInner::Kernel(Some(trace)),
            control_handle: Some(query_handle),
        })
    }

    /// Central place to enable all user-mode providers.
    fn register_user_providers<F>(
        &self,
        shared_callback: F,
    ) -> (TraceBuilder<UserTrace>, Vec<GUID>)
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = shared_callback.clone();
        let file_provider = providers::user_trace_kernel_file::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFile(evt));
        });

        let process_cb = shared_callback.clone();
        let process_provider = providers::user_trace_kernel_process::build_provider(move |evt| {
            process_cb(ProviderEvent::KernelProcess(evt));
        });

        let builder = UserTrace::new()
            .enable(file_provider)
            .enable(process_provider);

        (
            builder,
            vec![
                providers::user_trace_kernel_file::PROVIDER_GUID,
                providers::user_trace_kernel_process::PROVIDER_GUID,
            ],
        )
    }
}

/// Internal trace type — either UserTrace or KernelTrace.
enum TraceInner {
    User(Option<UserTrace>),
    Kernel(Option<KernelTrace>),
}

/// A running ETW trace session. Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    session_name: String,
    inner: TraceInner,
    /// The session's control handle (from `ControlTraceW(QUERY)`).
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl EtwTraceSession {
    /// Explicitly shuts down the trace session ahead of Drop.
    #[allow(dead_code)]
    pub fn stop(&mut self) -> Result<(), TraceError> {
        self.stop_inner()
    }

    /// Shared cleanup logic used by both `stop()` and `Drop`.
    fn stop_inner(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping trace session '{}'...", self.session_name);

        let mut result = Ok(());

        match &mut self.inner {
            TraceInner::User(trace_opt) => {
                if let Some(trace) = trace_opt.take() {
                    if let Err(e) = trace.stop() {
                        log::error!("trace.stop() failed: {:?}", e);
                        result = Err(e);
                    }
                }
            }
            TraceInner::Kernel(trace_opt) => {
                if let Some(trace) = trace_opt.take() {
                    if let Err(e) = trace.stop() {
                        log::error!("trace.stop() failed: {:?}", e);
                        result = Err(e);
                    }
                }
            }
        }

        if let Err(e) = stop_trace_by_name(&self.session_name) {
            log::debug!("stop_trace_by_name fallback: {:?}", e);
        }

        result
    }
}

impl Drop for EtwTraceSession {
    fn drop(&mut self) {
        log::info!("Cleaning up ETW resources for '{}'...", self.session_name);
        let _ = self.stop_inner();
    }
}

// ---------------------------------------------------------------------------
//  PERFINFO_GROUPMASK via TraceSetInformation
// ---------------------------------------------------------------------------

/// Set PERFINFO_GROUPMASK via TraceSetInformation (TraceSystemTraceEnableFlagsInfo).
///
/// This must be called after the trace is opened but before ProcessTrace.
fn set_group_mask(
    control_handle: CONTROLTRACE_HANDLE,
    group_mask: [u32; 8],
    enable_flags: u32,
) -> Result<(), TraceError> {
    let mut masks = group_mask;
    // OR in the EnableFlags so they are not zeroed when replacing the groupmask.
    // Masks[0] corresponds to EnableFlags.
    masks[0] |= enable_flags;

    // TraceSystemTraceEnableFlagsInfo = 4
    const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;

    let result = unsafe {
        Etw::TraceSetInformation(
            control_handle,
            std::mem::transmute(TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO),
            masks.as_ptr() as *const std::ffi::c_void,
            std::mem::size_of::<[u32; 8]>() as u32,
        )
    }
    .ok();

    if let Err(e) = result {
        log::error!("TraceSetInformation (GroupMask) failed: {:?}", e);
        return Err(TraceError::EtwNativeError(
            ferrisetw::native::EvntraceNativeError::IoError(
                std::io::Error::from_raw_os_error(e.code().0),
            ),
        ));
    }

    log::debug!("Set PERFINFO_GROUPMASK to {:?}", masks);
    Ok(())
}

// ---------------------------------------------------------------------------
//  ControlTraceW(QUERY) — extract the session handle
// ---------------------------------------------------------------------------

/// Query the control handle for a session by name.
fn query_control_handle(session_name: &str) -> Result<CONTROLTRACE_HANDLE, std::io::Error> {
    let name_wide: Vec<u16> = session_name.encode_utf16().chain(std::iter::once(0)).collect();
    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let name_buf_size = (200 + 1) * 2; // TRACE_NAME_MAX_CHARS + 1, in bytes
    let total_size = header_size + name_buf_size;

    let mut buffer = vec![0u8; total_size];

    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
    props.Wnode.BufferSize = total_size as u32;
    props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    props.Wnode.Guid = GUID::zeroed();
    props.LoggerNameOffset = header_size as u32;
    props.LogFileNameOffset = 0;

    let name_ptr = unsafe { buffer.as_mut_ptr().add(header_size) as *mut u16 };
    unsafe {
        std::ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
    }

    let result = unsafe {
        Etw::ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(name_ptr as *const u16),
            props as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    }
    .ok();

    if let Err(e) = result {
        return Err(std::io::Error::from_raw_os_error(e.code().0));
    }

    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
    Ok(CONTROLTRACE_HANDLE {
        Value: handle_value,
    })
}

// ---------------------------------------------------------------------------
//  EnableTraceEx2(CAPTURE_STATE) — request rundown
// ---------------------------------------------------------------------------

fn trigger_capture_state(
    handle: CONTROLTRACE_HANDLE,
    provider_guid: GUID,
) -> Result<(), TraceError> {
    let result = unsafe {
        Etw::EnableTraceEx2(
            handle,
            &provider_guid as *const GUID,
            Etw::EVENT_CONTROL_CODE_CAPTURE_STATE.0,
            0, // TRACE_LEVEL_NONE
            0, // match any keyword
            0, // match all keyword
            0, // timeout
            None,
        )
    }
    .ok();

    if let Err(e) = result {
        log::error!("EnableTraceEx2 CAPTURE_STATE failed: {:?}", e);
        return Err(TraceError::EtwNativeError(
            ferrisetw::native::EvntraceNativeError::IoError(
                std::io::Error::from_raw_os_error(e.code().0),
            ),
        ));
    }

    log::debug!("Triggered capture state for {provider_guid:?}");
    Ok(())
}

// ---------------------------------------------------------------------------
//  Debug-only verification
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
fn verify_control_handle_user(trace: &UserTrace, query_handle: CONTROLTRACE_HANDLE) {
    let debug_str = format!("{trace:?}");

    let marker = "control_handle: CONTROLTRACE_HANDLE { Value: ";
    if let Some(start) = debug_str.find(marker) {
        let rest = &debug_str[start + marker.len()..];
        if let Some(end) = rest.find(" }") {
            let value_str = &rest[..end];
            if let Ok(parsed) = value_str.parse::<u64>() {
                let debug_handle = CONTROLTRACE_HANDLE { Value: parsed };
                if debug_handle.Value == query_handle.Value {
                    log::info!(
                        "Rundown handle OK — ControlTraceW handle matches UserTrace \
                         Debug handle (Value = {})",
                        query_handle.Value,
                    );
                } else {
                    log::warn!(
                        "Rundown handle MISMATCH — UserTrace Debug = {}, \
                         ControlTraceW = {} (using ControlTraceW handle anyway)",
                        debug_handle.Value,
                        query_handle.Value,
                    );
                }
                return;
            }
        }
    }
    log::warn!(
        "Could not parse control_handle from UserTrace Debug output \
         (format may have changed); query_handle Value = {}",
        query_handle.Value,
    );
}
