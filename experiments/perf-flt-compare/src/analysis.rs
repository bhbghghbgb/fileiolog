use std::collections::{BTreeSet, HashMap};
use std::fmt;

/// Per-config aggregated fingerprint.
#[derive(Debug, Clone, Default)]
pub struct Fingerprint {
    pub total: usize,
    pub fast: usize,
    pub nonfast: usize,
    /// major_function -> count
    pub majors: HashMap<u32, usize>,
}

impl Fingerprint {    pub fn fast_frac(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.fast as f64 / self.total as f64
        }
    }

    pub fn add(&mut self, ev: &crate::event::RawEvent) {
        self.total += 1;
        if ev.is_fast() {
            self.fast += 1;
        } else {
            self.nonfast += 1;
        }
        *self.majors.entry(ev.major_function()).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &Fingerprint) {
        self.total += other.total;
        self.fast += other.fast;
        self.nonfast += other.nonfast;
        for (k, v) in &other.majors {
            *self.majors.entry(*k).or_insert(0) += v;
        }
    }
pub fn major_jaccard(&self, other: &Fingerprint) -> f64 {
        jaccard(&self.majors, &other.majors)
    }

    /// Weighted fraction of this fingerprint's events whose MajorFunction is
    /// /not/ present in `other`. 1.0 = fully disjoint, 0.0 = fully covered.
    pub fn disjoint_frac(&self, other: &Fingerprint) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let d: f64 = self
            .majors
            .iter()
            .filter(|(k, _)| !other.majors.contains_key(k))
            .map(|(_, v)| *v as f64)
            .sum();
        d / self.total as f64
    }
}

fn jaccard(a: &HashMap<u32, usize>, b: &HashMap<u32, usize>) -> f64 {
    let aset: BTreeSet<u32> = a.keys().copied().collect();
    let bset: BTreeSet<u32> = b.keys().copied().collect();
    let inter = aset.intersection(&bset).count();
    let union = aset.union(&bset).count();
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

/// Result of one pass of the experiment.
#[derive(Debug, Clone)]
pub struct PassResult {
    pub fastio: Fingerprint,
    pub io: Fingerprint,
    pub both: Fingerprint,
    pub both_fast: Fingerprint,
    pub both_nonfast: Fingerprint,
}

#[derive(Debug, Clone)]
pub struct Verdict {
    pub num_passes: usize,
    pub fastio: Fingerprint,
    pub io: Fingerprint,
    pub both: Fingerprint,
    pub both_fast: Fingerprint,
    pub both_nonfast: Fingerprint,
    /// Discriminator validation: measured IrpPtr==0 fraction in FASTIO vs IO runs.
    pub fastio_fast_frac: f64,
    pub io_fast_frac: f64,
    pub scores: Vec<(Hypothesis, f64)>,
    pub best: Hypothesis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hypothesis {
    Disjoint,
    Subset,
    PartialOverlap,
    Same,
}

impl fmt::Display for Hypothesis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hypothesis::Disjoint => write!(f, "Disjoint / exclusive"),
            Hypothesis::Subset => write!(f, "FASTIO is a (proper) subset of IO"),
            Hypothesis::PartialOverlap => write!(f, "Partial overlap"),
            Hypothesis::Same => write!(f, "Same events"),
        }
    }
}

pub fn fingerprint(events: &[crate::event::RawEvent]) -> Fingerprint {
    let mut fp = Fingerprint::default();
    for ev in events {
        fp.add(ev);
    }
    fp
}

/// Single-pass: fingerprint each config and partition the `both` run.
pub fn score_pass(
    _pass: usize,
    fastio: &[crate::event::RawEvent],
    io: &[crate::event::RawEvent],
    both: &[crate::event::RawEvent],
) -> PassResult {
    let fp_fastio = fingerprint(fastio);
    let fp_io = fingerprint(io);
    let fp_both = fingerprint(both);

    let mut both_fast = Fingerprint::default();
    let mut both_nonfast = Fingerprint::default();
    for ev in both {
        if ev.is_fast() {
            both_fast.add(ev);
        } else {
            both_nonfast.add(ev);
        }
    }

    PassResult {
        fastio: fp_fastio,
        io: fp_io,
        both: fp_both,
        both_fast,
        both_nonfast,
    }
}

pub fn analyze(passes: &[PassResult]) -> Verdict {
    let mut fastio = Fingerprint::default();
    let mut io = Fingerprint::default();
    let mut both = Fingerprint::default();
    let mut both_fast = Fingerprint::default();
    let mut both_nonfast = Fingerprint::default();

    for p in passes {
        fastio.merge(&p.fastio);
        io.merge(&p.io);
        both.merge(&p.both);
        both_fast.merge(&p.both_fast);
        both_nonfast.merge(&p.both_nonfast);
    }

    let fastio_fast_frac = fastio.fast_frac();
    let io_fast_frac = io.fast_frac();

    // Discriminator validation: does IrpPtr==0 cleanly separate?
    // 0.0 = bad discriminator, 1.0 = perfect.
    // fastio should be ~all fast (IrpPtr==0), io should be ~none fast.
    let disc_sharpness = (fastio_fast_frac * (1.0 - io_fast_frac)).max(0.0);

    // MajorFunction relationships.
    let fastio_disjoint_io = fastio.disjoint_frac(&io); // 1 = fastio majors absent from io
    let io_disjoint_fastio = io.disjoint_frac(&fastio); // 1 = io majors absent from fastio
    let j_fastio_io = fastio.major_jaccard(&io);
    let j_fast_bothfast = fastio.major_jaccard(&both_fast);
    let j_io_bothnonfast = io.major_jaccard(&both_nonfast);

    let mut scores: Vec<(Hypothesis, f64)> = Vec::new();

    // H1 Disjoint: fastio & io major-mass are mutually absent, discriminator is sharp,
    // and the `both` partition inherits the correct implications.
    let disc_disjoint = j_fast_bothfast.min(j_io_bothnonfast);
    let disjoint = (0.4 * disc_sharpness
        + 0.4 * ((fastio_disjoint_io + io_disjoint_fastio) / 2.0)
        + 0.2 * disc_disjoint) * 100.0;
    scores.push((Hypothesis::Disjoint, disjoint));

    // H2 Subset: fastio is fully covered by io (fastio_disjoint_io ~ 0) while io
    // extends beyond fastio (io_disjoint_fastio > 0).
    let subset =
        (0.5 * (1.0 - fastio_disjoint_io) + 0.3 * io_disjoint_fastio + 0.2 * disc_sharpness)
            * 100.0;
    scores.push((Hypothesis::Subset, subset));

    // H3 Partial overlap: majors overlap strongly (jaccard ~0.5) but neither side is empty.
    let partial =
        (0.5 * (1.0 - (j_fastio_io - 0.5).abs().min(0.5)) + 0.5 * (1.0 - disc_sharpness)) * 100.0;
    scores.push((Hypothesis::PartialOverlap, partial));

    // H4 Same: major sets near-identical and both partition matches both flags.
    let same = (0.5 * j_fastio_io.min(1.0) + 0.3 * j_fast_bothfast.min(1.0)
        + 0.2 * j_io_bothnonfast.min(1.0))
        * 100.0;
    scores.push((Hypothesis::Same, same));

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let best = scores[0].0;

    Verdict {
        num_passes: passes.len(),
        fastio,
        io,
        both,
        both_fast,
        both_nonfast,
        fastio_fast_frac,
        io_fast_frac,
        scores,
        best,
    }
}