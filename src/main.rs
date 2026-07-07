mod event;
mod providers;

use ferrisetw::trace::UserTrace;

fn main() {
    let file_provider = providers::kernel_file::build_provider();

    let trace = UserTrace::new()
        .named(String::from("FileIoLog"))
        .enable(file_provider)
        .start_and_process()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let _ = trace.stop();
}
