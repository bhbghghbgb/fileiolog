// ETL trace replay test — scaffolded, not yet runnable.
//
// To run this test you need a recorded .etl capture of
// Microsoft-Windows-Kernel-File events.  Record one with:
//
//   xperf -on "Microsoft-Windows-Kernel-File" -start FileIoLog
//   ... exercise the scenario ...
//   xperf -stop FileIoLog -d kernel-file.etl
//
// Then copy kernel-file.etl into the crate root and rename the
// path below, or set the FILEIOLOG_ETL environment variable.

use ferrisetw::trace::FileTrace;
use ferrisetw::schema_locator::SchemaLocator;

/// Path to the .etl fixture file.
/// Replace this with the actual path once you have captured a trace.
const ETL_PATH: &str = "tests/fixtures/kernel-file.etl";

#[test]
#[ignore = "no .etl fixture file yet; capture one and update ETL_PATH"]
fn replay_kernel_file_events() {
    let etl_path = std::path::PathBuf::from(ETL_PATH);
    if !etl_path.exists() {
        panic!(
            "ETL fixture not found at {}. Capture a trace first.\n\
             See the comments at the top of this file for instructions.",
            ETL_PATH
        );
    }

    // Use a shared collection to gather parsed events across threads.
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    let trace = FileTrace::new(etl_path, move |_record: &ferrisetw::EventRecord, _locator: &SchemaLocator| {
        if let Ok(mut guard) = events_clone.lock() {
            guard.push(());
        }
    })
    .start_and_process()
    .expect("FileTrace::start_and_process failed");

    // The trace runs on a separate thread; wait for it to finish.
    // Dropping the FileTrace handle stops processing.
    drop(trace);

    let collected = events.lock().unwrap();
    assert!(
        !collected.is_empty(),
        "expected at least one event from the replay"
    );
}
