use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ferrisetw::EventRecord;
use ferrisetw::provider::*;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::*;
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, PROCESSTRACE_HANDLE, WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

// ── ETW Constants ──────────────────────────────────────────────────────────

const FILE_IO_GUID: GUID = GUID::from_u128(0x90cbdc39_4a3e_11d1_84f4_0000f80464e3);
const PERF_FLT_IO: u32 = 0x80100000;
const PERF_FLT_FASTIO: u32 = 0x80200000;

// ── Tee Logger (stderr + file) ─────────────────────────────────────────────

use std::sync::OnceLock;

struct TeeLogger {
    file: OnceLock<Mutex<fs::File>>,
}

impl log::Log for TeeLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let msg = format!("{}\n", record.args());
        eprint!("{}", msg);
        if let Some(f) = self.file.get() {
            if let Ok(mut guard) = f.lock() {
                let _ = guard.write_all(msg.as_bytes());
            }
        }
    }
    fn flush(&self) {
        if let Some(f) = self.file.get() {
            if let Ok(mut guard) = f.lock() {
                let _ = guard.flush();
            }
        }
    }
}

static TEE_LOGGER: TeeLogger = TeeLogger {
    file: OnceLock::new(),
};

fn init_tee_logger() {
    let f = fs::File::create("flt-compare-results.txt").expect("Cannot create log file");
    TEE_LOGGER
        .file
        .set(Mutex::new(f))
        .expect("Logger already initialized");
    log::set_logger(&TEE_LOGGER).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Info);
}

// ── Data Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct RawEvent {
    opcode: u8,
    version: u8,
    timestamp: u64,
    #[allow(dead_code)]
    process_id: u32,
    #[allow(dead_code)]
    thread_id: u32,
}

// ── Trace Session ──────────────────────────────────────────────────────────

struct TraceConfig {
    session_name: String,
    enable_flags: u32,
    group_mask: Option<[u32; 8]>,
}

struct KernelTraceSession {
    config: TraceConfig,
    events: Arc<Mutex<Vec<RawEvent>>>,
    trace: Option<KernelTrace>,
    trace_handle: Option<PROCESSTRACE_HANDLE>,
    control_handle: Option<CONTROLTRACE_HANDLE>,
}

impl KernelTraceSession {
    fn new(config: TraceConfig) -> Self {
        Self {
            config,
            events: Arc::new(Mutex::new(Vec::new())),
            trace: None,
            trace_handle: None,
            control_handle: None,
        }
    }

    fn start(&mut self) -> Result<PROCESSTRACE_HANDLE, Box<dyn std::error::Error>> {
        let events = self.events.clone();

        let kernel_provider =
            kernel_providers::KernelProvider::new(FILE_IO_GUID, self.config.enable_flags);

        let provider = Provider::kernel(&kernel_provider)
            .level(0xFF)
            .any(0)
            .all(0)
            .add_callback(move |record: &EventRecord, _schema_locator: &SchemaLocator| {
                if let Ok(mut evts) = events.lock() {
                    evts.push(RawEvent {
                        opcode: record.opcode(),
                        version: record.version(),
                        timestamp: record.raw_timestamp() as u64,
                        process_id: record.process_id(),
                        thread_id: record.thread_id(),
                    });
                }
            })
            .build();

        let builder = KernelTrace::new()
            .named(self.config.session_name.clone())
            .enable(provider)
            .stop_if_exist(true);

        let (trace, trace_handle) = builder
            .start()
            .map_err(|e| format!("Trace start failed: {:?}", e))?;
        self.trace = Some(trace);
        self.trace_handle = Some(trace_handle);

        let control_handle = self.query_control_handle()?;
        self.control_handle = Some(control_handle);

        if let Some(mask) = self.config.group_mask {
            self.set_group_mask(mask)?;
        }

        Ok(trace_handle)
    }

    fn start_processing(&self) -> std::thread::JoinHandle<()> {
        let handle = self.trace_handle.expect("Trace not started");
        std::thread::spawn(move || {
            use ferrisetw::trace::TraceTrait;
            let _ = <KernelTrace as TraceTrait>::process_from_handle(handle);
        })
    }

    #[allow(dead_code)]
    fn request_rundown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let control_handle = self.control_handle.ok_or("Control handle not available")?;
        unsafe {
            Etw::EnableTraceEx2(
                control_handle,
                &FILE_IO_GUID as *const GUID,
                Etw::EVENT_CONTROL_CODE_CAPTURE_STATE.0,
                0,
                0,
                0,
                0,
                None,
            )
        }
        .ok()
        .map_err(|e| format!("EnableTraceEx2 failed: {:?}", e))?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(control_handle) = self.control_handle {
            let mut buffer = self.build_trace_properties();
            let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
            unsafe {
                Etw::ControlTraceW(
                    control_handle,
                    PCWSTR::null(),
                    props as *mut EVENT_TRACE_PROPERTIES,
                    Etw::EVENT_TRACE_CONTROL_STOP,
                )
            }
            .ok()
            .map_err(|e| format!("ControlTraceW STOP failed: {:?}", e))?;
        }
        self.trace.take();
        Ok(())
    }

    fn take_events(&self) -> Vec<RawEvent> {
        std::mem::take(&mut *self.events.lock().unwrap())
    }

    fn build_trace_properties(&self) -> Vec<u8> {
        let name_wide: Vec<u16> = self
            .config
            .session_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
        let name_buf_size = (200 + 1) * 2;
        let total_size = header_size + name_buf_size;
        let mut buffer = vec![0u8; total_size];
        let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
        props.Wnode.BufferSize = total_size as u32;
        props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
        props.Wnode.Guid = GUID::zeroed();
        props.LoggerNameOffset = header_size as u32;
        props.LogFileNameOffset = 0;
        let name_ptr = unsafe { buffer.as_mut_ptr().add(header_size) as *mut u16 };
        unsafe {
            std::ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
        }
        buffer
    }

    fn query_control_handle(&self) -> Result<CONTROLTRACE_HANDLE, Box<dyn std::error::Error>> {
        let mut buffer = self.build_trace_properties();
        let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
        let name_ptr = unsafe {
            buffer.as_mut_ptr().add(props.LoggerNameOffset as usize) as *const u16
        };
        unsafe {
            Etw::ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                PCWSTR::from_raw(name_ptr),
                props as *mut EVENT_TRACE_PROPERTIES,
                Etw::EVENT_TRACE_CONTROL_QUERY,
            )
        }
        .ok()
        .map_err(|e| format!("ControlTraceW QUERY failed: {:?}", e))?;
        let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
        Ok(CONTROLTRACE_HANDLE {
            Value: handle_value,
        })
    }

    fn set_group_mask(&self, masks: [u32; 8]) -> Result<(), Box<dyn std::error::Error>> {
        let control_handle = self.control_handle.ok_or("Control handle not available")?;
        let mut group_mask_data = [0u32; 8];
        group_mask_data.copy_from_slice(&masks);
        const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;
        unsafe {
            Etw::TraceSetInformation(
                control_handle,
                std::mem::transmute(TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO),
                group_mask_data.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of::<[u32; 8]>() as u32,
            )
        }
        .ok()
        .map_err(|e| format!("TraceSetInformation failed: {:?}", e))?;
        Ok(())
    }
}

impl Drop for KernelTraceSession {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ── File Operations ────────────────────────────────────────────────────────

fn trigger_file_ops() {
    let dir = Path::new("C:\\temp_flt_compare");
    let _ = fs::create_dir_all(dir);
    for i in 0..8 {
        let _ = fs::write(dir.join(format!("test_{}.txt", i)), format!("data {}", i));
    }
    for i in 0..8 {
        let _ = fs::read(dir.join(format!("test_{}.txt", i)));
    }
    for i in 0..8 {
        let _ = fs::write(dir.join(format!("test_{}.txt", i)), format!("upd {}", i));
    }
    for _ in 0..3 {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let _ = e.metadata();
            }
        }
    }
    for i in 0..4 {
        let old = dir.join(format!("test_{}.txt", i));
        let new = dir.join(format!("renamed_{}.txt", i));
        let _ = fs::rename(&old, &new);
    }
    for i in 0..4 {
        let old = dir.join(format!("renamed_{}.txt", i));
        let new = dir.join(format!("test_{}.txt", i));
        let _ = fs::rename(&old, &new);
    }
    for i in 0..8 {
        let _ = fs::metadata(dir.join(format!("test_{}.txt", i)));
    }
    for i in 0..4 {
        use std::io::Write;
        let path = dir.join(format!("test_{}.txt", i));
        if let Ok(mut f) = fs::OpenOptions::new().write(true).open(&path) {
            let _ = f.write_all(b"flush");
            let _ = f.flush();
        }
    }
    for i in 0..4 {
        let _ = fs::remove_file(dir.join(format!("test_{}.txt", i)));
    }
    let _ = fs::remove_dir_all(dir);
}

fn opcode_label(opcode: u8) -> &'static str {
    match opcode {
        0 => "Name",
        32 => "FileCreate",
        35 => "FileDelete",
        36 => "FileRundown",
        64 => "Create",
        65 => "Cleanup",
        66 => "Close",
        67 => "Read",
        68 => "Write",
        69 => "SetInfo",
        70 => "DeleteInfo",
        71 => "Rename",
        72 => "DirEnum",
        73 => "Flush",
        74 => "QueryInfo",
        75 => "FSControl",
        76 => "OpEnd",
        77 => "DirNotify",
        79 => "DeletePath",
        80 => "RenamePath",
        81 => "SetLinkPath",
        96 => "PreOpInit",
        97 => "PostOpInit",
        98 => "PreOpCompletion",
        99 => "PostOpCompletion",
        100 => "PreOpFailure",
        101 => "PostOpFailure",
        _ => "UNKNOWN",
    }
}

fn count_by_opcode(events: &[RawEvent]) -> HashMap<u8, usize> {
    let mut counts = HashMap::new();
    for e in events {
        *counts.entry(e.opcode).or_insert(0) += 1;
    }
    counts
}

fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    let group_index = ((mask_value >> 29) & 0x07) as usize;
    masks[group_index] = mask_value;
    masks
}

// ── Main ───────────────────────────────────────────────────────────────────

fn main() {
    init_tee_logger();

    log::info!("=== PERF_FLT_IO vs PERF_FLT_FASTIO Comparison ===");
    log::info!("Running two ETW sessions simultaneously to compare events.\n");

    let num_runs = 3;
    let mut all_io_events: Vec<RawEvent> = Vec::new();
    let mut all_fastio_events: Vec<RawEvent> = Vec::new();

    for run in 1..=num_runs {
        log::info!("--- Run {}/{} ---", run, num_runs);

        let mut session_io = KernelTraceSession::new(TraceConfig {
            session_name: format!("FltCmp-FltIo-R{}", run),
            enable_flags: 0,
            group_mask: Some(build_group_mask(PERF_FLT_IO)),
        });
        let mut session_fastio = KernelTraceSession::new(TraceConfig {
            session_name: format!("FltCmp-FltFIO-R{}", run),
            enable_flags: 0,
            group_mask: Some(build_group_mask(PERF_FLT_FASTIO)),
        });

        if let Err(e) = session_io.start() {
            log::error!("Failed to start FltIo session: {:?}", e);
            continue;
        }
        if let Err(e) = session_fastio.start() {
            log::error!("Failed to start FltFastIo session: {:?}", e);
            let _ = session_io.stop();
            continue;
        }

        let proc_io = session_io.start_processing();
        let proc_fastio = session_fastio.start_processing();

        std::thread::sleep(Duration::from_millis(800));

        log::info!("  Triggering file operations...");
        trigger_file_ops();

        log::info!("  Waiting for events (5s)...");
        std::thread::sleep(Duration::from_secs(5));

        log::info!("  Stopping sessions...");
        let _ = session_io.stop();
        let _ = session_fastio.stop();
        let _ = proc_io.join();
        let _ = proc_fastio.join();

        let io_events = session_io.take_events();
        let fastio_events = session_fastio.take_events();
        log::info!("  FltIo: {} events, FltFastIo: {} events", io_events.len(), fastio_events.len());

        all_io_events.extend(io_events);
        all_fastio_events.extend(fastio_events);

        if run < num_runs {
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    log::info!(
        "\n=== Combined: {} FltIo, {} FltFastIo ===\n",
        all_io_events.len(),
        all_fastio_events.len()
    );

    analyze_and_compare(&all_io_events, &all_fastio_events);
}

// ── Analysis ───────────────────────────────────────────────────────────────

fn analyze_and_compare(io_events: &[RawEvent], fastio_events: &[RawEvent]) {
    log::info!("=============================================================");
    log::info!("  ANALYSIS: PERF_FLT_IO vs PERF_FLT_FASTIO");
    log::info!("=============================================================\n");

    let io_counts = count_by_opcode(io_events);
    let fastio_counts = count_by_opcode(fastio_events);
    let io_opcodes: HashSet<u8> = io_counts.keys().copied().collect();
    let fastio_opcodes: HashSet<u8> = fastio_counts.keys().copied().collect();

    // ── Distribution table ──
    log::info!("Opcode distribution:");
    log::info!("{:<22} {:>8} {:>8} {:>8}", "Event", "FltIo", "FastIo", "Diff");
    log::info!("{}", "-".repeat(52));
    let mut all_opcodes: Vec<u8> = io_opcodes.union(&fastio_opcodes).copied().collect();
    all_opcodes.sort();
    for op in &all_opcodes {
        let io_c = io_counts.get(op).copied().unwrap_or(0);
        let fio_c = fastio_counts.get(op).copied().unwrap_or(0);
        log::info!(
            "{:<22} {:>8} {:>8} {:>+8}",
            opcode_label(*op),
            io_c,
            fio_c,
            io_c as isize - fio_c as isize
        );
    }

    // ── FltIo-specific opcodes ──
    log::info!("\nFltIo-specific events (opcodes 96-101):");
    for op in 96..=101u8 {
        log::info!(
            "  {:<22} {:>8} {:>8}",
            opcode_label(op),
            io_counts.get(&op).copied().unwrap_or(0),
            fastio_counts.get(&op).copied().unwrap_or(0)
        );
    }

    // ── Heuristic 1: Jaccard Similarity (opcode set overlap) ──
    let intersection: HashSet<u8> = io_opcodes.intersection(&fastio_opcodes).copied().collect();
    let union_set: HashSet<u8> = io_opcodes.union(&fastio_opcodes).copied().collect();
    let jaccard = if union_set.is_empty() {
        0.0
    } else {
        intersection.len() as f64 / union_set.len() as f64
    };
    log::info!("\n[Heuristic 1] Jaccard similarity (opcode sets): {:.3}", jaccard);
    log::info!("  intersection={:?}, io_only={:?}, fastio_only={:?}",
        intersection.iter().map(|o| opcode_label(*o)).collect::<Vec<_>>(),
        io_opcodes.difference(&fastio_opcodes).map(|o| opcode_label(*o)).collect::<Vec<_>>(),
        fastio_opcodes.difference(&io_opcodes).map(|o| opcode_label(*o)).collect::<Vec<_>>(),
    );

    // ── Heuristic 2: Subset checks ──
    let io_subset_of_fastio = io_opcodes.is_subset(&fastio_opcodes);
    let fastio_subset_of_io = fastio_opcodes.is_subset(&io_opcodes);
    log::info!("[Heuristic 2] Subset checks:");
    log::info!("  PERF_FLT_IO ⊆ PERF_FLT_FASTIO: {}", io_subset_of_fastio);
    log::info!("  PERF_FLT_FASTIO ⊆ PERF_FLT_IO: {}", fastio_subset_of_io);

    // ── Heuristic 3: Count-weighted overlap ──
    let mut matched_count = 0u64;
    let mut total_count = 0u64;
    for op in &all_opcodes {
        let io_c = io_counts.get(op).copied().unwrap_or(0) as u64;
        let fio_c = fastio_counts.get(op).copied().unwrap_or(0) as u64;
        matched_count += io_c.min(fio_c);
        total_count += io_c.max(fio_c);
    }
    let count_similarity = if total_count == 0 {
        0.0
    } else {
        matched_count as f64 / total_count as f64
    };
    log::info!("[Heuristic 3] Count-weighted overlap: {:.3}", count_similarity);

    // ── Heuristic 4: Per-opcode count ratio ──
    let mut ratio_sum = 0.0;
    let mut ratio_count = 0usize;
    for op in &all_opcodes {
        let io_c = io_counts.get(op).copied().unwrap_or(0) as f64;
        let fio_c = fastio_counts.get(op).copied().unwrap_or(0) as f64;
        if io_c > 0.0 && fio_c > 0.0 {
            ratio_sum += (io_c / fio_c).min(fio_c / io_c);
            ratio_count += 1;
        }
    }
    let count_ratio_score = if ratio_count == 0 {
        0.0
    } else {
        ratio_sum / ratio_count as f64
    };
    log::info!("[Heuristic 4] Per-opcode count ratio: {:.3}", count_ratio_score);

    // ── Heuristic 5: FltIo-specific opcode presence ──
    let flt_opcodes: HashSet<u8> = (96..=101).collect();
    let io_has_flt: HashSet<u8> = io_opcodes.intersection(&flt_opcodes).copied().collect();
    let fastio_has_flt: HashSet<u8> = fastio_opcodes.intersection(&flt_opcodes).copied().collect();
    let flt_both = io_has_flt == fastio_has_flt;
    log::info!("[Heuristic 5] FltIo opcodes (96-101) identical: {}", flt_both);
    log::info!("  io_has={:?}, fastio_has={:?}",
        io_has_flt.iter().map(|o| opcode_label(*o)).collect::<Vec<_>>(),
        fastio_has_flt.iter().map(|o| opcode_label(*o)).collect::<Vec<_>>(),
    );

    // ── Heuristic 6: Timestamp correlation for matched opcodes ──
    // For opcodes present in both sessions, check if event counts are proportional
    let mut count_ratios: Vec<(u8, f64)> = Vec::new();
    for op in &all_opcodes {
        let io_c = io_counts.get(op).copied().unwrap_or(0) as f64;
        let fio_c = fastio_counts.get(op).copied().unwrap_or(0) as f64;
        if io_c > 0.0 && fio_c > 0.0 {
            count_ratios.push((*op, io_c / fio_c));
        }
    }
    log::info!("[Heuristic 6] Count ratios (FltIo/FastIo) per opcode:");
    for (op, ratio) in &count_ratios {
        log::info!("  {:<22} {:.3}", opcode_label(*op), ratio);
    }

    // ── Final Verdict ──
    log::info!("\n=============================================================");
    log::info!("  FINAL VERDICT");
    log::info!("=============================================================\n");

    log::info!("Confidence scores:");
    let identical = io_opcodes == fastio_opcodes;
    log::info!("  Identical opcode sets:    {} (score: {:.1}%)", identical, if identical { 100.0 } else { 0.0 });
    log::info!("  Jaccard similarity:       {:.1}%", jaccard * 100.0);
    log::info!("  Count-weighted overlap:   {:.1}%", count_similarity * 100.0);
    log::info!("  FltIo opcodes identical:  {} (score: {:.1}%)", flt_both, if flt_both { 100.0 } else { 0.0 });
    log::info!("  PERF_FLT_IO ⊆ FAST_IO:   {}", io_subset_of_fastio);
    log::info!("  FAST_IO ⊆ PERF_FLT_IO:   {}", fastio_subset_of_io);

    log::info!("\nConclusion:");
    if identical {
        log::info!("  Answer: One is a subset of another (they are IDENTICAL).");
        log::info!("  Both PERF_FLT_IO and PERF_FLT_FASTIO enable the exact same set of events.");
        log::info!("  The flags appear to be aliases or both enable the same underlying bitmask.");
    } else if io_subset_of_fastio {
        log::info!("  Answer: One is a subset of another.");
        log::info!("  PERF_FLT_IO is a SUBSET of PERF_FLT_FASTIO.");
        log::info!("  PERF_FLT_FASTIO includes everything in PERF_FLT_IO plus additional events.");
    } else if fastio_subset_of_io {
        log::info!("  Answer: One is a subset of another.");
        log::info!("  PERF_FLT_FASTIO is a SUBSET of PERF_FLT_IO.");
        log::info!("  PERF_FLT_IO includes everything in PERF_FLT_FASTIO plus additional events.");
    } else if intersection.is_empty() {
        log::info!("  Answer: The two's events are EXCLUSIVE.");
        log::info!("  No overlap in enabled opcodes.");
    } else {
        log::info!("  Answer: There are some collisions but neither is a subset.");
        log::info!("  Shared opcodes: {:?}",
            intersection.iter().map(|o| opcode_label(*o)).collect::<Vec<_>>());
        log::info!("  FltIo only: {:?}",
            io_opcodes.difference(&fastio_opcodes).map(|o| opcode_label(*o)).collect::<Vec<_>>());
        log::info!("  FastIo only: {:?}",
            fastio_opcodes.difference(&io_opcodes).map(|o| opcode_label(*o)).collect::<Vec<_>>());
    }
}
