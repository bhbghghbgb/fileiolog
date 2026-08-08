use std::sync::{Arc, Mutex};

use ferrisetw::EventRecord;
use ferrisetw::provider::*;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::*;
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, PROCESSTRACE_HANDLE, WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

use crate::event::{FltIoCompletionEvent, RawEvent};

/// Parse a Pointer property and convert to usize via the EtwPropConvert trait.
fn parse_ptr(parser: &ferrisetw::parser::Parser, name: &str) -> usize {
    match parser.try_parse::<ferrisetw::parser::Pointer>(name) {
        Ok(p) => fileiolog::etw::EtwPropConvert::<ferrisetw::parser::Pointer>::convert(p),
        Err(_) => 0,
    }
}

const FILE_IO_GUID: GUID = GUID::from_u128(0x90cbdc39_4a3e_11d1_84f4_0000f80464e3);

pub struct TraceConfig {
    pub session_name: String,
    pub group_mask: [u32; 8],
}

pub struct KernelTraceSession {
    config: TraceConfig,
    trace: Option<KernelTrace>,
    trace_handle: Option<PROCESSTRACE_HANDLE>,
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl KernelTraceSession {
    pub fn new(config: TraceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            trace: None,
            trace_handle: None,
            control_handle: None,
        })
    }

    pub fn start(
        &mut self,
        events: Arc<Mutex<Vec<RawEvent>>>,
    ) -> Result<PROCESSTRACE_HANDLE, Box<dyn std::error::Error>> {
        let events_clone = events;

        let kernel_provider = kernel_providers::KernelProvider::new(
            FILE_IO_GUID,
            0, // flags set via group mask
        );

        let provider = Provider::kernel(&kernel_provider)
            .level(0xFF)
            .any(0)
            .all(0)
            .add_callback(
                move |record: &EventRecord, schema_locator: &SchemaLocator| {
                    let opcode = record.opcode();
                    let version = record.version();
                    let event_id = record.event_id();

                    // Only capture FltIoCompletion events (opcodes 98, 99)
                    if opcode != 98 && opcode != 99 {
                        return;
                    }

                    let schema = match schema_locator.event_schema(record) {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let parser = ferrisetw::parser::Parser::create(record, &schema);

                    let initial_time: u64 = parser.try_parse("InitialTime").unwrap_or(0);
                    let routine_addr = parse_ptr(&parser, "RoutineAddr");
                    let file_object = parse_ptr(&parser, "FileObject");
                    let file_context = parse_ptr(&parser, "FileContext");
                    let irp_ptr = parse_ptr(&parser, "IrpPtr");
                    let callback_data_ptr = parse_ptr(&parser, "CallbackDataPtr");
                    let major_function: u32 = parser.try_parse("MajorFunction").unwrap_or(0);

                    let event = RawEvent {
                        opcode,
                        event_id,
                        version,
                        timestamp: record.raw_timestamp() as u64,
                        process_id: record.process_id(),
                        thread_id: record.thread_id(),
                        flt: FltIoCompletionEvent {
                            initial_time,
                            routine_addr,
                            file_object,
                            file_context,
                            irp_ptr,
                            callback_data_ptr,
                            major_function,
                        },
                    };

                    if let Ok(mut evts) = events_clone.lock() {
                        evts.push(event);
                    }
                },
            )
            .build();

        let builder = KernelTrace::new()
            .named(self.config.session_name.clone())
            .enable(provider)
            .stop_if_exist(true);

        let (trace, trace_handle) = builder
            .start()
            .map_err(|e| format!("Trace start failed: {:?}", e))?;
        self.trace = Some(trace);
        self.trace_handle = Some(trace_handle);

        let control_handle = self.query_control_handle()?;
        self.control_handle = Some(control_handle);

        // Set the group mask
        self.set_group_mask(self.config.group_mask)?;

        Ok(trace_handle)
    }

    #[allow(dead_code)]
    pub fn get_trace_handle(&self) -> PROCESSTRACE_HANDLE {
        self.trace_handle.expect("Trace not started")
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(control_handle) = self.control_handle {
            let mut buffer = self.build_trace_properties();
            let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };

            unsafe {
                Etw::ControlTraceW(
                    control_handle,
                    PCWSTR::null(),
                    props as *mut EVENT_TRACE_PROPERTIES,
                    Etw::EVENT_TRACE_CONTROL_STOP,
                )
            }
            .ok()
            .map_err(|e| format!("ControlTraceW STOP failed: {:?}", e))?;
        }

        self.trace.take();
        Ok(())
    }

    fn build_trace_properties(&self) -> Vec<u8> {
        let name_wide: Vec<u16> = self
            .config
            .session_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
        let name_buf_size = (200 + 1) * 2;
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

        buffer
    }

    fn query_control_handle(&self) -> Result<CONTROLTRACE_HANDLE, Box<dyn std::error::Error>> {
        let mut buffer = self.build_trace_properties();
        let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
        let name_ptr = unsafe {
            buffer.as_mut_ptr().add(props.LoggerNameOffset as usize) as *const u16
        };

        unsafe {
            Etw::ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                PCWSTR::from_raw(name_ptr),
                props as *mut EVENT_TRACE_PROPERTIES,
                Etw::EVENT_TRACE_CONTROL_QUERY,
            )
        }
        .ok()
        .map_err(|e| format!("ControlTraceW QUERY failed: {:?}", e))?;

        let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
        Ok(CONTROLTRACE_HANDLE {
            Value: handle_value,
        })
    }

    fn set_group_mask(&self, masks: [u32; 8]) -> Result<(), Box<dyn std::error::Error>> {
        let control_handle = self.control_handle.ok_or("Control handle not available")?;

        const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;

        let result = unsafe {
            Etw::TraceSetInformation(
                control_handle,
                std::mem::transmute::<i32, Etw::TRACE_QUERY_INFO_CLASS>(
                    TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO,
                ),
                masks.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of::<[u32; 8]>() as u32,
            )
        }
        .ok();

        if let Err(e) = result {
            return Err(format!("TraceSetInformation (GroupMask) failed: {:?}", e).into());
        }

        Ok(())
    }
}

impl Drop for KernelTraceSession {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
