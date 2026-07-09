use ferrisetw::UserTrace;
use ferrisetw::trace::{TraceBuilder, TraceError, stop_trace_by_name};

use crate::provider_event::ProviderEvent;
use crate::providers;

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
    pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        log::info!("Creating new ETW session: '{}'...", self.session_name);

        let trace = self
            .register_providers(shared_callback)
            .named(self.session_name.clone())
            .start_and_process()?;

        log::info!("ETW Trace session '{}' is now active.", self.session_name);
        log::info!("{:?}", trace);
        Ok(EtwTraceSession {
            session_name: self.session_name,
            trace: Some(trace),
        })
    }

    /// Central place to enable all desired providers.
    /// Add new providers here with additional `.enable(...)` calls.
    fn register_providers<F>(&self, shared_callback: F) -> TraceBuilder<UserTrace>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        let file_cb = shared_callback;
        let file_provider = providers::kernel_file::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFile(evt));
        });

        UserTrace::new().enable(file_provider)
        // ── Add future providers here ──
        // .enable(another_provider)
    }
}

/// A running ETW trace session. Call `stop()` or let `Drop` handle cleanup.
pub struct EtwTraceSession {
    session_name: String,
    trace: Option<UserTrace>,
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
