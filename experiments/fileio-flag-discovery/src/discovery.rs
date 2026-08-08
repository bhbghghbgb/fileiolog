use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::event_types::EventTypeInfo;
use crate::events::{self, ParsedFileIoEvent};
use crate::flags::{self, Flag};
use crate::output;
use crate::trace_session::{KernelTraceSession, TraceConfig};

pub(crate) const RUNS_PER_COMBO: usize = 3;
pub(crate) const MAX_COMBO_SIZE: usize = 2;

pub(crate) struct EventDiscovery {
    pub best_size: usize,
    pub combinations: Vec<Vec<usize>>,
    pub observed_versions: HashSet<u8>,
}

pub(crate) fn discover(
    flags: &[Flag],
    event_types: &[EventTypeInfo],
    output_dir: &Path,
) -> HashMap<u8, EventDiscovery> {
    let mut discovered: HashMap<u8, EventDiscovery> = HashMap::new();
    let total = event_types.len();

    for size in 1..=MAX_COMBO_SIZE {
        let combos = gen_combos(flags.len(), size);
        log::info!("========================================");
        log::info!(
            "Phase {}: Testing {} combinations of {} flags",
            size,
            combos.len(),
            size
        );
        log::info!("========================================");

        let mut new_this_phase: Vec<String> = Vec::new();

        for (ci, indices) in combos.iter().enumerate() {
            log::info!(
                "  [{}/{}] {}",
                ci + 1,
                combos.len(),
                flags::combo_name(flags, indices)
            );

            let config = flags::merge_flags(flags, indices);
            let mut combo_opcodes: HashSet<u8> = HashSet::new();

            for ri in 0..RUNS_PER_COMBO {
                log::info!("    Run {}/{}", ri + 1, RUNS_PER_COMBO);
                let raw = run_single_test(&config);

                // Collect this run's opcodes and versions
                let mut run_opcodes: HashSet<u8> = HashSet::new();
                let mut run_versions: HashMap<u8, HashSet<u8>> = HashMap::new();
                for event in &raw {
                    run_versions
                        .entry(event.opcode)
                        .or_default()
                        .insert(event.version);
                    run_opcodes.insert(event.opcode);
                }
                combo_opcodes.extend(run_opcodes.iter());

                // Update discovered incrementally after each run
                for &opcode in &run_opcodes {
                    if !event_types.iter().any(|et| et.opcode == opcode) {
                        continue;
                    }
                    update_discovery(
                        &mut discovered,
                        opcode,
                        size,
                        indices,
                        &run_versions,
                        event_types,
                        &mut new_this_phase,
                    );
                }

                output::write_run_file(
                    output_dir,
                    size,
                    ci,
                    ri,
                    flags,
                    indices,
                    &run_opcodes,
                    &discovered,
                    event_types,
                );
            }

            output::write_combo_file(
                output_dir,
                size,
                ci,
                flags,
                indices,
                RUNS_PER_COMBO,
                &combo_opcodes,
                &discovered,
                event_types,
            );
            if ci < combos.len() - 1 {
                std::thread::sleep(Duration::from_secs(1));
            }
        }

        if !new_this_phase.is_empty() {
            log::info!("  New events discovered at size {}:", size);
            for name in &new_this_phase {
                log::info!("    {}", name);
            }
        }

        let count = discovered.values().filter(|d| d.best_size <= size).count();
        log::info!(
            "Phase {} complete. Discovered: {}/{} event types.",
            size,
            count,
            total
        );

        output::write_phase_file(
            output_dir,
            size,
            combos.len(),
            flags,
            &discovered,
            event_types,
        );

        if count >= total {
            log::info!("All event types discovered! Stopping early.");
            break;
        }
        log::info!("");
        std::thread::sleep(Duration::from_secs(2));
    }

    discovered
}

fn update_discovery(
    discovered: &mut HashMap<u8, EventDiscovery>,
    opcode: u8,
    size: usize,
    indices: &[usize],
    run_versions: &HashMap<u8, HashSet<u8>>,
    event_types: &[EventTypeInfo],
    new_this_phase: &mut Vec<String>,
) {
    let entry = discovered.entry(opcode).or_insert_with(|| EventDiscovery {
        best_size: usize::MAX,
        combinations: Vec::new(),
        observed_versions: HashSet::new(),
    });

    if let Some(vers) = run_versions.get(&opcode) {
        entry.observed_versions.extend(vers);
    }

    if size < entry.best_size {
        entry.best_size = size;
        entry.combinations = vec![indices.to_vec()];
        if let Some(et) = event_types.iter().find(|et| et.opcode == opcode) {
            new_this_phase.push(format!(
                "{} [{}] (Opcode={})",
                et.event_name, et.class_name, opcode
            ));
        }
    } else if size == entry.best_size && !entry.combinations.contains(&indices.to_vec()) {
        entry.combinations.push(indices.to_vec());
    }
}

fn run_single_test(config: &TraceConfig) -> Vec<events::FileIoRawEvent> {
    let collected: Arc<Mutex<Vec<events::FileIoRawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let parsed: Arc<Mutex<Vec<ParsedFileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let mut session = match KernelTraceSession::new(TraceConfig {
        session_name: config.session_name.clone(),
        enable_flags: config.enable_flags,
        group_mask: config.group_mask,
    }) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create trace session: {:?}", e);
            return Vec::new();
        }
    };

    let _handle = match session.start(collected.clone(), parsed.clone()) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start trace: {:?}", e);
            return Vec::new();
        }
    };

    let proc_handle = session.get_trace_handle();
    let thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(proc_handle);
    });

    std::thread::sleep(Duration::from_millis(500));
    crate::file_ops::trigger_all_file_operations();
    std::thread::sleep(Duration::from_secs(5));
    let _ = session.request_rundown();
    std::thread::sleep(Duration::from_secs(2));
    let _ = session.stop();
    let _ = thread.join();

    collected.lock().unwrap().clone()
}

fn gen_combos(n: usize, k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if k > n {
        return vec![];
    }
    let mut result = Vec::new();
    let mut combo = Vec::new();
    gen_recursive(n, k, 0, &mut combo, &mut result);
    result
}

fn gen_recursive(
    n: usize,
    k: usize,
    start: usize,
    combo: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if combo.len() == k {
        result.push(combo.clone());
        return;
    }
    for i in start..n {
        combo.push(i);
        gen_recursive(n, k, i + 1, combo, result);
        combo.pop();
    }
}
