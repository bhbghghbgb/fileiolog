# ETW Kernel Tracing Reference Links

## Official Microsoft Documentation

### EVENT_TRACE_PROPERTIES (EnableFlags)
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_properties

### TraceSetInformation (for PERFINFO_GROUPMASK)
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-tracesetinformation

### TRACE_QUERY_INFO_CLASS (TraceSystemTraceEnableFlagsInfo = 0x04)
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/ne-evntrace-trace_query_info_class

### StartTraceW
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-starttracew

### ControlTraceW
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-controltracew

### EnableTraceEx2
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-enabletraceex2

### CloseTrace
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-closetrace

## Unofficial / Reverse-Engineered Documentation

### PERFINFO_GROUPMASK (Geoff Chappell)
- https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntwmi/perfinfo_groupmask.htm

### TraceSetInformation / TraceQueryInformation (Geoff Chappell)
- https://www.geoffchappell.com/studies/windows/win32/advapi32/api/etw/logapi/set.htm
- https://www.geoffchappell.com/studies/windows/win32/advapi32/api/etw/logapi/query.htm

### EVENT_TRACE_GROUPMASK_INFORMATION (Geoff Chappell)
- https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntetw/event_trace_groupmask_information.htm

## Ferrisetw Library

### Repository
- https://github.com/n4r1b/ferrisetw

### Kernel Trace Example
- https://github.com/n4r1b/ferrisetw/blob/main/examples/kernel_trace.rs

### Trace Module (start/process/stop)
- See `etw_learning_resources/ferrisetw/src/trace.rs`

### Provider Module (kernel_providers)
- See `etw_learning_resources/ferrisetw/src/provider/kernel_providers.rs`

### Native evntrace (control_trace, close_trace)
- See `etw_learning_resources/ferrisetw/src/native/evntrace.rs`

## FileIo MOF Classes

### V0 (EventVersion 0)
- FileIo_V0_Name: EventType(0) = Name

### V1 (EventVersion 1)
- FileIo_V1_Name: EventType{0, 32} = Name, FileCreate

### V2 (EventVersion 2)
- FileIo_V2_Name: EventType{0, 32, 35, 36} = Name, FileCreate, FileDelete, FileRundown
- FileIo_V2_MapFile: EventType{37, 38, 39, 40} = MapFile, UnmapFile, MapFileDCStart, MapFileDCEnd
- FileIo_V2_Create: EventType(64) = Create
- FileIo_V2_SimpleOp: EventType{65, 66, 73} = Cleanup, Close, Flush
- FileIo_V2_ReadWrite: EventType{67, 68} = Read, Write
- FileIo_V2_Info: EventType{69, 70, 71, 74, 75} = SetInfo, Delete, Rename, QueryInfo, FSControl
- FileIo_V2_DirEnum: EventType{72, 77} = DirEnum, DirNotify
- FileIo_V2_OpEnd: EventType(76) = OperationEnd

### V3 (EventVersion 3)
- FileIo_Name: EventType{0, 32, 35, 36} = Name, FileCreate, FileDelete, FileRundown
- FileIo_Create: EventType(64) = Create
- FileIo_SimpleOp: EventType{65, 66, 73} = Cleanup, Close, Flush
- FileIo_ReadWrite: EventType{67, 68} = Read, Write
- FileIo_Info: EventType{69, 70, 71, 74, 75} = SetInfo, Delete, Rename, QueryInfo, FSControl
- FileIo_DirEnum: EventType{72, 77} = DirEnum, DirNotify
- FileIo_OpEnd: EventType(76) = OperationEnd
- FileIo_PathOperation: EventType{79, 80, 81} = DletePath, RenamePath, SetLinkPath
- FltIoInit: EventType{96, 97} = PreOpInit, PostOpInit
- FltIoCompletion: EventType{98, 99} = PreOpCompletion, PostOpCompletion
- FltIoFailure: EventType{100, 101} = PreOpFailure, PostOpFailure

## EnableFlags (FileIo-related)

| Flag | Value | Events Enabled |
|------|-------|----------------|
| EVENT_TRACE_FLAG_DISK_FILE_IO | 0x00000200 | FileIo_Name (requires DISK_IO) |
| EVENT_TRACE_FLAG_FILE_IO | 0x02000000 | FileIo_OpEnd |
| EVENT_TRACE_FLAG_FILE_IO_INIT | 0x04000000 | FileIo_Create, FileIo_DirEnum, FileIo_Info, FileIo_ReadWrite, FileIo_SimpleOp |
| EVENT_TRACE_FLAG_VAMAP | 0x00008000 | MapFile/UnmapFile events (V2+) |

## PERFINFO_GROUPMASK (FileIo-related)

| Mask | Value | Events Enabled |
|------|-------|----------------|
| PERF_FLT_IO_INIT | 0x80080000 | FltIoInit (PreOpInit, PostOpInit) |
| PERF_FLT_IO | 0x80100000 | FltIoCompletion (PreOpCompletion, PostOpCompletion) |
| PERF_FLT_FASTIO | 0x80200000 | FastIO events |
| PERF_FLT_IO_FAILURE | 0x80400000 | FltIoFailure (PreOpFailure, PostOpFailure) |
| PERF_VAMAP | 0x00008000 | MapFile/UnmapFile events (same as EVENT_TRACE_FLAG_VAMAP) |
