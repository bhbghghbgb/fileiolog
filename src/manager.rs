use std::thread::JoinHandle;

use ferrisetw::UserTrace;
use ferrisetw::trace::{TraceBuilder, TraceError, TraceTrait, stop_trace_by_name};
use windows::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE;
use windows::core::GUID;

use crate::provider_event::ProviderEvent;
use crate::providers;
use crate::rundown::request_rundown;

// ---------------------------------------------------------------------------
//  Builder
// ---------------------------------------------------------------------------

/// Builder for configuring an ETW trace session.
/// Consumed by `start()` which returns a `EtwTraceSession`.
pub struct EtwTraceManager {
    session_name: String,
}

impl EtwTraceManager {
    pub fn new(session_name: &str) -> Self {
        let session_name = session_name.to_string();

        // Pre-emptively stop any orphan sessions from previous crashes
        let kernel_name = format!("{session_name}-Kernel");
        let user_name = format!("{session_name}-User");

        log::info!(
            "Checking for any lingering ETW sessions named '{kernel_name}' / '{user_name}'..."
        );
        match stop_trace_by_name(&kernel_name) {
            Ok(()) => log::debug!("No lingering kernel session detected or clean bypass"),
            Err(e) => log::warn!("Found and terminated a stale kernel session: {e:?}"),
        }
        match stop_trace_by_name(&user_name) {
            Ok(()) => log::debug!("No lingering user session detected or clean bypass"),
            Err(e) => log::warn!("Found and terminated a stale user session: {e:?}"),
        }

        Self { session_name }
    }

    /// Starts both a UserTrace and a KernelTrace session simultaneously.
    ///
    /// Each session runs on its own background thread. The callback is cloned
    /// for each session; the caller must ensure it satisfies `Send + Sync`.
    pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        log::info!("Creating dual ETW sessions: '{}'...", self.session_name);

        let kernel_name = format!("{}-Kernel", self.session_name);
        let user_name = format!("{}-User", self.session_name);

        // Start the user trace first (less disruptive if it fails)
        let user = start_user_session(&user_name, shared_callback.clone())?;

        // Start the kernel trace
        let kernel = match start_kernel_session(&kernel_name, shared_callback) {
            Ok(k) => k,
            Err(e) => {
                // Kernel failed — tear down the user session before returning
                log::error!("Kernel session failed, stopping user session: {e:?}");
                let mut user = user;
                let _ = user.stop();
                let _ = stop_trace_by_name(&user_name);
                return Err(e);
            }
        };

        log::info!("Both ETW sessions '{}' are now active.", self.session_name);
        Ok(EtwTraceSession {
            base_name: self.session_name,
            user,
            kernel,
        })
    }
}

// ---------------------------------------------------------------------------
//  Running sub-sessions
// ---------------------------------------------------------------------------

/// A running user-mode ETW trace session.
pub struct UserTraceSession {
    session_name: String,
    trace: Option<UserTrace>,
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
    thread: Option<JoinHandle<()>>,
}

impl UserTraceSession {
    fn stop(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping user trace session '{}'...", self.session_name);

        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("UserTrace.stop() failed: {e:?}");
            }
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

/// A running kernel-mode ETW trace session with PERFINFO_GROUPMASK support.
pub struct KernelTraceSession {
    session_name: String,
    trace: Option<ferrisetw::trace::KernelTrace>,
    #[allow(dead_code)]
    control_handle: Option<CONTROLTRACE_HANDLE>,
    thread: Option<JoinHandle<()>>,
}

impl KernelTraceSession {
    fn stop(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping kernel trace session '{}'...", self.session_name);

        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("KernelTrace.stop() failed: {e:?}");
            }
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
//  Running dual session
// ---------------------------------------------------------------------------

/// A running ETW trace session managing both user and kernel traces.
/// Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    base_name: String,
    user: UserTraceSession,
    kernel: KernelTraceSession,
}

impl EtwTraceSession {
    /// Explicitly shuts down both trace sessions ahead of Drop.
    #[allow(dead_code)]
    pub fn stop(&mut self) -> Result<(), TraceError> {
        self.stop_inner()
    }

    fn stop_inner(&mut self) -> Result<(), TraceError> {
        log::info!("Stopping trace sessions '{}'...", self.base_name);

        let mut result = Ok(());

        if let Err(e) = self.user.stop() {
            log::error!("User session stop failed: {e:?}");
            result = Err(e);
        }
        if let Err(e) = self.kernel.stop() {
            log::error!("Kernel session stop failed: {e:?}");
            if result.is_ok() {
                result = Err(e);
            }
        }

        // Fallback: ensure session names are cleaned up
        let user_name = format!("{}-User", self.base_name);
        let kernel_name = format!("{}-Kernel", self.base_name);
        if let Err(e) = stop_trace_by_name(&user_name) {
            log::debug!("stop_trace_by_name fallback (user): {e:?}");
        }
        if let Err(e) = stop_trace_by_name(&kernel_name) {
            log::debug!("stop_trace_by_name fallback (kernel): {e:?}");
        }

        result
    }
}

impl Drop for EtwTraceSession {
    fn drop(&mut self) {
        log::info!("Cleaning up ETW resources for '{}'...", self.base_name);
        let _ = self.stop_inner();
    }
}

// ---------------------------------------------------------------------------
//  Session startup helpers
// ---------------------------------------------------------------------------

/// Start a UserTrace session.
fn start_user_session<F>(
    session_name: &str,
    shared_callback: F,
) -> Result<UserTraceSession, TraceError>
where
    F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
{
    let (builder, provider_guids) = register_user_providers(shared_callback);

    let (trace, trace_handle) = builder.named(session_name.to_string()).start()?;

    let query_handle = request_rundown(session_name, &provider_guids).map_err(|e| {
        TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
    })?;

    #[cfg(debug_assertions)]
    verify_control_handle(&trace, query_handle);

    let thread = std::thread::spawn(move || {
        let _ = UserTrace::process_from_handle(trace_handle);
    });

    log::info!("User trace session '{}' is now active.", session_name);
    log::info!("{:?}", trace);

    Ok(UserTraceSession {
        session_name: session_name.to_string(),
        trace: Some(trace),
        control_handle: Some(query_handle),
        thread: Some(thread),
    })
}

/// Start a KernelTrace session with group mask support.
fn start_kernel_session<F>(
    session_name: &str,
    shared_callback: F,
) -> Result<KernelTraceSession, TraceError>
where
    F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
{
    let file_cb = shared_callback.clone();
    let file_provider = providers::kernel_trace_fileio::build_provider(move |evt| {
        file_cb(ProviderEvent::KernelFileIo(evt));
    });

    let builder = ferrisetw::trace::KernelTrace::new()
        .named(session_name.to_string())
        .enable(file_provider)
        .stop_if_exist(true);

    let (trace, trace_handle) = builder.start()?;

    let provider_guids = vec![providers::kernel_trace_fileio::PROVIDER_GUID];
    let query_handle = request_rundown(session_name, &provider_guids).map_err(|e| {
        TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
    })?;

    providers::kernel_trace_fileio::apply_group_mask(query_handle).map_err(|e| {
        TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
    })?;

    log::info!(
        "Applied PERFINFO_GROUPMASK to session '{}': {:?}",
        session_name,
        providers::kernel_trace_fileio::GROUP_MASK
    );

    let thread = std::thread::spawn(move || {
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(trace_handle);
    });

    log::info!("Kernel trace session '{}' is now active.", session_name);
    log::info!("{:?}", trace);

    Ok(KernelTraceSession {
        session_name: session_name.to_string(),
        trace: Some(trace),
        control_handle: Some(query_handle),
        thread: Some(thread),
    })
}

/// Register all user-mode providers and return the builder + GUIDs for rundown.
fn register_user_providers<F>(
    shared_callback: F,
) -> (TraceBuilder<UserTrace>, Vec<GUID>)
where
    F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
{
    let file_cb = shared_callback.clone();
    let file_provider = providers::user_trace_kernel_file::build_provider(move |evt| {
        file_cb(ProviderEvent::KernelFile(evt));
    });

    let process_cb = shared_callback;
    let process_provider = providers::user_trace_kernel_process::build_provider(move |evt| {
        process_cb(ProviderEvent::KernelProcess(evt));
    });

    let builder = UserTrace::new()
        .enable(file_provider)
        .enable(process_provider);
    // ── Add future providers here ──

    (
        builder,
        vec![
            providers::user_trace_kernel_file::PROVIDER_GUID,
            providers::user_trace_kernel_process::PROVIDER_GUID,
        ],
    )
}

// ---------------------------------------------------------------------------
//  Debug-only verification
// ---------------------------------------------------------------------------
#[cfg(debug_assertions)]
fn verify_control_handle(trace: &UserTrace, query_handle: CONTROLTRACE_HANDLE) {
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
