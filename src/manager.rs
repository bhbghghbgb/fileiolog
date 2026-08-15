use ferrisetw::UserTrace;
use ferrisetw::trace::{TraceBuilder, TraceError, TraceTrait, stop_trace_by_name};
use windows::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE;
use windows::core::GUID;

use crate::provider_event::ProviderEvent;
use crate::providers;
use crate::rundown::request_rundown;

/// Builder for configuring an ETW trace session.
/// Consumed by `start()` which returns a `EtwTraceSession`.
pub struct EtwTraceManager {
    session_name: String,
    /// If true, use KernelTrace with group mask support.
    use_kernel_trace: bool,
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
            use_kernel_trace: false,
        }
    }

    /// Enable kernel trace mode with PERFINFO_GROUPMASK support.
    ///
    /// When enabled, the trace will use `KernelTrace` instead of `UserTrace`,
    /// allowing extended flags to be set via `TraceSetInformation`.
    ///
    /// This is required for kernel sessions that need extended event types
    /// not covered by the standard EnableFlags (e.g., minifilter events).
    pub fn with_kernel_trace(mut self) -> Self {
        self.use_kernel_trace = true;
        self
    }

    /// Starts the ETW trace. Accepts a single unified callback processing `ProviderEvent`.
    ///
    /// Always requests rundown (DCStart/DCEnd) for every enabled provider after
    /// the session starts but before `ProcessTrace` begins processing events.
    ///
    /// When `with_kernel_trace()` was called, uses `KernelTrace` with group mask support.
    /// Otherwise, uses `UserTrace`.
    pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        log::info!("Creating new ETW session: '{}'...", self.session_name);

        if self.use_kernel_trace {
            self.start_kernel(shared_callback)
        } else {
            self.start_user(shared_callback)
        }
    }

    /// Start a UserTrace session (default mode).
    fn start_user<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        // Build the provider list and collect their GUIDs for the rundown request.
        let (builder, provider_guids) = self.register_user_providers(shared_callback);

        // Step 1: StartTraceW → EnableTraceEx2(ENABLE) for each provider → OpenTraceW.
        let (trace, trace_handle) = builder.named(self.session_name.clone()).start()?;

        // Step 2: Request rundown (EnableTraceEx2 CAPTURE_STATE)
        // Must happen before ProcessTrace (see krabsetw etw.hpp:375-378).
        let query_handle = request_rundown(&self.session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        // Step 3: (debug only) verify the ControlTraceW-obtained handle matches
        // the private control_handle we can only see through Debug formatting.
        #[cfg(debug_assertions)]
        verify_control_handle(&trace, query_handle);

        // Step 4: Spawn the blocking ProcessTrace on a background thread.
        std::thread::spawn(move || {
            let _ = UserTrace::process_from_handle(trace_handle);
        });

        log::info!("ETW Trace session '{}' is now active.", self.session_name);
        log::info!("{:?}", trace);
        Ok(EtwTraceSession {
            session_name: self.session_name,
            inner: EtwTraceSessionInner::User {
                trace: Some(trace),
            },
            control_handle: Some(query_handle),
        })
    }

    /// Start a KernelTrace session with group mask support.
    ///
    /// This uses the `kernel_trace_fileio` provider which supports extended flags
    /// via PERFINFO_GROUPMASK. The group mask is applied via `TraceSetInformation`
    /// after the trace is started but before `ProcessTrace` begins.
    fn start_kernel<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = shared_callback.clone();
        let file_provider = providers::kernel_trace_fileio::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFileIo(evt));
        });

        // Build the kernel trace
        let builder = ferrisetw::trace::KernelTrace::new()
            .named(self.session_name.clone())
            .enable(file_provider)
            .stop_if_exist(true);

        // Step 1: Start the trace
        let (trace, trace_handle) = builder.start()?;

        // Step 2: Query the control handle
        let provider_guids = vec![providers::kernel_trace_fileio::PROVIDER_GUID];
        let query_handle = request_rundown(&self.session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        // Step 3: Apply the group mask via TraceSetInformation
        // This must happen after the trace is started but before ProcessTrace begins.
        providers::kernel_trace_fileio::apply_group_mask(query_handle).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        log::info!(
            "Applied PERFINFO_GROUPMASK to session '{}': {:?}",
            self.session_name,
            providers::kernel_trace_fileio::GROUP_MASK
        );

        // Step 4: Spawn the blocking ProcessTrace on a background thread.
        std::thread::spawn(move || {
            let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(trace_handle);
        });

        log::info!("ETW Kernel Trace session '{}' is now active.", self.session_name);
        log::info!("{:?}", trace);
        Ok(EtwTraceSession {
            session_name: self.session_name,
            inner: EtwTraceSessionInner::Kernel {
                trace: Some(trace),
            },
            control_handle: Some(query_handle),
        })
    }

    /// Central place to enable all desired user-mode providers.
    /// Returns the builder plus the list of provider GUIDs (needed for rundown).
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
        // ── Add future providers here ──
        // let builder = builder.enable(another_provider);
        // guids.push(another_provider.guid());

        (
            builder,
            vec![
                providers::user_trace_kernel_file::PROVIDER_GUID,
                providers::user_trace_kernel_process::PROVIDER_GUID,
            ],
        )
    }
}

/// A running ETW trace session. Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    session_name: String,
    inner: EtwTraceSessionInner,
    /// The session's control handle (from `ControlTraceW(QUERY)`).
    /// Used for requesting rundown and future control operations.
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

enum EtwTraceSessionInner {
    User {
        trace: Option<UserTrace>,
    },
    Kernel {
        trace: Option<ferrisetw::trace::KernelTrace>,
    },
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
            EtwTraceSessionInner::User { trace } => {
                if let Some(trace) = trace.take() {
                    if let Err(e) = trace.stop() {
                        log::error!("trace.stop() failed: {:?}", e);
                        result = Err(e);
                    }
                }
            }
            EtwTraceSessionInner::Kernel { trace } => {
                if let Some(trace) = trace.take() {
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
//  Debug-only verification: compare the ControlTraceW-obtained handle with
//  the private control_handle visible only through the Debug representation.
// ---------------------------------------------------------------------------
#[cfg(debug_assertions)]
fn verify_control_handle(trace: &UserTrace, query_handle: CONTROLTRACE_HANDLE) {
    let debug_str = format!("{trace:?}");

    // The Debug output for UserTrace includes:
    //   control_handle: CONTROLTRACE_HANDLE { Value: <N> }
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
