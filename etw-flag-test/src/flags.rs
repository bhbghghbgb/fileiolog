#![allow(dead_code)]

/// EnableFlags constants for kernel trace sessions.
///
/// These are the documented flags that can be set in EVENT_TRACE_PROPERTIES.EnableFlags.
/// Values from: https://learn.microsoft.com/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_properties
pub mod enable_flags {
    /// Enables FileIo_Name events (event types 0, 32, 35, 36).
    /// Requires EVENT_TRACE_FLAG_DISK_IO to also be set for disk-level correlation.
    pub const EVENT_TRACE_FLAG_DISK_FILE_IO: u32 = 0x00000200;

    /// Enables FileIo_OpEnd events (event type 76).
    pub const EVENT_TRACE_FLAG_FILE_IO: u32 = 0x02000000;

    /// Enables FileIo_Create, FileIo_SimpleOp, FileIo_ReadWrite, FileIo_Info, FileIo_DirEnum events.
    /// Covers event types: 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 77.
    pub const EVENT_TRACE_FLAG_FILE_IO_INIT: u32 = 0x04000000;

    /// Enables MapFile/UnmapFile events (event types 37, 38, 39, 40) via Masks[0].
    /// On Windows 8+ this maps to PERF_VAMAP.
    pub const EVENT_TRACE_FLAG_VAMAP: u32 = 0x00008000;
}

/// PERFINFO_GROUPMASK related constants.
///
/// The PERFINFO_GROUPMASK extends EnableFlags from 32 bits to 256 bits (8 x u32).
/// Each Masks[] element holds bits for a group of related event types.
/// The high 3 bits of each flag value encode the group index.
///
/// Reference: https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntwmi/perfinfo_groupmask.htm
pub mod group_mask {
    /// Bit extraction: index = (value & 0xe0000000) >> 29
    pub const PERF_MASK_INDEX: u32 = 0xe0000000;
    /// Bit extraction: group = value & ~PERF_MASK_INDEX
    pub const PERF_MASK_GROUP: u32 = !PERF_MASK_INDEX;

    // ── Masks[0] ────────────────────────────────────────────────────
    // These are equivalent to EnableFlags (the low 32 bits).

    /// PERF_FILE_IO = EVENT_TRACE_FLAG_FILE_IO
    /// Enables FileIo_OpEnd (event type 76).
    pub const PERF_FILE_IO: u32 = 0x02000000;

    /// PERF_FILE_IO_INIT = EVENT_TRACE_FLAG_FILE_IO_INIT
    /// Enables Create, SimpleOp, ReadWrite, Info, DirEnum events.
    pub const PERF_FILE_IO_INIT: u32 = 0x04000000;

    /// PERF_FILENAME = EVENT_TRACE_FLAG_DISK_FILE_IO
    /// Enables FileIo_Name events (0, 32, 35, 36).
    pub const PERF_FILENAME: u32 = 0x00000200;

    /// PERF_VAMAP = EVENT_TRACE_FLAG_VAMAP
    /// Enables MapFile/UnmapFile events (37, 38, 39, 40).
    pub const PERF_VAMAP: u32 = 0x00008000;

    // ── Masks[4] ────────────────────────────────────────────────────
    // Minifilter-related flags (undocumented for enabling via ETW).

    /// PERF_FLT_IO_INIT - Enables FltIoInit events (96, 97).
    /// Group index 4, value 0x80080000.
    pub const PERF_FLT_IO_INIT: u32 = 0x80080000;

    /// PERF_FLT_IO - Enables FltIoCompletion events (98, 99).
    /// Group index 4, value 0x80100000.
    pub const PERF_FLT_IO: u32 = 0x80100000;

    /// PERF_FLT_FASTIO - Enables minifilter fastio callback events.
    /// Group index 4, value 0x80200000.
    pub const PERF_FLT_FASTIO: u32 = 0x80200000;

    /// PERF_FLT_IO_FAILURE - Enables FltIoFailure events (100, 101).
    /// Group index 4, value 0x80400000.
    pub const PERF_FLT_IO_FAILURE: u32 = 0x80400000;

    // ── Masks[0] additional (for completeness) ───────────────────────

    /// PERF_DISK_IO_INIT - Enables DiskIo_TypeGroup2 events.
    /// Group index 0, value 0x00000400.
    pub const PERF_DISK_IO_INIT: u32 = 0x00000400;

    /// PERF_DISK_IO - Enables DiskIo_TypeGroup1 and TypeGroup3 events.
    /// Group index 0, value 0x00000300.
    pub const PERF_DISK_IO: u32 = 0x00000300;

    /// Helper: set a group mask bit in a PERFINFO_GROUPMASK Masks[] array.
    pub fn set_mask(masks: &mut [u32; 8], flag: u32) {
        let index = ((flag & PERF_MASK_INDEX) >> 29) as usize;
        masks[index] |= flag & PERF_MASK_GROUP;
    }

    /// Helper: clear all masks.
    pub fn clear_masks(masks: &mut [u32; 8]) {
        masks.fill(0);
    }

    /// Helper: create a PERFINFO_GROUPMASK with a single flag set.
    pub fn single(flag: u32) -> [u32; 8] {
        let mut masks = [0u32; 8];
        set_mask(&mut masks, flag);
        masks
    }
}

/// A test configuration: either an EnableFlags value or a PERFINFO_GROUPMASK.
#[derive(Debug, Clone)]
pub enum TestConfig {
    /// Test with EnableFlags only (documented path).
    EnableFlags(u32),
    /// Test with PERFINFO_GROUPMASK (undocumented extended path).
    GroupMask([u32; 8]),
}

impl TestConfig {
    pub fn name(&self) -> String {
        match self {
            Self::EnableFlags(flags) => format!("EnableFlags=0x{:08X}", flags),
            Self::GroupMask(masks) => {
                let non_zero: Vec<String> = masks
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| **v != 0u32)
                    .map(|(i, &v)| format!("Masks[{}]=0x{:08X}", i, v))
                    .collect();
                format!("GroupMask({})", non_zero.join(", "))
            }
        }
    }

    /// All individual FileIo-related test configurations to iterate over.
    pub fn fileio_test_cases() -> Vec<(String, Self)> {
        use enable_flags::*;
        use group_mask::*;

        vec![
            // ── EnableFlags tests ──
            (
                "EVENT_TRACE_FLAG_FILE_IO_INIT".into(),
                Self::EnableFlags(EVENT_TRACE_FLAG_FILE_IO_INIT),
            ),
            (
                "EVENT_TRACE_FLAG_FILE_IO".into(),
                Self::EnableFlags(EVENT_TRACE_FLAG_FILE_IO),
            ),
            (
                "EVENT_TRACE_FLAG_DISK_FILE_IO".into(),
                Self::EnableFlags(EVENT_TRACE_FLAG_DISK_FILE_IO),
            ),
            (
                "EVENT_TRACE_FLAG_VAMAP".into(),
                Self::EnableFlags(EVENT_TRACE_FLAG_VAMAP),
            ),
            // Combined flags
            (
                "FILE_IO_INIT + FILE_IO".into(),
                Self::EnableFlags(EVENT_TRACE_FLAG_FILE_IO_INIT | EVENT_TRACE_FLAG_FILE_IO),
            ),
            (
                "FILE_IO_INIT + FILE_IO + DISK_FILE_IO".into(),
                Self::EnableFlags(
                    EVENT_TRACE_FLAG_FILE_IO_INIT | EVENT_TRACE_FLAG_FILE_IO | EVENT_TRACE_FLAG_DISK_FILE_IO,
                ),
            ),
            (
                "ALL_FILEIO_FLAGS".into(),
                Self::EnableFlags(
                    EVENT_TRACE_FLAG_FILE_IO_INIT
                        | EVENT_TRACE_FLAG_FILE_IO
                        | EVENT_TRACE_FLAG_DISK_FILE_IO
                        | EVENT_TRACE_FLAG_VAMAP,
                ),
            ),
            // ── GroupMask tests (undocumented) ──
            (
                "PERF_FLT_IO_INIT".into(),
                Self::GroupMask(single(PERF_FLT_IO_INIT)),
            ),
            (
                "PERF_FLT_IO".into(),
                Self::GroupMask(single(PERF_FLT_IO)),
            ),
            (
                "PERF_FLT_IO_FAILURE".into(),
                Self::GroupMask(single(PERF_FLT_IO_FAILURE)),
            ),
            (
                "PERF_FLT_FASTIO".into(),
                Self::GroupMask(single(PERF_FLT_FASTIO)),
            ),
            (
                "ALL_FLT_MASKS".into(),
                Self::GroupMask({
                    let mut m = [0u32; 8];
                    set_mask(&mut m, PERF_FLT_IO_INIT);
                    set_mask(&mut m, PERF_FLT_IO);
                    set_mask(&mut m, PERF_FLT_IO_FAILURE);
                    set_mask(&mut m, PERF_FLT_FASTIO);
                    m
                }),
            ),
            (
                "ALL_FILEIO_MASKS".into(),
                Self::GroupMask({
                    let mut m = [0u32; 8];
                    set_mask(&mut m, PERF_FILE_IO);
                    set_mask(&mut m, PERF_FILE_IO_INIT);
                    set_mask(&mut m, PERF_FILENAME);
                    set_mask(&mut m, PERF_VAMAP);
                    set_mask(&mut m, PERF_FLT_IO_INIT);
                    set_mask(&mut m, PERF_FLT_IO);
                    set_mask(&mut m, PERF_FLT_IO_FAILURE);
                    set_mask(&mut m, PERF_FLT_FASTIO);
                    m
                }),
            ),
        ]
    }
}
