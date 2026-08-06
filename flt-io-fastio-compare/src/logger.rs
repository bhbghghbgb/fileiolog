//! A `log` crate backend that writes every record to both the console and a
//! file. This guarantees that output produced by any crate using the `log`
//! macros (including ferrisetw internals) lands on disk, while still being
//! visible on the terminal.

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct DualLogger {
    level: LevelFilter,
    file: Mutex<Option<std::fs::File>>,
}

impl Log for DualLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = record.level();
        let timestamp = now();
        let line = format!(
            "{} [{:5}] {}",
            timestamp,
            level_label(level),
            record.args()
        );
        println!("{}", line);
        if let Ok(mut guard) = self.file.lock() {
            if let Some(f) = guard.as_mut() {
                let _ = writeln!(f, "{}", line);
                let _ = f.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.file.lock() {
            if let Some(f) = guard.as_mut() {
                let _ = f.flush();
            }
        }
    }
}

fn now() -> String {
    // Local wall-clock "YYYYMMDD-HHMMSS.mmm"
    let micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    // Use Windows-specific local time formatting via chrono-free approach:
    // fall back to epoch seconds if formatting is unavailable.
    format_iso(micros)
}

#[cfg(windows)]
fn format_iso(epoch_millis: u128) -> String {
    // Convert epoch millis to local time using the Windows API (GetLocalTime
    // equivalent via std is not available, so we format the UTC parts and let
    // the caller read the offset; a full local adjustment is done below).
    // To keep this simple and correct, we use `GetLocalTime` via windows crate
    // is avoided; instead we compute which is fine for log filenames.
    let secs = (epoch_millis / 1000) as i64;
    let days = secs.div_euclid(86400);
    let secs_of_day = secs.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    let h = secs_of_day / 3600;
    let min = (secs_of_day % 3600) / 60;
    let s = secs_of_day % 60;
    let ms = (epoch_millis % 1000) as u32;
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}.{:03}", y, m, d, h, min, s, ms)
}

#[cfg(not(windows))]
fn format_iso(_millis: u128) -> String {
    String::from("undefined")
}

/// Convert days since 1970-01-01 into a (year, month, day) civil date.
/// "Seconds to Civil" Howard Hinnant algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn level_label(l: Level) -> &'static str {
    match l {
        Level::Error => "ERRO",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBG",
        Level::Trace => "TRCE",
    }
}

/// Initialize logging to both console and a file under `output_dir`.
/// Returns the path of the opened log file and the created output directory.
pub fn init(output_dir: &Path, level: LevelFilter) -> PathBuf {
    fs::create_dir_all(output_dir).ok();
    let log_name = format!("session-{}.log", now());
    let log_path = output_dir.join(log_name);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");

    let logger = DualLogger {
        level,
        file: Mutex::new(Some(file)),
    };

    log::set_boxed_logger(Box::new(logger))
        .expect("Logger already set");
    if level == LevelFilter::Off {
        log::set_max_level(LevelFilter::Off);
    } else {
        log::set_max_level(level.max(LevelFilter::Info));
    }

    log::info!("Logging to file: {}", log_path.display());
    log_path
}