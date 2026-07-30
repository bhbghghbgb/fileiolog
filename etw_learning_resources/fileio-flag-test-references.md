# FileIo Flag Test - Reference Links

## Official Microsoft Documentation

- [EVENT_TRACE_PROPERTIES structure](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_properties)
- [TRACE_QUERY_INFO_CLASS enumeration](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/ne-evntrace-trace_query_info_class)
- [TraceQueryInformation function](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-tracequeryinformation)
- [TraceSetInformation function](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-tracesetinformation)
- [ControlTraceW function](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-controltracew)
- [StartTraceW function](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-starttracew)
- [FileIo class (V2)](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio)
- [FileIo_Name class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-name)
- [FileIo_Create class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-create)
- [FileIo_ReadWrite class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-readwrite)
- [FileIo_SimpleOp class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-simpleop)
- [FileIo_Info class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-info)
- [FileIo_OpEnd class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-opend)
- [FileIo_DirEnum class](https://learn.microsoft.com/en-us/windows/desktop/ETW/fileio-direnum)

## Unofficial Documentation

- [PERFINFO_GROUPMASK - Geoff Chappell](https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntwmi/perfinfo_groupmask.htm)
- [TraceSetInformation - Geoff Chappell](https://www.geoffchappell.com/studies/windows/win32/advapi32/api/etw/logapi/set.htm)
- [TraceQueryInformation - Geoff Chappell](https://www.geoffchappell.com/studies/windows/win32/advapi32/api/etw/logapi/query.htm)
- [EVENT_TRACE_GROUPMASK_INFORMATION - Geoff Chappell](https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/ntetw/event_trace_groupmask_information.htm)

## System Informer / phnt Headers

- [ntwmi.h (PERFINFO_GROUPMASK, CONTROLTRACE_ID)](https://github.com/processhacker/processhacker/blob/master/phnt/include/ntwmi.h)
- [CONTROLTRACE_ID = ULONG64, obtainable from ControlTrace Wnode.HistoricalContext]

## Ferrisetw Library

- [ferrisetw GitHub](https://github.com/n4r1b/ferrisetw)
- [ferrisetw trace.rs (KernelTrace implementation)](https://github.com/n4r1b/ferrisetw/blob/master/src/trace.rs)
- [ferrisetw evntrace.rs (native ETW wrappers)](https://github.com/n4r1b/ferrisetw/blob/master/src/native/evntrace.rs)
- [ferrisetw etw_types.rs (EventTraceProperties wrapper)](https://github.com/n4r1b/ferrisetw/blob/master/src/native/etw_types.rs)

## Rust Crates

- [windows crate (v0.61)](https://crates.io/crates/windows)
- [ntapi crate (PERFINFO_GROUPMASK in ntexapi module)](https://crates.io/crates/ntapi)
- [ntapi docs.rs - PERFINFO_GROUPMASK](https://docs.rs/ntapi/latest/ntapi/ntexapi/struct.PERFINFO_GROUPMASK.html)

## Key Findings

### EnableFlags (32-bit, Masks[0])

| Flag | Value | Event Types |
|------|-------|-------------|
| EVENT_TRACE_FLAG_DISK_FILE_IO | 0x00000200 | FileIo_Name (0, 32, 35, 36) |
| EVENT_TRACE_FLAG_FILE_IO | 0x02000000 | FileIo_OpEnd (76) |
| EVENT_TRACE_FLAG_FILE_IO_INIT | 0x04000000 | FileIo_Create (64), DirEnum (72,77), Info (69,70,71,74,75), ReadWrite (67,68), SimpleOp (65,66,73) |

### PERFINFO_GROUPMASK (Extended, Masks[1]+)

| Mask Index | Name | Value | Event Types |
|------------|------|-------|-------------|
| Masks[4] | PERF_FLT_IO_INIT | 0x80080000 | FltIoInit (96, 97) |
| Masks[4] | PERF_FLT_IO | 0x80100000 | FltIoCompletion (98, 99) |
| Masks[4] | PERF_FLT_IO_FAILURE | 0x80400000 | FltIoFailure (100, 101) |

### CONTROLTRACE_ID / CONTROLTRACE_HANDLE

- Obtained from `StartTraceW` output
- Also obtainable from `ControlTraceW(0, name, props, QUERY)` via `Wnode.HistoricalContext`
- Used as session handle for `TraceQueryInformation` and `TraceSetInformation`

### FileIo Event Versions

- **V0**: Only FileIo_Name (0: Name)
- **V1**: FileIo_Name (0: Name, 32: FileCreate)
- **V2**: Full set documented in official docs (no MapFile in docs)
- **V3**: Full set from MOF code (includes FltIoInit, FltIoFailure, FltIoCompletion, MapFile via V2 MapFile class)
