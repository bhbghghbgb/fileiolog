//! Orchestrator for the PERF_FLT_IO vs PERF_FLT_FASTIO experiment.
//!
//!  1. Sequential baseline passes (one flag at a time) showing which
//!     opcodes/versions each flag produces in isolation.
//!  2. Concurrent comparison passes: two sessions, one with `PERF_FLT_IO`, one
//!     with `PERF_FLT_FASTIO`, both listening to the same live kernel event
//!     stream while a shared workload runs.
//!  3. A control pass (IO vs IO) that calibrates how much cross-session overlap
//!     to expect under the *same* flag (start skew + buffer loss).
//!
//! All four hypotheses are scored 0-100% and the heuristics that pass/fail for
//! each answer are reported, both to the console and to files.

mod compare;
mod file_ops;
mod flt_events;
mod logger;
mod session;

use std::fs;
use std::path::{Path, PathBuf};

use log::LevelFilter;

use compare::{AnswerScore, MatchStats, OpcodeCount, aggregate, score_answers};
use session::{PERF_FLT_FASTIO, PERF_FLT_IO, run_dual, run_single};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut passes = 3usize;
    let mut baseline_passes = 1usize;
    let mut run_control = true;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--passes" => { if i + 1 < args.len() { passes = args[i+1].parse().unwrap_or(3); i += 1; } }
            "--baseline" => { if i + 1 < args.len() { baseline_passes = args[i+1].parse().unwrap_or(1); i += 1; } }
            "--no-control" => run_control = false,
            _ => {}
        }
        i += 1;
    }

    let out_dir = make_out_dir();
    logger::init(&out_dir, LevelFilter::Info);

    log::info!("=== PERF_FLT_IO (0x80100000) vs PERF_FLT_FASTIO (0x80200000) ===");
    log::info!(
        "compare passes: {} | baseline passes: {} | control: {}",
        passes, baseline_passes, run_control
    );
    log::info!("output directory: {}", out_dir.display());
    log::info!("");

    // ── 1. Baseline (isolated single-flag sessions) ───────────────────────
    if baseline_passes > 0 {
        log::info!("── BASELINE: each flag in isolation ────────────────────────");
        for p in 0..baseline_passes {
            log::info!("  [baseline {}/{}] PERF_FLT_IO only ...", p + 1, baseline_passes);
            let io = run_single(&format!("FltCmp-Bio-{}-{}", p, std::process::id()), PERF_FLT_IO, true);
            let fast = run_single(&format!("FltCmp-Bfast-{}-{}", p, std::process::id()), PERF_FLT_FASTIO, true);

            save_events(&out_dir, &format!("baseline-io-{}.json", p), &io);
            save_events(&out_dir, &format!("baseline-fast-{}.json", p), &fast);

            log::info!("  ── Io-only opcodes ──");
            print_opcodes(&io, "IO  ");
            log::info!("  ── FastIo-only opcodes ──");
            print_opcodes(&fast, "FAST");
            log::info!("");
        }
    }

    // ── 2. Concurrent compare passes ─────────────────────────────────────
    let mut compare_passes: Vec<MatchStats> = Vec::new();
    let mut control_cov: f64 = 0.9;

    log::info!("=== CONCURRENT COMPARE (IO vs FASTIO, one workload) ===");
    for p in 0..passes {
        let name_a = format!("FltCmp-A-{}-{}", p, std::process::id());
        let name_b = format!("FltCmp-B-{}-{}", p, std::process::id());
        log::info!("");
        log::info!("  ── Pass {}/{}: IO ({}) vs FASTIO ({}) ──", p + 1, passes, name_a, name_b);

        let (io_ev, fast_ev) = run_dual(&name_a, PERF_FLT_IO, &name_b, PERF_FLT_FASTIO, true);
        save_events(&out_dir, &format!("pass-{}-io.json", p), &io_ev);
        save_events(&out_dir, &format!("pass-{}-fast.json", p), &fast_ev);

        let stats = compare::compare(&fast_ev, &io_ev, &format!("pass-{}", p + 1));
        compare_passes.push(stats.clone());
        print_stats_table(&stats);
    }

    // ── 3. Control pass (IO vs IO) to calibrate coverage ─────────────────
    if run_control {
        log::info!("");
        log::info!("=== CONTROL: IO vs IO (calibration) ===");
        let name_a = format!("FltCtl-A-{}", std::process::id());
        let name_b = format!("FltCtl-B-{}", std::process::id());
        let (io1, io2) = run_dual(&name_a, PERF_FLT_IO, &name_b, PERF_FLT_IO, true);
        save_events(&out_dir, "control-io1.json", &io1);
        save_events(&out_dir, "control-io2.json", &io2);
        let c_stats = compare::compare(&io2, &io1, "control");
        if c_stats.fast_total > 0 {
            control_cov = c_stats.matched as f64 / c_stats.fast_total as f64;
        }
        log::info!("  Control symmetric coverage: {:.3}", control_cov);
        print_stats_table(&c_stats);
        log::info!("");
    }

    // ── 4. Aggregate + verdict ───────────────────────────────────────────
    let agg = aggregate(&compare_passes);
    log::info!("");
    log::info!("============ FINAL VERDICT ============");
    print_stats_table(&agg);

    let scores = score_answers(&agg, control_cov);
    let best = scores
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    log::info!("");
    log::info!("Calibrated symmetric coverage (control): {:.3}", control_cov);
    log::info!("");
    log::info!("Answer compatibility scores:");
    for s in &scores {
        let badge = if s.number == best.number { "   <-- BEST" } else { "" };
        log::info!(
            "  [{}] {:.1}%  {}{}",
            s.number,
            s.score * 100.0,
            s.label,
            badge
        );
        if !s.passed.is_empty() {
            log::info!("         passed: {}", s.passed.join(", "));
        }
        if !s.failed.is_empty() {
            log::info!("         failed: {}", s.failed.join(", "));
        }
    }
    log::info!("");
    log::info!("CONCLUSION: {}", best.label);
    log::info!("Confidence: {:.1}%", best.score * 100.0);

    write_verdict(&out_dir, &agg, &scores, best, control_cov);
    log::info!("Done. See {} for report + session logs.", out_dir.display());
}

fn make_out_dir() -> PathBuf {
    let dir = PathBuf::from("output")
        .join(format!("run-{}", std::process::id()));
    fs::create_dir_all(&dir).ok();
    dir
}

fn save_events(dir: &Path, name: &str, events: &[session::CapturedEvent]) {
    match serde_json::to_string_pretty(events) {
        Ok(json) => {
            if let Err(e) = fs::write(dir.join(name), json) {
                log::warn!("failed to write {}: {}", name, e);
            }
        }
        Err(e) => log::warn!("failed to serialize {}: {}", name, e),
    }
}

fn opcode_label(opcode: u8) -> &'static str {
    match opcode {
        96 => "PreOpInit",
        97 => "PostOpInit",
        98 => "PreOpCompletion",
        99 => "PostOpCompletion",
        100 => "PreOpFailure",
        101 => "PostOpFailure",
        _ => "?(FLT?)",
    }
}

fn print_opcodes(events: &[session::CapturedEvent], tag: &str) {
    let mut map: std::collections::HashMap<(u8, u8), usize> = std::collections::HashMap::new();
    for e in events {
        *map.entry((e.opcode, e.version)).or_insert(0) += 1;
    }
    let mut v: Vec<_> = map.into_iter().collect();
    v.sort_by_key(|((op, ver), _)| (*op, *ver));
    if v.is_empty() {
        log::info!("    {} (none)", tag);
    }
    for ((op, ver), count) in v {
        log::info!("    {} opcode={:>3} ver={}  {}: {}", tag, op, ver, opcode_label(op), count);
    }
}

fn print_stats_table(s: &MatchStats) {
    let pc = |f: f64| format!("{:.3}", f);
    log::info!("  ── {} ──", s.name);
    log::info!("  io_total={} fast_total={} matched={}", s.io_total, s.fast_total, s.matched);
    log::info!(
        "  fast_to_io={} io_to_fast={} jaccard={} count_ratio(fast/io)={}",
        pc(s.fast_to_io), pc(s.io_to_fast), pc(s.jaccard), pc(s.count_ratio)
    );
    log::info!("  fast_only={} io_only={} opcodes_identical={}", s.fast_only, s.io_only, s.opcodes_identical);
    log::info!("  io opcodes   : {}", format_oc(&s.io_opcodes));
    log::info!("  fast opcodes : {}", format_oc(&s.fast_opcodes));
}

fn format_oc(list: &[OpcodeCount]) -> String {
    if list.is_empty() {
        return "(none)".to_string();
    }
    list.iter()
        .map(|o| format!("{}(/op{},ver{})={}", opcode_label(o.opcode), o.opcode, o.version, o.count))
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_verdict(
    out_dir: &Path,
    agg: &MatchStats,
    scores: &[AnswerScore],
    best: &AnswerScore,
    control_cov: f64,
) {
    let mut text = String::from("PERF_FLT_IO (0x80100000) vs PERF_FLT_FASTIO (0x80200000)\n");
    text.push_str(&format!(
        "io_total={} fast_total={} matched={}\n",
        agg.io_total, agg.fast_total, agg.matched
    ));
    text.push_str(&format!(
        "fast_to_io={:.3} io_to_fast={:.3} jaccard={:.3} count_ratio={:.3}\n",
        agg.fast_to_io, agg.io_to_fast, agg.jaccard, agg.count_ratio
    ));
    text.push_str(&format!(
        "fast_only={} io_only={} opcodes_identical={}\n",
        agg.fast_only, agg.io_only, agg.opcodes_identical
    ));
    text.push_str(&format!("control symmetric coverage={:.3}\n", control_cov));
    text.push_str("\nScores:\n");
    for s in scores {
        text.push_str(&format!("  [{}] {:.1}%  {}\n", s.number, s.score * 100.0, s.label));
        if !s.passed.is_empty() {
            text.push_str(&format!("     passed: {}\n", s.passed.join(", ")));
        }
        if !s.failed.is_empty() {
            text.push_str(&format!("     failed: {}\n", s.failed.join(", ")));
        }
    }
    text.push_str(&format!("\nCONCLUSION: {}\n", best.label));
    text.push_str(&format!("Confidence: {:.1}%\n", best.score * 100.0));
    if let Err(e) = fs::write(out_dir.join("verdict.txt"), text) {
        log::warn!("failed writing verdict.txt: {}", e);
    }
}