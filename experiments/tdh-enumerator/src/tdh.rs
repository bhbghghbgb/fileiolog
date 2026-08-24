use std::alloc::Layout;

use ferrisetw::EventRecord;
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::System::Diagnostics::Etw::{self, TRACE_EVENT_INFO};

use crate::types::{
    EventObservation, EventTypeId, EventTypeInfo, PropertyCountInfo, PropertyInfo,
    PropertyLengthInfo, format_win32_error,
};

/// Transmute a ferrisetw EventRecord reference to a raw EVENT_RECORD pointer.
///
/// # Safety
///
/// This relies on EventRecord being #[repr(transparent)] over EVENT_RECORD.
#[inline(always)]
unsafe fn as_raw_event_record(
    record: &EventRecord,
) -> &windows::Win32::System::Diagnostics::Etw::EVENT_RECORD {
    unsafe {
        &*(record as *const EventRecord
            as *const windows::Win32::System::Diagnostics::Etw::EVENT_RECORD)
    }
}

/// Get the user data buffer from an EVENT_RECORD
unsafe fn get_user_data(record: &EventRecord) -> &[u8] {
    unsafe {
        let raw = as_raw_event_record(record);
        let ptr = raw.UserData as *const u8;
        let len = raw.UserDataLength as usize;
        if ptr.is_null() || len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

/// Build an EventTypeId from an EventRecord (lightweight, no TDH calls).
pub fn build_type_id(record: &EventRecord) -> EventTypeId {
    EventTypeId {
        provider_guid: record.provider_id(),
        event_id: record.event_id(),
        version: record.version(),
        opcode: record.opcode(),
    }
}

/// Build a lightweight EventObservation from an EventRecord (no TDH calls).
/// Stores raw user data bytes; hex formatting deferred to serialization.
pub fn build_observation(record: &EventRecord, type_key: &str) -> EventObservation {
    let user_data = unsafe { get_user_data(record) };

    // Cap at 256 bytes
    let cap = user_data.len().min(256);
    EventObservation {
        type_key: type_key.to_string(),
        process_id: record.process_id(),
        thread_id: record.thread_id(),
        timestamp: record.raw_timestamp(),
        user_data_bytes: user_data[..cap].to_vec(),
    }
}

// ── TDH schema extraction (expensive, called once per event type) ────────

/// Raw wrapper over TRACE_EVENT_INFO buffer
struct TraceEventInfoBuffer {
    data: *const u8,
    layout: Layout,
}

unsafe impl Send for TraceEventInfoBuffer {}
unsafe impl Sync for TraceEventInfoBuffer {}

impl TraceEventInfoBuffer {
    fn as_raw(&self) -> &TRACE_EVENT_INFO {
        unsafe { &*(self.data as *const TRACE_EVENT_INFO) }
    }
}

impl Drop for TraceEventInfoBuffer {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                std::alloc::dealloc(self.data as *mut u8, self.layout);
            }
        }
    }
}

/// Call TdhGetEventInformation to get the TRACE_EVENT_INFO buffer
fn get_trace_event_info(record: &EventRecord) -> Result<TraceEventInfoBuffer, String> {
    let raw_ptr = unsafe { as_raw_event_record(record) };

    let mut buffer_size: u32 = 0;
    let status = unsafe { Etw::TdhGetEventInformation(raw_ptr, None, None, &mut buffer_size) };
    if status != ERROR_INSUFFICIENT_BUFFER.0 {
        return Err(format!(
            "TdhGetEventInformation (size query) failed: {}",
            format_win32_error(status),
        ));
    }

    if buffer_size == 0 {
        return Err("TdhGetEventInformation returned zero buffer size".into());
    }

    let layout = Layout::from_size_align(buffer_size as usize, 8)
        .map_err(|e| format!("Layout error: {}", e))?;

    let data = unsafe { std::alloc::alloc(layout) };
    if data.is_null() {
        return Err("Allocation failed".into());
    }

    let status = unsafe {
        Etw::TdhGetEventInformation(
            raw_ptr,
            None,
            Some(data as *mut TRACE_EVENT_INFO),
            &mut buffer_size,
        )
    };

    if status != 0 {
        unsafe {
            std::alloc::dealloc(data, layout);
        }
        return Err(format!(
            "TdhGetEventInformation failed: {}",
            format_win32_error(status),
        ));
    }

    Ok(TraceEventInfoBuffer { data, layout })
}

/// Extract a UTF-16 string from a TRACE_EVENT_INFO buffer at a given offset
fn extract_string(data_ptr: *const u8, offset: u32) -> String {
    if offset == 0 {
        return String::new();
    }
    let ptr = unsafe { data_ptr.offset(offset as isize) };
    if ptr.is_null() {
        return String::new();
    }
    let wide_ptr = ptr as *const u16;
    unsafe {
        let mut len = 0;
        while *wide_ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(wide_ptr, len);
        String::from_utf16_lossy(slice)
    }
}

/// Format GUID from raw bytes
fn format_guid(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        d1, d2, d3, d4[0], d4[1], d4[2], d4[3], d4[4], d4[5], d4[6], d4[7]
    )
}

/// Sanitize a GUID into a filesystem-safe string for use as a file key.
pub fn sanitize_key_static(guid: &windows::core::GUID) -> String {
    format!(
        "{:08x}_{:04x}_{:04x}_{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}

/// Extract complete event type information using TDH (expensive).
///
/// This calls TdhGetEventInformation and iterates over all properties.
/// Should only be called once per unique event type.
pub fn extract_event_type_info(record: &EventRecord) -> Result<EventTypeInfo, String> {
    let te_info_buf = get_trace_event_info(record)?;
    let te_info = te_info_buf.as_raw();
    let data_ptr = te_info_buf.data;

    // Extract schema metadata
    let provider_name = extract_string(data_ptr, te_info.ProviderNameOffset);
    let opcode_name = extract_string(data_ptr, te_info.OpcodeNameOffset);

    // Extract property definitions
    let property_count = te_info.PropertyCount as usize;
    let mut properties = Vec::with_capacity(property_count);

    for i in 0..property_count {
        let prop_ptr = unsafe { te_info.EventPropertyInfoArray.as_ptr().add(i) };
        let prop = unsafe { &*prop_ptr };

        let name = extract_string(data_ptr, prop.NameOffset);
        let flags = prop.Flags.0 as u32;

        // Extract type information from the union
        let (in_type, out_type) = if (flags & 0x1) == 0 {
            // PROPERTY_STRUCT is not set, access nonStructType
            let non_struct = unsafe { prop.Anonymous1.nonStructType };
            (non_struct.InType, non_struct.OutType)
        } else {
            (0, 0) // Struct type, skip type info
        };

        // Extract length
        let length = if (flags & 0x2) != 0 {
            // PROPERTY_PARAM_LENGTH is set
            let index = unsafe { prop.Anonymous3.lengthPropertyIndex };
            PropertyLengthInfo::Index(index)
        } else {
            let len = unsafe { prop.Anonymous3.length };
            PropertyLengthInfo::Fixed(len)
        };

        // Extract count
        let count = if (flags & 0x4) != 0 {
            // PROPERTY_PARAM_COUNT is set
            let index = unsafe { prop.Anonymous2.countPropertyIndex };
            Some(PropertyCountInfo::Index(index))
        } else {
            let cnt = unsafe { prop.Anonymous2.count };
            if cnt > 1 {
                Some(PropertyCountInfo::Fixed(cnt))
            } else {
                None
            }
        };

        properties.push(PropertyInfo {
            name,
            in_type,
            out_type,
            length,
            count,
            flags,
        });
    }

    // Format provider GUID
    let raw_guid = record.provider_id();
    let provider_guid = format_guid(
        raw_guid.data1,
        raw_guid.data2,
        raw_guid.data3,
        &raw_guid.data4,
    );

    Ok(EventTypeInfo {
        provider_guid,
        event_id: record.event_id(),
        opcode: record.opcode(),
        version: record.version(),
        provider_name,
        opcode_name,
        properties,
    })
}
