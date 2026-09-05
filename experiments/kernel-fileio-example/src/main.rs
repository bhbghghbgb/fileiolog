#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser as ClapParser;
use ferrisetw::EventRecord;
use ferrisetw::GUID;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::provider::kernel_providers::KernelProvider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::KernelTrace;
use ferrisetw::trace::stop_trace_by_name;
use serde::Serialize;
use windows::Win32::System::Diagnostics::Etw::EVENT_TRACE_FLAG_DISK_FILE_IO;
use windows::Win32::System::Diagnostics::Etw::EVENT_TRACE_FLAG_DISK_IO;
use windows::Win32::System::Diagnostics::Etw::EVENT_TRACE_FLAG_FILE_IO;
use windows::Win32::System::Diagnostics::Etw::EVENT_TRACE_FLAG_FILE_IO_INIT;

#[derive(Debug, ClapParser)]
#[command(name = "kernel-fileio-example")]
#[command(about = "Basic kernel FileIO ETW trace example")]
struct Args {
    /// Output directory for results
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

// ── Event structs matching MOF definitions ─────────────────────────
// Source: https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/ETW/fileio.md

/// FileIo\_Name — opcodes 0, 32, 35, 36
#[derive(Debug, Clone, Serialize)]
struct FileIoName {
    file_object: u64,
    file_name: String,
}

/// FileIo\_SimpleOp — opcodes 65, 66, 73
#[derive(Debug, Clone, Serialize)]
struct FileIoSimpleOp {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
}

/// FileIo\_Create — opcode 64
#[derive(Debug, Clone, Serialize)]
struct FileIoCreate {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    create_options: u32,
    file_attributes: u32,
    share_access: u32,
    open_path: String,
}

/// FileIo\_ReadWrite — opcodes 67, 68
#[derive(Debug, Clone, Serialize)]
struct FileIoReadWrite {
    offset: u64,
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    io_size: u32,
    io_flags: u32,
}

/// FileIo\_Info — opcodes 69, 70, 71, 74, 75
#[derive(Debug, Clone, Serialize)]
struct FileIoInfo {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    extra_info: u64,
    info_class: u32,
}

/// FileIo\_DirEnum — opcodes 72, 77
#[derive(Debug, Clone, Serialize)]
struct FileIoDirEnum {
    irp_ptr: u64,
    ttid: u32,
    file_object: u64,
    file_key: u64,
    length: u32,
    info_class: u32,
    file_index: u32,
    file_name: String,
}

/// FileIo\_OpEnd — opcode 76
#[derive(Debug, Clone, Serialize)]
struct FileIoOpEnd {
    irp_ptr: u64,
    extra_info: u64,
    nt_status: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ParsedEvent {
    Name(FileIoName),
    SimpleOp(FileIoSimpleOp),
    Create(FileIoCreate),
    ReadWrite(FileIoReadWrite),
    Info(FileIoInfo),
    DirEnum(FileIoDirEnum),
    OpEnd(FileIoOpEnd),
}

#[derive(Debug, Clone, Serialize)]
struct CapturedEvent {
    opcode: u8,
    opcode_name: String,
    event_id: u16,
    version: u32,
    process_id: u32,
    thread_id: u32,
    timestamp: i64,
    parsed: Option<ParsedEvent>,
}

#[derive(Serialize)]
struct OutputData {
    events: Vec<CapturedEvent>,
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    total_events: usize,
    opcode_counts: std::collections::HashMap<String, usize>,
}

// ── Parse helpers ──────────────────────────────────────────────────

fn parse_name(parser: &Parser) -> FileIoName {
    FileIoName {
        file_object: parser.try_parse("FileObject").unwrap(),
        file_name: parser.try_parse("FileName").unwrap(),
    }
}

fn parse_simple_op(parser: &Parser) -> FileIoSimpleOp {
    FileIoSimpleOp {
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        ttid: parser.try_parse("TTID").unwrap(),
        file_object: parser.try_parse("FileObject").unwrap(),
        file_key: parser.try_parse("FileKey").unwrap(),
    }
}

fn parse_create(parser: &Parser) -> FileIoCreate {
    FileIoCreate {
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        ttid: parser.try_parse("TTID").unwrap(),
        file_object: parser.try_parse("FileObject").unwrap(),
        create_options: parser.try_parse("CreateOptions").unwrap(),
        file_attributes: parser.try_parse("FileAttributes").unwrap(),
        share_access: parser.try_parse("ShareAccess").unwrap(),
        open_path: parser.try_parse("OpenPath").unwrap(),
    }
}

fn parse_read_write(parser: &Parser) -> FileIoReadWrite {
    FileIoReadWrite {
        offset: parser.try_parse("Offset").unwrap(),
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        ttid: parser.try_parse("TTID").unwrap(),
        file_object: parser.try_parse("FileObject").unwrap(),
        file_key: parser.try_parse("FileKey").unwrap(),
        io_size: parser.try_parse("IoSize").unwrap(),
        io_flags: parser.try_parse("IoFlags").unwrap(),
    }
}

fn parse_info(parser: &Parser) -> FileIoInfo {
    FileIoInfo {
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        ttid: parser.try_parse("TTID").unwrap(),
        file_object: parser.try_parse("FileObject").unwrap(),
        file_key: parser.try_parse("FileKey").unwrap(),
        extra_info: parser.try_parse("ExtraInfo").unwrap(),
        info_class: parser.try_parse("InfoClass").unwrap(),
    }
}

fn parse_dir_enum(parser: &Parser) -> FileIoDirEnum {
    FileIoDirEnum {
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        ttid: parser.try_parse("TTID").unwrap(),
        file_object: parser.try_parse("FileObject").unwrap(),
        file_key: parser.try_parse("FileKey").unwrap(),
        length: parser.try_parse("Length").unwrap(),
        info_class: parser.try_parse("InfoClass").unwrap(),
        file_index: parser.try_parse("FileIndex").unwrap(),
        file_name: parser.try_parse("FileName").unwrap(),
    }
}

fn parse_op_end(parser: &Parser) -> FileIoOpEnd {
    FileIoOpEnd {
        irp_ptr: parser.try_parse("IrpPtr").unwrap(),
        extra_info: parser.try_parse("ExtraInfo").unwrap(),
        nt_status: parser.try_parse("NtStatus").unwrap(),
    }
}

// ── Event dispatch ─────────────────────────────────────────────────

fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0 => "Name",
        32 => "FileCreate",
        35 => "FileDelete",
        36 => "FileRundown",
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
        _ => "Unknown",
    }
}

fn parse_event(opcode: u8, parser: &Parser) -> Option<ParsedEvent> {
    Some(match opcode {
        0 | 32 | 35 | 36 => ParsedEvent::Name(parse_name(parser)),
        64 => ParsedEvent::Create(parse_create(parser)),
        65 | 66 | 73 => ParsedEvent::SimpleOp(parse_simple_op(parser)),
        67 | 68 => ParsedEvent::ReadWrite(parse_read_write(parser)),
        69 | 70 | 71 | 74 | 75 => ParsedEvent::Info(parse_info(parser)),
        72 | 77 => ParsedEvent::DirEnum(parse_dir_enum(parser)),
        76 => ParsedEvent::OpEnd(parse_op_end(parser)),
        _ => return None,
    })
}

fn main() {
    let args = Args::parse();
    let output_dir = &args.output;

    let _ = std::fs::create_dir_all(output_dir);
    fileiolog::logging::init_logging(output_dir, "kernel-fileio-example");

    log::info!("Starting old NT Kernel FileIO trace...");

    let events: Arc<Mutex<Vec<CapturedEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    let callback = move |record: &EventRecord, schema_locator: &SchemaLocator| {
        let opcode = record.opcode();
        let name = opcode_name(opcode).to_string();

        let parsed = match schema_locator.event_schema(record) {
            Ok(schema) => {
                let parser = Parser::create(record, &schema);
                parse_event(opcode, &parser)
            }
            Err(_) => None,
        };

        println!("[FileIO-{}] {:?}", name, parsed.as_ref().unwrap_or(&ParsedEvent::Name(FileIoName { file_object: 0, file_name: "<parse error>".into() })));

        let event = CapturedEvent {
            opcode,
            opcode_name: name,
            event_id: record.event_id(),
            version: record.version() as u32,
            process_id: record.process_id(),
            thread_id: record.thread_id(),
            timestamp: record.raw_timestamp(),
            parsed,
        };

        if let Ok(mut evts) = events_clone.lock() {
            evts.push(event);
        }
    };

    // Enable ALL kernel FileIO flags to capture every event type
    let file_io_provider = Provider::kernel(&KernelProvider::new(
        GUID::from_values(
            0x90cbdc39,
            0x4a3e,
            0x11d1,
            [0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3],
        ),
        EVENT_TRACE_FLAG_DISK_FILE_IO.0,
    ))
    .add_callback(callback)
    .build();

    let session_name = "FileIoKernelTrace";

    if let Ok(_) = stop_trace_by_name(session_name) {
        log::info!("Stopped orphan trace session.");
    } else {
        log::info!("No orphan trace session found.");
    }

    let trace = KernelTrace::new()
        .named(session_name.to_string())
        .enable(file_io_provider)
        .start_and_process()
        .expect("Failed to start kernel trace (run as Administrator!)");

    log::info!("Kernel FileIO trace active for 3 seconds...");

    // Trigger file operations to generate ETW events
    log::info!("Triggering file operations...");
    let bin = file_ops_trigger::bin_path();
    let _ = std::process::Command::new(&bin)
        .output()
        .map_err(|e| log::warn!("Failed to invoke file-ops-trigger: {}", e));

    std::thread::sleep(Duration::from_secs(3));

    log::info!("Stopping trace session by name to allow rundown processing...");
    if let Err(e) = stop_trace_by_name(session_name) {
        log::warn!("Failed to stop trace by name: {:?}", e);
    }

    log::info!("Kernel FileIO trace waiting for rundown events to be processed...");
    std::thread::sleep(Duration::from_secs(3));

    log::info!("Trace stopped and rundown events should have been processed.");

    drop(trace);

    // Collect results
    let collected = events.lock().unwrap().clone();
    log::info!("Captured {} events total", collected.len());

    // Build summary
    let mut opcode_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for event in &collected {
        *opcode_counts.entry(event.opcode_name.clone()).or_insert(0) += 1;
    }

    let total_events = collected.len();
    let output = OutputData {
        events: collected,
        summary: Summary {
            total_events,
            opcode_counts,
        },
    };

    // Write JSON output
    if let Err(e) = fs::create_dir_all(output_dir) {
        log::error!("Failed to create output directory: {}", e);
        return;
    }

    let json_path = output_dir.join("kernel_fileio_output.json");
    match serde_json::to_string_pretty(&output) {
        Ok(json) => {
            if let Err(e) = fs::write(&json_path, &json) {
                log::error!("Failed to write {}: {}", json_path.display(), e);
            } else {
                log::info!("JSON output saved to {}", json_path.display());
            }
        }
        Err(e) => {
            log::error!("Failed to serialize results: {}", e);
        }
    }

    // Write human-readable text output
    let txt_path = output_dir.join("kernel_fileio_output.txt");
    let mut txt = String::from("=== Kernel FileIO Trace Results ===\n\n");
    txt.push_str(&format!("Total events: {}\n\n", output.summary.total_events));
    txt.push_str("Event counts by opcode:\n");
    let mut sorted_counts: Vec<_> = output.summary.opcode_counts.iter().collect();
    sorted_counts.sort_by_key(|(name, _)| (*name).clone());
    for (name, count) in &sorted_counts {
        txt.push_str(&format!("  {}: {}\n", name, count));
    }
    txt.push_str("\n--- Events ---\n");
    for event in &output.events {
        txt.push_str(&format!(
            "[{}] id={}, v={}, pid={}, tid={}, ts={}\n",
            event.opcode_name, event.event_id, event.version,
            event.process_id, event.thread_id, event.timestamp
        ));
        if let Some(ref parsed) = event.parsed {
            txt.push_str(&format!("  {:?}\n", parsed));
        }
    }

    if let Err(e) = fs::write(&txt_path, &txt) {
        log::error!("Failed to write {}: {}", txt_path.display(), e);
    } else {
        log::info!("Text output saved to {}", txt_path.display());
    }
}
