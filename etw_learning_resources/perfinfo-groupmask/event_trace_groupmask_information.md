# EVENT_TRACE_GROUPMASK_INFORMATION

The EVENT_TRACE_GROUPMASK_INFORMATION structure is one of many that the ZwQuerySystemInformation (or NtQuerySystemInformation) and ZwSetSystemInformation (or NtSetSystemInformation) functions expect in their information buffer when given the information class SystemPerformanceTraceInformation (0x1F). This particular structure is selected when the first dword in the information buffer on input is EventTraceGroupMaskInformation (0x01).

See also: [PERFINFO_GROUPMASK](perfinfo_groupmask.md) and [TraceSetInformation](tracesetinformation.md) and [EVENT_TRACE_INFORMATION_CLASS](event_trace_information_class.md)

## Usage

At least in user mode, the EVENT_TRACE_GROUPMASK_INFORMATION structure arguably exists only to support the documented ADVAPI32 or SECHOST functions TraceQueryInformation and TraceSetInformation for their information class TraceSystemTraceEnableFlagsInfo (0x04). Well-behaved user-mode software executing above ADVAPI32 does not call NtQuerySystemInformation or NtSetSystemInformation but prefers TraceQueryInformation and TraceSetInformation and therefore has no need of this structure.

Or so might go the theory or principle. Against it is that Microsoft's documentation of TraceQueryInformation and TraceSetInformation, as perused online today (30th November 2016), does not tell programmers what form of information to expect or provide. Indeed, what it does say is arguably misleading, for it suggests that the TraceSystemTraceEnableFlagsInfo case works just with "the setting for the EnableFlags for the system trace provider" and directs attention to the documentation of the EVENT_TRACE_PROPERTIES structure. In fact, the case provides for a significant extension of those flags.

## Documentation Status

The EVENT_TRACE_GROUPMASK_INFORMATION structure is not documented.

A few public disclosures are known from Microsoft, though not as any sort of plain-English documentation. One is that a previously unpublished header named NTETW.H was published in the original and Version 1511 editions of the Windows Driver Kit (WDK) for Windows 10, and this header contains a C-language definition of the structure.

Were it not for this limited and possibly unintended disclosure of NTETW.H, a practical equivalent of the C-language definition (but missing comments, of course) would anyway be known from type information in symbol files. But this too has the look of an oversight. Type information for this structure has never appeared in any public symbol files for the kernel or for the obvious low-level user-mode DLLs. It has instead slipped out in symbol files for a smattering of higher-level user-mode DLLs, starting with Windows 8. For these few, the readily available symbol files actually are private symbol files and show that the unpublished NTETW.H was included when compiling the corresponding binaries.

Type information also has been published in a statically linked library, named CLFSMGMT.LIB, which Microsoft distributes with the Software Development Kit (SDK) starting for Windows Vista. This does not have the forensic quality as has type information in symbol files for the binaries that ship with an operating system, for although it is as accurate for when the library was built, there is no requirement that the library have been built with the operating system that it targets. There can be, and often is, some discrepancy, and there is anyway not always a fresh library for each service pack.

## Layout

The EVENT_TRACE_GROUPMASK_INFORMATION is 0x30 bytes in both 32-bit and 64-bit Windows. Offsets, names and types in the table that follows are from type information in symbol files and libraries, and from the published C-language definition, as described above.

| Offset | Definition | Versions | Input or Output |
|--------|------------|----------|-----------------|
| 0x00 | EVENT_TRACE_INFORMATION_CLASS EventTraceInformationClass; | 6.0 and higher | input |
| 0x08 | TRACEHANDLE TraceHandle; | 6.0 and higher | input |
| 0x10 | PERFINFO_GROUPMASK EventTraceGroupMasks; | 6.0 and higher | output for query; input for set |

When the structure is built by TraceQueryInformation or TraceSetInformation, the TraceHandle is the SessionHandle argument and the EventTraceGroupMasks is copied to or from the InformationLength bytes at TraceInformation.

## Behaviour

The EVENT_TRACE_GROUPMASK_INFORMATION structure is meaningful only as input to one case each of the ZwQuerySystemInformation and ZwSetSystemInformation functions. The behaviour is as well picked up here. This review takes as understood all the general points and shorthands that are noted in the separate attempt at documenting the functions, and takes as granted that the information class is SystemPerformanceTraceInformation and that the information buffer is exactly the size of an EVENT_TRACE_GROUPMASK_INFORMATION in which the EventTraceInformationClass is EventTraceGroupMaskInformation.

Note that although EventTraceGroupMaskInformation is valid for querying in version 6.0 and higher, versions before 6.2 reject it for setting. The returned error code is ERROR_NOT_IMPLEMENTED.

Whether to query or set, the TraceHandle selects an event logger. Specifically, the low 16 bits are the logger ID or are 0xFFFF to select the NT Kernel Logger. This interpretation of 0xFFFF is formalised by the definition of a macro KERNEL_LOGGER_ID in the NTWMI_X.H header from the early editions of the WDK for Windows 10. If the logger ID does not select an active logger to which the function can arrange exclusive access, the function returns STATUS_WMI_INSTANCE_NOT_FOUND. The function also fails, but returning STATUS_INVALID_PARAMETER, if the logger context's logger mode does not include EVENT_TRACE_SYSTEM_LOGGER_MODE (0x02000000).

### Query

The essential work when querying is to extract the logger's PERFINFO_GROUPMASK as the EventTraceGroupMasks member of the output. This PERFINFO_GROUPMASK that is produced as output is not necessarily exactly what the kernel keeps for the logger but may instead have been translated for compatibility with the EnableFlags that is documented for the EVENT_TRACE_PROPERTIES structure as input to the StartTrace and ControlTrace functions.

### Set

To set information, the caller must have TRACELOG_GUID_ENABLE access to the logger. Without it, the function fails, typically returning STATUS_ACCESS_DENIED.

The essential work when setting is to load the logger's PERFINFO_GROUPMASK from the EventTraceGroupMasks member of the input. As with querying, translation may be involved for compatibility.

The given group masks specify which types of event the kernel is to generate (itself or on behalf of low-level user-mode modules such as NTDLL) for the logger. For most types of event, enabling is trivial: for others, not so much. For example, asking to enable PERF_PROFILE (0x20000002) or PERF_PMC_PROFILE (0x20000400) without SeSystemProfilePrivilege causes the function to fail, returning STATUS_PRIVILEGE_NOT_HELD. Another is that asking to enable PERF_CONTEXT_SWITCH (0x20000004) and PERF_COMPACT_CSWITCH (0x20000100) when they are not already both enabled may require the preparation of per-processor buffers to support the efficient batching of data about thread switches, and can fail such that the function returns STATUS_NO_MEMORY. There need not be an end to such examples, of course.