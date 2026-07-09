#![allow(dead_code)]

use std::time::Duration;

use ferrisetw::EventRecord;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::provider::kernel_providers;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::KernelTrace;
use ferrisetw::trace::stop_trace_by_name;

// ── Event structs matching MOF definitions ─────────────────────────
// Source: https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/ETW/fileio.md

/// FileIo\_Name — opcodes 0, 32, 35, 36
#[derive(Debug)]
struct FileIoName {
    file_object: u32,
    file_name: String,
}

/// FileIo\_SimpleOp — opcodes 65, 66, 73
#[derive(Debug)]
struct FileIoSimpleOp {
    irp_ptr: u32,
    ttid: u32,
    file_object: u32,
    file_key: u32,
}

/// FileIo\_Create — opcode 64
#[derive(Debug)]
struct FileIoCreate {
    irp_ptr: u32,
    ttid: u32,
    file_object: u32,
    create_options: u32,
    file_attributes: u32,
    share_access: u32,
    open_path: String,
}

/// FileIo\_ReadWrite — opcodes 67, 68
#[derive(Debug)]
struct FileIoReadWrite {
    offset: u64,
    irp_ptr: u32,
    ttid: u32,
    file_object: u32,
    file_key: u32,
    io_size: u32,
    io_flags: u32,
}

/// FileIo\_Info — opcodes 69, 70, 71, 74, 75
#[derive(Debug)]
struct FileIoInfo {
    irp_ptr: u32,
    ttid: u32,
    file_object: u32,
    file_key: u32,
    extra_info: u32,
    info_class: u32,
}

/// FileIo\_DirEnum — opcodes 72, 77
#[derive(Debug)]
struct FileIoDirEnum {
    irp_ptr: u32,
    ttid: u32,
    file_object: u32,
    file_key: u32,
    length: u32,
    info_class: u32,
    file_index: u32,
    file_name: String,
}

/// FileIo\_OpEnd — opcode 76
#[derive(Debug)]
struct FileIoOpEnd {
    irp_ptr: u32,
    extra_info: u32,
    nt_status: u32,
}

// ── Parse helpers ──────────────────────────────────────────────────

fn parse_name(parser: &Parser) -> FileIoName {
    FileIoName {
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        file_name: parser.try_parse("FileName").unwrap_or_default(),
    }
}

fn parse_simple_op(parser: &Parser) -> FileIoSimpleOp {
    FileIoSimpleOp {
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        ttid: parser.try_parse("TTID").unwrap_or(0),
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        file_key: parser.try_parse("FileKey").unwrap_or(0),
    }
}

fn parse_create(parser: &Parser) -> FileIoCreate {
    FileIoCreate {
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        ttid: parser.try_parse("TTID").unwrap_or(0),
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        create_options: parser.try_parse("CreateOptions").unwrap_or(0),
        file_attributes: parser.try_parse("FileAttributes").unwrap_or(0),
        share_access: parser.try_parse("ShareAccess").unwrap_or(0),
        open_path: parser.try_parse("OpenPath").unwrap_or_default(),
    }
}

fn parse_read_write(parser: &Parser) -> FileIoReadWrite {
    FileIoReadWrite {
        offset: parser.try_parse("Offset").unwrap_or(0),
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        ttid: parser.try_parse("TTID").unwrap_or(0),
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        file_key: parser.try_parse("FileKey").unwrap_or(0),
        io_size: parser.try_parse("IoSize").unwrap_or(0),
        io_flags: parser.try_parse("IoFlags").unwrap_or(0),
    }
}

fn parse_info(parser: &Parser) -> FileIoInfo {
    FileIoInfo {
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        ttid: parser.try_parse("TTID").unwrap_or(0),
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        file_key: parser.try_parse("FileKey").unwrap_or(0),
        extra_info: parser.try_parse("ExtraInfo").unwrap_or(0),
        info_class: parser.try_parse("InfoClass").unwrap_or(0),
    }
}

fn parse_dir_enum(parser: &Parser) -> FileIoDirEnum {
    FileIoDirEnum {
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        ttid: parser.try_parse("TTID").unwrap_or(0),
        file_object: parser.try_parse("FileObject").unwrap_or(0),
        file_key: parser.try_parse("FileKey").unwrap_or(0),
        length: parser.try_parse("Length").unwrap_or(0),
        info_class: parser.try_parse("InfoClass").unwrap_or(0),
        file_index: parser.try_parse("FileIndex").unwrap_or(0),
        file_name: parser.try_parse("FileName").unwrap_or_default(),
    }
}

fn parse_op_end(parser: &Parser) -> FileIoOpEnd {
    FileIoOpEnd {
        irp_ptr: parser.try_parse("IrpPtr").unwrap_or(0),
        extra_info: parser.try_parse("ExtraInfo").unwrap_or(0),
        nt_status: parser.try_parse("NtStatus").unwrap_or(0),
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

#[derive(Debug)]
enum ParsedEvent {
    Name(FileIoName),
    SimpleOp(FileIoSimpleOp),
    Create(FileIoCreate),
    ReadWrite(FileIoReadWrite),
    Info(FileIoInfo),
    DirEnum(FileIoDirEnum),
    OpEnd(FileIoOpEnd),
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting old NT Kernel FileIO trace...");

    let callback = |record: &EventRecord, schema_locator: &SchemaLocator| {
        let opcode = record.opcode();
        let name = opcode_name(opcode);

        match schema_locator.event_schema(record) {
            Ok(schema) => {
                let parser = Parser::create(record, &schema);

                if let Some(parsed) = parse_event(opcode, &parser) {
                    println!("[FileIO-{}] {:?}", name, parsed);
                }
            }
            Err(err) => {
                println!("[FileIO-{}] schema error: {:?}", name, err);
            }
        }
    };

    // Enable ALL kernel FileIO flags to capture every event type
    let file_io_provider = Provider::kernel(&kernel_providers::FILE_IO_PROVIDER)
        .add_callback(callback.clone())
        .build();
    let file_io_init_provider = Provider::kernel(&kernel_providers::FILE_INIT_IO_PROVIDER)
        .add_callback(callback.clone())
        .build();
    let disk_file_io_provider = Provider::kernel(&kernel_providers::DISK_FILE_IO_PROVIDER)
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
        .enable(file_io_init_provider)
        .enable(disk_file_io_provider)
        .start_and_process()
        .expect("Failed to start kernel trace (run as Administrator!)");

    log::info!("Kernel FileIO trace active for 30 seconds...");
    std::thread::sleep(Duration::from_secs(30));

    trace.stop().unwrap();
    log::info!("Trace stopped.");
}
