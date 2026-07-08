use ferrisetw::UserTrace;
use ferrisetw::trace::{TraceError, stop_trace_by_name};

use crate::provider_event::ProviderEvent;
use crate::providers;

pub struct EtwTraceManager {
    session_name: String,
    // Store the active trace once running so its own Drop/stop can be fired
    trace: Option<UserTrace>,
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

        Self {
            session_name,
            trace: None,
        }
    }

    /// Starts the ETW trace. Accepts a single unified callback processing `ProviderEvent`.
    pub fn start<F>(&mut self, shared_callback: F) -> Result<(), TraceError>
    where
        F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
    {
        // Fail immediately if a trace is already running!
        if self.trace.is_some() {
            log::warn!("Trace session '{}' is already active.", self.session_name);
            return Ok(()); // Or return an Error if preferred
        }

        log::info!("Creating new ETW session: '{}'...", self.session_name);

        // 1. Adapt the shared callback to what kernel_file expects using `.clone()`
        let file_cb = shared_callback.clone();
        let file_provider = providers::kernel_file::build_provider(move |evt| {
            file_cb(ProviderEvent::KernelFile(evt)); // Automatically lifts into ProviderEvent
        });

        // 2. Build the trace session (chaining multiple .enable() calls as needed)
        let trace_result = UserTrace::new()
            .named(self.session_name.clone())
            .enable(file_provider)
            // .enable(future_provider) // You can chain more here seamlessly
            .start_and_process();

        // 3. Even if start_and_process returns an Err, a partial session might exist.
        // We assign whatever trace handle we got to `self.trace` so `Drop` can clean it up.
        match trace_result {
            Ok(active_trace) => {
                self.trace = Some(active_trace);
                log::info!("ETW Trace session '{}' is now active.", self.session_name);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed to fully start ETW trace session: {:?}", err);
                Err(err.into())
            }
        }
    }

    /// Explicitly shuts down the trace session ahead of Drop.
    pub fn stop(&mut self) -> Result<(), TraceError> {
        log::info!(
            "Explicitly stopping trace session '{}'...",
            self.session_name
        );

        let mut result = Ok(());

        // 1. Consume the internal trace object so it isn't processed again in Drop
        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("Library trace.stop() failed: {:?}", e);
                result = Err(e);
            }
        }

        // 2. Run the cleanup fallback rule by name for good measure
        if let Err(e) = stop_trace_by_name(&self.session_name) {
            log::debug!("Force fallback cleanup skipped or already clean: {:?}", e);
        }

        result
    }
}

/// RAII Implementation ensures cleanup happens no matter how the thread/scope exits.
impl Drop for EtwTraceManager {
    fn drop(&mut self) {
        log::info!("Cleaning up ETW resources for '{}'...", self.session_name);

        // First attempt: clean close using the library's method if available
        if let Some(trace) = self.trace.take() {
            if let Err(e) = trace.stop() {
                log::error!("Library trace.stop() failed: {:?}", e);
            } else {
                log::info!("Library trace stopped safely.");
            }
        }

        // Second attempt: force close by name for good measure (covers start errors/stale sessions)
        log::info!(
            "Ensuring trace session '{}' is completely unmounted...",
            self.session_name
        );
        if let Err(e) = stop_trace_by_name(&self.session_name) {
            log::debug!(
                "Final stop_trace_by_name check completed (session already gone or empty): {:?}",
                e
            );
        } else {
            log::info!("Fallback session cleanup by name successful.");
        }
    }
}
