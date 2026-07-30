# ETW FileIo Undocumented Behavior References

## Key Reference Pages

### PERFINFO_GROUPMASK Structure
- https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntwmi/perfinfo_groupmask.htm
  - Full list of all 8 Masks[] elements and their bit definitions
  - Maps EnableFlags to PERFINFO_GROUPMASK Masks[0] equivalents
  - Documents undocumented group masks (Masks[1]-[7])

### EVENT_TRACE_GROUPMASK_INFORMATION
- https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntetw/event_trace_groupmask_information.htm
  - Structure layout used by NtQuerySystemInformation/NtSetSystemInformation
  - TraceSetInformation/TraceQueryInformation with TraceSystemTraceEnableFlagsInfo (0x04)

### NTWMI.H (WDK header with PERF defines)
- https://github.com/winsiderss/phnt/blob/master/ntwmi.h
  - PERF_FLT_IO_INIT (0x80080000) - Minifilter IO Init
  - PERF_FLT_IO (0x80100000) - Minifilter IO
  - PERF_FLT_FASTIO (0x80200000) - Minifilter FastIO
  - PERF_FLT_IO_FAILURE (0x80400000) - Minifilter IO Failure
  - PERF_FILE_IO (0x02000000) = EVENT_TRACE_FLAG_FILE_IO
  - PERF_FILE_IO_INIT (0x04000000) = EVENT_TRACE_FLAG_FILE_IO_INIT
  - PERF_VAMAP (0x00008000) = EVENT_TRACE_FLAG_VAMAP
  - Event type packing: Group (bits 8..12) + Type (bits 0..7)
  - PERF_GET_MASK_INDEX / PERF_GET_MASK_GROUP macros

### TraceSetInformation API
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-tracesetinformation
  - Use TraceSystemTraceEnableFlagsInfo (0x04) information class
  - Pass PERFINFO_GROUPMASK as the information buffer

### Microsoft FileIo V2 Official Docs
- https://learn.microsoft.com/en-us/windows/win32/etw/fileio
  - Lists EVENT_TRACE_FLAG_DISK_FILE_IO, EVENT_TRACE_FLAG_FILE_IO, EVENT_TRACE_FLAG_FILE_IO_INIT
  - Documents event types 0-77

### EVENT_TRACE_TYPE definitions for Filter Manager
- EVENT_TRACE_TYPE_FLT_PREOP_INIT = 0x60 (96)
- EVENT_TRACE_TYPE_FLT_POSTOP_INIT = 0x61 (97)
- EVENT_TRACE_TYPE_FLT_PREOP_COMPLETION = 0x62 (98)
- EVENT_TRACE_TYPE_FLT_POSTOP_COMPLETION = 0x63 (99)
- EVENT_TRACE_TYPE_FLT_PREOP_FAILURE = 0x64 (100)
- EVENT_TRACE_TYPE_FLT_POSTOP_FAILURE = 0x65 (101)

### EVENT_TRACE_GROUP_FILE
- EVENT_TRACE_GROUP_FILE = 0x0400

## FileIo Event Type to WMI Log Type Mapping

| Event Type | WMI Log Type | MOF Class | Description |
|------------|-------------|-----------|-------------|
| 0 | 0x0400 | FileIo_Name | Name |
| 32 | 0x0420 | FileIo_Name | FileCreate |
| 35 | 0x0423 | FileIo_Name | FileDelete |
| 36 | 0x0424 | FileIo_Name | FileRundown |
| 37 | 0x0425 | FileIo_V2_MapFile | MapFile |
| 38 | 0x0426 | FileIo_V2_MapFile | UnmapFile |
| 39 | 0x0427 | FileIo_V2_MapFile | MapFileDCStart |
| 40 | 0x0428 | FileIo_V2_MapFile | MapFileDCEnd |
| 64 | 0x0440 | FileIo_Create | Create |
| 65 | 0x0441 | FileIo_SimpleOp | Cleanup |
| 66 | 0x0442 | FileIo_SimpleOp | Close |
| 67 | 0x0443 | FileIo_ReadWrite | Read |
| 68 | 0x0444 | FileIo_ReadWrite | Write |
| 69 | 0x0445 | FileIo_Info | SetInfo |
| 70 | 0x0446 | FileIo_Info | Delete |
| 71 | 0x0447 | FileIo_Info | Rename |
| 72 | 0x0448 | FileIo_DirEnum | DirEnum |
| 73 | 0x0449 | FileIo_SimpleOp | Flush |
| 74 | 0x044A | FileIo_Info | QueryInfo |
| 75 | 0x044B | FileIo_Info | FSControl |
| 76 | 0x044C | FileIo_OpEnd | OperationEnd |
| 77 | 0x044D | FileIo_DirEnum | DirNotify |
| 79 | 0x044F | FileIo_PathOperation | DeletePath |
| 80 | 0x0450 | FileIo_PathOperation | RenamePath |
| 81 | 0x0451 | FileIo_PathOperation | SetLinkPath |
| 96 | 0x0460 | FltIoInit | PreOpInit |
| 97 | 0x0461 | FltIoInit | PostOpInit |
| 98 | 0x0462 | FltIoCompletion | PreOpCompletion |
| 99 | 0x0463 | FltIoCompletion | PostOpCompletion |
| 100 | 0x0464 | FltIoFailure | PreOpFailure |
| 101 | 0x0465 | FltIoFailure | PostOpFailure |

## Known EnableFlags → Event Type Mapping (Official + Unofficial)

### EVENT_TRACE_FLAG_DISK_FILE_IO (0x00000200)
- FileIo_Name: 0 (Name), 32 (FileCreate), 35 (FileDelete), 36 (FileRundown)
- Requires EVENT_TRACE_FLAG_DISK_IO for disk-level events

### EVENT_TRACE_FLAG_FILE_IO (0x02000000)
- FileIo_OpEnd: 76 (OperationEnd)

### EVENT_TRACE_FLAG_FILE_IO_INIT (0x04000000)
- FileIo_Create: 64
- FileIo_SimpleOp: 65 (Cleanup), 66 (Close), 73 (Flush)
- FileIo_ReadWrite: 67 (Read), 68 (Write)
- FileIo_Info: 69 (SetInfo), 70 (Delete), 71 (Rename), 74 (QueryInfo), 75 (FSControl)
- FileIo_DirEnum: 72 (DirEnum), 77 (DirNotify)

### EVENT_TRACE_FLAG_VAMAP (0x00008000) [Masks[0]]
- FileIo_V2_MapFile: 37 (MapFile), 38 (UnmapFile), 39 (MapFileDCStart), 40 (MapFileDCEnd)

### PERF_FLT_IO_INIT (0x80080000) [Masks[4]]
- FltIoInit: 96 (PreOpInit), 97 (PostOpInit)

### PERF_FLT_IO (0x80100000) [Masks[4]]
- FltIoCompletion: 98 (PreOpCompletion), 99 (PostOpCompletion)

### PERF_FLT_IO_FAILURE (0x80400000) [Masks[4]]
- FltIoFailure: 100 (PreOpFailure), 101 (PostOpFailure)

### Unknown/Undocumented Mapping
- FileIo_PathOperation: 79 (DeletePath), 80 (RenamePath), 81 (SetLinkPath)
  - Possibly requires a specific PERFINFO_GROUPMASK bit not yet identified
