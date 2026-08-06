use std::collections::HashMap;

use crate::event::RawEvent;

/// Result of comparing two event sets
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub count_a: usize,
    pub count_b: usize,
    pub ratio: f64,
    pub matched_pairs: usize,
    pub match_ratio_a: f64,
    pub match_ratio_b: f64,
    pub major_func_dist_a: HashMap<u32, usize>,
    pub major_func_dist_b: HashMap<u32, usize>,
    pub distribution_match: bool,
    pub unique_to_a: usize,
    pub unique_to_b: usize,
    #[allow(dead_code)]
    pub heuristic_scores: HashMap<String, f64>,
}

/// Match events between two sessions by temporal + field proximity.
///
/// Two events are considered "the same" if:
/// - |timestamp_a - timestamp_b| < tolerance
/// - process_id matches
/// - thread_id matches
/// - major_function matches
///
/// Returns (matched_pairs, unique_to_a, unique_to_b).
fn match_events(a: &[RawEvent], b: &[RawEvent], tolerance: u64) -> (usize, usize, usize) {
    let mut b_used = vec![false; b.len()];
    let mut matched = 0;

    for ea in a {
        let mut found = false;
        for (j, eb) in b.iter().enumerate() {
            if b_used[j] {
                continue;
            }
            let ts_diff = ea.timestamp.abs_diff(eb.timestamp);
            if ts_diff <= tolerance
                && ea.process_id == eb.process_id
                && ea.thread_id == eb.thread_id
                && ea.flt.major_function == eb.flt.major_function
            {
                b_used[j] = true;
                matched += 1;
                found = true;
                break;
            }
        }
        // If not found, it's unique to A (but we don't count here; we count after)
        let _ = found;
    }

    let unique_to_a = a.len() - matched;
    let unique_to_b = b_used.iter().filter(|&&u| !u).count();

    (matched, unique_to_a, unique_to_b)
}

/// Compare MajorFunction distributions between two event sets
fn compare_distributions(
    a: &[RawEvent],
    b: &[RawEvent],
) -> (HashMap<u32, usize>, HashMap<u32, usize>, bool) {
    let dist_a: HashMap<u32, usize> = a.iter().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.flt.major_function).or_insert(0) += 1;
        acc
    });
    let dist_b: HashMap<u32, usize> = b.iter().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.flt.major_function).or_insert(0) += 1;
        acc
    });

    let match_dist = dist_a == dist_b;
    (dist_a, dist_b, match_dist)
}

/// Score how consistent a ratio is across multiple passes.
/// Low variance → high consistency.
fn consistency_score(ratios: &[f64]) -> f64 {
    if ratios.len() < 2 {
        return 1.0;
    }
    let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let variance = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / ratios.len() as f64;
    let cv = variance.sqrt() / mean.max(0.001);
    // cv=0 → score=1.0, cv=0.5 → score=0.0
    (1.0 - cv * 2.0).max(0.0)
}

/// Analyze multiple passes of concurrent dual-session traces.
///
/// Each pass produces one `ComparisonResult`. This function aggregates them
/// into a final verdict with confidence scores for each hypothesis.
pub fn analyze_passes(passes: &[ComparisonResult]) -> AnalysisVerdict {
    let n = passes.len() as f64;

    // Collect ratios (B/A, where A=PERF_FLT_IO, B=PERF_FLT_FASTIO)
    let ratios: Vec<f64> = passes.iter().map(|p| p.ratio).collect();
    let mean_ratio = ratios.iter().sum::<f64>() / n;
    let ratio_consistency = consistency_score(&ratios);

    // Check if ratio is always <= 1.0 (FASTIO <= IO)
    let all_le_1 = ratios.iter().all(|r| *r <= 1.0 + 0.05); // 5% tolerance
    let all_ge_1 = ratios.iter().all(|r| *r >= 1.0 - 0.05);
    let all_eq_1 = all_le_1 && all_ge_1;

    // Check distribution match consistency
    let dist_match_count = passes.iter().filter(|p| p.distribution_match).count();
    let dist_match_ratio = dist_match_count as f64 / n;

    // Check match ratio consistency
    let match_ratios_a: Vec<f64> = passes.iter().map(|p| p.match_ratio_a).collect();
    let match_ratios_b: Vec<f64> = passes.iter().map(|p| p.match_ratio_b).collect();
    let mean_match_a = match_ratios_a.iter().sum::<f64>() / n;
    let mean_match_b = match_ratios_b.iter().sum::<f64>() / n;

    // Check exclusive event counts
    let total_unique_to_a: usize = passes.iter().map(|p| p.unique_to_a).sum();
    let total_unique_to_b: usize = passes.iter().map(|p| p.unique_to_b).sum();

    // Score each hypothesis
    let mut scores: HashMap<String, f64> = HashMap::new();

    // H1: PERF_FLT_FASTIO events ⊂ PERF_FLT_IO events (FASTIO is subset of IO)
    // Evidence: ratio <= 1, distributions match, many FASTIO events match IO events
    let h1_ratio_score = if all_le_1 { 1.0 } else { 0.0 };
    let h1_dist_score = dist_match_ratio;
    let h1_match_score = mean_match_b; // fraction of FASTIO events found in IO
    let h1 = (h1_ratio_score * 0.3 + h1_dist_score * 0.3 + h1_match_score * 0.4) * 100.0;
    scores.insert("FASTIO ⊂ IO".to_string(), h1);

    // H2: The two sets are exclusive (no overlap)
    // Evidence: low match ratios, distributions differ
    let h2_match_score = 1.0 - ((mean_match_a + mean_match_b) / 2.0);
    let h2_dist_score = 1.0 - dist_match_ratio;
    let h2 = (h2_match_score * 0.5 + h2_dist_score * 0.5) * 100.0;
    scores.insert("Exclusive (no overlap)".to_string(), h2);

    // H3: Partial overlap (some collisions, neither is subset)
    // Evidence: moderate match ratios, some exclusive events on both sides
    let has_both_exclusive = total_unique_to_a > 0 && total_unique_to_b > 0;
    let h3_overlap = if has_both_exclusive {
        ((mean_match_a + mean_match_b) / 2.0).min(1.0 - (mean_match_a + mean_match_b) / 2.0)
    } else {
        0.0
    };
    let h3_dist_mismatch = 1.0 - dist_match_ratio;
    let h3 = (h3_overlap * 0.5 + h3_dist_mismatch * 0.5) * 100.0;
    scores.insert("Partial overlap".to_string(), h3);

    // H4: They are the same (identical event sets)
    // Evidence: ratio ≈ 1, distributions match, high match ratios on both sides
    let h4_ratio_score = if all_eq_1 { 1.0 } else { (1.0 - (mean_ratio - 1.0).abs()).max(0.0) };
    let h4_dist_score = dist_match_ratio;
    let h4_match_score = (mean_match_a + mean_match_b) / 2.0;
    let h4_consistency = ratio_consistency;
    let h4 = (h4_ratio_score * 0.25 + h4_dist_score * 0.25 + h4_match_score * 0.3 + h4_consistency * 0.2) * 100.0;
    scores.insert("Same events".to_string(), h4);

    // Find best hypothesis
    let best = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, v)| (k.clone(), *v))
        .unwrap_or_default();

    AnalysisVerdict {
        num_passes: passes.len(),
        mean_ratio,
        ratio_consistency,
        distribution_match_rate: dist_match_ratio,
        mean_match_ratio_a: mean_match_a,
        mean_match_ratio_b: mean_match_b,
        total_unique_to_a,
        total_unique_to_b,
        heuristic_scores: scores,
        best_hypothesis: best,
        passes: passes.to_vec(),
    }
}

/// Final verdict from analyzing multiple passes
#[derive(Debug, Clone)]
pub struct AnalysisVerdict {
    pub num_passes: usize,
    pub mean_ratio: f64,
    pub ratio_consistency: f64,
    pub distribution_match_rate: f64,
    pub mean_match_ratio_a: f64,
    pub mean_match_ratio_b: f64,
    pub total_unique_to_a: usize,
    pub total_unique_to_b: usize,
    pub heuristic_scores: HashMap<String, f64>,
    pub best_hypothesis: (String, f64),
    pub passes: Vec<ComparisonResult>,
}

/// Run a single-pass comparison between two event sets.
pub fn compare_sessions(a: &[RawEvent], b: &[RawEvent]) -> ComparisonResult {
    // Match with 10μs tolerance (10,000 * 100ns units)
    let tolerance = 10_000;
    let (matched, unique_to_a, unique_to_b) = match_events(a, b, tolerance);

    let (dist_a, dist_b, dist_match) = compare_distributions(a, b);

    let ratio = if a.is_empty() {
        0.0
    } else {
        b.len() as f64 / a.len() as f64
    };

    let match_ratio_a = if a.is_empty() {
        0.0
    } else {
        matched as f64 / a.len() as f64
    };

    let match_ratio_b = if b.is_empty() {
        0.0
    } else {
        matched as f64 / b.len() as f64
    };

    ComparisonResult {
        count_a: a.len(),
        count_b: b.len(),
        ratio,
        matched_pairs: matched,
        match_ratio_a,
        match_ratio_b,
        major_func_dist_a: dist_a.clone(),
        major_func_dist_b: dist_b.clone(),
        distribution_match: dist_match,
        unique_to_a,
        unique_to_b,
        heuristic_scores: HashMap::new(),
    }
}
