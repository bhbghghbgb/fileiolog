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
}

impl EtwTraceManager {
    pub fn new(session_name: &str) -> Self {
        let session_name = session_name.to_string();

        // Pre-emptively stop any orphan sessions from previous crashes
        log::info!(
            "Checking for any lingering ETW sessions named '{}'...",
            session_name
        );
        if let Err(e) = stop_trace_by_name(&session_name) {
            log::debug!("No lingering session detected or clean bypass: {:?}", e);
        } else {
            log::warn!(
                "Found and terminated a stale trace session: '{}'.",
                session_name
            );
        }

        Self { session_name }
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

        // Build the provider list and collect their GUIDs for the rundown request.
        let (builder, provider_guids) = self.register_providers(shared_callback);

        // Step 1: StartTraceW → EnableTraceEx2(ENABLE) for each provider → OpenTraceW.
        let (trace, trace_handle) = builder.named(self.session_name.clone()).start()?;

        // Step 2: Request rundown (EnableTraceEx2 CAPTURE_STATE)
        // Must happen before ProcessTrace (see krabsetw etw.hpp:375-378).
        let query_handle = request_rundown(&self.session_name, &provider_guids).map_err(|e| {
            TraceError::EtwNativeError(ferrisetw::native::EvntraceNativeError::IoError(e))
        })?;
        // let query_handle = CONTROLTRACE_HANDLE { Value: 0 };

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
            trace: Some(trace),
            control_handle: Some(query_handle),
        })
    }

    /// Central place to enable all desired providers.
    /// Returns the builder plus the list of provider GUIDs (needed for rundown).
    /// Add new providers here with additional `.enable(...)` calls.
    fn register_providers<F>(&self, shared_callback: F) -> (TraceBuilder<UserTrace>, Vec<GUID>)
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = shared_callback;
        let file_provider = providers::kernel_file::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFile(evt));
        });

        let guid = file_provider.guid();
        let builder = UserTrace::new().enable(file_provider);
        // ── Add future providers here ──
        // let builder = builder.enable(another_provider);
        // guids.push(another_provider.guid());

        (builder, vec![guid])
    }
}

/// A running ETW trace session. Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    session_name: String,
    trace: Option<UserTrace>,
    /// The session's control handle (from `ControlTraceW(QUERY)`).
    /// Used for requesting rundown and future control operations.
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

        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("trace.stop() failed: {:?}", e);
                result = Err(e);
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
