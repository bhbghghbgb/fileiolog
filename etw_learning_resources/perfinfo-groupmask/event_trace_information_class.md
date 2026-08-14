# EVENT_TRACE_INFORMATION_CLASS

The EVENT_TRACE_INFORMATION_CLASS is an enumeration whose values are intended as the first dword in the information buffer when the ZwQuerySystemInformation (or NtQuerySystemInformation) and ZwSetSystemInformation (or NtSetSystemInformation) functions are given the information class SystemPerformanceTraceInformation (0x1F).

See also: [EVENT_TRACE_GROUPMASK_INFORMATION](event_trace_groupmask_information.md)

The EVENT_TRACE_INFORMATION_CLASS enumeration is not documented.

A few public disclosures are known from Microsoft, though not as any sort of plain-English documentation. One is that a previously unpublished header named NTETW.H was published in the original and Version 1511 editions of the Windows Driver Kit (WDK) for Windows 10, and this header contains a C-language definition of the enumeration.

Were it not for this limited and possibly unintended disclosure of NTETW.H, a practical equivalent of the C-language definition (but missing comments, of course) would anyway be known from type information in symbol files. But this too has the look of an oversight. Type information for this enumeration has never appeared in any public symbol files for the kernel or for the obvious low-level user-mode DLLs. It has instead slipped out in symbol files for a smattering of higher-level user-mode DLLs, starting with Windows 8. For these few, the readily available symbol files actually are private symbol files and show that the unpublished NTETW.H was included when compiling the corresponding binaries. Type information also has been published in a statically linked library, named CLFSMGMT.LIB, which Microsoft distributes with the Software Development Kit (SDK) starting for Windows Vista.

## Values

For the table that follows, Microsoft's names are known for version 6.2 and higher from type information in symbol files and libraries, and from the limited publication of a C-language definition, as described above.

Of the many defined values, some can be used successfully only to query or only to set:

| Value | Name | Versions | Query Or Set |
|-------|------|----------|--------------|
| 0x00 | EventTraceKernelVersionInformation | 6.0 and higher | query |
| 0x01 | EventTraceGroupMaskInformation | 6.0 to 6.1 | query |
| | | 6.2 and higher | both |
| 0x02 | EventTracePerformanceInformation | 6.0 and higher | query |
| 0x03 | EventTraceTimeProfileInformation | 6.0 and higher | both |
| 0x04 | EventTraceSessionSecurityInformation | 6.0 and higher | query |
| 0x05 | EventTraceSpinlockInformation | 6.1 and higher | both |
| 0x06 | EventTraceStackTracingInformation | 6.1 and higher | both |
| 0x07 | EventTraceExecutiveResourceInformation | 6.1 and higher | both |
| 0x08 | EventTraceHeapTracingInformation | 6.1 and higher | query |
| 0x09 | EventTraceHeapSummaryTracingInformation | 6.1 and higher | query |
| 0x0A | EventTracePoolTagFilterInformation | 6.1 and higher | both |
| 0x0B | EventTracePebsTracingInformation | 6.2 and higher | set |
| 0x0C | EventTraceProfileConfigInformation | 6.2 and higher | set |
| 0x0D | EventTraceProfileSourceListInformation | 6.2 and higher | query |
| 0x0E | EventTraceProfileEventListInformation | 6.2 and higher | set |
| 0x0F | EventTraceProfileCounterListInformation | 6.2 and higher | set |
| 0x10 | EventTraceStackCachingInformation | 6.2 and higher | set |
| 0x11 | EventTraceObjectTypeFilterInformation | 6.2 and higher | set |
| 0x12 | EventTraceSoftRestartInformation | 1607 and higher | both |
| 0x13 | EventTraceLastBranchConfigurationInformation | 1709 and higher | set |
| 0x14 | EventTraceLastBranchEventListInformation | 1709 and higher | set |
| 0x15 | EventTraceProfileSourceAddInformation | 1803 and higher | set |
| 0x16 | EventTraceProfileSourceRemoveInformation | 1803 and higher | set |
| 0x17 | EventTraceProcessorTraceConfigurationInformation | 1803 and higher | set |
| 0x18 | EventTraceProcessorTraceEventListInformation | 1803 and higher | set |
| 0x19 | EventTraceCoverageSamplerInformation | 1803 and higher | both |
| 0x1A | MaxEventTraceInfoClass | 6.0 and higher | |

## Behaviour

In its role as the first dword of input in the information buffer for ZwQuerySystemInformation and ZwSetSystemInformation when given the information class SystemPerformanceTraceInformation, the EVENT_TRACE_INFORMATION_CLASS enumeration subdivides the behaviour of these functions—which is as well picked up here. This review takes as understood all the general points and shorthands that are noted in the separate attempt at documenting the functions, and takes as granted that the information class is SystemPerformanceTraceInformation and that the information buffer is at least large enough for an EVENT_TRACE_INFORMATION_CLASS.

If the EVENT_TRACE_INFORMATION_CLASS on input is not listed above as valid for the function, then the function returns STATUS_NOT_IMPLEMENTED.

Each EVENT_TRACE_INFORMATION_CLASS is associated with a structure that is at least the start of what the function produces as its output or expects as input. Mostly, the structure has no other purpose. Rather than have a separate page for each information class and then another for the corresponding structure, the remainder of this page gives for each information class a brief description of the general behaviour, and then the meaning of whatever the function puts in the structure or interprets in it is taken up, if at all, in the separate documentation of the structure.

A unified presentation of these cases is very much the sort of thing that isn't well settled until all the cases have been examined. Of necessity this is a bit of an open-ended project, and commercial imperatives may mean the project must be abandoned. Please beware that the draft colour signifies rough notes and tentative thoughts that I offer only on the basis that they may (or may not) be better than nothing.