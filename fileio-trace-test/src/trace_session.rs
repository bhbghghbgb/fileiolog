use std::ptr;
use std::sync::{Arc, Mutex};

use ferrisetw::EventRecord;
use ferrisetw::provider::*;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::*;
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, PROCESSTRACE_HANDLE, WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

use crate::events;

/// FileIo Provider GUID
const FILE_IO_GUID: GUID = GUID::from_u128(0x90cbdc39_4a3e_11d1_84f4_0000f80464e3);

/// Configuration for a trace session
pub struct TraceConfig {
    pub session_name: String,
    /// EnableFlags to set via the kernel provider (OR'd into the trace)
    pub enable_flags: Option<u32>,
    /// Optional PERFINFO_GROUPMASK to set via TraceSetInformation (for extended masks)
    pub group_mask: Option<[u32; 8]>,
}

/// A kernel trace session with lifecycle management
pub struct KernelTraceSession {
    config: TraceConfig,
    trace: Option<ferrisetw::trace::KernelTrace>,
    trace_handle: Option<PROCESSTRACE_HANDLE>,
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl KernelTraceSession {
    /// Create a new kernel trace session
    pub fn new(config: TraceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(KernelTraceSession {
            config,
            trace: None,
            trace_handle: None,
            control_handle: None,
        })
    }

    /// Start the trace session and return the trace handle for processing.
    /// Events will be pushed to the provided `collected_events` vector.
    pub fn start(
        &mut self,
        collected_events: Arc<Mutex<Vec<events::FileIoEvent>>>,
    ) -> Result<PROCESSTRACE_HANDLE, Box<dyn std::error::Error>> {
        let events_clone = collected_events;

        // Create a kernel provider with the FILE_IO_GUID and the specified flags
        // The library will OR these flags into the trace's EnableFlags
        let kernel_provider = kernel_providers::KernelProvider::new(
            FILE_IO_GUID,
            self.config.enable_flags.unwrap_or(0),
        );

        let provider = Provider::kernel(&kernel_provider)
            .level(0xFF) // LogAll
            .any(0) // Match all keywords
            .all(0)
            .add_callback(
                move |record: &EventRecord, _schema_locator: &SchemaLocator| {
                    let opcode = record.opcode();
                    let event_id = record.event_id();
                    let version = record.version();
                    let timestamp = record.raw_timestamp() as u64;
                    let process_id = record.process_id();
                    let thread_id = record.thread_id();

                    // Log the event
                    events::log_event(opcode, event_id, version, timestamp, process_id, thread_id);

                    // Store the event
                    if let Ok(mut events) = events_clone.lock() {
                        events.push(events::FileIoEvent {
                            opcode,
                            event_id,
                            version,
                            timestamp,
                            process_id,
                            thread_id,
                        });
                    }
                },
            )
            .build();

        // Build the kernel trace with stop_if_exist to handle lingering sessions
        let builder = KernelTrace::new()
            .named(self.config.session_name.clone())
            .enable(provider)
            .stop_if_exist(true);

        // Start the trace (without processing yet)
        let (trace, trace_handle) = builder
            .start()
            .map_err(|e| format!("Trace start failed: {:?}", e))?;
        self.trace = Some(trace);
        self.trace_handle = Some(trace_handle);

        // Get the control handle by querying the session
        let control_handle = self.query_control_handle()?;
        self.control_handle = Some(control_handle);

        // If we have a group mask, set it via TraceSetInformation
        // This is for extended masks that can't be set via EnableFlags
        if let Some(mask) = self.config.group_mask {
            self.set_group_mask(mask)?;
        }

        Ok(trace_handle)
    }

    /// Get the trace handle for processing
    pub fn get_trace_handle(&self) -> PROCESSTRACE_HANDLE {
        self.trace_handle.expect("Trace not started")
    }

    /// Request rundown (DCEnd events) for all providers
    pub fn request_rundown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let control_handle = self.control_handle.ok_or("Control handle not available")?;

        let result = unsafe {
            Etw::EnableTraceEx2(
                control_handle,
                &FILE_IO_GUID as *const GUID,
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
            return Err(format!("EnableTraceEx2 failed: {:?}", e).into());
        }

        log::debug!("Rundown requested for FileIo provider");
        Ok(())
    }

    /// Stop the trace session
    ///
    /// Sends EVENT_TRACE_CONTROL_STOP directly via the control handle,
    /// allowing the ProcessTrace thread to process remaining events
    /// (including rundown) before it naturally exits.
    ///
    /// We must NOT call trace.stop() or rely on KernelTrace::drop() here,
    /// because ferrisetw's non_consuming_stop calls CloseTrace *before*
    /// ControlTrace(STOP), which aborts the background thread and drops
    /// all rundown events.
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(control_handle) = self.control_handle {
            let mut buffer = self.build_trace_properties();
            let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };

            unsafe {
                windows::Win32::System::Diagnostics::Etw::ControlTraceW(
                    control_handle,
                    windows::core::PCWSTR::null(),
                    props as *mut EVENT_TRACE_PROPERTIES,
                    windows::Win32::System::Diagnostics::Etw::EVENT_TRACE_CONTROL_STOP,
                )
            }
            .ok()
            .map_err(|e| format!("ControlTraceW STOP failed: {:?}", e))?;
        }

        // Drop the trace object. Its Drop impl will call close_trace +
        // control_trace(STOP), but the session is already stopped so
        // control_trace will harmlessly fail.
        self.trace.take();

        Ok(())
    }

    /// Build an EVENT_TRACE_PROPERTIES buffer populated with the session name.
    fn build_trace_properties(&self) -> Vec<u8> {
        let name_wide: Vec<u16> = self
            .config
            .session_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
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
            ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
        }

        buffer
    }

    /// Query the control handle by calling ControlTraceW with QUERY
    fn query_control_handle(&self) -> Result<CONTROLTRACE_HANDLE, Box<dyn std::error::Error>> {
        let mut buffer = self.build_trace_properties();

        let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
        let name_ptr = unsafe {
            buffer.as_mut_ptr().add(props.LoggerNameOffset as usize) as *const u16
        };

        let result = unsafe {
            Etw::ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                PCWSTR::from_raw(name_ptr),
                props as *mut EVENT_TRACE_PROPERTIES,
                Etw::EVENT_TRACE_CONTROL_QUERY,
            )
        }
        .ok();

        if let Err(e) = result {
            return Err(format!("ControlTraceW QUERY failed: {:?}", e).into());
        }

        let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
        Ok(CONTROLTRACE_HANDLE {
            Value: handle_value,
        })
    }

    /// Set PERFINFO_GROUPMASK via TraceSetInformation
    fn set_group_mask(&self, masks: [u32; 8]) -> Result<(), Box<dyn std::error::Error>> {
        let control_handle = self.control_handle.ok_or("Control handle not available")?;

        // PERFINFO_GROUPMASK is 8 ULONGs = 32 bytes
        // We use TraceSystemTraceEnableFlagsInfo (4) to set the extended mask

        // Build the PERFINFO_GROUPMASK structure
        let mut group_mask_data = [0u32; 8];
        group_mask_data.copy_from_slice(&masks);

        // TraceSystemTraceEnableFlagsInfo = 4
        const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;

        let result = unsafe {
            Etw::TraceSetInformation(
                control_handle,
                std::mem::transmute(TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO),
                group_mask_data.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of::<[u32; 8]>() as u32,
            )
        }
        .ok();

        if let Err(e) = result {
            return Err(format!("TraceSetInformation (GroupMask) failed: {:?}", e).into());
        }

        log::debug!("Set PERFINFO_GROUPMASK to {:?}", masks);
        Ok(())
    }
}

impl Drop for KernelTraceSession {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
