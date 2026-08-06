//! Correlation between concurrently-run sessions and derivation of the
//! relationship between `PERF_FLT_IO` and `PERF_FLT_FASTIO`.
//!
//! Because both sessions watch the *same* live kernel event stream, the same
//! underlying event produces a record in each session that shares an identity
//! key: opcode, version, raw kernel timestamp, pid, tid, and payload signature.
//! We therefore match the two event multisets on that key.
//!
//! Interpretation of the coverage measures:
//!   * `fast_to_io = matched / fast_total` – fraction of FAST events that IO
//!     also emits. Near 1.0 means FAST ⊆ IO.
//!   * `io_to_fast = matched / io_total`   – fraction of IO events that FAST
//!     also emits. Well below 1.0 means IO has extra events.
//!   * FAST ⊂ IO  ⇒ fast_to_io ≈ 1.0 but io_to_fast < 1.0   (asymmetric).
//!   * identical   ⇒ both ≈ 1.0 and counts match.
//!   * exclusive   ⇒ both ≈ 0 (and/or disjoint opcode sets).

use std::collections::HashMap;

use crate::session::CapturedEvent;

#[derive(Debug, Clone)]
pub struct OpcodeCount {
    pub opcode: u8,
    pub version: u8,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct MatchStats {
    pub name: String,
    pub fast_total: usize,
    pub io_total: usize,
    pub matched: usize,
    pub fast_only: usize,
    pub io_only: usize,
    pub fast_to_io: f64,
    pub io_to_fast: f64,
    pub jaccard: f64,
    /// fast_total / io_total.
    pub count_ratio: f64,
    pub fast_opcodes: Vec<OpcodeCount>,
    pub io_opcodes: Vec<OpcodeCount>,
    pub opcodes_identical: bool,
}

type MatchKey = (u8, u8, u64, u32, u32, u64); // op, ver, ts, pid, tid, sig

fn event_key(e: &CapturedEvent) -> MatchKey {
    (e.opcode, e.version, e.timestamp, e.pid, e.tid, e.sig)
}

fn opcode_counts(evs: &[CapturedEvent]) -> Vec<OpcodeCount> {
    let mut map: HashMap<(u8, u8), usize> = HashMap::new();
    for e in evs {
        *map.entry((e.opcode, e.version)).or_insert(0) += 1;
    }
    let mut v: Vec<OpcodeCount> = map
        .into_iter()
        .map(|((op, ver), count)| OpcodeCount { opcode: op, version: ver, count })
        .collect();
    v.sort_by_key(|o| (o.opcode, o.version));
    v
}

/// Trim both event lists to the common time window so that session-start skew
/// and teardown do not produce asymmetric edge effects.
fn trim_window(
    fast: &[CapturedEvent],
    io: &[CapturedEvent],
) -> (Vec<CapturedEvent>, Vec<CapturedEvent>) {
    let (Some(t0a), Some(t1a)) = (
        fast.iter().map(|e| e.timestamp).min(),
        fast.iter().map(|e| e.timestamp).max(),
    ) else {
        return (Vec::new(), Vec::new());
    };
    let (Some(t0b), Some(t1b)) = (
        io.iter().map(|e| e.timestamp).min(),
        io.iter().map(|e| e.timestamp).max(),
    ) else {
        return (Vec::new(), Vec::new());
    };
    let lo = t0a.max(t0b);
    let hi = t1a.min(t1b);
    (
        fast.iter()
            .filter(|e| e.timestamp >= lo && e.timestamp <= hi)
            .cloned()
            .collect(),
        io.iter()
            .filter(|e| e.timestamp >= lo && e.timestamp <= hi)
            .cloned()
            .collect(),
    )
}

fn make_multiset(evs: &[CapturedEvent]) -> HashMap<MatchKey, usize> {
    let mut m: HashMap<MatchKey, usize> = HashMap::new();
    for e in evs {
        *m.entry(event_key(e)).or_insert(0) += 1;
    }
    m
}

/// Compare two concurrently collected sessions (window-trimmed).
/// The first argument is treated as the "FAST" side, the second as "IO".
pub fn compare(fast: &[CapturedEvent], io: &[CapturedEvent], name: &str) -> MatchStats {
    let (fast, io) = trim_window(fast, io);
    let fa = make_multiset(&fast);
    let ib = make_multiset(&io);

    let mut matched = 0usize;
    let mut fast_only = 0usize;
    let mut io_only = 0usize;
    for (k, ca) in &fa {
        let cb = ib.get(k).copied().unwrap_or(0);
        matched += *ca.min(&cb);
        if *ca > cb {
            fast_only += ca - cb;
        }
    }
    for (k, cb) in &ib {
        let ca = fa.get(k).copied().unwrap_or(0);
        if *cb > ca {
            io_only += cb - ca;
        }
    }

    let fast_total = fast.len();
    let io_total = io.len();
    let fast_to_io = if fast_total > 0 { matched as f64 / fast_total as f64 } else { 0.0 };
    let io_to_fast = if io_total > 0 { matched as f64 / io_total as f64 } else { 0.0 };
    let jaccard = if fast_total + io_total - matched > 0 {
        matched as f64 / (fast_total as f64 + io_total as f64 - matched as f64)
    } else {
        0.0
    };
    let count_ratio = if io_total > 0 { fast_total as f64 / io_total as f64 } else { 0.0 };

    let fast_opcodes = opcode_counts(&fast);
    let io_opcodes = opcode_counts(&io);
    let fset: std::collections::HashSet<(u8, u8)> =
        fast_opcodes.iter().map(|o| (o.opcode, o.version)).collect();
    let iset: std::collections::HashSet<(u8, u8)> =
        io_opcodes.iter().map(|o| (o.opcode, o.version)).collect();
    let opcodes_identical = !fset.is_empty() && fset == iset;

    MatchStats {
        name: name.to_string(),
        fast_total,
        io_total,
        matched,
        fast_only,
        io_only,
        fast_to_io,
        io_to_fast,
        jaccard,
        count_ratio,
        fast_opcodes,
        io_opcodes,
        opcodes_identical,
    }
}

#[derive(Debug, Clone)]
pub struct HeuristicResult {
    pub name: &'static str,
    pub value: f64,
    pub passed: bool,
    pub supports: Vec<u8>,
    pub contradicts: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AnswerScore {
    pub number: u8,
    pub label: &'static str,
    pub score: f64,
    pub passed: Vec<&'static str>,
    pub failed: Vec<&'static str>,
}

const COV_LOW: f64 = 0.10;
const COV_FLOOR: f64 = 0.10;

/// Threshold for "strong coverage", calibrated against a control run of the
/// same flag on both sides (which absorbs start-skew / buffer-loss).
fn cov_high(control_cov: f64) -> f64 {
    0.9_f64.max(control_cov * 0.90)
}

pub fn evaluate(stats: &MatchStats, control_cov: f64) -> Vec<HeuristicResult> {
    let high = cov_high(control_cov);
    let fast_cov = stats.fast_to_io;
    let io_cov = stats.io_to_fast;
    let cr = stats.count_ratio;

    let mut out = Vec::new();

    out.push(HeuristicResult {
        name: "opcodes identical",
        value: if stats.opcodes_identical { 1.0 } else { 0.0 },
        passed: stats.opcodes_identical,
        supports: vec![4, 1],
        contradicts: vec![2],
    });

    out.push(HeuristicResult {
        name: "fast covered by io",
        value: fast_cov,
        passed: fast_cov >= high,
        supports: vec![1, 4],
        contradicts: vec![2, 3],
    });

    out.push(HeuristicResult {
        name: "io covered by fast",
        value: io_cov,
        passed: io_cov >= high,
        supports: vec![4],
        contradicts: vec![1, 2, 3],
    });

    out.push(HeuristicResult {
        name: "io larger",
        value: cr,
        passed: cr <= 0.66,
        supports: vec![1],
        contradicts: vec![4],
    });

    let sym = fast_cov.min(io_cov);
    out.push(HeuristicResult {
        name: "symmetric",
        value: sym,
        passed: fast_cov >= high && io_cov >= high,
        supports: vec![4],
        contradicts: vec![1, 2, 3],
    });

    let excl = (1.0 - fast_cov).max(0.0)
        * (1.0 - io_cov).max(0.0)
        * if stats.opcodes_identical { 0.0 } else { 1.0 };
    out.push(HeuristicResult {
        name: "exclusive",
        value: excl,
        passed: fast_cov <= COV_FLOOR && io_cov <= COV_FLOOR,
        supports: vec![2],
        contradicts: vec![1, 3, 4],
    });

    let partial = 1.0 - (2.0 * fast_cov - 1.0).abs();
    out.push(HeuristicResult {
        name: "partial overlap",
        value: partial,
        passed: fast_cov >= COV_LOW && fast_cov <= high && io_cov >= COV_LOW && io_cov <= high,
        supports: vec![3],
        contradicts: vec![1, 2, 4],
    });

    out
}

/// Compute 0..1 compatibility scores for each of the four answers.
pub fn score_answers(stats: &MatchStats, control_cov: f64) -> Vec<AnswerScore> {
    let hb: std::collections::HashMap<&str, bool> = evaluate(stats, control_cov)
        .into_iter()
        .map(|h| (h.name, h.passed))
        .collect();

    let fast_cov = stats.fast_to_io;
    let io_cov = stats.io_to_fast;
    let cr = stats.count_ratio;
    let h4 = if cr >= 1.0 { 0.0 } else { ((1.0 - cr) / (1.0 - 0.66)).clamp(0.0, 1.0) };
    let excl = (1.0 - fast_cov).max(0.0)
        * (1.0 - io_cov).max(0.0)
        * if stats.opcodes_identical { 0.0 } else { 1.0 };
    let partial = 1.0 - (2.0 * fast_cov - 1.0).abs();
    let op_id = if stats.opcodes_identical { 1.0 } else { 0.0 };

    let s_subset = 0.35 * fast_cov
        + 0.30 * (1.0 - io_cov)
        + 0.20 * h4
        + 0.15 * (1.0 - excl);
    let s_same = 0.35 * fast_cov
        + 0.30 * io_cov
        + 0.20 * (1.0 - h4)
        + 0.15 * op_id;
    let s_excl = 0.40 * excl
        + 0.30 * (1.0 - op_id)
        + 0.15 * (1.0 - fast_cov)
        + 0.15 * (1.0 - io_cov);
    let s_part = 0.40 * partial
        + 0.30 * (1.0 - excl)
        + 0.15 * (1.0 - fast_cov)
        + 0.15 * (1.0 - io_cov);

    let picked = |keys: &[&str]| keys.iter().filter(|k| hb.get(*k).copied().unwrap_or(false)).copied().collect::<Vec<_>>();

    vec![
        AnswerScore {
            number: 1,
            label: "PERF_FLT_FASTIO is a SUBSET of PERF_FLT_IO (IO superset)",
            score: s_subset,
            passed: picked(&["fast covered by io", "io larger"]),
            failed: picked(&["io covered by fast", "symmetric"]),
        },
        AnswerScore {
            number: 2,
            label: "Events are EXCLUSIVE (disjoint)",
            score: s_excl,
            passed: picked(&["exclusive"]),
            failed: picked(&["opcodes identical", "fast covered by io", "io covered by fast"]),
        },
        AnswerScore {
            number: 3,
            label: "PARTIAL OVERLAP / collisions",
            score: s_part,
            passed: picked(&["partial overlap"]),
            failed: picked(&["symmetric", "exclusive"]),
        },
        AnswerScore {
            number: 4,
            label: "They are the SAME",
            score: s_same,
            passed: picked(&["opcodes identical", "io covered by fast", "symmetric"]),
            failed: picked(&["io larger"]),
        },
    ]
}

/// Average several passes into a single representative pass.
pub fn aggregate(passes: &[MatchStats]) -> MatchStats {
    if passes.is_empty() {
        return MatchStats {
            name: "aggregate".into(),
            fast_total: 0, io_total: 0, matched: 0, fast_only: 0, io_only: 0,
            fast_to_io: 0.0, io_to_fast: 0.0, jaccard: 0.0, count_ratio: 0.0,
            fast_opcodes: vec![], io_opcodes: vec![], opcodes_identical: false,
        };
    }
    let n = passes.len() as f64;
    MatchStats {
        name: format!("aggregate ({} passes)", passes.len()),
        fast_total: passes.iter().map(|p| p.fast_total).sum::<usize>() / passes.len(),
        io_total: passes.iter().map(|p| p.io_total).sum::<usize>() / passes.len(),
        matched: passes.iter().map(|p| p.matched).sum::<usize>() / passes.len(),
        fast_only: passes.iter().map(|p| p.fast_only).sum::<usize>() / passes.len(),
        io_only: passes.iter().map(|p| p.io_only).sum::<usize>() / passes.len(),
        fast_to_io: passes.iter().map(|p| p.fast_to_io).sum::<f64>() / n,
        io_to_fast: passes.iter().map(|p| p.io_to_fast).sum::<f64>() / n,
        jaccard: passes.iter().map(|p| p.jaccard).sum::<f64>() / n,
        count_ratio: passes.iter().map(|p| p.count_ratio).sum::<f64>() / n,
        fast_opcodes: majority_opcodes(passes, true),
        io_opcodes: majority_opcodes(passes, false),
        opcodes_identical: passes.iter().all(|p| p.opcodes_identical),
    }
}

fn majority_opcodes(passes: &[MatchStats], fast_side: bool) -> Vec<OpcodeCount> {
    let mut map: HashMap<(u8, u8), usize> = HashMap::new();
    for p in passes {
        let list = if fast_side { &p.fast_opcodes } else { &p.io_opcodes };
        for oc in list {
            *map.entry((oc.opcode as u16, oc.version as u16)).or_insert(0) += oc.count;
        }
    }
    let mut v: Vec<OpcodeCount> = map
        .into_iter()
        .map(|((op, ver), count)| OpcodeCount { opcode: op as u8, version: ver as u8, count })
        .collect();
    v.sort_by_key(|o| (o.opcode, o.version));
    v
}