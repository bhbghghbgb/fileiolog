use std::sync::{Arc, Mutex};
use std::time::Duration;

use fileio_trace_test::build_group_mask;
use fileio_trace_test::events::{FileIoRawEvent, ParsedFileIoEvent};
use fileio_trace_test::file_ops;
use fileio_trace_test::trace_session::{KernelTraceSession, TraceConfig};

const PERF_FLT_IO: u32 = 0x80100000;
const PERF_FLT_FASTIO: u32 = 0x80200000;

/// FltIoCompletion opcodes
const OPCODE_PRE_OP_COMPLETION: u8 = 98;
const OPCODE_POST_OP_COMPLETION: u8 = 99;

/// Timestamp matching tolerance in nanoseconds (100 microseconds)
const TIMESTAMP_TOLERANCE_NS: u64 = 100_000;

const NUM_RUNS: usize = 3;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== PERF_FLT_FASTIO vs PERF_FLT_IO Comparison Test ===");
    log::info!("Comparing FltIoCompletion events (opcodes 98/99) under two flags:");
    log::info!("  PERF_FLT_FASTIO = 0x{:08X}", PERF_FLT_FASTIO);
    log::info!("  PERF_FLT_IO     = 0x{:08X}", PERF_FLT_IO);
    log::info!("Running {} iterations...\n", NUM_RUNS);

    let mut run_results: Vec<RunResult> = Vec::new();

    for run in 0..NUM_RUNS {
        log::info!("--- Run {}/{} ---", run + 1, NUM_RUNS);
        let result = run_comparison();
        log::info!(
            "  FASTIO events: {}, FLT_IO events: {}, matched: {}",
            result.count_fastio,
            result.count_flt_io,
            result.matched_count
        );
        run_results.push(result);

        if run < NUM_RUNS - 1 {
            log::info!("  Pausing 2s before next run...\n");
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    log::info!("\n=== AGGREGATE RESULTS ===");
    display_aggregate(&run_results);
}

struct RunResult {
    count_fastio: usize,
    count_flt_io: usize,
    /// Unique FltIoCompletion events seen by FASTIO only
    unique_to_fastio: usize,
    /// Unique FltIoCompletion events seen by FLT_IO only
    unique_to_flt_io: usize,
    /// Events matched across both sessions (same timestamp within tolerance)
    matched_count: usize,
    /// Count of PreOpCompletion (opcode 98) in each
    pre_count_fastio: usize,
    pre_count_flt_io: usize,
    /// Count of PostOpCompletion (opcode 99) in each
    post_count_fastio: usize,
    post_count_flt_io: usize,
}

fn run_comparison() -> RunResult {
    let events_a: Arc<Mutex<Vec<FileIoRawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_b: Arc<Mutex<Vec<FileIoRawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let parsed_a: Arc<Mutex<Vec<ParsedFileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let parsed_b: Arc<Mutex<Vec<ParsedFileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let config_a = TraceConfig {
        session_name: "FlagCompare-FASTIO".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(PERF_FLT_FASTIO)),
    };
    let config_b = TraceConfig {
        session_name: "FlagCompare-FLTIO".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(PERF_FLT_IO)),
    };

    let mut session_a = KernelTraceSession::new(config_a).expect("Failed to create session A");
    let mut session_b = KernelTraceSession::new(config_b).expect("Failed to create session B");

    let handle_a = session_a
        .start(events_a.clone(), parsed_a.clone())
        .expect("Failed to start session A");
    let handle_b = session_b
        .start(events_b.clone(), parsed_b.clone())
        .expect("Failed to start session B");

    let thread_a = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(handle_a);
    });
    let thread_b = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(handle_b);
    });

    std::thread::sleep(Duration::from_millis(500));

    log::info!("  Triggering file operations...");
    file_ops::trigger_all_file_operations();

    log::info!("  Waiting for events (5s)...");
    std::thread::sleep(Duration::from_secs(5));

    let _ = session_a.request_rundown();
    let _ = session_b.request_rundown();
    std::thread::sleep(Duration::from_secs(2));

    log::info!("  Stopping sessions...");
    let _ = session_a.stop();
    let _ = session_b.stop();
    let _ = thread_a.join();
    let _ = thread_b.join();

    let raw_a = events_a.lock().unwrap().clone();
    let raw_b = events_b.lock().unwrap().clone();

    analyze_results(&raw_a, &raw_b)
}

fn analyze_results(raw_a: &[FileIoRawEvent], raw_b: &[FileIoRawEvent]) -> RunResult {
    // Filter FltIoCompletion events
    let flt_a: Vec<&FileIoRawEvent> = raw_a
        .iter()
        .filter(|e| e.opcode == OPCODE_PRE_OP_COMPLETION || e.opcode == OPCODE_POST_OP_COMPLETION)
        .collect();
    let flt_b: Vec<&FileIoRawEvent> = raw_b
        .iter()
        .filter(|e| e.opcode == OPCODE_PRE_OP_COMPLETION || e.opcode == OPCODE_POST_OP_COMPLETION)
        .collect();

    let count_fastio = flt_a.len();
    let count_flt_io = flt_b.len();

    let pre_count_fastio = flt_a.iter().filter(|e| e.opcode == OPCODE_PRE_OP_COMPLETION).count();
    let pre_count_flt_io = flt_b.iter().filter(|e| e.opcode == OPCODE_PRE_OP_COMPLETION).count();
    let post_count_fastio = flt_a.iter().filter(|e| e.opcode == OPCODE_POST_OP_COMPLETION).count();
    let post_count_flt_io = flt_b.iter().filter(|e| e.opcode == OPCODE_POST_OP_COMPLETION).count();

    // Timestamp matching: for each event in A, find closest event in B with same opcode
    let mut matched = 0usize;
    let mut matched_a_flags = vec![false; flt_a.len()];
    let mut matched_b_flags = vec![false; flt_b.len()];

    for (i, ea) in flt_a.iter().enumerate() {
        for (j, eb) in flt_b.iter().enumerate() {
            if matched_b_flags[j] {
                continue;
            }
            if ea.opcode != eb.opcode {
                continue;
            }
            let diff = ea.timestamp.abs_diff(eb.timestamp);
            if diff <= TIMESTAMP_TOLERANCE_NS {
                matched += 1;
                matched_a_flags[i] = true;
                matched_b_flags[j] = true;
                break;
            }
        }
    }

    let unique_to_fastio = matched_a_flags.iter().filter(|&&m| !m).count();
    let unique_to_flt_io = matched_b_flags.iter().filter(|&&m| !m).count();

    RunResult {
        count_fastio,
        count_flt_io,
        unique_to_fastio,
        unique_to_flt_io,
        matched_count: matched,
        pre_count_fastio,
        pre_count_flt_io,
        post_count_fastio,
        post_count_flt_io,
    }
}

fn display_aggregate(results: &[RunResult]) {
    let total_fastio: usize = results.iter().map(|r| r.count_fastio).sum();
    let total_flt_io: usize = results.iter().map(|r| r.count_flt_io).sum();
    let total_matched: usize = results.iter().map(|r| r.matched_count).sum();
    let total_unique_fastio: usize = results.iter().map(|r| r.unique_to_fastio).sum();
    let total_unique_flt_io: usize = results.iter().map(|r| r.unique_to_flt_io).sum();

    log::info!("Total FltIoCompletion events:");
    log::info!("  PERF_FLT_FASTIO: {}", total_fastio);
    log::info!("  PERF_FLT_IO:     {}", total_flt_io);
    log::info!("  Matched (both):  {}", total_matched);
    log::info!("  FASTIO only:     {}", total_unique_fastio);
    log::info!("  FLT_IO only:     {}", total_unique_flt_io);
    log::info!("");

    // Per-opcode breakdown
    let pre_fastio: usize = results.iter().map(|r| r.pre_count_fastio).sum();
    let pre_flt_io: usize = results.iter().map(|r| r.pre_count_flt_io).sum();
    let post_fastio: usize = results.iter().map(|r| r.post_count_fastio).sum();
    let post_flt_io: usize = results.iter().map(|r| r.post_count_flt_io).sum();
    log::info!("Per-opcode breakdown:");
    log::info!("  PreOpCompletion (98):  FASTIO={}, FLT_IO={}", pre_fastio, pre_flt_io);
    log::info!("  PostOpCompletion (99): FASTIO={}, FLT_IO={}", post_fastio, post_flt_io);
    log::info!("");

    // Score hypotheses
    log::info!("=== HYPOTHESIS SCORING ===");
    log::info!("");

    // Hypothesis 1: PERF_FLT_FASTIO ⊂ PERF_FLT_IO
    // Evidence: every FASTIO event should also appear in FLT_IO
    let score_fastio_subset = if total_fastio == 0 {
        100.0 // vacuously true
    } else {
        (total_matched as f64 / total_fastio as f64) * 100.0
    };
    log::info!(
        "H1: PERF_FLT_FASTIO ⊂ PERF_FLT_IO  (all FASTIO events also in FLT_IO)"
    );
    log::info!(
        "    Score: {:.1}%  (matched {}/{} FASTIO events)",
        score_fastio_subset,
        total_matched,
        total_fastio
    );
    log::info!("    => {}", if score_fastio_subset > 95.0 { "SUPPORTED" } else if score_fastio_subset > 50.0 { "WEAK" } else { "REFUTED" });
    log::info!("");

    // Hypothesis 2: PERF_FLT_IO ⊂ PERF_FLT_FASTIO
    // Evidence: every FLT_IO event should also appear in FASTIO
    let score_fltio_subset = if total_flt_io == 0 {
        100.0
    } else {
        (total_matched as f64 / total_flt_io as f64) * 100.0
    };
    log::info!(
        "H2: PERF_FLT_IO ⊂ PERF_FLT_FASTIO  (all FLT_IO events also in FASTIO)"
    );
    log::info!(
        "    Score: {:.1}%  (matched {}/{} FLT_IO events)",
        score_fltio_subset,
        total_matched,
        total_flt_io
    );
    log::info!("    => {}", if score_fltio_subset > 95.0 { "SUPPORTED" } else if score_fltio_subset > 50.0 { "WEAK" } else { "REFUTED" });
    log::info!("");

    // Hypothesis 3: Exclusive (no overlap)
    // Evidence: no matched events
    let score_exclusive = if total_fastio + total_flt_io == 0 {
        0.0 // can't determine
    } else {
        let overlap_ratio = total_matched as f64 / (total_fastio + total_flt_io) as f64;
        (1.0 - overlap_ratio) * 100.0
    };
    log::info!(
        "H3: Exclusive  (no overlap between the two flags)"
    );
    log::info!(
        "    Score: {:.1}%  (overlap ratio: {:.1}%)",
        score_exclusive,
        if total_fastio + total_flt_io > 0 {
            total_matched as f64 / (total_fastio + total_flt_io) as f64 * 100.0
        } else {
            0.0
        }
    );
    log::info!("    => {}", if score_exclusive > 95.0 { "SUPPORTED" } else if score_exclusive > 50.0 { "WEAK" } else { "REFUTED" });
    log::info!("");

    // Hypothesis 4: Collisions (partial overlap, neither is strict subset)
    // Evidence: some overlap exists, but neither H1 nor H2 score is >95%
    let score_collisions = if score_fastio_subset > 95.0 || score_fltio_subset > 95.0 {
        0.0 // one is subset, so not "collisions only"
    } else {
        let overlap_ratio = if total_fastio + total_flt_io > 0 {
            total_matched as f64 / (total_fastio + total_flt_io) as f64
        } else {
            0.0
        };
        overlap_ratio * 100.0 // higher overlap = more collisions
    };
    log::info!(
        "H4: Collisions  (partial overlap, neither is a strict subset)"
    );
    log::info!(
        "    Score: {:.1}%  (overlap exists but neither is a full subset)",
        score_collisions
    );
    log::info!("    => {}", if score_collisions > 50.0 { "SUPPORTED" } else { "REFUTED" });
    log::info!("");

    // Final verdict
    log::info!("=== VERDICT ===");
    let scores = [
        ("H1: FASTIO ⊂ FLT_IO", score_fastio_subset),
        ("H2: FLT_IO ⊂ FASTIO", score_fltio_subset),
        ("H3: Exclusive", score_exclusive),
        ("H4: Collisions", score_collisions),
    ];
    let best = scores.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    log::info!(
        "Best supported: {} ({:.1}%)",
        best.0,
        best.1
    );

    if total_fastio == 0 && total_flt_io == 0 {
        log::warn!("WARNING: No FltIoCompletion events received in either session.");
        log::warn!("This may indicate that neither flag enables these events, or the");
        log::warn!("system has no active minifilter operations during the test window.");
    }
}
