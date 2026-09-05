use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

/// A logger that writes to both stderr and a log file.
/// Respects the RUST_LOG environment variable for filtering.
struct DualWriter {
    file: Mutex<Option<BufWriter<File>>>,
    filter: log::LevelFilter,
}

impl log::Log for DualWriter {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} [{}] {}: {}\n",
            timestamp(),
            record.level(),
            record.target(),
            record.args()
        );
        eprint!("{}", line);
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut w) = *guard {
                let _ = w.write_all(line.as_bytes());
                let _ = w.flush();
            }
        }
    }

    fn flush(&self) {
        let _ = io::stderr().flush();
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut w) = *guard {
                let _ = w.flush();
            }
        }
    }
}

fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn parse_rust_log() -> log::LevelFilter {
    match std::env::var("RUST_LOG") {
        Ok(val) => match val.to_lowercase().as_str() {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" | "warning" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info,
        },
        Err(_) => log::LevelFilter::Info,
    }
}

/// Initialize logging to both console (stderr) and a log file in the given output directory.
/// The log file is named `<prefix>.log`. Respects RUST_LOG for level filtering.
pub fn init_logging(output_dir: &Path, prefix: &str) {
    let log_path = output_dir.join(format!("{}.log", prefix));
    let _ = fs::create_dir_all(output_dir);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()
        .map(|f| BufWriter::with_capacity(8192, f));

    let filter = parse_rust_log();

    let writer = DualWriter {
        file: Mutex::new(file),
        filter,
    };

    log::set_boxed_logger(Box::new(writer)).unwrap();
    log::set_max_level(filter);

    log::info!("Logging to {}", log_path.display());
}
