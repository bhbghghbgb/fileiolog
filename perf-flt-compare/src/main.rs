mod analysis;
mod event;
mod session;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use event::{build_group_mask, RawEvent};
use session::{KernelTraceSession, TraceConfig};

const PERF_FLT_IO: u32 = 0x80100000;
const PERF_FLT_FASTIO: u32 = 0x80200000;

const NUM_PASSES: usize = 5;
const SESSION_WARMUP_MS: u64 = 500;
const COLLECTION_SECS: u64 = 8;
const PAUSE_BETWEEN_PASSES_SECS: u64 = 2;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== PERF_FLT_IO vs PERF_FLT_FASTIO Comparison ===");
    log::info!(
        "PERF_FLT_IO      = 0x{:08X} (FltIoCompletion events)",
        PERF_FLT_IO
    );
    log::info!(
        "PERF_FLT_FASTIO  = 0x{:08X} (FltIoCompletion events)",
        PERF_FLT_FASTIO
    );
    log::info!(
        "Running {} concurrent dual-session passes...",
        NUM_PASSES
    );
    log::info!("");

    let mut all_passes: Vec<analysis::ComparisonResult> = Vec::new();

    for pass_num in 1..=NUM_PASSES {
        log::info!("--- Pass {}/{} ---", pass_num, NUM_PASSES);

        let (events_io, events_fastio) = run_pass(pass_num);

        log::info!(
            "  PERF_FLT_IO captured {} FltIoCompletion events",
            events_io.len()
        );
        log::info!(
            "  PERF_FLT_FASTIO captured {} FltIoCompletion events",
            events_fastio.len()
        );

        let result = analysis::compare_sessions(&events_io, &events_fastio);

        log::info!(
            "  Ratio (FASTIO/IO): {:.4}",
            result.ratio
        );
        log::info!(
            "  Matched pairs: {} (A→B: {:.2}%, B→A: {:.2}%)",
            result.matched_pairs,
            result.match_ratio_a * 100.0,
            result.match_ratio_b * 100.0
        );
        log::info!(
            "  Unique to IO: {}, Unique to FASTIO: {}",
            result.unique_to_a,
            result.unique_to_b
        );
        log::info!(
            "  MajorFunction distributions match: {}",
            result.distribution_match
        );
        if !result.distribution_match {
            log::info!("    IO dist:      {:?}", result.major_func_dist_a);
            log::info!("    FASTIO dist:  {:?}", result.major_func_dist_b);
        }

        all_passes.push(result);

        if pass_num < NUM_PASSES {
            log::info!(
                "  Pausing {}s before next pass...",
                PAUSE_BETWEEN_PASSES_SECS
            );
            std::thread::sleep(Duration::from_secs(PAUSE_BETWEEN_PASSES_SECS));
        }
    }

    // Analyze all passes
    log::info!("");
    log::info!("=== ANALYSIS ===");
    let verdict = analysis::analyze_passes(&all_passes);
    display_verdict(&verdict);

    // Save results to file
    save_results(&verdict);
}

/// Run a single pass: start both sessions simultaneously, collect events.
fn run_pass(pass_num: usize) -> (Vec<RawEvent>, Vec<RawEvent>) {
    let events_io: Arc<Mutex<Vec<RawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_fastio: Arc<Mutex<Vec<RawEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let io_events = events_io.clone();
    let fastio_events = events_fastio.clone();

    // Session names must be unique across passes
    let io_name = format!("FltCmp_IO_P{}", pass_num);
    let fastio_name = format!("FltCmp_FIO_P{}", pass_num);

    // Create sessions
    let io_config = TraceConfig {
        session_name: io_name.clone(),
        group_mask: build_group_mask(PERF_FLT_IO),
    };
    let fastio_config = TraceConfig {
        session_name: fastio_name.clone(),
        group_mask: build_group_mask(PERF_FLT_FASTIO),
    };

    let mut io_session = match KernelTraceSession::new(io_config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create IO session: {:?}", e);
            return (Vec::new(), Vec::new());
        }
    };
    let mut fastio_session = match KernelTraceSession::new(fastio_config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create FASTIO session: {:?}", e);
            return (Vec::new(), Vec::new());
        }
    };

    // Start both sessions
    let io_handle = match io_session.start(io_events) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start IO session: {:?}", e);
            return (Vec::new(), Vec::new());
        }
    };
    let fastio_handle = match fastio_session.start(fastio_events) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start FASTIO session: {:?}", e);
            return (Vec::new(), Vec::new());
        }
    };

    // Spawn processing threads
    let io_thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(io_handle);
    });
    let fastio_thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ =
            <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(fastio_handle);
    });

    // Warm up
    std::thread::sleep(Duration::from_millis(SESSION_WARMUP_MS));

    // Trigger file I/O to generate FltIoCompletion events
    log::info!("  Triggering file I/O operations...");
    trigger_io();

    // Wait for events to arrive
    log::info!(
        "  Collecting events ({}s)...",
        COLLECTION_SECS
    );
    std::thread::sleep(Duration::from_secs(COLLECTION_SECS));

    // Stop both sessions
    log::info!("  Stopping sessions...");
    let _ = io_session.stop();
    let _ = fastio_session.stop();

    // Wait for processing threads
    let _ = io_thread.join();
    let _ = fastio_thread.join();

    let io_result = events_io.lock().unwrap().clone();
    let fastio_result = events_fastio.lock().unwrap().clone();

    (io_result, fastio_result)
}

/// Trigger file I/O operations to generate FltIoCompletion events.
fn trigger_io() {
    let test_dir = Path::new("C:\\temp_flt_compare");

    let _ = fs::create_dir_all(test_dir);

    // Create files
    for i in 0..10 {
        let path = test_dir.join(format!("test_{}.dat", i));
        let _ = fs::write(&path, format!("data {}", i));
    }

    // Read files
    for i in 0..10 {
        let path = test_dir.join(format!("test_{}.dat", i));
        let _ = fs::read(&path);
    }

    // Write files
    for i in 0..10 {
        let path = test_dir.join(format!("test_{}.dat", i));
        let _ = fs::write(&path, format!("updated {}", i));
    }

    // Flush
    for i in 0..5 {
        let path = test_dir.join(format!("test_{}.dat", i));
        if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&path) {
            use std::io::Write;
            let _ = file.flush();
        }
    }

    // Delete files
    for i in 0..10 {
        let path = test_dir.join(format!("test_{}.dat", i));
        let _ = fs::remove_file(&path);
    }

    let _ = fs::remove_dir_all(test_dir);
}

/// Display the final verdict
fn display_verdict(verdict: &analysis::AnalysisVerdict) {
    log::info!("Number of passes: {}", verdict.num_passes);
    log::info!(
        "Mean ratio (FASTIO/IO): {:.4}",
        verdict.mean_ratio
    );
    log::info!(
        "Ratio consistency: {:.2}%",
        verdict.ratio_consistency * 100.0
    );
    log::info!(
        "Distribution match rate: {:.2}%",
        verdict.distribution_match_rate * 100.0
    );
    log::info!(
        "Mean match ratio A→B (IO→FASTIO): {:.2}%",
        verdict.mean_match_ratio_a * 100.0
    );
    log::info!(
        "Mean match ratio B→A (FASTIO→IO): {:.2}%",
        verdict.mean_match_ratio_b * 100.0
    );
    log::info!(
        "Total unique to IO (across all passes): {}",
        verdict.total_unique_to_a
    );
    log::info!(
        "Total unique to FASTIO (across all passes): {}",
        verdict.total_unique_to_b
    );
    log::info!("");

    log::info!("--- Heuristic Scores ---");
    let mut sorted_scores: Vec<_> = verdict.heuristic_scores.iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    for (hypothesis, score) in &sorted_scores {
        log::info!("  {:<25} {:.1}%", hypothesis, score);
    }

    log::info!("");
    log::info!(
        ">>> BEST HYPOTHESIS: {} ({:.1}% confidence)",
        verdict.best_hypothesis.0,
        verdict.best_hypothesis.1
    );

    // Interpretation
    log::info!("");
    log::info!("--- Interpretation ---");
    match verdict.best_hypothesis.0.as_str() {
        "FASTIO ⊂ IO" => {
            log::info!("PERF_FLT_FASTIO events are a strict subset of PERF_FLT_IO events.");
            log::info!("PERF_FLT_IO captures all FltIoCompletion events that PERF_FLT_FASTIO captures, plus additional ones.");
        }
        "Same events" => {
            log::info!("PERF_FLT_FASTIO and PERF_FLT_IO enable the exact same FltIoCompletion events.");
            log::info!("The difference in observed counts is due to ETW timing/buffer noise.");
        }
        "Exclusive (no overlap)" => {
            log::info!("PERF_FLT_FASTIO and PERF_FLT_IO enable completely different sets of FltIoCompletion events.");
        }
        "Partial overlap" => {
            log::info!("PERF_FLT_FASTIO and PERF_FLT_IO have partially overlapping but non-identical FltIoCompletion event sets.");
        }
        _ => {
            log::info!("Unable to determine a clear relationship.");
        }
    }
}

/// Save results to a JSON file
fn save_results(verdict: &analysis::AnalysisVerdict) {
    let output = serde_json::json!({
        "summary": {
            "num_passes": verdict.num_passes,
            "mean_ratio": verdict.mean_ratio,
            "ratio_consistency": verdict.ratio_consistency,
            "distribution_match_rate": verdict.distribution_match_rate,
            "mean_match_ratio_a": verdict.mean_match_ratio_a,
            "mean_match_ratio_b": verdict.mean_match_ratio_b,
            "total_unique_to_io": verdict.total_unique_to_a,
            "total_unique_to_fastio": verdict.total_unique_to_b,
            "best_hypothesis": verdict.best_hypothesis.0,
            "confidence": verdict.best_hypothesis.1,
        },
        "heuristic_scores": verdict.heuristic_scores,
        "passes": verdict.passes.iter().enumerate().map(|(i, p)| {
            serde_json::json!({
                "pass": i + 1,
                "count_io": p.count_a,
                "count_fastio": p.count_b,
                "ratio": p.ratio,
                "matched_pairs": p.matched_pairs,
                "match_ratio_a": p.match_ratio_a,
                "match_ratio_b": p.match_ratio_b,
                "distribution_match": p.distribution_match,
                "unique_to_io": p.unique_to_a,
                "unique_to_fastio": p.unique_to_b,
            })
        }).collect::<Vec<_>>(),
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
