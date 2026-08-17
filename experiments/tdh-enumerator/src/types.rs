use serde::{Deserialize, Serialize};

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

/// Information about a single property in an event schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub in_type: u16,
    pub in_type_name: String,
    pub out_type: u16,
    pub out_type_name: String,
    pub length: PropertyLengthInfo,
    pub count: Option<PropertyCountInfo>,
    pub flags: u32,
    pub flags_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyLengthInfo {
    Fixed(u16),
    Index(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyCountInfo {
    Fixed(u16),
    Index(u16),
}

/// Schema-level information about an event type.
/// This is extracted once per unique EventTypeId and cached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeInfo {
    // Identity
    pub provider_guid: String,
    pub event_id: u16,
    pub opcode: u8,
    pub version: u8,

    // Schema metadata (from TDH)
    pub provider_name: String,
    pub task_name: String,
    pub opcode_name: String,
    pub decoding_source: String,

    // Property definitions
    pub properties: Vec<PropertyInfo>,
}

/// Runtime event observation (lightweight, no schema extraction).
/// Collected for every event but does not trigger TDH calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventObservation {
    pub type_key: String,
    pub process_id: u32,
    pub thread_id: u32,
    pub timestamp: i64,
    pub user_data_length: usize,
    pub user_data_hex: String,
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
