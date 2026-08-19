# Event Tracing MOF Qualifiers - Win32 apps | Microsoft Learn

Use the qualifiers defined in this section when creating your provider MOF class, event MOF class, event type MOF class, and the properties of the event type MOF class. For an example that includes some of these qualifiers, see [Publishing Your Event Schema](publishing-your-event-schema).

## Provider MOF class qualifiers

The following table lists the qualifiers you can specify on a provider MOF class.

| Qualifier | Data type | Description |
| --- | --- | --- |
| **Guid** | **String** | Required. String Guid that uniquely identifies a provider. For example, Guid("{3F92E6E0-9886-434e-85DB-0D11D3904C0A}"). This is the same GUID you use when you call the [**RegisterTraceGuids**](/en-us/windows/win32/api/evntrace/nf-evntrace-registertraceguidsa) function to register your provider. |

## Event MOF class qualifiers

The following table lists the qualifiers you can specify on an event class (the parent class that groups related event type classes).

| Qualifier | Data type | Description |
| --- | --- | --- |
| **Guid** | **String** | Required. String Guid that identifies a class of events. For example, Guid("{3F92E6E0-9886-434e-85DB-0D11D3904C0A}"). Event providers use the Guid to set the [**EVENT\_TRACE\_HEADER.Guid**](/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_header) member, so that consumers can determine the class of events they are receiving. |
| **EventVersion** | **Integer** | This qualifier is optional for the latest version of an event trace class and is required for all older versions of the class. The latest version of the class either does not specify the **EventVersion** qualifier or has the highest version number. Version numbers begin with 0, for example, EventVersion(0).Typically, when you create a new version of the class, you also rename the previous version to &lt;classname&gt;\_Vn, where n is an incremental number starting at 0. For an example, see [**FileIo**](fileio) and [**FileIo\_V0**](fileio-v0). |

## Event type MOF class qualifiers

The following table lists the qualifiers you can specify on an event type class (the class that defines the event property data).

| Qualifier | Value | Description |
| --- | --- | --- |
| **EventType** | **Integer** | Required. Identifies the event type class. For example, EventType(1). The event provider uses the same event type value to set [**EVENT\_TRACE\_HEADER.Class.Type**](/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_header). If the same MOF class is used for multiple event types (because they use the same event data), specify the event type value as an array of integers, for example, EventType{12,15}. |
| **EventTypeName** | **String** | Optional. Describes the event type. For example, EventTypeName("Start"). If the same MOF class is used for multiple event types (because they use the same event data), specify the event type name value as an array of strings, for example, EventTypeName{"Start", "End"}. The elements of the EventTypeName array correspond directly to the EventType array. |

## Property qualifiers

The following table lists the qualifiers you can specify on a property.

| Qualifier | Description |
| --- | --- |
| **BitMap** | Specifies the bit positions that map to string values. If you specify this qualifier, you must also specify the **BitValues** qualifier. |
| **BitValues** | String values. If the **BitMap** qualifier is also specified, the strings correspond directly to the values in the **BitMap** qualifier. Otherwise, assume the property value is a one-based index into the value strings (bit one corresponds to the first string in the list). |
| **Extension** | Provides additional information on how to consume (interpret) the data. The extension value is case-insensitive. Include the value in quotes, for example, Extension("Guid"). Possible extension values are: <br>- Guid<br>    - Indicates the property data is a Guid. The MOF data type must be **object**. The payload is expected to be a **GUID** structure.<br>- IPAddr and IPAddrV4<br>    - The data is an IP V4 address. The MOF data type must be **object**. The payload is expected to be an unsigned long. Each byte of the unsigned long represents one of the four parts of the IP address (p1.p2.p3.p4). The low-order byte contains the value for p1, the next byte contains the value for p2, and so on.**Prior to Windows Vista:** The IPAddrV4 extension is not supported.<br>- IPAddrV6<br>    - The data is an IP V6 address. The MOF data type must be **object**. The payload is expected to be an **IN6\_ADDR** structure. **Prior to Windows Vista:** The IPAddrV6 extension is not supported.<br>- NoPrint<br>    - Indicates that the consumer should not print this data.<br>- Port<br>    - The data identifies a port number. The MOF data type must be **object**. The payload is expected to be an unsigned short.<br>- RString<br>    - Newline characters were replaced with spaces. The payload is expected to be a null-terminated, ANSI string.<br>- RWString<br>    - Newline characters were replaced with spaces. The payload is expected to be a null-terminated, wide-character string.<br>- Sid<br>    - The data represents a binary blob SID. The MOF data type must be **object**.  The SID is of a variable length. The value contained in the first 4-bytes (**ULONG**) indicates whether the blob contains a SID. If the first 4-bytes (**ULONG**) of the blob is nonzero, the blob contains a SID. The first part of the blob contains the TOKEN\_USER (the structure is aligned on an 8-byte boundary) and the second part contains the SID. To address the SID portion of the blob:<br>- Set a byte pointer to the beginning of the blob<br>- Multiply the pointer size for the event log by 2 and add the product to the byte pointer (the **PointerSize** member of [**TRACE\_LOGFILE\_HEADER**](/en-us/windows/win32/api/evntrace/ns-evntrace-trace_logfile_header) contains the pointer size value)<br><br> You can use the following macro to determine the length of the SID. <br><br>```<br>#define SeLengthSid( Sid ) \<br>  (8 + (4 * ((SID *)Sid)->SubAuthorityCount))<br>```<br>- SizeT<br>    - Indicates that the property contains a pointer value. The size of the pointer value depends on the operating system used to log the event; the payload will contain a 4-byte value for 32-bit systems or an 8-byte value for 64-bit systems. The MOF data type must be **object**. Consumers should ignore the data type and **Format** qualifier if the property includes the **SizeT** extension. To determine the size of data to read for the property, use: <br>- The **PointerSize** member of [**TRACE\_LOGFILE\_HEADER**](/en-us/windows/win32/api/evntrace/ns-evntrace-trace_logfile_header)<br>- The **Flags** member of [**EVENT\_HEADER**](/en-us/windows/desktop/api/evntcons/ns-evntcons-event_header)<br><br>**Prior to Windows Vista:** The **PointerSize** value may not be accurate. For example, on a 64-bit computer, a 32-bit application will log 4-byte pointers; however, the session will set **PointerSize** to 8.<br>- Variant<br>    - The data represents a blob. The first four bytes (uint32) indicate the size of the blob. The MOF data type must be **object**.<br>- WmiTime<br>    - Translates the time stamp to system time. The MOF data type must be **object**. The payload is expected to be an unsigned 64-bit integer.**Prior to Windows Vista:** Not available. |
| **Format** | Defines the format of the property data. For example, including Format("w") on a string property indicates the string is a wide string. Possible values are: <br><br>| Term | Description |<br>| --- | --- |<br>| c | Display the property value as an ASCII character. You can use this qualifier with **uint8** data types. |<br>| s | Treat the array of characters as a null-terminated string. The string is a wide-character string if the data type is **char16**; otherwise, the string is an ASCII character string. |<br>| w | The property value is a wide-character string. You can use this qualifier with **string** data types. |<br>| x | Display the property value as a hexadecimal number. You can use this qualifier with 16-, 32-, and 64-bit integer data types. | |
| --- | --- |
| **Pointer** | Indicates that the property contains a pointer value. The size of the pointer value depends on the operating system used to log the event; the payload will contain a 4-byte value for 32-bit systems or an 8-byte value for 64-bit systems. The MOF data type must be **object**.<br><br>Consumers should ignore the data type and **Format** qualifier if the property includes the **SizeT** extension. To determine the size of data to read for the property, use:<br><br>- The **PointerSize** member of [**TRACE\_LOGFILE\_HEADER**](/en-us/windows/win32/api/evntrace/ns-evntrace-trace_logfile_header)<br>- The **Flags** member of [**EVENT\_HEADER**](/en-us/windows/desktop/api/evntcons/ns-evntcons-event_header)<br><br><br>**Prior to Windows Vista:** The **PointerSize** value may not be accurate. For example, on a 64-bit computer, a 32-bit application will log 4-byte pointers; however, the session will set **PointerSize** to 8.<br><br>Note that some events use **PointerType** instead of **Pointer**; do not use **PointerType**. |
| **StringTermination** | Indicates how the string property is terminated. For example, StringTermination("NullTerminated") indicates the string property is null-terminated. Possible values are: <br>- **Counted**<br>    - The length of the string is embedded at the beginning of the string as a **USHORT** value.<br>- **NotCounted**<br>    - The string is not null-terminated and the length of the string is not embedded at the beginning of the string. In this case, the string should be the last element and occupy all the space to the end of the event data.<br>- **NullTerminated**<br>    - The string is null-terminated. If you do not specify the **StringTermination** qualifier, the string is assumed to be null-terminated.<br>- **ReverseCounted**<br>    - The length of the string is embedded at the beginning of the string as a **USHORT** value in big-endian format. |
| **ValueDescriptions** | Provides descriptions for each value in the **Values** qualifier. The [**TdhEnumerateProviderFieldInformation**](/en-us/windows/desktop/api/Tdh/nf-tdh-tdhenumerateproviderfieldinformation) and [**TdhQueryProviderFieldInformation**](/en-us/windows/desktop/api/Tdh/nf-tdh-tdhqueryproviderfieldinformation) functions return these descriptions when you try to retrieve keyword and level information. The descriptions are optional. If you do not provide the descriptions, the functions return **NULL**. See Specifying level and enable flags values for a provider for more details. |
| **ValueMap** | Specifies the integer index or flag values that map to string values. If you specify this qualifier, you must also specify the **Values** qualifier, and optionally the **ValueType** qualifier. Note that ETW does not support the WMI option of having strings for value map values. <br>The following example shows how to use the ValueMap, Values, and ValueType qualifiers.<br><br><br>```<br>ValueType("flag"),<br>ValueMap {"0x01", "0x02", "0x04", "0x08"},<br>Values {"ValueMapFlag1", "ValueMapFlag2", "ValueMapFlag4", "ValueMapFlag8"}]<br>``` |
| **Values** | String values. If the **ValueMap** qualifier is also specified, the strings correspond directly to the values in the **ValueMap** qualifier. Otherwise, assume the property value is a zero-based index into the value strings. |
| **ValueType** | Indicates if the **ValueMap** values are integer index values or bit flag values. If you do not specify this qualifier, integer index values are assumed. To specify that the values are integer index values, use ValueType("index"). To specify that the values are bit flag values, use ValueType("flag"). |
| **WmiDataId** | Each property must contain the **WmiDataId** qualifier. **WmiDataId** defines the order in which the consumer reads the event data. The value for **WmiDataId** starts at 1 and increments for each property in the class. For example, WmiDataId(1). |
| **XMLFragment** | Indicates that the data is in XML format and ready to display without further formatting. |

## Specifying level and enable flags values for a provider

To document the level and enable flags that a controller would use to enable your provider, include the "Level" and "Flags" properties in your provider MOF class. The Level and Flags property names are case sensitive. The properties must include the **Values** and **ValueMap** qualifiers, which specify the possible level and enable flag values. The **ValueMap** for the enable flag values must be bit (flag) values. The **ValueDescriptions** qualifier is optional but you should use it to provide descriptions for each possible value. The descriptions are used when someone calls the [**TdhEnumerateProviderFieldInformation**](/en-us/windows/desktop/api/Tdh/nf-tdh-tdhenumerateproviderfieldinformation) and [**TdhQueryProviderFieldInformation**](/en-us/windows/desktop/api/Tdh/nf-tdh-tdhqueryproviderfieldinformation) functions to get the possible level and enable flags (keywords) values for the provider.

The following shows a provider class that specifies the possible level and enable flags values.

```syntax
[Dynamic,
 Description("IIS_Trace") : amended,
 guid("{3a2a4e84-4c21-4981-ae10-3fda0d9b0f83}"),
 locale("MS\\0x409")]
class IIS_Trace : EventTrace
{
    [Description ("Enable Flags") : amended,
        ValueDescriptions{
             "Allow_tracing_only_selected_requests ",
             "IIS_authentication_events ",
             "IIS_security_events ",
             "IIS_filter_events ",
             "IIS_static_file_events ",
             "IIS_CGI_events ",
             "IIS_compression_events ",
             "IIS_cache_events ",
             "IIS_request_notifications_events ",
             "IIS_module_events ",
             "IIS_FastCGI_events "},
        DefineValues{
             "UseUrlFilter",
             "IISAuthentication",
             "IISSecurity",
             "IISFilter",
             "IISStaticFile",
             "IISCGI",
             "IISCompression",
             "IISCache",
             "IISRequestNotification",
             "IISModule",
             "IISFastCGI"},
        Values{
             "UseUrlFilter",
             "IISAuthentication",
             "IISSecurity",
             "IISFilter",
             "IISStaticFile",
             "IISCGI",
             "IISCompression",
             "IISCache",
             "IISRequestNotification",
             "IISModule",
             "IISFastCGI"},
        ValueMap{
             "0x00000001",
             "0x00000002",
             "0x00000004",
             "0x00000008",
             "0x00000010",
             "0x00000020",
             "0x00000040",
             "0x00000080",
             "0x00000100",
             "0x00000200",
             "0x00001000"}: amended
    ]
    uint32 Flags;

    [Description ("Levels") : amended,
        ValueDescriptions{
            "Abnormal exit or termination",
            "Severe errors that need logging",
            "Warnings such as allocation failure",
            "Includes non-error cases",
            "Detailed traces from intermediate steps" } : amended,
         DefineValues{
            "TRACE_LEVEL_FATAL",
            "TRACE_LEVEL_ERROR",
            "TRACE_LEVEL_WARNING"
            "TRACE_LEVEL_INFORMATION",
            "TRACE_LEVEL_VERBOSE" },
        Values{
            "Fatal",
            "Error",
            "Warning",
            "Information",
            "Verbose" },
        ValueMap{
            "0x1",
            "0x2",
            "0x3",
            "0x4",
            "0x5" },
        ValueType("index")
    ]
    uint32 Level;
};
```
