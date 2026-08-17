use serde::{Deserialize, Serialize};

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

/// Complete information about a received event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    // Event header fields
    pub provider_guid: String,
    pub event_id: u16,
    pub opcode: u8,
    pub version: u8,
    pub level: u8,
    pub keyword: u64,
    pub process_id: u32,
    pub thread_id: u32,
    pub timestamp: i64,
    pub activity_id: String,

    // Schema metadata (from TDH)
    pub provider_name: String,
    pub task_name: String,
    pub opcode_name: String,
    pub decoding_source: String,

    // Property definitions
    pub properties: Vec<PropertyInfo>,

    // Parsed property values (best-effort)
    pub property_values: Vec<PropertyValue>,

    // Raw user data
    pub user_data_hex: String,
    pub user_data_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValue {
    pub name: String,
    pub in_type_name: String,
    pub raw_hex: String,
    pub display_value: String,
    pub parse_error: Option<String>,
}
