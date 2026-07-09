use std::time::Duration;

use ferrisetw::EventRecord;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::provider::kernel_providers;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::*;

/// Map old NT kernel FileIO opcodes to human-readable names.
/// Ref: https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/ETW/fileio.md
fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0 => "FileName (Map/Rundown)",
        32 => "Create (Name)",
        35 => "Delete (Name)",
        36 => "FileRundown",
        64 => "Create",
        65 => "Cleanup",
        66 => "Close",
        67 => "Read",
        68 => "Write",
        69 => "SetInformation",
        70 => "Delete",
        71 => "Rename",
        72 => "DirEnum",
        73 => "Flush",
        74 => "QueryInformation",
        75 => "Fsctl",
        76 => "OperationEnd",
        77 => "DirNotify",
        _ => "Unknown",
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting old NT Kernel FileIO trace...");

    let callback = |record: &EventRecord, schema_locator: &SchemaLocator| {
        let opcode = record.opcode();
        let op_name = opcode_name(opcode);

        match schema_locator.event_schema(record) {
            Ok(schema) => {
                let parser = Parser::create(record, &schema);
                let provider_name = schema.provider_name();
                let file_key: Option<u64> = parser.try_parse("FileKey").ok();
                let file_object: Option<u64> = parser.try_parse("FileObject").ok();
                let file_name: Option<String> = parser.try_parse("FileName").ok();
                let file_path: Option<String> = parser.try_parse("FilePath").ok();
                let irp: Option<u64> = parser.try_parse("Irp").ok();
                let io_size: Option<u32> = parser.try_parse("IOSize").ok();
                let thread_id: Option<u32> = parser.try_parse("IssuingThreadId").ok();

                println!(
                    "[FileIO] opcode={} ({}) | provider={} | FileKey={:?} | FileObject={:?} | FileName={:?} | FilePath={:?} | Irp={:?} | IOSize={:?} | ThreadId={:?}",
                    opcode,
                    op_name,
                    provider_name,
                    file_key,
                    file_object,
                    file_name,
                    file_path,
                    irp,
                    io_size,
                    thread_id,
                );
            }
            Err(err) => {
                println!(
                    "[FileIO] opcode={} ({}) | schema error: {:?}",
                    opcode, op_name, err
                );
            }
        }
    };

    let provider = Provider::kernel(&kernel_providers::FILE_IO_PROVIDER)
        .add_callback(callback)
        .build();

    let session_name = "FileIoKernelTrace";

    if let Ok(_) = stop_trace_by_name(session_name) {
        log::info!("Stopped an orphan trace session.")
    } else {
        log::info!("Orphan trace session not found.")
    }

    let trace = KernelTrace::new()
        .named(session_name.to_string())
        .enable(provider)
        .start_and_process()
        .expect("Failed to start kernel trace (run as Administrator!)");

    log::info!("Kernel FileIO trace active for 30 seconds...");
    std::thread::sleep(Duration::from_secs(30));

    trace.stop().unwrap();
    log::info!("Trace stopped.");
}
