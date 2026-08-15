use std::sync::Arc;

use ferrisetw::UserTrace;
use ferrisetw::trace::{TraceError, TraceTrait, stop_trace_by_name};
use windows::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE;

use crate::provider_event::ProviderEvent;
use crate::providers;
use crate::rundown::request_rundown;

/// Builder for configuring an ETW trace session.
/// Consumed by `start()` which returns a `EtwTraceSession`.
pub struct EtwTraceManager {
    session_name: String,
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
        Self { session_name }
    }

    /// Starts both UserTrace and KernelTrace sessions simultaneously.
    ///
    /// Each session runs on its own thread. The callback is shared via `Arc`
    /// and will be called from both threads in parallel.
    ///
    /// The callback type is `Fn(ProviderEvent) + Send + Sync + Clone + 'static`,
    /// which allows it to be called from multiple threads simultaneously.
    pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        log::info!("Creating new ETW sessions for '{}'...", self.session_name);

        let user_session_name = format!("{}-User", self.session_name);
        let kernel_session_name = format!("{}-Kernel", self.session_name);

        // Wrap callback in Arc for thread-safe sharing across both sessions
        let callback = Arc::new(shared_callback);

        // Start UserTrace session
        let user_result = self.start_user(&user_session_name, callback.clone());

        // Start KernelTrace session
        let kernel_result = self.start_kernel(&kernel_session_name, callback);

        // Both sessions must succeed
        let (user_session, kernel_session) = match (user_result, kernel_result) {
            (Ok(u), Ok(k)) => (u, k),
            (Err(e), _) | (_, Err(e)) => {
                log::error!("Failed to start one or both ETW sessions: {:?}", e);
                return Err(e);
            }
        };

        log::info!("Both ETW sessions for '{}' are now active.", self.session_name);
        Ok(EtwTraceSession {
            user_session: Some(user_session),
            kernel_session: Some(kernel_session),
        })
    }

    /// Start a UserTrace session with standard kernel providers.
    fn start_user<F>(
        &self,
        session_name: &str,
        callback: Arc<F>,
    ) -> Result<UserTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = callback.clone();
        let file_provider = providers::user_trace_kernel_file::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFile(evt));
        });

        let process_cb = callback.clone();
        let process_provider = providers::user_trace_kernel_process::build_provider(move |evt| {
            process_cb(ProviderEvent::KernelProcess(evt));
        });

        let provider_guids = vec![
            providers::user_trace_kernel_file::PROVIDER_GUID,
            providers::user_trace_kernel_process::PROVIDER_GUID,
        ];

        let builder = UserTrace::new()
            .enable(file_provider)
            .enable(process_provider);

        // Start the trace
        let (trace, trace_handle) = builder.named(session_name.to_string()).start()?;

        // Request rundown
        let query_handle = request_rundown(session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        #[cfg(debug_assertions)]
        verify_control_handle(&trace, query_handle);

        // Spawn the blocking ProcessTrace on a background thread
        std::thread::spawn(move || {
            let _ = UserTrace::process_from_handle(trace_handle);
        });

        log::info!("UserTrace session '{}' is now active.", session_name);
        Ok(UserTraceSession {
            session_name: session_name.to_string(),
            trace: Some(trace),
            control_handle: Some(query_handle),
        })
    }

    /// Start a KernelTrace session with extended flags (PERFINFO_GROUPMASK) support.
    fn start_kernel<F>(
        &self,
        session_name: &str,
        callback: Arc<F>,
    ) -> Result<KernelTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = callback.clone();
        let file_provider = providers::kernel_trace_fileio::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFileIo(evt));
        });

        let provider_guids = vec![providers::kernel_trace_fileio::PROVIDER_GUID];

        let builder = ferrisetw::trace::KernelTrace::new()
            .named(session_name.to_string())
            .enable(file_provider)
            .stop_if_exist(true);

        // Step 1: Start the trace
        let (trace, trace_handle) = builder.start()?;

        // Step 2: Query the control handle
        let query_handle = request_rundown(session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        // Step 3: Apply the group mask via TraceSetInformation
        providers::kernel_trace_fileio::apply_group_mask(query_handle).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;

        log::info!(
            "Applied PERFINFO_GROUPMASK to session '{}': {:?}",
            session_name,
            providers::kernel_trace_fileio::GROUP_MASK
        );

        // Step 4: Spawn the blocking ProcessTrace on a background thread
        std::thread::spawn(move || {
            let _ =
                <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(trace_handle);
        });

        log::info!("KernelTrace session '{}' is now active.", session_name);
        Ok(KernelTraceSession {
            session_name: session_name.to_string(),
            trace: Some(trace),
            control_handle: Some(query_handle),
        })
    }
}

/// A running UserTrace session.
pub struct UserTraceSession {
    session_name: String,
    trace: Option<UserTrace>,
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl UserTraceSession {
    fn stop_inner(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping UserTrace session '{}'...", self.session_name);

        let mut result = Ok(());

        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("UserTrace.stop() failed: {:?}", e);
                result = Err(e);
            }
        }

        if let Err(e) = stop_trace_by_name(&self.session_name) {
            log::debug!("stop_trace_by_name fallback for UserTrace: {:?}", e);
        }

        result
    }
}

impl Drop for UserTraceSession {
    fn drop(&mut self) {
        log::info!("Cleaning up UserTrace resources for '{}'...", self.session_name);
        let _ = self.stop_inner();
    }
}

/// A running KernelTrace session.
pub struct KernelTraceSession {
    session_name: String,
    trace: Option<ferrisetw::trace::KernelTrace>,
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl KernelTraceSession {
    fn stop_inner(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping KernelTrace session '{}'...", self.session_name);

        let mut result = Ok(());

        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("KernelTrace.stop() failed: {:?}", e);
                result = Err(e);
            }
        }

        if let Err(e) = stop_trace_by_name(&self.session_name) {
            log::debug!("stop_trace_by_name fallback for KernelTrace: {:?}", e);
        }

        result
    }
}

impl Drop for KernelTraceSession {
    fn drop(&mut self) {
        log::info!(
            "Cleaning up KernelTrace resources for '{}'...",
            self.session_name
        );
        let _ = self.stop_inner();
    }
}

/// A running ETW trace session managing both UserTrace and KernelTrace.
/// Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    user_session: Option<UserTraceSession>,
    kernel_session: Option<KernelTraceSession>,
}

impl EtwTraceSession {
    /// Explicitly shuts down both trace sessions ahead of Drop.
    pub fn stop(&mut self) -> Result<(), TraceError> {
        let mut result = Ok(());

        if let Some(ref mut session) = self.user_session {
            if let Err(e) = session.stop_inner() {
                log::error!("Failed to stop UserTrace: {:?}", e);
                result = Err(e);
            }
        }

        if let Some(ref mut session) = self.kernel_session {
            if let Err(e) = session.stop_inner() {
                log::error!("Failed to stop KernelTrace: {:?}", e);
                result = Err(e);
            }
        }

        result
    }
}

impl Drop for EtwTraceSession {
    fn drop(&mut self) {
        log::info!("Cleaning up all ETW resources...");
        let _ = self.stop();
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
