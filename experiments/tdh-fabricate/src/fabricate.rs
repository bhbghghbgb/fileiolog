//! Core module for fabricating EVENT_RECORD structures in memory.
//!
//! Since ferrisetw's EventRecord is `#[repr(transparent)]` over Windows' EVENT_RECORD,
//! we can create raw EVENT_RECORDs in memory and transmute them to &EventRecord for
//! calling TdhGetEventInformation.

use std::alloc::Layout;

use ferrisetw::EventRecord;
use windows::Win32::System::Diagnostics::Etw::{
    EVENT_DESCRIPTOR, EVENT_HEADER, EVENT_RECORD,
};
use windows::core::GUID;

/// A fabricated EVENT_RECORD that owns its memory.
///
/// This struct ensures proper alignment and lifetime management
/// for manually constructed EVENT_RECORD data.
pub struct FabricatedRecord {
    /// The raw EVENT_RECORD laid out in a properly aligned buffer.
    /// We use a boxed layout to ensure the buffer lives as long as needed.
    buffer: Vec<u8>,
    layout: Layout,
}

impl FabricatedRecord {
    /// Create a new fabricated record with all fields zeroed except the ones we set.
    pub fn new() -> Self {
        let layout = Layout::new::<EVENT_RECORD>();
        let mut buffer = vec![0u8; layout.size()];

        // Set the EVENT_RECORD header size
        unsafe {
            let record = buffer.as_mut_ptr() as *mut EVENT_RECORD;
            (*record).EventHeader.Size = std::mem::size_of::<EVENT_HEADER>() as u16;
        }

        Self { buffer, layout }
    }

    /// Get a reference to the raw EVENT_RECORD
    pub fn raw_record(&self) -> &EVENT_RECORD {
        unsafe { &*(self.buffer.as_ptr() as *const EVENT_RECORD) }
    }

    /// Get a mutable reference to the raw EVENT_RECORD
    pub fn raw_record_mut(&mut self) -> &mut EVENT_RECORD {
        unsafe { &mut *(self.buffer.as_mut_ptr() as *mut EVENT_RECORD) }
    }

    /// Transmute to a ferrisetw EventRecord reference.
    ///
    /// # Safety
    ///
    /// The underlying EVENT_RECORD must have valid size, alignment, and
    /// the buffer must outlive the returned reference.
    pub unsafe fn as_event_record(&self) -> &EventRecord {
        unsafe { &*(self.raw_record() as *const EVENT_RECORD as *const EventRecord) }
    }

    // ── Builder methods for setting EVENT_HEADER fields ──────

    /// Set the ProviderId GUID
    pub fn set_provider_id(&mut self, guid: GUID) {
        self.raw_record_mut().EventHeader.ProviderId = guid;
    }

    /// Set the full EventDescriptor
    pub fn set_event_descriptor(&mut self, descriptor: EVENT_DESCRIPTOR) {
        self.raw_record_mut().EventHeader.EventDescriptor = descriptor;
    }

    /// Build and set an EVENT_DESCRIPTOR from individual fields
    pub fn set_descriptor(
        &mut self,
        id: u16,
        version: u8,
        channel: u8,
        level: u8,
        opcode: u8,
        task: u16,
        keyword: u64,
    ) {
        unsafe {
            let desc = &mut self.raw_record_mut().EventHeader.EventDescriptor;
            // EVENT_DESCRIPTOR is packed; we need to set fields carefully.
            // Layout: Id(u16), Version(u8), Channel(u8), Level(u8), Opcode(u8), Task(u16), Keyword(u64)
            // But due to packing, we write byte-by-byte or use std::ptr::write_unaligned

            let desc_ptr = desc as *mut EVENT_DESCRIPTOR as *mut u8;

            // Id (u16, little-endian)
            desc_ptr.add(0).write(id as u8);
            desc_ptr.add(1).write((id >> 8) as u8);
            // Version (u8)
            desc_ptr.add(2).write(version);
            // Channel (u8)
            desc_ptr.add(3).write(channel);
            // Level (u8)
            desc_ptr.add(4).write(level);
            // Opcode (u8)
            desc_ptr.add(5).write(opcode);
            // Task (u16, little-endian)
            desc_ptr.add(6).write(task as u8);
            desc_ptr.add(7).write((task >> 8) as u8);
            // Keyword (u64, little-endian)
            desc_ptr.add(8).write(keyword as u8);
            desc_ptr.add(9).write((keyword >> 8) as u8);
            desc_ptr.add(10).write((keyword >> 16) as u8);
            desc_ptr.add(11).write((keyword >> 24) as u8);
            desc_ptr.add(12).write((keyword >> 32) as u8);
            desc_ptr.add(13).write((keyword >> 40) as u8);
            desc_ptr.add(14).write((keyword >> 48) as u8);
            desc_ptr.add(15).write((keyword >> 56) as u8);
        }
    }

    /// Set the EVENT_HEADER Flags field
    pub fn set_flags(&mut self, flags: u16) {
        self.raw_record_mut().EventHeader.Flags = flags;
    }

    /// Set the EVENT_HEADER EventProperty field
    pub fn set_event_property(&mut self, property: u16) {
        self.raw_record_mut().EventHeader.EventProperty = property;
    }

    /// Set the BufferContext (logger ID, processor, etc.)
    pub fn set_buffer_context(&mut self, logger_id: u16, processor_number: u8) {
        let ctx = &mut self.raw_record_mut().BufferContext;
        ctx.LoggerId = logger_id;
        ctx.Anonymous.Anonymous.ProcessorNumber = processor_number;
    }

    /// Set the UserData pointer and length.
    ///
    /// The data must outlive this FabricatedRecord.
    pub fn set_user_data(&mut self, data: &[u8]) {
        // We need the data to be in the same allocation or a separate allocation
        // that outlives this record. For simplicity, we'll store data inline
        // by extending our buffer.
        //
        // Actually, for fabricated records that we use immediately and discard,
        // we can point UserData at a static or leaked buffer.
        // But the cleanest approach: allocate data, leak it, and point to it.
        //
        // WARNING: This leaks memory. Only use for short-lived experiments.
        let ptr = data.as_ptr() as *mut std::ffi::c_void;
        self.raw_record_mut().UserData = ptr;
        self.raw_record_mut().UserDataLength = data.len() as u16;
    }

    /// Set ExtendedData (usually null/zero for fabricated records)
    pub fn set_extended_data(&mut self, count: u16, ptr: *mut std::ffi::c_void) {
        self.raw_record_mut().ExtendedDataCount = count;
        self.raw_record_mut().ExtendedData = ptr as *mut _;
    }

    /// Set ProcessId and ThreadId (useful for some TDH code paths)
    pub fn set_process_thread(&mut self, pid: u32, tid: u32) {
        self.raw_record_mut().EventHeader.ProcessId = pid;
        self.raw_record_mut().EventHeader.ThreadId = tid;
    }

    /// Set TimeStamp
    pub fn set_timestamp(&mut self, ts: i64) {
        // Write the i64 directly to the TimeStamp field via raw pointer
        unsafe {
            let raw = self.raw_record_mut();
            let ts_ptr = &mut raw.EventHeader.TimeStamp as *mut _ as *mut i64;
            *ts_ptr = ts;
        }
    }

    /// Set ActivityId
    pub fn set_activity_id(&mut self, guid: GUID) {
        self.raw_record_mut().EventHeader.ActivityId = guid;
    }

    /// Set UserContext
    pub fn set_user_context(&mut self, ctx: *mut std::ffi::c_void) {
        self.raw_record_mut().UserContext = ctx;
    }

    /// Create from an existing EVENT_RECORD by copying it
    pub fn from_raw(raw: &EVENT_RECORD) -> Self {
        let layout = Layout::new::<EVENT_RECORD>();
        let mut buffer = vec![0u8; layout.size()];

        unsafe {
            std::ptr::copy_nonoverlapping(
                raw as *const EVENT_RECORD,
                buffer.as_mut_ptr() as *mut EVENT_RECORD,
                1,
            );
        }

        Self { buffer, layout }
    }

    /// Create from a ferrisetw EventRecord by copying the underlying EVENT_RECORD
    pub fn from_event_record(record: &EventRecord) -> Self {
        let raw = unsafe { &*(record as *const EventRecord as *const EVENT_RECORD) };
        Self::from_raw(raw)
    }

    /// Create from raw bytes (previously captured EVENT_RECORD data)
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let layout = Layout::new::<EVENT_RECORD>();
        let mut buffer = vec![0u8; layout.size()];

        // Copy as much as fits
        let copy_len = bytes.len().min(buffer.len());
        buffer[..copy_len].copy_from_slice(&bytes[..copy_len]);

        Self { buffer, layout }
    }
}

/// Helper to create a standard EVENT_DESCRIPTOR
pub fn make_descriptor(
    id: u16,
    version: u8,
    channel: u8,
    level: u8,
    opcode: u8,
    task: u16,
    keyword: u64,
) -> EVENT_DESCRIPTOR {
    // EVENT_DESCRIPTOR is #[repr(C)] packed(1), so we write byte-by-byte
    let mut desc = std::mem::MaybeUninit::<EVENT_DESCRIPTOR>::uninit();
    let desc_ptr = desc.as_mut_ptr() as *mut u8;

    unsafe {
        // Id (u16, little-endian)
        desc_ptr.add(0).write(id as u8);
        desc_ptr.add(1).write((id >> 8) as u8);
        // Version (u8)
        desc_ptr.add(2).write(version);
        // Channel (u8)
        desc_ptr.add(3).write(channel);
        // Level (u8)
        desc_ptr.add(4).write(level);
        // Opcode (u8)
        desc_ptr.add(5).write(opcode);
        // Task (u16, little-endian)
        desc_ptr.add(6).write(task as u8);
        desc_ptr.add(7).write((task >> 8) as u8);
        // Keyword (u64, little-endian)
        desc_ptr.add(8).write(keyword as u8);
        desc_ptr.add(9).write((keyword >> 8) as u8);
        desc_ptr.add(10).write((keyword >> 16) as u8);
        desc_ptr.add(11).write((keyword >> 24) as u8);
        desc_ptr.add(12).write((keyword >> 32) as u8);
        desc_ptr.add(13).write((keyword >> 40) as u8);
        desc_ptr.add(14).write((keyword >> 48) as u8);
        desc_ptr.add(15).write((keyword >> 56) as u8);
    }

    unsafe { desc.assume_init() }
}

/// Well-known FileIo kernel provider GUID
pub const FILEIO_PROVIDER_GUID: GUID = GUID {
    data1: 0x90cbdc39,
    data2: 0x4a3e,
    data3: 0x11d1,
    data4: [0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3],
};

/// Well-known SystemTraceProvider GUID (legacy kernel tracing)
pub const SYSTEM_TRACE_PROVIDER_GUID: GUID = GUID {
    data1: 0x9e9bba3c,
    data2: 0x2e38,
    data3: 0x11d3,
    data4: [0x9a, 0x10, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0xfd],
};

/// Well-known DiskIo provider GUID
pub const DISKIO_PROVIDER_GUID: GUID = GUID {
    data1: 0x01853a65,
    data2: 0x418f,
    data3: 0x3f73,
    data4: [0x98, 0x13, 0x64, 0xd6, 0x3f, 0xe1, 0xcd, 0xb8],
};

/// Well-known Process provider GUID
pub const PROCESS_PROVIDER_GUID: GUID = GUID {
    data1: 0x22fb2cd6,
    data2: 0x0e7b,
    data3: 0x422b,
    data4: [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16],
};

/// Well-known Thread provider GUID
pub const THREAD_PROVIDER_GUID: GUID = GUID {
    data1: 0x3d6fa8d1,
    data2: 0xfe05,
    data3: 0x11d0,
    data4: [0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd9, 0x30, 0xc5],
};

/// Microsoft-Windows-Kernel-Process manifest provider GUID (user-mode, registered in WINEVT)
pub const KERNEL_PROCESS_MANIFEST_GUID: GUID = GUID {
    data1: 0x22fb2cd6,
    data2: 0x0e7b,
    data3: 0x422b,
    data4: [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16],
};

/// Microsoft-Windows-Kernel-FileIo manifest provider GUID (user-mode version)
pub const KERNEL_FILEIO_MANIFEST_GUID: GUID = GUID {
    data1: 0xedd08927,
    data2: 0x9cc4,
    data3: 0x4e65,
    data4: [0xb9, 0x70, 0xc2, 0x56, 0x0f, 0xb5, 0xc2, 0x89],
};

impl Default for FabricatedRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FabricatedRecord {
    fn drop(&mut self) {
        // No special cleanup needed; Vec handles deallocation
    }
}
