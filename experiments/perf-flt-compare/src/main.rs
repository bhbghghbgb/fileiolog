mod analysis;
mod event;
mod session;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use analysis::{Hypothesis, PassResult, Verdict};
use event::{Config, RawEvent};
use session::{KernelTraceSession, TraceConfig};

const NUM_PASSES: usize = 5;
const SESSION_WARMUP_MS: u64 = 500;
const COLLECTION_SECS: u64 = 8;
const PAUSE_BETWEEN_CONFIGS_SECS: u64 = 1;
const PAUSE_BETWEEN_PASSES_SECS: u64 = 2;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== PERF_FLT_IO vs PERF_FLT_FASTIO: intrinsic-discriminator comparison ===");
    log::info!("Research basis: PERF_FLT_IO = SYSTEM_IOFILTER_KW_GENERAL (IRP-based),");
    log::info!("                PERF_FLT_FASTIO = SYSTEM_IOFILTER_KW_FASTIO (cached fast I/O).");
    log::info!("Hypothesis to validate empirically: fast-I/O events have IrpPtr == 0.");
    log::info!("");
    log::info!("Running {} passes x 3 configs (FASTIO, IO, BOTH), same fixed workload each.",
        NUM_PASSES);
    log::info!("");

    let mut passes: Vec<PassResult> = Vec::new();

    for pass in 1..=NUM_PASSES {
        log::info!("--- Pass {}/{} ---", pass, NUM_PASSES);

        let mut fastio = Vec::new();
        let mut io = Vec::new();
        let mut both = Vec::new();

        for cfg in Config::ALL {
            log::info!("  Config: {} (group 0x{:08X})", cfg.name(), cfg.group_value());
            let events = run_config(pass, cfg);
            log::info!(
                "    captured {} FltIoCompletion events ({} fast, {} non-fast)",
                events.len(),
                events.iter().filter(|e| e.is_fast()).count(),
                events.iter().filter(|e| !e.is_fast()).count()
            );
            match cfg {
                Config::FastIoOnly => fastio = events,
                Config::IoOnly => io = events,
                Config::Both => both = events,
            }

            if cfg != Config::Both {
                std::thread::sleep(Duration::from_secs(PAUSE_BETWEEN_CONFIGS_SECS));
            }
        }

        let pr = analysis::score_pass(pass, &fastio, &io, &both);
        log::info!(
            "  [pass {}] IrpPtr==0 fraction: FASTIO={:.1}%, IO={:.1}%",
            pass,
            pr.fastio.fast_frac() * 100.0,
            pr.io.fast_frac() * 100.0
        );
        log::info!(
            "  [pass {}] Both partition: fast={} events ({} majors), nonfast={} events ({} majors)",
            pass,
            pr.both_fast.total,
            pr.both_fast.majors.len(),
            pr.both_nonfast.total,
            pr.both_nonfast.majors.len()
        );

        passes.push(pr);

        if pass < NUM_PASSES {
            log::info!("  Pausing {}s before next pass...", PAUSE_BETWEEN_PASSES_SECS);
            std::thread::sleep(Duration::from_secs(PAUSE_BETWEEN_PASSES_SECS));
        }
    }

    log::info!("");
    log::info!("=== ANALYSIS (pooled over {} passes) ===", passes.len());
    let verdict = analysis::analyze(&passes);
    display_verdict(&verdict);
    save_results(&verdict);
}

/// Run a single trace session for the given configuration.
fn run_config(pass: usize, cfg: Config) -> Vec<RawEvent> {
    let events: Arc<Mutex<Vec<RawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cb = events.clone();

    let session_name = format!("FltCmp_{}_{}", cfg.name(), pass);

    let trace_config = TraceConfig {
        session_name: session_name.clone(),
        group_mask: event::build_group_mask(cfg.group_value()),
    };

    let mut session = match KernelTraceSession::new(trace_config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create session {}: {:?}", session_name, e);
            return Vec::new();
        }
    };

    let handle = match session.start(events_cb) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start session {}: {:?}", session_name, e);
            return Vec::new();
        }
    };

    let proc_thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(handle);
    });

    std::thread::sleep(Duration::from_millis(SESSION_WARMUP_MS));

    trigger_io();

    std::thread::sleep(Duration::from_secs(COLLECTION_SECS));

    let _ = session.stop();
    let _ = proc_thread.join();

    events.lock().unwrap().clone()
}

/// Identical, fixed file-system workload used by every configuration so that
/// fingerprints are directly comparable.
fn trigger_io() {
    let test_dir = Path::new("C:\\temp_flt_compare");

    let _ = fs::create_dir_all(test_dir);

    // Create files (IRP-heavy path).
    for i in 0..10 {
        let path = test_dir.join(format!("t_{}.dat", i));
        let _ = fs::write(&path, format!("data {}", i));
    }

    // Read back (cached -> fast I/O eligible).
    for i in 0..10 {
        let path = test_dir.join(format!("t_{}.dat", i));
        let _ = fs::read(&path);
    }

    // Write again (cached -> fast I/O eligible).
    for i in 0..10 {
        let path = test_dir.join(format!("t_{}.dat", i));
        let _ = fs::write(&path, format!("updated {}", i));
    }

    // Open + write + flush via OpenOptions.
    for i in 0..5 {
        let path = test_dir.join(format!("t_{}.dat", i));
        if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&path) {
            use std::io::Write;
            let _ = file.write_all(b"flush payload");
            let _ = file.flush();
        }
    }

    // Memory-mapped read+write.
    trigger_mmap(test_dir);

    // Delete files (IRP-heavy path).
    for i in 0..10 {
        let path = test_dir.join(format!("t_{}.dat", i));
        let _ = fs::remove_file(&path);
    }

    let _ = fs::remove_dir_all(test_dir);
}

fn trigger_mmap(dir: &Path) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path = dir.join("t_mmap.bin");
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.write_all(&[0u8; 8192]).unwrap();
        file.flush().unwrap();
    }
    let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();
    let mut mmap = unsafe { memmap2::MmapMut::map_mut(&file).unwrap() };
    let pattern: Vec<u8> = (0..256).map(|b| b as u8).collect();
    for slot in mmap.chunks_mut(256) {
        slot.copy_from_slice(&pattern);
    }
    mmap.flush().unwrap();
    for chunk in mmap.chunks(256) {
        let _sum: u64 = chunk.iter().map(|&b| b as u64).sum();
    }
    drop(mmap);
    drop(file);
    let _ = fs::remove_file(&path);
}

/// Human-readable report of the verdict.
fn display_verdict(v: &Verdict) {
    log::info!("Number of passes: {}", v.num_passes);
    log::info!("");

    log::info!("--- Discriminator validation (IrpPtr == 0) ---");
    log::info!("  FASTIO-only run: {:.1}% of events have IrpPtr==0", v.fastio_fast_frac * 100.0);
    log::info!("  IO-only run:     {:.1}% of events have IrpPtr==0", v.io_fast_frac * 100.0);
    if v.fastio_fast_frac > 0.8 && v.io_fast_frac < 0.2 {
        log::info!("  -> IrpPtr==0 is a VALID discriminator (FASTIO events carry no real IRP).");
    } else {
        log::info!("  -> IrpPtr==0 is NOT a clean discriminator; results below are suggestive only.");
    }
    log::info!("");

    log::info!("--- Event totals (pooled) ---");
    log::info!("  FASTIO: {} events (fast={}, nonfast={})", v.fastio.total, v.fastio.fast, v.fastio.nonfast);
    log::info!("  IO:     {} events (fast={}, nonfast={})", v.io.total, v.io.fast, v.io.nonfast);
    log::info!("  BOTH:   {} events", v.both.total);
    log::info!("    fast partition:    {} events", v.both_fast.total);
    log::info!("    non-fast partition:{} events", v.both_nonfast.total);
    log::info!("");

    log::info!("--- MajorFunction set comparison ---");
    let all_majors: BTreeMap<u32, usize> = v
        .both
        .majors
        .keys()
        .chain(v.fastio.majors.keys())
        .map(|&k| (k, 0))
        .collect();
    for (maj, _) in all_majors {
        let f = v.fastio.majors.get(&maj).copied().unwrap_or(0);
        let i = v.io.majors.get(&maj).copied().unwrap_or(0);
        let bf = v.both_fast.majors.get(&maj).copied().unwrap_or(0);
        let bn = v.both_nonfast.majors.get(&maj).copied().unwrap_or(0);
        log::info!(
            "  MJ_{:>2}: FASTIO={}  IO={}  | BOTH-fast={}  BOTH-nonfast={}",
            maj, f, i, bf, bn
        );
    }
    log::info!("");

    log::info!("--- Hypothesis scores ---");
    for (hyp, score) in &v.scores {
        log::info!("  {:<30} {:.1}%", format!("{}", hyp), score);
    }
    log::info!("");
    log::info!(">>> BEST HYPOTHESIS: {} ({:.1}%)", v.best, v.scores[0].1);

    match v.best {
        Hypothesis::Disjoint => {
            log::info!(
                "Interpretation: PERF_FLT_FASTIO and PERF_FLT_IO emit DIFFERENT event instances."
            );
            log::info!(
                "Fast I/O (cached, IrpPtr==0) vs IRP-based I/O; sets are mutually exclusive."
            );
        }
        Hypothesis::Subset => {
            log::info!(
                "Interpretation: FASTIO events are a subset of IO events (same underlying event,"
            );
            log::info!("  extra Fast I/O instances only surface under the IO flag).");
        }
        Hypothesis::PartialOverlap => {
            log::info!(
                "Interpretation: the two flags share some instances but each also emits unique ones."
            );
        }
        Hypothesis::Same => {
            log::info!("Interpretation: the flags are effectively identical for FltIoCompletion.");
        }
    }
}

fn save_results(v: &Verdict) {
    let output = serde_json::json!({
        "num_passes": v.num_passes,
        "discriminator": {
            "fastio_fast_frac": v.fastio_fast_frac,
            "io_fast_frac": v.io_fast_frac,
        },
        "totals": {
            "fastio": { "total": v.fastio.total, "fast": v.fastio.fast, "nonfast": v.fastio.nonfast },
            "io": { "total": v.io.total, "fast": v.io.fast, "nonfast": v.io.nonfast },
            "both": v.both.total,
            "both_fast": v.both_fast.total,
            "both_nonfast": v.both_nonfast.total,
        },
        "major_functions": v.both.majors.keys().map(|maj| {
            serde_json::json!({
                "mj": maj,
                "fastio": v.fastio.majors.get(maj).copied().unwrap_or(0),
                "io": v.io.majors.get(maj).copied().unwrap_or(0),
                "both_fast": v.both_fast.majors.get(maj).copied().unwrap_or(0),
                "both_nonfast": v.both_nonfast.majors.get(maj).copied().unwrap_or(0),
            })
        }).collect::<Vec<_>>(),
        "scores": v.scores.iter().map(|(h, s)| (h.to_string(), s)).collect::<Vec<_>>(),
        "best": v.best.to_string(),
    });

    let path = "flt_compare_results.json";
    match serde_json::to_string_pretty(&output) {
        Ok(json) => {
            if let Err(e) = fs::write(path, json) {
                log::error!("Failed to write {}: {}", path, e);
            } else {
                log::info!("Results saved to {}", path);
            }
        }
        Err(e) => {
            log::error!("Failed to serialize results: {}", e);
        }
    }
}