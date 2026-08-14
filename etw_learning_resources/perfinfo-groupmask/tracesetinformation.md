# TraceSetInformation

This function sets properties for Event Tracing for Windows (ETW) in general or for a trace session.

See also: [PERFINFO_GROUPMASK](./perfinfo_groupmask.md)

## Declaration

```c
ULONG 
TraceSetInformation (
    TRACEHANDLE SessionHandle, 
    TRACE_INFO_CLASS InformationClass, 
    PVOID TraceInformation, 
    ULONG InformationLength);
```

## Parameters

- **SessionHandle** - Selects a trace session, also known as an event logger, to configure.
- **InformationClass** - Tells what sort of information is being set.
- **TraceInformation** - Address and size (in bytes) of a buffer that provides the information. What the function expects of this buffer depends on the information class.
- **InformationLength** - Can be zero for some information classes, in which case TraceInformation may need to be NULL.

## Return Value

The function returns ERROR_SUCCESS if successful, else a non-zero error code.

## Availability

The TraceSetInformation function is exported from ADVAPI32 in version 6.1 and higher. Starting with version 6.2, however, the implementation in ADVAPI32 is just a stub for calling the true implementation via the API Set api-ms-win-eventing-controller-l1-1-0.dll.

## Documentation Status

The TraceSetInformation function is documented. However, for most of the information classes the documentation is essentially useless since although it sketches (not always accurately) the purpose of the expected information it does not present the format.

## Behaviour

The following implementation notes are from inspection of SECHOST from the original release of Windows 10 only. They may some day get revised to account for earlier versions. Meanwhile, where anything is added about earlier versions, take it not as an attempt at comprehensiveness but as a bonus from my being unable to resist a trip down memory lane or at least a quick look into the history.

The function supports the following information classes:

| Value | Name | Versions |
|-------|------|----------|
| 0x03 | TraceStackTracingInfo | 6.1 and higher |
| 0x04 | TraceSystemTraceEnableFlagsInfo | 6.2 and higher |
| 0x05 | TraceSampledProfileIntervalInfo | 6.2 and higher |
| 0x06 | TraceProfileSourceConfigInfo | 6.2 and higher |
| 0x08 | TracePmcEventListInfo | 6.2 and higher |
| 0x09 | TracePmcCounterListInfo | 6.2 and higher |
| 0x0A | TraceSetDisallowList | 10.0 and higher |

Given any other, the function returns ERROR_NOT_SUPPORTED.

### TraceSystemTraceEnableFlagsInfo (0x04)

The information class TraceSystemTraceEnableFlagsInfo specifies which groups of NT Kernel Logger events to enable for the given trace session.

The information buffer must provide an array of 32-bit group masks. The first is compatible with the EnableFlags that are documented for the EVENT_TRACE_PROPERTIES structure that is the input to such functions as StartTrace and ControlTrace. There can be at most eight group masks, which altogether make a PERFINFO_GROUPMASK structure. If the information buffer would provide more or is not an exact fit for a whole number of group masks, the function returns ERROR_INVALID_PARAMETER.

The given group masks are sent to the kernel's information class EventTraceGroupMaskInformation (0x01). The kernel receives an EVENT_TRACE_GROUPMASK_INFORMATION structure into which the function places the SessionHandle as the TraceHandle and the given group masks as the EventTraceGroupMasks. Each group mask that isn't specified to the function gets specified to the kernel as zero.

This is the primary mechanism for enabling extended flags that cannot be set normally when starting the etw kernel tracing session's using only EnableFlags. Using PERFINFO_GROUPMASK with TraceSetInformation allows enabling many more event bits than the 32 bits available through EnableFlags in the EVENT_TRACE_PROPERTIES structure.