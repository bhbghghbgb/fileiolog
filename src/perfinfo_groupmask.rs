/// PERFINFO_GROUPMASK constants and builder macro for extended kernel trace flags.
///
/// The `PERFINFO_GROUPMASK` extends the 32-bit `EnableFlags` to 256 bits (8 × 32)
/// by partitioning flags into 8 groups. Each flag's high 3 bits encode its group index:
///
/// - Group 0 (0x00xxxxxx): mirrors `EnableFlags` (EVENT_TRACE_FLAG_*)
/// - Group 1 (0x20xxxxxx): context switch, DPC, interrupt, dispatcher, profile, etc.
/// - Group 2 (0x40xxxxxx): syscall, heap, timer, idle, etc.
/// - Group 3 (0x60xxxxxx): (reserved, no flags defined yet)
/// - Group 4 (0x80xxxxxx): optical IO, object handles, minifilter, WDF, etc.
/// - Group 5 (0xA0xxxxxx): hibernate rundown
/// - Group 6 (0xC0xxxxxx): sysconfig groups
/// - Group 7 (0xE0xxxxxx): cluster, memory control
///
/// # Example
///
/// ```ignore
/// use fileiolog::perfinfo_groupmask::{group_mask, PERF_FLT_IO_INIT, PERF_FLT_IO};
///
/// let mask: [u32; 8] = group_mask![PERF_FLT_IO_INIT, PERF_FLT_IO];
/// // mask[4] will be 0x80180000 (both flags are in group 4)
/// ```

// ── Masks[0]: mirrors EnableFlags ──────────────────────────────

pub const PERF_PROCESS: u32 = 0x00000001;
pub const PERF_THREAD: u32 = 0x00000002;
pub const PERF_LOADER: u32 = 0x00000004;
pub const PERF_PERF_COUNTER: u32 = 0x00000008;
pub const PERF_DISK_IO: u32 = 0x00000300;
pub const PERF_FILENAME: u32 = 0x00000200;
pub const PERF_DISK_FILE_IO: u32 = 0x00000200; // alias for PERF_FILENAME
pub const PERF_DISK_IO_INIT: u32 = 0x00000400;
pub const PERF_ALL_FAULTS: u32 = 0x00001000;
pub const PERF_HARD_FAULTS: u32 = 0x00002000;
pub const PERF_VAMAP: u32 = 0x00008000;
pub const PERF_NETWORK: u32 = 0x00010000;
pub const PERF_REGISTRY: u32 = 0x00020000;
pub const PERF_DBGPRINT: u32 = 0x00040000;
pub const PERF_JOB: u32 = 0x00080000;
pub const PERF_ALPC: u32 = 0x00100000;
pub const PERF_SPLIT_IO: u32 = 0x00200000;
pub const PERF_DEBUG_EVENTS: u32 = 0x00400000;
pub const PERF_FILE_IO: u32 = 0x02000000;
pub const PERF_FILE_IO_INIT: u32 = 0x04000000;
pub const PERF_NO_SYSCONFIG: u32 = 0x10000000;

// ── Masks[1] ──────────────────────────────────────────────────

pub const PERF_MEMORY: u32 = 0x20000001;
pub const PERF_PROFILE: u32 = 0x20000002;
pub const PERF_CONTEXT_SWITCH: u32 = 0x20000004;
pub const PERF_FOOTPRINT: u32 = 0x20000008;
pub const PERF_DRIVERS: u32 = 0x20000010;
pub const PERF_POOL: u32 = 0x20000040;
pub const PERF_POOLTRACE: u32 = 0x20000041;
pub const PERF_DPC: u32 = 0x20000080;
pub const PERF_COMPACT_CSWITCH: u32 = 0x20000100;
pub const PERF_DISPATCHER: u32 = 0x20000200;
pub const PERF_PMC_PROFILE: u32 = 0x20000400;
pub const PERF_PROFILING: u32 = 0x20000402;
pub const PERF_PROCESS_INSWAP: u32 = 0x20000800;
pub const PERF_AFFINITY: u32 = 0x20001000;
pub const PERF_PRIORITY: u32 = 0x20002000;
pub const PERF_INTERRUPT: u32 = 0x20004000;
pub const PERF_VIRTUAL_ALLOC: u32 = 0x20008000;
pub const PERF_SPINLOCK: u32 = 0x20010000;
pub const PERF_SYNC_OBJECTS: u32 = 0x20020000;
pub const PERF_DPC_QUEUE: u32 = 0x20040000;
pub const PERF_MEMINFO: u32 = 0x20080000;
pub const PERF_CONTMEM_GEN: u32 = 0x20100000;
pub const PERF_SPINLOCK_CNTRS: u32 = 0x20200000;
pub const PERF_SPININSTR: u32 = 0x20210000;
pub const PERF_SESSION: u32 = 0x20400000;
pub const PERF_MEMINFO_WS: u32 = 0x20800000;
pub const PERF_KERNEL_QUEUE: u32 = 0x21000000;
pub const PERF_INTERRUPT_STEER: u32 = 0x22000000;
pub const PERF_SHOULD_YIELD: u32 = 0x24000000;
pub const PERF_WS: u32 = 0x28000000;

// ── Masks[2] ──────────────────────────────────────────────────

pub const PERF_ANTI_STARVATION: u32 = 0x40000001;
pub const PERF_PROCESS_FREEZE: u32 = 0x40000002;
pub const PERF_PFN_LIST: u32 = 0x40000004;
pub const PERF_WS_DETAIL: u32 = 0x40000008;
pub const PERF_WS_ENTRY: u32 = 0x40000010;
pub const PERF_HEAP: u32 = 0x40000020;
pub const PERF_SYSCALL: u32 = 0x40000040;
pub const PERF_UMS: u32 = 0x40000080;
pub const PERF_BACKTRACE: u32 = 0x40000100;
pub const PERF_VULCAN: u32 = 0x40000200;
pub const PERF_OBJECTS: u32 = 0x40000400;
pub const PERF_EVENTS: u32 = 0x40000800;
pub const PERF_FULLTRACE: u32 = 0x40001000;
pub const PERF_DFSS: u32 = 0x40002000;
pub const PERF_PREFETCH: u32 = 0x40004000;
pub const PERF_PROCESSOR_IDLE: u32 = 0x40008000;
pub const PERF_CPU_CONFIG: u32 = 0x40010000;
pub const PERF_TIMER: u32 = 0x40020000;
pub const PERF_CLOCK_INTERRUPT: u32 = 0x40040000;
pub const PERF_LOAD_BALANCER: u32 = 0x40080000;
pub const PERF_CLOCK_TIMER: u32 = 0x40100000;
pub const PERF_IDLE_SELECTION: u32 = 0x40200000;
pub const PERF_IPI: u32 = 0x40400000;
pub const PERF_IO_TIMER: u32 = 0x40800000;
pub const PERF_REG_HIVE: u32 = 0x41000000;
pub const PERF_REG_NOTIF: u32 = 0x42000000;
pub const PERF_PPM_EXIT_LATENCY: u32 = 0x44000000;
pub const PERF_WORKER_THREAD: u32 = 0x48000000;

// ── Masks[3]: reserved (no flags defined) ─────────────────────

// ── Masks[4] ──────────────────────────────────────────────────

pub const PERF_OPTICAL_IO: u32 = 0x80000001;
pub const PERF_OPTICAL_IO_INIT: u32 = 0x80000002;
pub const PERF_DLL_INFO: u32 = 0x80000008;
pub const PERF_DLL_FLUSH_WS: u32 = 0x80000010;
pub const PERF_OB_HANDLE: u32 = 0x80000040;
pub const PERF_OB_OBJECT: u32 = 0x80000080;
pub const PERF_WAKE_DROP: u32 = 0x80000200;
pub const PERF_WAKE_EVENT: u32 = 0x80000400;
pub const PERF_DEBUGGER: u32 = 0x80000800;
pub const PERF_PROC_ATTACH: u32 = 0x80001000;
pub const PERF_WAKE_COUNTER: u32 = 0x80002000;
pub const PERF_POWER: u32 = 0x80008000;
pub const PERF_SOFT_TRIM: u32 = 0x80010000;
pub const PERF_CC: u32 = 0x80020000;
pub const PERF_FLT_IO_INIT: u32 = 0x80080000;
pub const PERF_FLT_IO: u32 = 0x80100000;
pub const PERF_FLT_FASTIO: u32 = 0x80200000;
pub const PERF_FLT_IO_FAILURE: u32 = 0x80400000;
pub const PERF_HV_PROFILE: u32 = 0x80800000;
pub const PERF_WDF_DPC: u32 = 0x81000000;
pub const PERF_WDF_INTERRUPT: u32 = 0x82000000;
pub const PERF_CACHE_FLUSH: u32 = 0x84000000;

// ── Masks[5] ──────────────────────────────────────────────────

pub const PERF_HIBER_RUNDOWN: u32 = 0xA0000001;

// ── Masks[6] ──────────────────────────────────────────────────

pub const PERF_SYSCFG_SYSTEM: u32 = 0xC0000001;
pub const PERF_SYSCFG_GRAPHICS: u32 = 0xC0000002;
pub const PERF_SYSCFG_STORAGE: u32 = 0xC0000004;
pub const PERF_SYSCFG_NETWORK: u32 = 0xC0000008;
pub const PERF_SYSCFG_SERVICES: u32 = 0xC0000010;
pub const PERF_SYSCFG_PNP: u32 = 0xC0000020;
pub const PERF_SYSCFG_OPTICAL: u32 = 0xC0000040;
pub const PERF_SYSCFG_ALL: u32 = 0xDFFFFFFF;

// ── Masks[7] ──────────────────────────────────────────────────

pub const PERF_CLUSTER_OFF: u32 = 0xE0000001;
pub const PERF_MEMORY_CONTROL: u32 = 0xE0000002;

// ── Build a PERFINFO_GROUPMASK from flag constants ─────────────
//
// Usage:
//
//   let mask = group_mask![PERF_FLT_IO_INIT, PERF_FLT_IO, PERF_FLT_IO_FAILURE];
//
// Each identifier must be a `u32` constant whose high 3 bits encode its
// group index (0..7).  The macro extracts the group and ORs the value
// into the correct element of the `[u32; 8]` array.

/// Build a `[u32; 8]` PERFINFO_GROUPMASK from one or more flag constants.
///
/// Each argument must be a `u32` constant whose high 3 bits encode its
/// group index.  The macro extracts the group index at compile time and
/// ORs the value into the correct element of the returned array.
///
/// # Examples
///
/// ```ignore
/// // Single flag — most common case, no boilerplate:
/// let mask = group_mask![PERF_FLT_IO_INIT];
///
/// // Multiple flags in the same group:
/// let mask = group_mask![PERF_FLT_IO_INIT, PERF_FLT_IO, PERF_FLT_IO_FAILURE];
///
/// // Flags across different groups:
/// let mask = group_mask![PERF_FILE_IO_INIT, PERF_FLT_IO];
/// ```
#[macro_export]
macro_rules! group_mask {
    ( $($flag:expr),+ $(,)? ) => {{
        let mut masks = [0u32; 8];
        $(
            // The high 3 bits of each flag encode its group index (0..7).
            // We shift right by 29 to obtain the group, then OR in.
            masks[((($flag) >> 29) & 0x07) as usize] |= $flag;
        )+
        masks
    }};
    () => { [0u32; 8] };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_flag_group4() {
        let m = group_mask![PERF_FLT_IO_INIT];
        assert_eq!(m[4], 0x80080000);
        assert_eq!(m.iter().filter(|&&v| v != 0).count(), 1);
    }

    #[test]
    fn multiple_flags_same_group() {
        let m = group_mask![PERF_FLT_IO_INIT, PERF_FLT_IO, PERF_FLT_IO_FAILURE];
        assert_eq!(m[4], 0x80080000 | 0x80100000 | 0x80400000);
    }

    #[test]
    fn flags_across_groups() {
        let m = group_mask![PERF_FILE_IO_INIT, PERF_FLT_IO];
        assert_eq!(m[0], 0x04000000);
        assert_eq!(m[4], 0x80100000);
    }

    #[test]
    fn empty_mask() {
        let m = group_mask![];
        assert_eq!(m, [0u32; 8]);
    }
}
