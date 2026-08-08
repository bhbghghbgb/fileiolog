use serde::{Deserialize, Serialize};

/// A received FltIoCompletion event from ETW
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub opcode: u8,
    pub event_id: u16,
    pub version: u8,
    pub timestamp: u64,
    pub process_id: u32,
    pub thread_id: u32,
    pub flt: FltIoCompletionEvent,
}

impl RawEvent {
    /// Fast I/O operations carry no real IRP, so IrpPtr is 0.
    pub fn is_fast(&self) -> bool {
        self.flt.irp_ptr == 0
    }

    pub fn major_function(&self) -> u32 {
        self.flt.major_function
    }
}

/// Parsed FltIoCompletion event data (opcodes 98/99)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FltIoCompletionEvent {
    pub initial_time: u64,
    pub routine_addr: usize,
    pub file_object: usize,
    pub file_context: usize,
    pub irp_ptr: usize,
    pub callback_data_ptr: usize,
    pub major_function: u32,
}

/// Group mask index: (value >> 29) & 0x07
fn group_index(value: u32) -> usize {
    ((value >> 29) & 0x07) as usize
}

/// Build a PERFINFO_GROUPMASK from a combined mask value
pub fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    masks[group_index(mask_value)] = mask_value;
    masks
}

/// The trace configuration under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Config {
    FastIoOnly,
    IoOnly,
    Both,
}

impl Config {
    pub const ALL: [Config; 3] = [Config::FastIoOnly, Config::IoOnly, Config::Both];

    pub fn name(self) -> &'static str {
        match self {
            Config::FastIoOnly => "FASTIO",
            Config::IoOnly => "IO",
            Config::Both => "BOTH",
        }
    }

    /// PERFINFO group value for this configuration.
    pub fn group_value(self) -> u32 {
        const PERF_FLT_IO: u32 = 0x80100000;
        const PERF_FLT_FASTIO: u32 = 0x80200000;
        match self {
            Config::FastIoOnly => PERF_FLT_FASTIO,
            Config::IoOnly => PERF_FLT_IO,
            Config::Both => PERF_FLT_IO | PERF_FLT_FASTIO,
        }
    }
}