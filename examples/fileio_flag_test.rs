//! FileIo EnableFlags & PERFINFO_GROUPMASK test
//!
//! Tests which EVENT_TRACE_FLAG_* and PERFINFO_GROUPMASK bits enable which
//! FileIo event types in an NT Kernel Logger session.
//!
//! Run with: `cargo run --example fileio_flag_test`
//! Requires Administrator privileges.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ferrisetw::EventRecord;
use ferrisetw::GUID;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::provider::kernel_providers::KernelProvider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{KernelTrace, TraceTrait};
use windows::core::PCWSTR;
use windows::Win32::System::Diagnostics::Etw;

// ─── Constants ────────────────────────────────────────────────────

const FILE_IO_GUID: GUID = GUID::from_values(
    0x90cbdc39,
    0x4a3e,
    0x11d1,
    [0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3],
);

const SESSION_NAME_PREFIX: &str = "FileIoFlagTest";

// Standard EnableFlags (Masks[0] equivalents)
const EVENT_TRACE_FLAG_FILE_IO: u32 = 0x02000000;
const EVENT_TRACE_FLAG_FILE_IO_INIT: u32 = 0x04000000;
const EVENT_TRACE_FLAG_DISK_FILE_IO: u32 = 0x00000200;
const EVENT_TRACE_FLAG_DISK_IO: u32 = 0x00000100;

// ─── PERFINFO_GROUPMASK definition ────────────────────────────────

/// Undocumented PERFINFO_GROUPMASK from ntwmi.h / Geoff Chappell.
/// 8 ULONG masks = 256 bits total.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct PerfInfoGroupMask {
    masks: [u32; 8],
}

impl PerfInfoGroupMask {
    fn new() -> Self {
        Self::default()
    }

    /// Set a bit using the conventional encoding: high 3 bits = mask index, low 29 bits = bit position.
    fn set_bit(&mut self, conventional_value: u32) {
        let mask_index = ((conventional_value >> 29) & 0x07) as usize;
        let bit = conventional_value & 0x1FFFFFFF;
        self.masks[mask_index] |= bit;
    }

    /// PERF_FLT_IO_INIT = Masks[4] bit 11
    fn flt_io_init() -> Self {
        let mut m = Self::new();
        m.masks[4] |= 0x00080000;
        m
    }

    /// PERF_FLT_IO = Masks[4] bit 16
    fn flt_io() -> Self {
        let mut m = Self::new();
        m.masks[4] |= 0x00100000;
        m
    }

    /// PERF_FLT_FASTIO = Masks[4] bit 17
    fn flt_fast_io() -> Self {
        let mut m = Self::new();
        m.masks[4] |= 0x00200000;
        m
    }

    /// PERF_FLT_IO_FAILURE = Masks[4] bit 18
    fn flt_io_failure() -> Self {
        let mut m = Self::new();
        m.masks[4] |= 0x00400000;
        m
    }

    /// All FltIo masks combined
    fn all_flt() -> Self {
        let mut m = Self::new();
        m.masks[4] |= 0x00780000; // bits 15-18 (FltIoInit|FltIo|FltFastIo|FltIoFailure)
        m
    }
}

// ─── FileIo Event Struct Definitions (V0) ─────────────────────────

#[derive(Debug)]
struct FileIoV0Name {
    file_object: u64,
    file_name: String,
}

// ─── FileIo Event Struct Definitions (V1) ─────────────────────────

#[derive(Debug)]
struct FileIoV1Name {
    file_object: u64,
    file_name: String,
}

// ─── FileIo Event Struct Definitions (V2) ─────────────────────────

#[derive(Debug)]
struct FileIoV2Name {
    file_object: u64,
    file_name: String,
}

#[derive(Debug)]
struct FileIoV2Create {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    create_options: u32,
    file_attributes: u32,
    share_access: u32,
    open_path: String,
}

#[derive(Debug)]
struct FileIoV2ReadWrite {
    offset: u64,
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    io_size: u32,
    io_flags: u32,
}

#[derive(Debug)]
struct FileIoV2SimpleOp {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
}

#[derive(Debug)]
struct FileIoV2Info {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    extra_info: u64,
    info_class: u32,
}

#[derive(Debug)]
struct FileIoV2DirEnum {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    length: u32,
    info_class: u32,
    file_index: u32,
    file_name: String,
}

#[derive(Debug)]
struct FileIoV2OpEnd {
    irp_ptr: u64,
    extra_info: u64,
    nt_status: u32,
}

#[derive(Debug)]
struct FileIoV2MapFile {
    view_base: u64,
    file_object: u64,
    misc_info: u64,
    view_size: u64,
    process_id: u32,
}

// ─── FileIo Event Struct Definitions (V3) ─────────────────────────

#[derive(Debug)]
struct FileIoV3Name {
    file_object: u64,
    file_name: String,
}

#[derive(Debug)]
struct FileIoV3Create {
    irp_ptr: u64,
    file_object: u64,
    ttid: u32,
    create_options: u32,
    file_attributes: u32,
    share_access: u32,
    open_path: String,
}

#[derive(Debug)]
struct FileIoV3ReadWrite {
    offset: u64,
    irp_ptr: u64,
    file_object: u64,
    file_key: u64,
    ttid: u32,
    io_size: u32,
    io_flags: u32,
}

#[derive(Debug)]
struct FileIoV3SimpleOp {
    irp_ptr: u64,
    file_object: u64,
    file_key: u64,
    ttid: u32,
}

#[derive(Debug)]
struct FileIoV3Info {
    irp_ptr: u64,
    file_object: u64,
    file_key: u64,
    extra_info: u64,
    ttid: u32,
    info_class: u32,
}

#[derive(Debug)]
struct FileIoV3DirEnum {
    irp_ptr: u64,
    file_object: u64,
    file_key: u64,
    ttid: u32,
    length: u32,
    info_class: u32,
    file_index: u32,
    file_name: String,
}

#[derive(Debug)]
struct FileIoV3OpEnd {
    irp_ptr: u64,
    extra_info: u64,
    nt_status: u32,
}

#[derive(Debug)]
struct FileIoV3PathOp {
    irp_ptr: u64,
    file_object: u64,
    file_key: u64,
    extra_info: u64,
    ttid: u32,
    info_class: u32,
    file_name: String,
}

#[derive(Debug)]
struct FltIoInit {
    routine_addr: u64,
    file_object: u64,
    file_context: u64,
    irp: u64,
    callback_data: u64,
    major_function: u32,
}

#[derive(Debug)]
struct FltIoCompletion {
    initial_time: u64,
    routine_addr: u64,
    file_object: u64,
    file_context: u64,
    irp: u64,
    callback_data: u64,
    major_function: u32,
}

#[derive(Debug)]
struct FltIoFailure {
    routine_addr: u64,
    file_object: u64,
    file_context: u64,
    irp: u64,
    callback_data: u64,
    major_function: u32,
    status: u32,
}

// ─── Event identification ─────────────────────────────────────────

fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0 => "Name",
        32 => "FileCreate",
        35 => "FileDelete",
        36 => "FileRundown",
        37 => "MapFile",
        38 => "UnmapFile",
        39 => "MapFileDCStart",
        40 => "MapFileDCEnd",
        64 => "Create",
        65 => "Cleanup",
        66 => "Close",
        67 => "Read",
        68 => "Write",
        69 => "SetInfo",
        70 => "Delete",
        71 => "Rename",
        72 => "DirEnum",
        73 => "Flush",
        74 => "QueryInfo",
        75 => "FSControl",
        76 => "OperationEnd",
        77 => "DirNotify",
        79 => "DeletePath",
        80 => "RenamePath",
        81 => "SetLinkPath",
        96 => "PreOpInit",
        97 => "PostOpInit",
        98 => "PreOpCompletion",
        99 => "PostOpCompletion",
        100 => "PreOpFailure",
        101 => "PostOpFailure",
        _ => "Unknown",
    }
}

fn class_name_for_event(version: u8, opcode: u8) -> &'static str {
    match (version, opcode) {
        // V0
        (0, 0) => "FileIo_V0_Name",
        // V1
        (1, 0 | 32) => "FileIo_V1_Name",
        // V2
        (2, 0 | 32 | 35 | 36) => "FileIo_V2_Name",
        (2, 64) => "FileIo_V2_Create",
        (2, 67 | 68) => "FileIo_V2_ReadWrite",
        (2, 65 | 66 | 73) => "FileIo_V2_SimpleOp",
        (2, 69 | 70 | 71 | 74 | 75) => "FileIo_V2_Info",
        (2, 72 | 77) => "FileIo_V2_DirEnum",
        (2, 76) => "FileIo_V2_OpEnd",
        (2, 37 | 38 | 39 | 40) => "FileIo_V2_MapFile",
        // V3
        (3, 0 | 32 | 35 | 36) => "FileIo_V3_Name",
        (3, 64) => "FileIo_V3_Create",
        (3, 67 | 68) => "FileIo_V3_ReadWrite",
        (3, 65 | 66 | 73) => "FileIo_V3_SimpleOp",
        (3, 69 | 70 | 71 | 74 | 75) => "FileIo_V3_Info",
        (3, 72 | 77) => "FileIo_V3_DirEnum",
        (3, 76) => "FileIo_V3_OpEnd",
        (3, 79 | 80 | 81) => "FileIo_V3_PathOp",
        (3, 96 | 97) => "FltIoInit",
        (3, 98 | 99) => "FltIoCompletion",
        (3, 100 | 101) => "FltIoFailure",
        _ => "",
    }
}

fn is_known_event(version: u8, opcode: u8) -> bool {
    !class_name_for_event(version, opcode).is_empty()
}

// ─── Event parsing ────────────────────────────────────────────────

fn parse_event(version: u8, opcode: u8, parser: &Parser) -> Option<Box<dyn std::fmt::Debug>> {
    match (version, opcode) {
        // V0
        (0, 0) => Some(Box::new(FileIoV0Name {
            file_object: parser.try_parse("FileObject").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V1
        (1, 0 | 32) => Some(Box::new(FileIoV1Name {
            file_object: parser.try_parse("FileObject").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V2 Name
        (2, 0 | 32 | 35 | 36) => Some(Box::new(FileIoV2Name {
            file_object: parser.try_parse("FileObject").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V2 Create
        (2, 64) => Some(Box::new(FileIoV2Create {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            create_options: parser.try_parse("CreateOptions").ok()?,
            file_attributes: parser.try_parse("FileAttributes").ok()?,
            share_access: parser.try_parse("ShareAccess").ok()?,
            open_path: parser.try_parse("OpenPath").ok()?,
        })),
        // V2 ReadWrite
        (2, 67 | 68) => Some(Box::new(FileIoV2ReadWrite {
            offset: parser.try_parse("Offset").ok()?,
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            io_size: parser.try_parse("IoSize").ok()?,
            io_flags: parser.try_parse("IoFlags").ok()?,
        })),
        // V2 SimpleOp
        (2, 65 | 66 | 73) => Some(Box::new(FileIoV2SimpleOp {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
        })),
        // V2 Info
        (2, 69 | 70 | 71 | 74 | 75) => Some(Box::new(FileIoV2Info {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            extra_info: parser.try_parse("ExtraInfo").ok()?,
            info_class: parser.try_parse("InfoClass").ok()?,
        })),
        // V2 DirEnum
        (2, 72 | 77) => Some(Box::new(FileIoV2DirEnum {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            length: parser.try_parse("Length").ok()?,
            info_class: parser.try_parse("InfoClass").ok()?,
            file_index: parser.try_parse("FileIndex").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V2 OpEnd
        (2, 76) => Some(Box::new(FileIoV2OpEnd {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            extra_info: parser.try_parse("ExtraInfo").ok()?,
            nt_status: parser.try_parse("NtStatus").ok()?,
        })),
        // V2 MapFile
        (2, 37 | 38 | 39 | 40) => Some(Box::new(FileIoV2MapFile {
            view_base: parser.try_parse("ViewBase").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            misc_info: parser.try_parse("MiscInfo").ok()?,
            view_size: parser.try_parse("ViewSize").ok()?,
            process_id: parser.try_parse("ProcessId").ok()?,
        })),
        // V3 Name
        (3, 0 | 32 | 35 | 36) => Some(Box::new(FileIoV3Name {
            file_object: parser.try_parse("FileObject").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V3 Create
        (3, 64) => Some(Box::new(FileIoV3Create {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            create_options: parser.try_parse("CreateOptions").ok()?,
            file_attributes: parser.try_parse("FileAttributes").ok()?,
            share_access: parser.try_parse("ShareAccess").ok()?,
            open_path: parser.try_parse("OpenPath").ok()?,
        })),
        // V3 ReadWrite
        (3, 67 | 68) => Some(Box::new(FileIoV3ReadWrite {
            offset: parser.try_parse("Offset").ok()?,
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            io_size: parser.try_parse("IoSize").ok()?,
            io_flags: parser.try_parse("IoFlags").ok()?,
        })),
        // V3 SimpleOp
        (3, 65 | 66 | 73) => Some(Box::new(FileIoV3SimpleOp {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
        })),
        // V3 Info
        (3, 69 | 70 | 71 | 74 | 75) => Some(Box::new(FileIoV3Info {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            extra_info: parser.try_parse("ExtraInfo").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            info_class: parser.try_parse("InfoClass").ok()?,
        })),
        // V3 DirEnum
        (3, 72 | 77) => Some(Box::new(FileIoV3DirEnum {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            length: parser.try_parse("Length").ok()?,
            info_class: parser.try_parse("InfoClass").ok()?,
            file_index: parser.try_parse("FileIndex").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // V3 OpEnd
        (3, 76) => Some(Box::new(FileIoV3OpEnd {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            extra_info: parser.try_parse("ExtraInfo").ok()?,
            nt_status: parser.try_parse("NtStatus").ok()?,
        })),
        // V3 PathOp
        (3, 79 | 80 | 81) => Some(Box::new(FileIoV3PathOp {
            irp_ptr: parser.try_parse("IrpPtr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_key: parser.try_parse("FileKey").ok()?,
            extra_info: parser.try_parse("ExtraInfo").ok()?,
            ttid: parser.try_parse("TTID").ok()?,
            info_class: parser.try_parse("InfoClass").ok()?,
            file_name: parser.try_parse("FileName").ok()?,
        })),
        // FltIoInit
        (3, 96 | 97) => Some(Box::new(FltIoInit {
            routine_addr: parser.try_parse("RoutineAddr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_context: parser.try_parse("FileContext").ok()?,
            irp: parser.try_parse("Irp").ok()?,
            callback_data: parser.try_parse("CallbackData").ok()?,
            major_function: parser.try_parse("MajorFunction").ok()?,
        })),
        // FltIoCompletion
        (3, 98 | 99) => Some(Box::new(FltIoCompletion {
            initial_time: parser.try_parse("InitialTime").ok()?,
            routine_addr: parser.try_parse("RoutineAddr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_context: parser.try_parse("FileContext").ok()?,
            irp: parser.try_parse("Irp").ok()?,
            callback_data: parser.try_parse("CallbackData").ok()?,
            major_function: parser.try_parse("MajorFunction").ok()?,
        })),
        // FltIoFailure
        (3, 100 | 101) => Some(Box::new(FltIoFailure {
            routine_addr: parser.try_parse("RoutineAddr").ok()?,
            file_object: parser.try_parse("FileObject").ok()?,
            file_context: parser.try_parse("FileContext").ok()?,
            irp: parser.try_parse("Irp").ok()?,
            callback_data: parser.try_parse("CallbackData").ok()?,
            major_function: parser.try_parse("MajorFunction").ok()?,
            status: parser.try_parse("Status").ok()?,
        })),
        _ => None,
    }
}

// ─── Shared state for callback ────────────────────────────────────

type SeenEvents = Arc<Mutex<HashMap<(u8, u8), &'static str>>>;
type WarnedUnknowns = Arc<Mutex<HashSet<(u8, u8)>>>;

fn create_callback(seen: SeenEvents, warned: WarnedUnknowns) -> impl FnMut(&EventRecord, &SchemaLocator) {
    move |record: &EventRecord, schema_locator: &SchemaLocator| {
        let version = record.version();
        let opcode = record.opcode();
        let name = opcode_name(opcode);

        match schema_locator.event_schema(record) {
            Ok(schema) => {
                let parser = Parser::create(record, &schema);
                let class = class_name_for_event(version, opcode);

                if !class.is_empty() {
                    if let Some(parsed) = parse_event(version, opcode, &parser) {
                        seen.lock().unwrap().insert((opcode, version), class);
                        log::trace!("[FileIo V{} opcode={} {}] {:?}", version, opcode, name, parsed);
                    } else {
                        log::trace!(
                            "[FileIo V{} opcode={} {}] (parse failed for {})",
                            version, opcode, name, class
                        );
                    }
                } else {
                    let mut w = warned.lock().unwrap();
                    if w.insert((opcode, version)) {
                        log::warn!(
                            "Unknown FileIo event: opcode={}, version={}, provider=\"{}\", task=\"{}\", opcode_name=\"{}\"",
                            opcode,
                            version,
                            schema.provider_name(),
                            schema.task_name(),
                            schema.opcode_name(),
                        );
                    } else {
                        log::debug!("Unknown FileIo event: opcode={}, version={}", opcode, version);
                    }
                }
            }
            Err(_) => {
                let mut w = warned.lock().unwrap();
                if w.insert((opcode, version)) {
                    log::warn!("FileIo event with unknown schema: opcode={}, version={}", opcode, version);
                } else {
                    log::debug!("FileIo event with unknown schema: opcode={}, version={}", opcode, version);
                }
            }
        }
    }
}

// ─── Native ETW API helpers ───────────────────────────────────────

/// Allocate a properly sized buffer for EVENT_TRACE_PROPERTIES + name buffers,
/// suitable for passing to ControlTraceW.
#[repr(C)]
struct PropertiesBuf {
    props: Etw::EVENT_TRACE_PROPERTIES,
    name_buf: [u16; 256],
    log_buf: [u16; 256],
}

impl PropertiesBuf {
    fn new(session_name: &str) -> Self {
        let mut buf = Self {
            props: unsafe { std::mem::zeroed() },
            name_buf: [0u16; 256],
            log_buf: [0u16; 256],
        };
        buf.props.Wnode.BufferSize = std::mem::size_of::<Self>() as u32;

        let name: Vec<u16> = session_name.encode_utf16().chain(std::iter::once(0)).collect();
        let len = name.len().min(255);
        buf.name_buf[..len].copy_from_slice(&name[..len]);

        buf.props.LoggerNameOffset = std::mem::offset_of!(Self, name_buf) as u32;
        buf.props.LogFileNameOffset = 0;

        buf
    }

    fn as_mut_ptr(&mut self) -> *mut Etw::EVENT_TRACE_PROPERTIES {
        &mut self.props as *mut _
    }
}

/// Extract Wnode.HistoricalContext from EVENT_TRACE_PROPERTIES.
/// This is the CONTROLTRACE_HANDLE / CONTROLTRACE_ID.
///
/// Layout of WNODE_HEADER:
///   offset 0: BufferSize (u32)
///   offset 4: ProviderId (u32)
///   offset 8: HistoricalContext (u64)  <-- this is what we want
fn extract_session_handle(props: &Etw::EVENT_TRACE_PROPERTIES) -> u64 {
    unsafe {
        let ptr = props as *const _ as *const u8;
        let bytes = std::slice::from_raw_parts(ptr.add(8), 8);
        u64::from_ne_bytes(bytes.try_into().unwrap())
    }
}

/// Query the running session's properties and return the session handle (CONTROLTRACE_HANDLE).
fn query_control_handle(session_name: &str) -> u64 {
    let mut buf = PropertiesBuf::new(session_name);
    let result = unsafe {
        Etw::ControlTraceW(
            Etw::CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(buf.name_buf.as_ptr()),
            buf.as_mut_ptr(),
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    };

    match result.ok() {
        Ok(()) => {
            let handle = extract_session_handle(&buf.props);
            log::info!("Got session handle: 0x{:016X}", handle);
            handle
        }
        Err(e) => {
            log::error!("ControlTraceW QUERY failed: {:?}", e);
            0
        }
    }
}

/// Set PERFINFO_GROUPMASK on a running session via TraceSetInformation.
fn set_group_mask(session_handle: u64, mask: &PerfInfoGroupMask) {
    let handle = Etw::CONTROLTRACE_HANDLE { Value: session_handle };
    let info_class = Etw::TRACE_QUERY_INFO_CLASS(4); // TraceSystemTraceEnableFlagsInfo

    let result = unsafe {
        Etw::TraceSetInformation(
            handle,
            info_class,
            mask as *const _ as *const _,
            std::mem::size_of::<PerfInfoGroupMask>() as u32,
        )
    };

    match result.ok() {
        Ok(()) => {
            log::info!("TraceSetInformation succeeded, group mask: {:?}", mask);
        }
        Err(e) => {
            log::error!("TraceSetInformation failed: {:?}", e);
        }
    }
}

/// Query current PERFINFO_GROUPMASK from a running session.
fn query_group_mask(session_handle: u64) -> PerfInfoGroupMask {
    let handle = Etw::CONTROLTRACE_HANDLE { Value: session_handle };
    let info_class = Etw::TRACE_QUERY_INFO_CLASS(4); // TraceSystemTraceEnableFlagsInfo
    let mut mask = PerfInfoGroupMask::new();

    let result = unsafe {
        Etw::TraceQueryInformation(
            handle,
            info_class,
            &mut mask as *mut _ as *mut _,
            std::mem::size_of::<PerfInfoGroupMask>() as u32,
            None,
        )
    };

    match result.ok() {
        Ok(()) => mask,
        Err(e) => {
            log::error!("TraceQueryInformation failed: {:?}", e);
            PerfInfoGroupMask::new()
        }
    }
}

/// Query current EnableFlags from a running session via ControlTraceW.
fn query_enable_flags(session_name: &str) -> u32 {
    let mut buf = PropertiesBuf::new(session_name);
    let result = unsafe {
        Etw::ControlTraceW(
            Etw::CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(buf.name_buf.as_ptr()),
            buf.as_mut_ptr(),
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    };

    match result.ok() {
        Ok(()) => buf.props.EnableFlags.0,
        Err(e) => {
            log::error!("ControlTraceW QUERY for EnableFlags failed: {:?}", e);
            0
        }
    }
}

// ─── File operation triggers ──────────────────────────────────────

fn trigger_file_operations() {
    let dir = std::env::temp_dir().join("etw_fileio_test");
    let _ = std::fs::create_dir_all(&dir);

    // Create a file
    let path = dir.join("test_file.txt");
    let _ = std::fs::write(&path, "hello world from etw test");

    // Read it
    let _ = std::fs::read(&path);

    // Get metadata (QueryInfo)
    if let Ok(meta) = std::fs::metadata(&path) {
        let _ = meta.len();
    }

    // Open and write
    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(&path) {
        use std::io::Write;
        let _ = f.write_all(b"overwritten data");
        let _ = f.flush();
    }

    // Enumerate directory (DirEnum)
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let _ = entry.file_name();
        }
    }

    // Rename
    let new_path = dir.join("test_file_renamed.txt");
    let _ = std::fs::rename(&path, &new_path);

    // Set metadata (SetInfo)
    if let Ok(_f) = std::fs::OpenOptions::new().write(true).open(&new_path) {
        // just opening triggers info queries
    }

    // Delete
    let _ = std::fs::remove_file(&new_path);

    // Cleanup
    let _ = std::fs::remove_dir(&dir);
}

// ─── Test configuration ───────────────────────────────────────────

struct TestConfig {
    name: String,
    description: String,
    enable_flags: u32,
    group_mask: Option<PerfInfoGroupMask>,
    trigger_ops: bool,
}

struct TestResult {
    name: String,
    description: String,
    seen: HashMap<(u8, u8), &'static str>,
}

fn build_test_configs() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    // ── EnableFlags tests ──
    configs.push(TestConfig {
        name: "DISK_FILE_IO".into(),
        description: "EVENT_TRACE_FLAG_DISK_FILE_IO (0x00000200) - FileIo_Name".into(),
        enable_flags: EVENT_TRACE_FLAG_DISK_FILE_IO | EVENT_TRACE_FLAG_DISK_IO,
        group_mask: None,
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FILE_IO".into(),
        description: "EVENT_TRACE_FLAG_FILE_IO (0x02000000) - OpEnd".into(),
        enable_flags: EVENT_TRACE_FLAG_FILE_IO,
        group_mask: None,
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FILE_IO_INIT".into(),
        description: "EVENT_TRACE_FLAG_FILE_IO_INIT (0x04000000) - Create, ReadWrite, SimpleOp, Info, DirEnum".into(),
        enable_flags: EVENT_TRACE_FLAG_FILE_IO_INIT,
        group_mask: None,
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FILE_IO_INIT+FILE_IO".into(),
        description: "EVENT_TRACE_FLAG_FILE_IO_INIT | FILE_IO".into(),
        enable_flags: EVENT_TRACE_FLAG_FILE_IO_INIT | EVENT_TRACE_FLAG_FILE_IO,
        group_mask: None,
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "ALL_FILE_FLAGS".into(),
        description: "DISK_FILE_IO + FILE_IO + FILE_IO_INIT".into(),
        enable_flags: EVENT_TRACE_FLAG_DISK_FILE_IO | EVENT_TRACE_FLAG_DISK_IO | EVENT_TRACE_FLAG_FILE_IO | EVENT_TRACE_FLAG_FILE_IO_INIT,
        group_mask: None,
        trigger_ops: true,
    });

    // ── PERFINFO_GROUPMASK tests (extended flags) ──
    configs.push(TestConfig {
        name: "FLT_IO_INIT".into(),
        description: "PERF_FLT_IO_INIT (Masks[4] 0x80080000) - FltIoInit".into(),
        enable_flags: 0,
        group_mask: Some(PerfInfoGroupMask::flt_io_init()),
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FLT_IO".into(),
        description: "PERF_FLT_IO (Masks[4] 0x80100000) - FltIoCompletion".into(),
        enable_flags: 0,
        group_mask: Some(PerfInfoGroupMask::flt_io()),
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FLT_FAST_IO".into(),
        description: "PERF_FLT_FASTIO (Masks[4] 0x80200000)".into(),
        enable_flags: 0,
        group_mask: Some(PerfInfoGroupMask::flt_fast_io()),
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "FLT_IO_FAILURE".into(),
        description: "PERF_FLT_IO_FAILURE (Masks[4] 0x80400000) - FltIoFailure".into(),
        enable_flags: 0,
        group_mask: Some(PerfInfoGroupMask::flt_io_failure()),
        trigger_ops: true,
    });

    configs.push(TestConfig {
        name: "ALL_FLT".into(),
        description: "All PERF_FLT_* flags combined".into(),
        enable_flags: 0,
        group_mask: Some(PerfInfoGroupMask::all_flt()),
        trigger_ops: true,
    });

    configs
}

// ─── Test execution ───────────────────────────────────────────────

fn run_test(config: &TestConfig) -> TestResult {
    let session_name = format!("{}_{}", SESSION_NAME_PREFIX, config.name);
    let seen: SeenEvents = Arc::new(Mutex::new(HashMap::new()));
    let warned: WarnedUnknowns = Arc::new(Mutex::new(HashSet::new()));

    let callback = create_callback(seen.clone(), warned.clone());

    let provider = Provider::kernel(&KernelProvider::new(FILE_IO_GUID, config.enable_flags))
        .add_callback(callback)
        .build();

    // Stop any existing session with this name
    let _ = ferrisetw::trace::stop_trace_by_name(&session_name);

    log::info!("Starting trace '{}' ...", config.name);

    let (trace, trace_handle) = match KernelTrace::new()
        .named(session_name.clone())
        .enable(provider)
        .start()
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to start trace '{}': {:?}", config.name, e);
            return TestResult {
                name: config.name.clone(),
                description: config.description.clone(),
                seen: HashMap::new(),
            };
        }
    };

    // Query the actual EnableFlags after start
    let actual_flags = query_enable_flags(&session_name);
    log::info!(
        "  EnableFlags: 0x{:08X} (requested: 0x{:08X})",
        actual_flags, config.enable_flags
    );

    // Get the session handle for extended flags
    let control_handle = query_control_handle(&session_name);

    if let Some(ref mask) = config.group_mask {
        if control_handle != 0 {
            log::info!("  Setting PERFINFO_GROUPMASK: {:?}", mask);
            set_group_mask(control_handle, mask);

            // Verify
            let queried = query_group_mask(control_handle);
            log::info!("  Queried PERFINFO_GROUPMASK: {:?}", queried);
        }
    }

    // Spawn the process thread
    let process_thread = thread::spawn(move || {
        KernelTrace::process_from_handle(trace_handle).ok();
    });

    // Give a moment for the trace to stabilize
    thread::sleep(Duration::from_millis(200));

    // Trigger file operations
    if config.trigger_ops {
        log::info!("  Triggering file operations...");
        trigger_file_operations();
        // Wait for events to arrive
        thread::sleep(Duration::from_secs(2));
    } else {
        log::info!("  No file operations triggered (waiting for user input)...");
        log::info!("  Press Enter to stop...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    // Stop the trace
    log::info!("  Stopping trace...");
    drop(trace);
    let _ = process_thread.join();

    let seen_events = seen.lock().unwrap().clone();
    log::info!("  Got {} event types", seen_events.len());

    TestResult {
        name: config.name.clone(),
        description: config.description.clone(),
        seen: seen_events,
    }
}

// ─── Results display ──────────────────────────────────────────────

fn print_results(results: &[TestResult]) {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  FileIo EnableFlags & PERFINFO_GROUPMASK Test Results");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    // Collect all unique event types seen across all tests
    let mut all_events: HashMap<(u8, u8), &'static str> = HashMap::new();
    for result in results {
        for (&(opcode, version), class) in &result.seen {
            all_events.entry((opcode, version)).or_insert(class);
        }
    }

    // Print a table: rows = event types, columns = test configs
    let mut event_list: Vec<_> = all_events.iter().collect();
    event_list.sort_by_key(|&(&(opcode, version), _)| (version, opcode));

    // Header
    print!("{:<40}", "Event Type");
    for result in results {
        print!(" {:<12}", result.name);
    }
    println!();
    println!("{}", "-".repeat(40 + results.len() * 13));

    // Rows
    for (&(opcode, version), &class) in event_list {
        let name = opcode_name(opcode);
        print!("V{} {:<3} {:<34}", version, opcode, format!("{} ({})", class, name));
        for result in results {
            if result.seen.contains_key(&(opcode, version)) {
                print!(" {:<12}", "YES");
            } else {
                print!(" {:<12}", "-");
            }
        }
        println!();
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Test Configurations");
    println!("═══════════════════════════════════════════════════════════════════════════");
    for result in results {
        println!("  {}: {}", result.name, result.description);
        println!("    Events seen: {}", result.seen.len());
    }
    println!();
}

// ─── Main ─────────────────────────────────────────────────────────

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    println!("FileIo EnableFlags & PERFINFO_GROUPMASK Test");
    println!("Requires Administrator privileges!");
    println!();
    println!("This program tests which EVENT_TRACE_FLAG_* and PERFINFO_GROUPMASK bits");
    println!("enable which FileIo event types in an NT Kernel Logger session.");
    println!();
    println!("For each test configuration, the program will:");
    println!("  1. Start a kernel trace with specific flags");
    println!("  2. Trigger file system operations");
    println!("  3. Record which event types were received");
    println!("  4. Stop the trace");
    println!();
    println!("Press Enter to begin...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    let configs = build_test_configs();
    let mut results = Vec::new();

    for config in &configs {
        println!();
        println!("─────────────────────────────────────────────────────────────────");
        println!("Test: {} - {}", config.name, config.description);
        println!("─────────────────────────────────────────────────────────────────");

        let result = run_test(config);
        results.push(result);
    }

    print_results(&results);
}
