# Getting ETW Notifications - Event Tracing for Windows

*Posted: Dec 13, 2023 4:10 by **adeyblue***

'Simple' sample of capturing ETW events. This one captures all the events relating to the changes in the system's timer resolution and the requests that each process makes for it. (ie `timeBeginPeriod()`, `timeEndPeriod()` and `NtSetTimerResolution()`)

It looks like loads of code because FB doesn't have any of the SDK includes for this so the first 300 or so lines are the required bits from the SDK. The next 100 are the definitions of the event data. You can parse the event data out programmatically with functions like `TdhGetEventInformation`, `TdhGetProperty`, etc but that seemed like more of a pain so I transcribed the structures instead.

The main problem on ETW's front with getting events from providers you didn't create (like the OS ones) is knowing the events they support and the data included with them.

```
wevtutil ep
```

will show all the event providers and

```
wevtutil gp <providerName>
```

will show the guid and keyword bits and lots of other stuff... but not the event ids, or what the event data format is. Which is kind of important.

Luckily someone's already dumped all the event ids, versions and data formats from the built-in os components (you can find the ones used in this sample in Manifest/Microsoft-Windows-Kernel-Power.tsv)

[Windows10EtwEvents on GitHub](https://github.com/jdu2600/Windows10EtwEvents)

Unfortunately, those don't include which keyword bits you need to enable to receive them (just the name of them), or the provider guid that `wevtutil` will show you. So every provider and event you want to listen to requires at least two things for you to do to put it together rather than just one, unless someone's already put the guids and keyword bits and event ids and data together. And for the Kernel Power provider specifically, someone has! The recently departed Geoff Chappell put them all in one handy dandy page, which made this much less of a nightmare than it might have otherwise been.

[Geoff Chappell - Microsoft-Windows-Kernel-Power](https://www.geoffchappell.com/studies/windows/km/ntoskrnl/events/microsoft-windows-kernel-power.htm)

Anyway, here's the code. To enable any providers for a session, you need to be running as Admin or be a member of the Performance Log Users user group on your computer, so run it as admin. I think you can make it not do that, but that's even more faff in an already large and faffy piece of code, so I didn't.

Any key should quit. It does have a little delay before anything appears.

## Code

```freebasic
#define _WIN32_WINNT &h0601
#include "windows.bi"
#include "crt/stdio.bi"

'' ETW bits that FB don't have

Type TRACEHANDLE As ULONG64
Type PTRACEHANDLE As TRACEHANDLE Ptr

Const INVALID_TRACEHANDLE_VALUE As TRACEHANDLE = &hffffffffull

'' WNODE definition
Type WNODE_HEADER

    As ULONG BufferSize        '' Size of entire buffer inclusive of this ULONG
    As ULONG ProviderId    '' Provider Id of driver returning this buffer
    As ULONG64 HistoricalContext  '' Logger use
    As LARGE_INTEGER TimeStamp '' Timestamp as returned in units of 100ns
                                 '' since 1/1/1601
    As GUID Guid                  '' Guid for data block returned with results
    As ULONG ClientContext
    As ULONG Flags             '' Flags, see below
End Type

Type EVENT_TRACE_PROPERTIES
    As WNODE_HEADER Wnode
    As ULONG BufferSize
    As ULONG MinimumBuffers               '' minimum to preallocate
    As ULONG MaximumBuffers               '' maximum buffers allowed
    As ULONG MaximumFileSize              '' maximum logfile size (in MBytes)
    As ULONG LogFileMode                  '' sequential, circular
    As ULONG FlushTimer                   '' buffer flush timer, in seconds
    As ULONG EnableFlags                  '' trace enable flags
    As LONG  AgeLimit                     '' unused

'' data returned to caller
    As ULONG NumberOfBuffers              '' no of buffers in use
    As ULONG FreeBuffers                  '' no of buffers free
    As ULONG EventsLost                   '' event records lost
    As ULONG BuffersWritten               '' no of buffers written to file
    As ULONG LogBuffersLost               '' no of logfile write failures
    As ULONG RealTimeBuffersLost          '' no of rt delivery failures
    As HANDLE LoggerThreadId              '' thread id of Logger
    As ULONG LogFileNameOffset            '' Offset to LogFileName
    As ULONG LoggerNameOffset             '' Offset to LoggerName
End Type

''
'' EVENT_DESCRIPTOR describes and categorizes an event.
''
Type EVENT_DESCRIPTOR

    As USHORT      Id
    As UCHAR       Version
    As UCHAR       Channel
    As UCHAR       Level
    As UCHAR       Opcode
    As USHORT      Task
    As ULONGLONG   Keyword

End Type

Type EVENT_HEADER

    As USHORT              Size                   '' Event Size
    As USHORT              HeaderType             '' Header Type
    As USHORT              Flags                  '' Flags
    As USHORT              EventProperty          '' User given event property
    As ULONG               ThreadId               '' Thread Id
    As ULONG               ProcessId              '' Process Id
    As LARGE_INTEGER       TimeStamp              '' Event Timestamp
    As GUID                ProviderId             '' Provider Id
    As EVENT_DESCRIPTOR    EventDescriptor        '' Event Descriptor
    As ULONG64             ProcessorTime          '' Processor Clock
                                               '' for private session events
    As GUID                ActivityId             '' Activity Id

End Type

Type ETW_BUFFER_CONTEXT
    As UCHAR   ProcessorNumber
    As UCHAR   Alignment
    As USHORT  LoggerId
End Type

Union EventTrace_Context
    As ULONG               Client
    As ETW_BUFFER_CONTEXT  Buffer
End Union

Type EVENT_TRACE
    As WNODE_HEADER      Header             '' Event trace header
    As ULONG                   InstanceId         '' Instance Id of this event
    As ULONG                   ParentInstanceId   '' Parent Instance Id.
    As GUID                    ParentGuid         '' Parent Guid;
    As PVOID                   MofData            '' Pointer to Variable Data
    As ULONG                   MofLength          '' Variable Datablock Length
    As EventTrace_Context Context
End Type

Type PEVENT_TRACE As EVENT_TRACE Ptr

Type EVENT_HEADER_EXTENDED_DATA_ITEM

    As USHORT      Reserved1                      '' Reserved for internal use
    As USHORT      ExtType                        '' Extended info type
    As USHORT      Linkage             :  1       '' Indicates additional extended
    As USHORT      Reserved2           : 15
    As USHORT      DataSize                       '' Size of extended info data
    As ULONGLONG   DataPtr                        '' Pointer to extended info data

End Type

Type PEVENT_HEADER_EXTENDED_DATA_ITEM As EVENT_HEADER_EXTENDED_DATA_ITEM Ptr

Type EVENT_RECORD

    As EVENT_HEADER        EventHeader            '' Event header
    As ETW_BUFFER_CONTEXT  BufferContext          '' Buffer context
    As USHORT              ExtendedDataCount      '' Number of extended
                                                '' data items
    As USHORT              UserDataLength         '' User data length
    As PEVENT_HEADER_EXTENDED_DATA_ITEM _
                        ExtendedData           '' '' Pointer to an array of extended data items
    As PVOID               UserData               '' Pointer to user data
    As PVOID               UserContext            '' Context from OpenTrace
End Type

Type PEVENT_RECORD As EVENT_RECORD Ptr

Type PEVENT_TRACE_PROPERTIES As EVENT_TRACE_PROPERTIES Ptr

union ETL_ModeUnion
    '' Mode of the logfile
    As ULONG               LogFile
    '' Processing flags used on Vista and above
    As ULONG               ProcessTrace
End Union

Type PEVENT_CALLBACK As Sub stdcall ( ByVal pEvent As PEVENT_TRACE )
Type PEVENT_RECORD_CALLBACK As Sub stdcall ( ByVal EventRecord As PEVENT_RECORD )

union ETL_CallbackUnion
    '' Callback with EVENT_TRACE
    As PEVENT_CALLBACK         Event
    '' Callback with EVENT_RECORD on Vista and above
    As PEVENT_RECORD_CALLBACK  EventRecord
End Union

''
'' This is the header for every logfile. The memory for LoggerName
'' and LogFileName must be contiguous adjacent to this structure
'' Allows both user-mode and kernel-mode to understand the header.
''
'' TRACE_LOGFILE_HEADER32 and TRACE_LOGFILE_HEADER64 structures
'' are also provided to simplify cross platform decoding of the
'' header event.
''

Type TraceLogileHeader_Info
    As ULONG   StartBuffers       '' Count of buffers written at start.
    As ULONG   PointerSize        '' Size of pointer type in bits
    As ULONG   EventsLost         '' Events losts during log session
    As ULONG   CpuSpeedInMHz      '' Cpu Speed in MHz
End Type

Union TraceLogileHeader_InstanceInfo
    As GUID LogInstanceGuid
    As TraceLogileHeader_Info info
End Union

Type TRACE_LOGFILE_HEADER
    As ULONG           BufferSize         '' Logger buffer size in Kbytes
    As ULONG           Version            '' Logger version
    As ULONG           ProviderVersion    '' defaults to NT version
    As ULONG           NumberOfProcessors '' Number of Processors
    As LARGE_INTEGER   EndTime            '' Time when logger stops
    As ULONG           TimerResolution    '' assumes timer is constant!!!
    As ULONG           MaximumFileSize    '' Maximum in Mbytes
    As ULONG           LogFileMode        '' specify logfile mode
    As ULONG           BuffersWritten     '' used to file start of Circular File
    As TraceLogileHeader_InstanceInfo InstanceInfo
    As LPCWSTR          LoggerName
    As LPCWSTR          LogFileName
    As TIME_ZONE_INFORMATION TimeZone
    As LARGE_INTEGER   BootTime
    As LARGE_INTEGER   PerfFreq           '' Reserved
    As LARGE_INTEGER   StartTime          '' Reserved
    As ULONG           ReservedFlags      '' ClockType
    As ULONG           BuffersLost
End Type

Type EVENT_TRACE_LOGFILEW_ As EVENT_TRACE_LOGFILEW

Type PEVENT_TRACE_BUFFER_CALLBACKW As Function stdcall(ByVal Logfile As EVENT_TRACE_LOGFILEW_ Ptr) As ULONG

Type EVENT_TRACE_LOGFILEW
    As LPCWSTR                  LogFileName      '' Logfile Name
    As LPCWSTR                  LoggerName       '' LoggerName
    As LONGLONG                CurrentTime      '' timestamp of last event
    As ULONG                   BuffersRead      '' buffers read to date
    As ETL_ModeUnion           Modes
    As EVENT_TRACE             CurrentEvent     '' Current Event from this stream.
    As TRACE_LOGFILE_HEADER    LogfileHeader    '' logfile header structure
    As PEVENT_TRACE_BUFFER_CALLBACKW BufferCallback  '' callback before each buffer is read
    ''
    '' following variables are filled for BufferCallback.
    ''
    As ULONG                   BufferSize
    As ULONG                   Filled
    As ULONG                   EventsLost
    ''
    '' following needs to be propaged to each buffer
    ''
    As ETL_CallbackUnion Callback

    As ULONG                   IsKernelTrace    '' TRUE for kernel logfile

    As Any Ptr                   Context          '' reserved for internal use
End Type

Type PEVENT_TRACE_LOGFILEW As EVENT_TRACE_LOGFILEW Ptr

''
'' EVENT_FILTER_DESCRIPTOR is used to pass in enable filter
'' data item to a user callback function.
''
Type EVENT_FILTER_DESCRIPTOR

    As ULONGLONG   Ptr_
    As ULONG       Size_
    As ULONG       Type_

End Type

Type PEVENT_FILTER_DESCRIPTOR As EVENT_FILTER_DESCRIPTOR Ptr

Type ENABLE_TRACE_PARAMETERS
    As ULONG                    Version
    AS ULONG                    EnableProperty
    As ULONG                    ControlFlags
    As GUID                     SourceId
    As PEVENT_FILTER_DESCRIPTOR EnableFilterDesc '' don't need this
End Type

Type PENABLE_TRACE_PARAMETERS As ENABLE_TRACE_PARAMETERS Ptr

#define EVENT_HEADER_PROPERTY_XML               &h0001
#define EVENT_HEADER_PROPERTY_FORWARDED_XML     &h0002
#define EVENT_HEADER_PROPERTY_LEGACY_EVENTLOG   &h0004

#define EVENT_HEADER_FLAG_EXTENDED_INFO         &h0001
#define EVENT_HEADER_FLAG_PRIVATE_SESSION       &h0002
#define EVENT_HEADER_FLAG_STRING_ONLY           &h0004
#define EVENT_HEADER_FLAG_TRACE_MESSAGE         &h0008
#define EVENT_HEADER_FLAG_NO_CPUTIME            &h0010
#define EVENT_HEADER_FLAG_32_BIT_HEADER         &h0020
#define EVENT_HEADER_FLAG_64_BIT_HEADER         &h0040
#define EVENT_HEADER_FLAG_CLASSIC_HEADER        &h0100

#define ENABLE_TRACE_PARAMETERS_VERSION 1
#define WNODE_FLAG_TRACED_GUID   &h00020000 '' denotes a trace
#define EVENT_TRACE_REAL_TIME_MODE          &h00000100  '' Real time mode on
#define EVENT_CONTROL_CODE_DISABLE_PROVIDER 0
#define EVENT_CONTROL_CODE_ENABLE_PROVIDER  1
#define EVENT_CONTROL_CODE_CAPTURE_STATE    2
#define PROCESS_TRACE_MODE_REAL_TIME                &h00000100
#define PROCESS_TRACE_MODE_RAW_TIMESTAMP            &h00001000
#define PROCESS_TRACE_MODE_EVENT_RECORD             &h10000000
#define TRACE_LEVEL_NONE        0   '' Tracing is not on
#define TRACE_LEVEL_CRITICAL    1   '' Abnormal exit or termination
#define TRACE_LEVEL_FATAL       1   '' Deprecated name for Abnormal exit or termination
#define TRACE_LEVEL_ERROR       2   '' Severe errors that need logging
#define TRACE_LEVEL_WARNING     3   '' Warnings such as allocation failure
#define TRACE_LEVEL_INFORMATION 4   '' Includes non-error cases(e.g.,Entry-Exit)
#define TRACE_LEVEL_VERBOSE     5   '' Detailed traces from intermediate steps

Extern "Windows"

Declare Function StartTraceW(ByVal Trace As PTRACEHANDLE, ByVal InstanceName As LPCWSTR, ByVal Properties As PEVENT_TRACE_PROPERTIES) As ULONG
Declare Function OpenTraceW(ByVal LogFile As PEVENT_TRACE_LOGFILEW) As TRACEHANDLE
Declare Function ProcessTrace( _
    ByVal HandleArray As PTRACEHANDLE, _
    ByVal HandleCount As ULONG, _
    ByVal StartTime As LPFILETIME, _
    ByVal EndTime As LPFILETIME _
) As ULONG
Declare Function StopTraceW( _
    ByVal Trace As TRACEHANDLE, _
    ByVal InstanceName As LPCWSTR, _
    ByVal Properties As PEVENT_TRACE_PROPERTIES _
) As ULONG

Declare Function CloseTrace(ByVal Trace As TRACEHANDLE) As ULONG

'' Windows 7
Declare Function EnableTraceEx2( _
    ByVal Trace As TRACEHANDLE, _
    ByVal ProviderId As LPCGUID, _
    ByVal ControlCode As ULONG, _
    ByVal Level As UCHAR, _
    ByVal MatchAnyKeyword As ULONGLONG, _
    ByVal MatchAllKeyword As ULONGLONG, _
    ByVal Timeout As ULONG, _
    ByVal EnableParameters As PENABLE_TRACE_PARAMETERS _
) As ULONG

End Extern

'' end of Windows ETW bits

'' https://www.geoffchappell.com/studies/windows/km/ntoskrnl/events/microsoft-windows-kernel-power.htm
'' wevtutil gp Microsoft-Windows-Kernel-Power
Static Shared g_MicrosoftKernelPowerDiagnostic As GUID = Type(&h331C3B3A, &h2005, &h44c2, { &hac, &h5e, &h77, &h22, &h0c, &h37, &hd6, &hb4})
'' This guid can be anything, it's nothing special
Static Shared g_traceSessionGuid As GUID = Type(&h11e0d73c, &h497d, &h4aad, { &h90, &ha4, &h51, &hfb, &hbc, &h6c, &h9b, &h40})

Type SessionContext
    As TRACEHANDLE hTrace
    As PEVENT_TRACE_PROPERTIES pProps
End Type

Static Shared g_sessCtx As SessionContext
Static Shared g_exit As BOOL

'' Event ids here
'' https://github.com/jdu2600/Windows10EtwEvents/blob/master/manifest/Microsoft-Windows-Kernel-Power.tsv
const POP_ETW_EVENT_STRS As USHORT = 63
const POP_ETW_EVENT_TIME_RESOLUTION_UPDATE As USHORT = 95
const POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN As USHORT = 96
const POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN As USHORT = 97
const POP_ETW_EVENT_KERNEL_STRS As USHORT = 98
const POP_ETW_EVENT_TIME_RESOLUTION_STACK_RUNDOWN As USHORT = 110

Type POP_ETW_EVENT_STRS_data_V0 '' Win Vista
    As ULONG requestedResolution
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength WCHARS
End Type

Type POP_ETW_EVENT_STRS_data_V1 '' Win 7+
    As ULONG requestedResolution
    As ULONG pid
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength WCHARS
    '' As ULONG subProcessTag; '' this comes after the app Name
End Type

Type POP_ETW_EVENT_STRS_data_V2 '' Win 11
    As ULONG requestedResolution
    As ULONG pid
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength WCHARS
    ''As ULONG subProcessTag '' this comes after the app Name
    ''As ULONG requestIgnored '' after subProcessTag
End Type

Type POP_ETW_EVENT_TIME_RESOLUTION_UPDATE_data
    As ULONG newResolution
End Type

Type POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN_data_V0 '' Win7 +
    As ULONG currentPeriod
    As ULONG minimumPeriod
    As ULONG maximumPeriod
    As ULONG kernelRequestCount
    As ULONG kernelRequestedPeriod
End Type
Type POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN_data_V1 '' Win10 1803+
    As ULONG currentPeriod
    As ULONG minimumPeriod
    As ULONG maximumPeriod
    As ULONG kernelRequestCount
    As ULONG kernelRequestedPeriod
    As ULONG internalSetPeriod
End Type

Type POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN_data_V0 '' Win7+
    As ULONG requestedPeriod
    As ULONG pid
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength characters
End Type
Type POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN_data_V1 '' Win11
    As ULONG requestedPeriod
    As ULONG pid
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength characters
    '' As ULONG requestIgnored '' after app name
End Type

Type POP_ETW_EVENT_KERNEL_STRS_data_V0
    As ULONG requestedResolution
End Type
Type POP_ETW_EVENT_KERNEL_STRS_data_V1
    As ULONG requestedResolution
    As ULONG tag
End Type

Type POP_ETW_EVENT_TIME_RESOLUTION_STACK_RUNDOWN_data '' Win7+
    As ULONG requestedPeriod
    As ULONG pid
    As USHORT appNameLength
    As WCHAR appName(ANYSIZE_ARRAY) '' appNameLength characters
    '' As ULONG stackSize
    '' As ULONG stack(ANYSIZE_ARRAY)
End Type

Sub ShutdownTracing(ByVal hTrace As TRACEHANDLE, ByVal pProps As EVENT_TRACE_PROPERTIES ptr)

    puts("Shutting down tracing")
    Dim stat as ULONG = EnableTraceEx2( _
        hTrace, _
        @g_MicrosoftKernelPowerDiagnostic, _
        EVENT_CONTROL_CODE_DISABLE_PROVIDER, _
        TRACE_LEVEL_NONE, _
        MAXDWORD64, _
        0, _
        INFINITE, _
        NULL _
    )
    If stat <> ERROR_SUCCESS Then
        printf(!"Disabling provider failed with error %lu\n", stat)
    End If

    stat = StopTraceW(hTrace, cast(PCWSTR, cast(Byte Ptr, pProps) + pProps->LoggerNameOffset), pProps)
    If stat <> ERROR_SUCCESS Then
        printf(!"Stopping trace session failed with error %lu\n", stat)
    End If
End Sub

Sub ProcessEvents stdcall(ByVal pEvent As PEVENT_RECORD)

    Dim ByRef evDesc As EVENT_DESCRIPTOR = pEvent->EventHeader.EventDescriptor
    Dim version As USHORT = evDesc.Version
    Dim evId As USHORT = evDesc.Id
    Dim pUserData As Any Ptr = pEvent->UserData
    If g_exit AndAlso (g_sessCtx.hTrace <> 0) Then
        '' This is here and not in the main thread as StopTrace will write to the props
        '' structure. If we do it in the other thread and this thread exits before the write
        '' completes, the app will crash since the buffer is on this thread's stack which will
        '' have been free
        ShutdownTracing(g_sessCtx.hTrace, g_sessCtx.pProps)
        g_sessCtx.hTrace = 0
    End If
    Select Case evId

        case POP_ETW_EVENT_STRS

            If version = 0 Then

                Dim pEvData As POP_ETW_EVENT_STRS_data_V0 ptr = pUserData
                wprintf(@WStr(!"Process %*s requested a resolution of %luns\n"), pEvData->appNameLength, @pEvData->appName(0), pEvData->requestedResolution)

            ElseIf (version = 1) OrElse (version = 2) Then

                Dim pEvData As POP_ETW_EVENT_STRS_data_V1 ptr = pUserData
                Dim pSubProcTag As ULONG ptr = cast(ULONG ptr, @pEvData->appName(0) + pEvData->appNameLength)
                wprintf(@WStr(!"Process %lu (%.*s, tag %lu), requested a resolution of %luns"), pEvData->pid, pEvData->appNameLength, @pEvData->appName(0), *pSubProcTag, pEvData->requestedResolution)
                If version = 2 Then
                    Dim pReqIgnored As ULONG ptr = pSubProcTag + 1
                    wprintf(@WStr(" - Request %s"), IIf(*pReqIgnored, @WStr("ignored"), @WStr("allowed")))
                End If
                putwchar(Asc(!"\n"))
            Else
                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_STRS\n"), version)
            End If

        case POP_ETW_EVENT_TIME_RESOLUTION_UPDATE

            If(version = 0) Then

                Dim pEvData As POP_ETW_EVENT_TIME_RESOLUTION_UPDATE_data ptr = pUserData
                wprintf(@WStr(!"Timer resolution updated to %luns by process %lu\n"), pEvData->newResolution, pEvent->EventHeader.ProcessId)

            Else

                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_TIME_RESOLUTION_UPDATE\n"), version)
            End If

        case POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN
        '' This event is only sent when EnableTraceEx2 is called with the DUMP_STATE value
            If (version = 0) OrElse (version = 1) Then

                Dim pEvData as POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN_data_V1 ptr = pUserData
                wprintf( _
                    @WStr(!"Current system values:\nCurrent period - %luns\nmin period - %luns\nmax period - %luns\nkernelRequestCount = %lu\nkernelRequestedPeriod = %luns\n"), _
                    pEvData->currentPeriod, _
                    pEvData->minimumPeriod, _
                    pEvData->maximumPeriod, _
                    pEvData->kernelRequestCount, _
                    pEvData->kernelRequestedPeriod _
                )
                If(version = 1) Then

                    wprintf(@WStr(!"Internal set period: %lu\n"), pEvData->internalSetPeriod)
                End If
                putwchar(Asc(!"\n"))

            Else

                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_TIME_RESOLUTION_RUNDOWN\n"), version)
            End If

        case POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN
        '' This event is only sent when EnableTraceEx2 is called with the DUMP_STATE value

            If (version = 0) OrElse (version = 1) Then

                Dim pEvData As POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN_data_V1 ptr = pUserData
                wprintf(@WStr("Process %.*s (%lu) has active resoution request %luns"), pEvData->appNameLength, @pEvData->appName(0), pEvData->pid, pEvData->requestedPeriod)
                If(version = 1) Then

                    Dim pIgnored As ULONG ptr = cast(ULONG ptr, @pEvData->appName(0) + pEvData->appNameLength)
                    wprintf(@WStr(" - request %s"), Iif(*pIgnored, @WStr("ignored"), @WStr("allowed")))
                End If
                putwchar(Asc(!"\n"))

            Else

                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_TIME_RESOLUTION_REQUEST_RUNDOWN\n"), version)
            End If

        case POP_ETW_EVENT_KERNEL_STRS

            If (version = 0) OrElse (version = 1) Then

                Dim pEvData As POP_ETW_EVENT_KERNEL_STRS_data_V1 ptr = pUserData
                wprintf(@WStr("A driver requested a timer resolution of %luns"), pEvData->requestedResolution)
                If(version = 1) Then

                    wprintf(@WStr(" - tag = %x"), pEvData->tag)
                End If
                putwchar(Asc(!"\n"))

            Else

                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_KERNEL_STRS\n"), version)
            End If

        case POP_ETW_EVENT_TIME_RESOLUTION_STACK_RUNDOWN
        '' This event is only sent when EnableTraceEx2 is called with the DUMP_STATE value

            If(version = 0) Then

                Dim pEvData as POP_ETW_EVENT_TIME_RESOLUTION_STACK_RUNDOWN_data ptr = pUserData
                Dim pStackCount As ULONG ptr = cast(ULONG ptr, @pEvData->appName(0) + pEvData->appNameLength)
                Dim is64Event As BOOL = pEvent->EventHeader.Flags And EVENT_HEADER_FLAG_64_BIT_HEADER
                Dim pStackEntries As ULONG ptr = pStackCount + 1
                wprintf(@WStr(!"Process %.*s (%lu) previously requested resoution %luns\n"), pEvData->appNameLength, @pEvData->appName(0), pEvData->pid, pEvData->requestedPeriod)
                wprintf(@WStr(!"From stack:\n"))
                If(is64Event) Then

                    For i As ULONG = 0 To (*pStackCount - 1) Step 2

                        wprintf(@WStr(!"%lu: %#I64x\n"), (i \ 2) + 1, *cast(ULONG64 ptr, @pStackEntries[i]))
                    Next

                Else

                    For i As ULONG = 0 To (*pStackCount - 1)

                        wprintf(@WStr(!"%lu: %#x\n"), i + 1, pStackEntries[i])
                    Next
                End If

            Else

                wprintf(@WStr(!"Unknown version %hu of POP_ETW_EVENT_TIME_RESOLUTION_STACK_RUNDOWN\n"), version)
            End If
    End Select
End Sub

'' The function ProcessTrace blocks the thread while messages are being delivered
'' (and as we're using realtime events instead of ones from a log file, it will never naturally end)
'' So it's not a good idea to put this in a UI thread
Sub DoTracing(ByVal p As Any Ptr)

    Dim buffer(1000) As Byte
    Dim pSessionName As WString Ptr = @WStr("TimerResoutionLoggerSession")
    Dim pProps As EVENT_TRACE_PROPERTIES ptr = cast(EVENT_TRACE_PROPERTIES ptr, @buffer(0))
    pProps->Wnode.BufferSize = UBound(buffer)
    pProps->Wnode.Flags = WNODE_FLAG_TRACED_GUID
    pProps->Wnode.ClientContext = 1 '' QPC clock resolution
    pProps->Wnode.Guid = g_traceSessionGuid
    pProps->LogFileMode = EVENT_TRACE_REAL_TIME_MODE
    pProps->MaximumFileSize = 1  '' 1 MB
    pProps->LoggerNameOffset = sizeof(EVENT_TRACE_PROPERTIES)
    *cast(WString Ptr, pProps + 1) = *pSessionName

    '' Unlike pretty much everything else your app creates, Windows won't
    '' stop or delete the trace session you set up if your app exits without
    '' stopping it. So if your app crashes and you start it again and call StartTrace again
    '' with the same name or guid - it'll fail saying it already exists
    ''
    '' This stop trace ensures that doesn't happen. There must be a way to get a handle to
    '' an existing session but
    StopTraceW(0, pSessionName, pProps)

    '' po:Diagnostic | po:TimerResolution
    Dim enableBits As ULONGLONG = &h4000000000004004
    Dim etp As ENABLE_TRACE_PARAMETERS
    Dim etl As EVENT_TRACE_LOGFILEW
    Dim hTraceConsumer As TRACEHANDLE
    Dim hTraceSession As TRACEHANDLE

    Dim stat As ULONG = StartTraceW(@hTraceSession, pSessionName, pProps)
    If (stat <> ERROR_SUCCESS) Then

        printf(!"StartTrace failed with error %lu\n", stat)
        Goto ExitPart
    End If

    wprintf(@WStr(!"Started trace session %s\n"), pSessionName)
    g_sessCtx.hTrace = hTraceSession
    g_sessCtx.pProps = pProps
    etl.LoggerName = pSessionName
    etl.Modes.ProcessTrace = PROCESS_TRACE_MODE_REAL_TIME Or PROCESS_TRACE_MODE_EVENT_RECORD
    etl.Callback.EventRecord = @ProcessEvents
    hTraceConsumer = OpenTraceW(@etl)
    if(hTraceConsumer = INVALID_TRACEHANDLE_VALUE) Then

        printf(!"OpenTrace failed with error %lu\n", GetLastError())
        Goto ExitPart
    End If

    etp.Version = ENABLE_TRACE_PARAMETERS_VERSION
    etp.SourceId = pProps->Wnode.Guid
    stat = EnableTraceEx2( _
        hTraceSession, _
        @g_MicrosoftKernelPowerDiagnostic, _
        EVENT_CONTROL_CODE_ENABLE_PROVIDER, _
        TRACE_LEVEL_INFORMATION, _
        enableBits, _
        0, _
        INFINITE, _
        @etp _
    )
    If(stat <> ERROR_SUCCESS) Then

        printf(!"EnableTraceEx failed with error %lu\n", stat)
        If(stat = ERROR_ACCESS_DENIED) Then
            puts("User must be an administrator or a member of the 'Performance Log Users' group")
        End If
        Goto ExitPart
    End If
    puts("Starting processing")
    '' Dump the initial state
    EnableTraceEx2( _
        hTraceSession, _
        @g_MicrosoftKernelPowerDiagnostic, _
        EVENT_CONTROL_CODE_CAPTURE_STATE, _
        TRACE_LEVEL_NONE, _
        0, _
        0, _
        INFINITE, _
        NULL _
    )
    ProcessTrace(@hTraceConsumer, 1, NULL, NULL)
ExitPart:
    CloseTrace(hTraceConsumer)
    StopTraceW(hTraceSession, pSessionName, pProps)
    g_exit = TRUE
End Sub

Dim pThread As Any Ptr = ThreadCreate(@DoTracing)
While g_exit = FALSE
    Sleep 5
    If InKey() <> "" Then g_exit = TRUE
Wend
Print "Exit signalled"
ThreadWait(pThread)
```

## Example Output

```
Started trace session TimerResoutionLoggerSession
Starting processing
Current system values:
Current period - 100000ns
min period - 5000ns
max period - 156001ns
kernelRequestCount = 0
kernelRequestedPeriod = 0ns

Process \Device\HarddiskVolume2\Program Files (x86)\Mozilla Firefox\firefox.exe (4824) previously requested resoution 10000ns
From stack:
1: 0x770aaeea
2: 0x7fef7dda710
3: 0x7fee8c04513
4: 0x7fee9809ff1
5: 0x7feec536cf2
6: 0x7feec537087
7: 0x7feec53a1d1
8: 0x7fee98e937b
9: 0x7fee98e710b
10: 0x7feeac8e37d
11: 0x7fee8cadf11
12: 0x7feec6c370e
13: 0x7feec6c3930
14: 0x7fee9392e5e
15: 0x7fee937a783
16: 0x7fee937a09d
Process 568 (\Device\HarddiskVolume2\Windows\System32\svchost.exe, tag 62), requested a resolution of 10000ns
Timer resolution updated to 10000ns by process 568
Process 568 (\Device\HarddiskVolume2\Windows\System32\svchost.exe, tag 62), requested a resolution of 100000ns
Timer resolution updated to 100000ns by process 568
Process 2236 (\Device\HarddiskVolume2\Program Files (x86)\Mozilla Firefox\firefox.exe, tag 0), requested a resolution of 110000ns
Process 2236 (\Device\HarddiskVolume2\Program Files (x86)\Mozilla Firefox\firefox.exe, tag 0), requested a resolution of 0ns
Process 2236 (\Device\HarddiskVolume2\Program Files (x86)\Mozilla Firefox\firefox.exe, tag 0), requested a resolution of 90000ns
Timer resolution updated to 50000ns by process 2236
Process 2236 (\Device\HarddiskVolume2\Program Files (x86)\Mozilla Firefox\firefox.exe, tag 0), requested a resolution of 0ns
Timer resolution updated to 100000ns by process 2236
```

The requests for 0ns seem to be an artifact of timeEndPeriod.
