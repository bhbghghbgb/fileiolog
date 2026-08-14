# PERFINFO_GROUPMASK

The PERFINFO_GROUPMASK describes the types of event that are enabled or are to be enabled in an NT Kernel Logger session.

## Usage

Historically, and as far as anyone still might know from Microsoft's documentation, the choice of events to enable in NT Kernel Logger sessions is managed from user mode through the EnableFlags member of the EVENT_TRACE_PROPERTIES structure. This is most of the input and output for such documented API functions as StartTrace and ControlTrace. Bits within the EnableFlags select types of event. The PERFINFO_GROUPMASK greatly extends this choice, by allowing for very many more bits, and thus types of event.

The modern and arguably cleanest way to use a PERFINFO_GROUPMASK is through the [TraceSetInformation](./tracesetinformation.md) functions. The PERFINFO_GROUPMASK structure is what this function produces as output or expects as input in its information buffer when given the information class TraceSystemTraceEnableFlagsInfo (0x04).

Beneath these user-mode API functions, the PERFINFO_GROUPMASK is part of the [EVENT_TRACE_GROUPMASK_INFORMATION](./event_trace_groupmask_information.md) structure that is expected by the ZwQuerySystemInformation or NtQuerySystemInformation functions and the ZwSetSystemInformation or NtSetSystemInformation functions when given their information class SystemPerformanceTraceInformation (0x1F) if the first dword in the information buffer on input is EventTraceGroupMaskInformation (0x01).

Internally, the kernel keeps an array of PERFINFO_GROUPMASK structures, one for each possible NT Kernel Logger session (the present capacity being eight), and a single structure that acts as a union of the others for quickly reckoning which types of event are enabled in at least one session.

## Documentation Status

The PERFINFO_GROUPMASK structure is not documented. Until relatively recently, it was known only from type information in public symbol files (in Windows Vista and higher). A C-language definition has been published by Microsoft in NTWMI.H from the Windows Driver Kit (WDK) for the original release of Windows 10 and for Version 1511. This disclosure is not repeated in subsequent editions and is here thought to be an oversight. Still, published it is, which means that this note uses Microsoft's names throughout, notably for the many bits that Microsoft has yet defined for types of event that can be monitored through an NT Kernel Logger.

## Layout

The PERFINFO_GROUPMASK structure is 0x20 bytes in both 32-bit and 64-bit Windows for all versions 5.2 and higher, but is 0x24 bytes in version 5.1. Symbol files for version 6.0 and higher show just the one member:

| Offset | Definition | Versions |
|--------|------------|----------|
| 0x00 | ULONG Masks[8]; | 5.2 and higher |
| 0x20 | unknown dword | 5.1 only |

The PERFINFO_GROUPMASK in version 5.1 certainly is nine dwords but the purpose of the last is not understood much beyond knowing that it is not treated just as an extra mask as if to allow for more types of event. That the last dword is formally separated from the Masks is mere supposition.

## Masks

The first point to the PERFINFO_GROUPMASK is simply that its eight array elements allow 256 bits instead of the 32 that can be passed to and fro in the EnableFlags. A second point to the PERFINFO_GROUPMASK is that each ULONG is a group of bits for enabling similar types of events. It will often be wanted that multiple types of event in the one group are enabled concurrently. This is helped by adopting a conventional representation of each supported type of event by one 32-bit value in which the high 3 bits encode the group's index within the array.

For instance, to enable the somewhat quirky tracing of what may be going wrong with the ERESOURCE structures that provide for synchronising exclusive and shared ownership of abstracted kernel-mode resources, the kernel needs to see that the 0x00020000 bit is set in Masks[1]. The conventional representation is the one symbol PERF_SYNC_OBJECTS whose numerical value is 0x20020000.

The merit of this convention is that values that have the same index—corresponding to types of events that are in the same group—are easily combined. For instance, two types of events for instrumenting spin locks are in the same group. Their numerical representations PERF_SPINLOCK (0x20010000) and PERF_SPINLOCK_CNTRS (0x20200000) can be combined as 0x20210000, for which Microsoft helpfully defines the symbol PERF_SPININSTR.

Macros in the semi-secret NTWMI.H provide the programmers of ETW clients with the easy means to set or clear the desired bit in the desired Masks element of their PERFINFO_GROUPMASK by supplying the address, of course, and just one symbol to select the bit. For the kernel's programmers, the same header defines an inline routine that tests for the bit, again just from one symbol and the address.

### Masks[0]

The first element arguably is the EnableFlags:

| Value | Name | Equivalent in EnableFlags | Versions |
|-------|------|---------------------------|----------|
| 0x00000001 | PERF_PROCESS | EVENT_TRACE_FLAG_PROCESS | 5.0 and higher |
| 0x00000002 | PERF_THREAD | EVENT_TRACE_FLAG_THREAD | 5.0 and higher |
| 0x00000003 | PERF_PROC_THREAD | | |
| 0x00000004 | PERF_LOADER | EVENT_TRACE_FLAG_IMAGE_LOAD | 5.0 and higher |
| 0x00000008 | PERF_PERF_COUNTER | EVENT_TRACE_FLAG_PROCESS_COUNTERS | 6.0 and higher |
| 0x00000010 | maps to PERF_CONTEXT_SWITCH in Mask[1] | EVENT_TRACE_FLAG_CSWITCH | 6.0 and higher |
| 0x00000020 | maps to PERF_DPC in Mask[1] | EVENT_TRACE_FLAG_DPC | 6.0 and higher |
| 0x00000040 | maps to PERF_INTERRUPT in Mask[1] | EVENT_TRACE_FLAG_INTERRUPT | 6.0 and higher |
| 0x00000080 | maps to PERF_SYSCALL in Mask[2] | EVENT_TRACE_FLAG_SYSTEMCALL | 6.0 and higher |
| 0x00000100 | | EVENT_TRACE_FLAG_DISK_IO | 5.0 and higher |
| 0x00000200 | PERF_FILENAME | EVENT_TRACE_FLAG_DISK_FILE_IO | 5.0 and higher |
| 0x00000300 | PERF_DISK_IO | | |
| 0x00000400 | PERF_DISK_IO_INIT | EVENT_TRACE_FLAG_DISK_IO_INIT | 6.0 and higher |
| 0x00000800 | maps to PERF_DISPATCHER in Mask[1] | EVENT_TRACE_FLAG_DISPATCHER | 6.1 and higher |
| 0x00001000 | PERF_ALL_FAULTS | EVENT_TRACE_FLAG_MEMORY_PAGE_FAULTS | 5.0 and higher |
| 0x00002000 | PERF_HARD_FAULTS | EVENT_TRACE_FLAG_MEMORY_HARD_FAULTS | 5.0 and higher |
| 0x00004000 | maps to PERF_VIRTUAL_ALLOC in Mask[1] | EVENT_TRACE_FLAG_VIRTUAL_ALLOC | 6.1 and higher |
| 0x00008000 | PERF_VAMAP | EVENT_TRACE_FLAG_VAMAP | 6.2 and higher |
| 0x00010000 | PERF_NETWORK | EVENT_TRACE_FLAG_NETWORK_TCPIP | 5.0 and higher |
| 0x00020000 | PERF_REGISTRY | EVENT_TRACE_FLAG_REGISTRY | 5.0 and higher |
| 0x00040000 | PERF_DBGPRINT | EVENT_TRACE_FLAG_DBGPRINT | 5.1 and higher |
| 0x00080000 | PERF_JOB | EVENT_TRACE_FLAG_JOB | 10.0 and higher |
| 0x00100000 | PERF_ALPC | EVENT_TRACE_FLAG_ALPC | 6.0 and higher |
| 0x00200000 | PERF_SPLIT_IO | EVENT_TRACE_FLAG_VOLMGR | late 5.2 only; EVENT_TRACE_FLAG_SPLIT_IO | 6.0 and higher |
| 0x00400000 | PERF_DEBUG_EVENTS | EVENT_TRACE_FLAG_DEBUG_EVENTS | 10.0 and higher |
| 0x00800000 | maps to PERF_DRIVERS in Mask[1] | EVENT_TRACE_FLAG_DRIVER | 6.0 and higher |
| 0x01000000 | maps to PERF_PROFILE in Mask[1] | EVENT_TRACE_FLAG_PROFILE | 6.0 and higher |
| 0x02000000 | PERF_FILE_IO | EVENT_TRACE_FLAG_FILE_IO | 6.0 and higher |
| 0x04000000 | PERF_FILE_IO_INIT | EVENT_TRACE_FLAG_FILE_IO_INIT | 6.0 and higher |
| 0x08000000 | apparently unused | | |
| 0x10000000 | PERF_NO_SYSCONFIG | EVENT_TRACE_FLAG_NO_SYSCONFIG | 6.2 and higher |
| 0x20000000 | | EVENT_TRACE_FLAG_ENABLE_RESERVE | 5.0 and higher |
| 0x40000000 | | EVENT_TRACE_FLAG_FORWARD_WMI | 5.0 and higher |
| 0x80000000 | | EVENT_TRACE_FLAG_EXTENSION | 5.0 and higher |

Indication of support for version 5.0 applies only to the bit in the EnableFlags, as the precursor to the PERFINFO_GROUPMASK.

Please note a general caution about versions that are indicated for any of these masks, above and below. This analysis is preliminary at best. That a version is indicated does mean that use is known in those versions. It does not mean that earlier versions have no support, just that I don't know of it or that I haven't yet recorded it here. It's a rough indication only. If it ever seems that presenting it is not better than nothing, it will be withdrawn.

### Masks[1]

| Value | Name | Versions |
|-------|------|----------|
| 0x20000001 | PERF_MEMORY | 5.1 and higher |
| 0x20000002 | PERF_PROFILE | 5.1 and higher |
| 0x20000004 | PERF_CONTEXT_SWITCH | 5.1 and higher |
| 0x20000008 | PERF_FOOTPRINT | 5.1 and higher |
| 0x20000010 | PERF_DRIVERS | 5.1 and higher |
| 0x20000020 | | |
| PERF_REFSET | | |
| 0x20000040 | | |
| PERF_POOL | 6.1 and higher |
| 0x20000041 | PERF_POOLTRACE | 6.1 and higher |
| 0x20000080 | PERF_DPC | 5.1 and higher |
| 0x20000100 | | |
| PERF_COMPACT_CSWITCH | 6.0 and higher |
| 0x20000200 | PERF_DISPATCHER | 6.0 and higher |
| 0x20000400 | PERF_PMC_PROFILE | 6.2 and higher |
| 0x20000402 | PERF_PROFILING | 6.2 and higher |
| 0x20000800 | PERF_PROCESS_INSWAP | 6.1 and higher |
| 0x20001000 | unknown | 5.1 to 5.2 |
| PERF_AFFINITY | 6.1 and higher |
| 0x20002000 | PERF_PRIORITY | 6.0 and higher |
| 0x20004000 | PERF_INTERRUPT | 5.1 and higher |
| 0x20008000 | PERF_VIRTUAL_ALLOC | 6.0 and higher |
| 0x20010000 | PERF_SPINLOCK | 6.1 and higher (x64); 6.2 and higher (x86) |
| 0x20020000 | PERF_SYNC_OBJECTS | 6.1 and higher |
| 0x20040000 | PERF_DPC_QUEUE | 6.2 and higher |
| 0x20080000 | PERF_MEMINFO | 6.0 and higher |
| 0x20100000 | PERF_CONTMEM_GEN | 6.0 and higher |
| 0x20200000 | PERF_SPINLOCK_CNTRS | 6.2 and higher |
| 0x20210000 | PERF_SPININSTR | 6.2 and higher |
| 0x20400000 | PERF_SESSION; PERF_PFSECTION | 6.2 and higher |
| 0x20800000 | PERF_MEMINFO_WS | |
| 0x21000000 | PERF_KERNEL_QUEUE | 6.2 and higher |
| 0x22000000 | PERF_INTERRUPT_STEER | |
| 0x24000000 | PERF_SHOULD_YIELD | |
| 0x28000000 | PERF_WS | 6.2 and higher |

### Masks[2]

| Value | Name | Versions |
|-------|------|----------|
| 0x40000001 | PERF_ANTI_STARVATION | 6.2 and higher |
| 0x40000002 | PERF_PROCESS_FREEZE | 6.2 and higher |
| 0x40000004 | PERF_PFN_LIST | |
| 0x40000008 | PERF_WS_DETAIL | |
| 0x40000010 | PERF_WS_ENTRY | |
| 0x40000020 | PERF_HEAP | 6.2 and higher |
| 0x40000040 | PERF_SYSCALL | 6.0 and higher |
| 0x40000080 | PERF_UMS | |
| 0x40000100 | PERF_BACKTRACE | |
| 0x40000200 | PERF_VULCAN | |
| 0x40000400 | PERF_OBJECTS | |
| 0x40000800 | PERF_EVENTS | |
| 0x40001000 | PERF_FULLTRACE | |
| 0x40002000 | PERF_DFSS | 6.1 and higher |
| 0x40004000 | PERF_PREFETCH | |
| 0x40008000 | PERF_PROCESSOR_IDLE | 6.1 and higher |
| 0x40010000 | PERF_CPU_CONFIG | |
| 0x40020000 | PERF_TIMER | 6.2 and higher |
| 0x40040000 | PERF_CLOCK_INTERRUPT | 6.2 and higher |
| 0x40080000 | PERF_LOAD_BALANCER | 6.2 and higher |
| 0x40100000 | PERF_CLOCK_TIMER | 6.2 and higher |
| 0x40200000 | PERF_IDLE_SELECTION | |
| 0x40400000 | PERF_IPI | 6.2 and higher |
| 0x40800000 | PERF_IO_TIMER | 6.2 and higher |
| 0x41000000 | PERF_REG_HIVE | 6.2 and higher |
| 0x42000000 | PERF_REG_NOTIF | 6.2 and higher |
| 0x44000000 | PERF_PPM_EXIT_LATENCY | 6.2 and higher |
| 0x48000000 | PERF_WORKER_THREAD | |

### Masks[3]

Apparently, no flags are yet defined for mask number 3.

### Masks[4]

| Value | Name | Versions |
|-------|------|----------|
| 0x80000001 | PERF_OPTICAL_IO | 6.2 and higher |
| 0x80000002 | PERF_OPTICAL_IO_INIT | 6.2 and higher |
| 0x80000004 | apparently unused | |
| 0x80000008 | PERF_DLL_INFO | |
| 0x80000010 | PERF_DLL_FLUSH_WS | |
| 0x80000020 | apparently unused | |
| 0x80000040 | PERF_OB_HANDLE | 6.2 and higher |
| 0x80000080 | PERF_OB_OBJECT | 6.2 and higher |
| 0x80000100 | apparently unused | |
| 0x80000200 | PERF_WAKE_DROP | 6.2 and higher |
| 0x80000400 | PERF_WAKE_EVENT | 6.2 and higher |
| 0x80000800 | PERF_DEBUGGER | |
| 0x80001000 | PERF_PROC_ATTACH | |
| 0x80002000 | PERF_WAKE_COUNTER | |
| 0x80004000 | apparently unused | |
| 0x80008000 | PERF_POWER | 5.1 and higher |
| 0x80010000 | PERF_SOFT_TRIM | |
| 0x80020000 | PERF_CC | 6.2 and higher |
| 0x80040000 | apparently unused | |
| 0x80080000 | PERF_FLT_IO_INIT | |
| 0x80100000 | PERF_FLT_IO | |
| 0x80200000 | PERF_FLT_FASTIO | |
| 0x80400000 | PERF_FLT_IO_FAILURE | |
| 0x80800000 | PERF_HV_PROFILE | |
| 0x81000000 | PERF_WDF_DPC | |
| 0x82000000 | PERF_WDF_INTERRUPT | |
| 0x84000000 | PERF_CACHE_FLUSH | |

### Masks[5]

| Value | Name | |
|-------|------|---|
| 0xA0000001 | PERF_HIBER_RUNDOWN | |

### Masks[6]

| Value | Name | |
|-------|------|---|
| 0xC0000001 | PERF_SYSCFG_SYSTEM | |
| 0xC0000002 | PERF_SYSCFG_GRAPHICS | |
| 0xC0000004 | PERF_SYSCFG_STORAGE | |
| 0xC0000008 | PERF_SYSCFG_NETWORK | |
| 0xC0000010 | PERF_SYSCFG_SERVICES | |
| 0xC0000020 | PERF_SYSCFG_PNP | |
| 0xC0000040 | PERF_SYSCFG_OPTICAL | |
| 0xDFFFFFFF | PERF_SYSCFG_ALL | |

### Masks[7]

| Value | Name | Versions |
|-------|------|----------|
| 0xE0000001 | PERF_CLUSTER_OFF | |
| 0xE0000002 | PERF_MEMORY_CONTROL | 6.2 and higher |

*This page was created on 4th December 2016 and was last modified on 29th December 2020.*

*Copyright © 2016-2020. Geoff Chappell. All rights reserved. Conditions apply.*