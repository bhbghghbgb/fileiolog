use std::fmt;

use serde::Serialize;

/// Lightweight key to identify an event type.
///
/// Based on how ferrisetw/krabsetw cache schemas:
/// - Provider GUID + Event ID + Version define the canonical event type
///   (per Microsoft docs, these 3 fields are the schema identity)
/// - Opcode is added because kernel providers use opcode to distinguish
///   sub-types within the same event ID (e.g. FileIo opcode 67=Read, 68=Write)
///
/// Level and keyword are runtime filtering parameters, not event type identifiers.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EventTypeId {
    pub provider_guid: windows::core::GUID,
    pub event_id: u16,
    pub version: u8,
    pub opcode: u8,
}

impl EventTypeId {
    #[allow(dead_code)]
    pub fn format_key(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}_{}_v{}_op{}",
            self.provider_guid.data1,
            self.provider_guid.data2,
            self.provider_guid.data3,
            self.event_id,
            self.version,
            self.opcode,
        )
    }
}

/// Information about a single property in an event schema.
/// Stores only raw numeric types; name strings are computed on demand.
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub in_type: u16,
    pub out_type: u16,
    pub length: PropertyLengthInfo,
    pub count: Option<PropertyCountInfo>,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub enum PropertyLengthInfo {
    Fixed(u16),
    Index(u16),
}

#[derive(Debug, Clone)]
pub enum PropertyCountInfo {
    Fixed(u16),
    Index(u16),
}

/// Schema-level information about an event type.
/// Stores only raw data; name strings are computed on demand for JSON output.
#[derive(Debug, Clone)]
pub struct EventTypeInfo {
    // Identity
    pub provider_guid: String,
    pub event_id: u16,
    pub opcode: u8,
    pub version: u8,

    // Schema metadata (from TDH)
    pub provider_name: String,
    pub opcode_name: String,

    // Property definitions
    pub properties: Vec<PropertyInfo>,
}

impl fmt::Display for EventTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} (opcode={}, v{}, id={}) - {} properties",
            self.provider_name,
            self.opcode_name,
            self.opcode,
            self.version,
            self.event_id,
            self.properties.len(),
        )
    }
}

impl fmt::Display for PropertyInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: InType={} OutType={}",
            self.name,
            in_type_name(self.in_type),
            out_type_name(self.out_type),
        )?;
        match self.length {
            PropertyLengthInfo::Fixed(len) => write!(f, " Len={}", len)?,
            PropertyLengthInfo::Index(idx) => write!(f, " LenIdx={}", idx)?,
        }
        if let Some(ref count) = self.count {
            match count {
                PropertyCountInfo::Fixed(cnt) => write!(f, " Count={}", cnt)?,
                PropertyCountInfo::Index(idx) => write!(f, " CountIdx={}", idx)?,
            }
        }
        if self.flags != 0 {
            write!(f, " Flags=0x{:08x}", self.flags)?;
        }
        Ok(())
    }
}

/// Get in_type name as static str (no allocation)
pub fn in_type_name(in_type: u16) -> &'static str {
    match in_type {
        0 => "InTypeNull",
        1 => "InTypeUnicodeString",
        2 => "InTypeAnsiString",
        3 => "InTypeInt8",
        4 => "InTypeUInt8",
        5 => "InTypeInt16",
        6 => "InTypeUInt16",
        7 => "InTypeInt32",
        8 => "InTypeUInt32",
        9 => "InTypeInt64",
        10 => "InTypeUInt64",
        11 => "InTypeFloat",
        12 => "InTypeDouble",
        13 => "InTypeBoolean",
        14 => "InTypeBinary",
        15 => "InTypeGuid",
        16 => "InTypePointer",
        17 => "InTypeFileTime",
        18 => "InTypeSystemTime",
        19 => "InTypeSid",
        20 => "InTypeHexInt32",
        21 => "InTypeHexInt64",
        300 => "InTypeCountedString",
        301 => "InTypeCountedAnsiString",
        _ => "Unknown",
    }
}

/// Get out_type name as static str (no allocation)
pub fn out_type_name(out_type: u16) -> &'static str {
    match out_type {
        0 => "OutTypeNull",
        1 => "OutTypeString",
        2 => "OutTypeDateTime",
        3 => "OutTypeInt8",
        4 => "OutTypeUInt8",
        5 => "OutTypeInt16",
        6 => "OutTypeUInt16",
        7 => "OutTypeInt32",
        8 => "OutTypeUInt32",
        9 => "OutTypeInt64",
        10 => "OutTypeUInt64",
        11 => "OutTypeFloat",
        12 => "OutTypeDouble",
        13 => "OutTypeBoolean",
        14 => "OutTypeGuid",
        15 => "OutTypeHexBinary",
        16 => "OutTypeHexInt8",
        17 => "OutTypeHexInt16",
        18 => "OutTypeHexInt32",
        19 => "OutTypeHexInt64",
        20 => "OutTypePid",
        21 => "OutTypeTid",
        22 => "OutTypePort",
        23 => "OutTypeIpv4",
        24 => "OutTypeIpv6",
        30 => "OutTypeWin32Error",
        31 => "OutTypeNtStatus",
        32 => "OutTypeHResult",
        34 => "OutTypeJson",
        35 => "OutTypeUtf8",
        36 => "OutTypePkcs7",
        37 => "OutTypeCodePointer",
        38 => "OutTypeDatetimeUtc",
        _ => "Unknown",
    }
}

/// Windows error code to name lookup
pub fn error_code_name(code: u32) -> &'static str {
    match code {
        0 => "ERROR_SUCCESS",
        2 => "ERROR_FILE_NOT_FOUND",
        6 => "ERROR_INVALID_HANDLE",
        8 => "ERROR_NOT_ENOUGH_MEMORY",
        13 => "ERROR_INVALID_DATA",
        87 => "ERROR_INVALID_PARAMETER",
        111 => "ERROR_ALREADY_EXISTS",
        1223 => "ERROR_CANCELLED",
        1168 => "ERROR_NOT_FOUND",
        4317 => "ERROR_NOT_FOUND",
        _ => "UNKNOWN",
    }
}

/// Runtime event observation (lightweight, no schema extraction).
/// Stores raw user data bytes; hex is formatted on demand during serialization.
#[derive(Debug, Clone)]
pub struct EventObservation {
    pub type_key: String,
    pub process_id: u32,
    pub thread_id: u32,
    pub timestamp: i64,
    pub user_data_bytes: Vec<u8>,
}

/// Serializable observation for JSON output (hex formatted)
#[derive(Serialize)]
pub struct SerializedObservation<'a> {
    pub type_key: &'a str,
    pub process_id: u32,
    pub thread_id: u32,
    pub timestamp: i64,
    pub user_data_hex: String,
}

impl<'a> From<&'a EventObservation> for SerializedObservation<'a> {
    fn from(obs: &'a EventObservation) -> Self {
        let mut hex = String::with_capacity(obs.user_data_bytes.len() * 2);
        for &b in &obs.user_data_bytes {
            use std::fmt::Write;
            let _ = write!(hex, "{:02x}", b);
        }
        SerializedObservation {
            type_key: &obs.type_key,
            process_id: obs.process_id,
            thread_id: obs.thread_id,
            timestamp: obs.timestamp,
            user_data_hex: hex,
        }
    }
}

/// Message sent from the ETW callback to the disk writer thread.
pub enum WriteCommand {
    /// A new event type was first observed - write its schema file
    NewType(EventTypeInfo),
    /// An observation of an event (first-seen or subsequent)
    Observation(EventObservation),
    /// Flush and close all files
    Shutdown,
}
