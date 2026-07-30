use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Represents a known FileIo event type
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileIoEventDef {
    pub event_id: u16,
    pub version: u8,
    pub class_name: &'static str,
    pub event_name: &'static str,
    pub description: &'static str,
}

/// Represents a received FileIo event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileIoEvent {
    pub event_id: u16,
    pub version: u8,
    pub timestamp: u64,
    pub process_id: u32,
    pub thread_id: u32,
}

/// Static registry of all known FileIo events
/// Key: (event_id, version)
pub static EVENT_REGISTRY: Lazy<HashMap<(u16, u8), FileIoEventDef>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // ── FileIo V0 (EventVersion 0) ──────────────────────────────
    map.insert((0, 0), FileIoEventDef {
        event_id: 0,
        version: 0,
        class_name: "FileIo_V0_Name",
        event_name: "Name",
        description: "File name event (V0)",
    });

    // ── FileIo V1 (EventVersion 1) ──────────────────────────────
    map.insert((0, 1), FileIoEventDef {
        event_id: 0,
        version: 1,
        class_name: "FileIo_V1_Name",
        event_name: "Name",
        description: "File name event (V1)",
    });
    map.insert((32, 1), FileIoEventDef {
        event_id: 32,
        version: 1,
        class_name: "FileIo_V1_Name",
        event_name: "FileCreate",
        description: "File create event (V1)",
    });

    // ── FileIo V2 (EventVersion 2) ──────────────────────────────
    map.insert((0, 2), FileIoEventDef {
        event_id: 0,
        version: 2,
        class_name: "FileIo_V2_Name",
        event_name: "Name",
        description: "File name event (V2)",
    });
    map.insert((32, 2), FileIoEventDef {
        event_id: 32,
        version: 2,
        class_name: "FileIo_V2_Name",
        event_name: "FileCreate",
        description: "File create event (V2)",
    });
    map.insert((35, 2), FileIoEventDef {
        event_id: 35,
        version: 2,
        class_name: "FileIo_V2_Name",
        event_name: "FileDelete",
        description: "File delete event (V2)",
    });
    map.insert((36, 2), FileIoEventDef {
        event_id: 36,
        version: 2,
        class_name: "FileIo_V2_Name",
        event_name: "FileRundown",
        description: "File rundown event (V2)",
    });
    map.insert((37, 2), FileIoEventDef {
        event_id: 37,
        version: 2,
        class_name: "FileIo_V2_MapFile",
        event_name: "MapFile",
        description: "Memory-mapped file event (V2)",
    });
    map.insert((38, 2), FileIoEventDef {
        event_id: 38,
        version: 2,
        class_name: "FileIo_V2_MapFile",
        event_name: "UnmapFile",
        description: "Memory-unmap file event (V2)",
    });
    map.insert((39, 2), FileIoEventDef {
        event_id: 39,
        version: 2,
        class_name: "FileIo_V2_MapFile",
        event_name: "MapFileDCStart",
        description: "Memory-mapped file DC start event (V2)",
    });
    map.insert((40, 2), FileIoEventDef {
        event_id: 40,
        version: 2,
        class_name: "FileIo_V2_MapFile",
        event_name: "MapFileDCEnd",
        description: "Memory-mapped file DC end event (V2)",
    });
    map.insert((64, 2), FileIoEventDef {
        event_id: 64,
        version: 2,
        class_name: "FileIo_V2_Create",
        event_name: "Create",
        description: "File create event (V2)",
    });
    map.insert((65, 2), FileIoEventDef {
        event_id: 65,
        version: 2,
        class_name: "FileIo_V2_SimpleOp",
        event_name: "Cleanup",
        description: "File cleanup event (V2)",
    });
    map.insert((66, 2), FileIoEventDef {
        event_id: 66,
        version: 2,
        class_name: "FileIo_V2_SimpleOp",
        event_name: "Close",
        description: "File close event (V2)",
    });
    map.insert((67, 2), FileIoEventDef {
        event_id: 67,
        version: 2,
        class_name: "FileIo_V2_ReadWrite",
        event_name: "Read",
        description: "File read event (V2)",
    });
    map.insert((68, 2), FileIoEventDef {
        event_id: 68,
        version: 2,
        class_name: "FileIo_V2_ReadWrite",
        event_name: "Write",
        description: "File write event (V2)",
    });
    map.insert((69, 2), FileIoEventDef {
        event_id: 69,
        version: 2,
        class_name: "FileIo_V2_Info",
        event_name: "SetInfo",
        description: "File set info event (V2)",
    });
    map.insert((70, 2), FileIoEventDef {
        event_id: 70,
        version: 2,
        class_name: "FileIo_V2_Info",
        event_name: "Delete",
        description: "File delete info event (V2)",
    });
    map.insert((71, 2), FileIoEventDef {
        event_id: 71,
        version: 2,
        class_name: "FileIo_V2_Info",
        event_name: "Rename",
        description: "File rename event (V2)",
    });
    map.insert((72, 2), FileIoEventDef {
        event_id: 72,
        version: 2,
        class_name: "FileIo_V2_DirEnum",
        event_name: "DirEnum",
        description: "Directory enumeration event (V2)",
    });
    map.insert((73, 2), FileIoEventDef {
        event_id: 73,
        version: 2,
        class_name: "FileIo_V2_SimpleOp",
        event_name: "Flush",
        description: "File flush event (V2)",
    });
    map.insert((74, 2), FileIoEventDef {
        event_id: 74,
        version: 2,
        class_name: "FileIo_V2_Info",
        event_name: "QueryInfo",
        description: "File query info event (V2)",
    });
    map.insert((75, 2), FileIoEventDef {
        event_id: 75,
        version: 2,
        class_name: "FileIo_V2_Info",
        event_name: "FSControl",
        description: "File system control event (V2)",
    });
    map.insert((76, 2), FileIoEventDef {
        event_id: 76,
        version: 2,
        class_name: "FileIo_V2_OpEnd",
        event_name: "OperationEnd",
        description: "File operation end event (V2)",
    });
    map.insert((77, 2), FileIoEventDef {
        event_id: 77,
        version: 2,
        class_name: "FileIo_V2_DirEnum",
        event_name: "DirNotify",
        description: "Directory notification event (V2)",
    });

    // ── FileIo V3 (EventVersion 3) ──────────────────────────────
    map.insert((0, 3), FileIoEventDef {
        event_id: 0,
        version: 3,
        class_name: "FileIo_Name",
        event_name: "Name",
        description: "File name event (V3)",
    });
    map.insert((32, 3), FileIoEventDef {
        event_id: 32,
        version: 3,
        class_name: "FileIo_Name",
        event_name: "FileCreate",
        description: "File create event (V3)",
    });
    map.insert((35, 3), FileIoEventDef {
        event_id: 35,
        version: 3,
        class_name: "FileIo_Name",
        event_name: "FileDelete",
        description: "File delete event (V3)",
    });
    map.insert((36, 3), FileIoEventDef {
        event_id: 36,
        version: 3,
        class_name: "FileIo_Name",
        event_name: "FileRundown",
        description: "File rundown event (V3)",
    });
    map.insert((64, 3), FileIoEventDef {
        event_id: 64,
        version: 3,
        class_name: "FileIo_Create",
        event_name: "Create",
        description: "File create event (V3)",
    });
    map.insert((65, 3), FileIoEventDef {
        event_id: 65,
        version: 3,
        class_name: "FileIo_SimpleOp",
        event_name: "Cleanup",
        description: "File cleanup event (V3)",
    });
    map.insert((66, 3), FileIoEventDef {
        event_id: 66,
        version: 3,
        class_name: "FileIo_SimpleOp",
        event_name: "Close",
        description: "File close event (V3)",
    });
    map.insert((67, 3), FileIoEventDef {
        event_id: 67,
        version: 3,
        class_name: "FileIo_ReadWrite",
        event_name: "Read",
        description: "File read event (V3)",
    });
    map.insert((68, 3), FileIoEventDef {
        event_id: 68,
        version: 3,
        class_name: "FileIo_ReadWrite",
        event_name: "Write",
        description: "File write event (V3)",
    });
    map.insert((69, 3), FileIoEventDef {
        event_id: 69,
        version: 3,
        class_name: "FileIo_Info",
        event_name: "SetInfo",
        description: "File set info event (V3)",
    });
    map.insert((70, 3), FileIoEventDef {
        event_id: 70,
        version: 3,
        class_name: "FileIo_Info",
        event_name: "Delete",
        description: "File delete info event (V3)",
    });
    map.insert((71, 3), FileIoEventDef {
        event_id: 71,
        version: 3,
        class_name: "FileIo_Info",
        event_name: "Rename",
        description: "File rename event (V3)",
    });
    map.insert((72, 3), FileIoEventDef {
        event_id: 72,
        version: 3,
        class_name: "FileIo_DirEnum",
        event_name: "DirEnum",
        description: "Directory enumeration event (V3)",
    });
    map.insert((73, 3), FileIoEventDef {
        event_id: 73,
        version: 3,
        class_name: "FileIo_SimpleOp",
        event_name: "Flush",
        description: "File flush event (V3)",
    });
    map.insert((74, 3), FileIoEventDef {
        event_id: 74,
        version: 3,
        class_name: "FileIo_Info",
        event_name: "QueryInfo",
        description: "File query info event (V3)",
    });
    map.insert((75, 3), FileIoEventDef {
        event_id: 75,
        version: 3,
        class_name: "FileIo_Info",
        event_name: "FSControl",
        description: "File system control event (V3)",
    });
    map.insert((76, 3), FileIoEventDef {
        event_id: 76,
        version: 3,
        class_name: "FileIo_OpEnd",
        event_name: "OperationEnd",
        description: "File operation end event (V3)",
    });
    map.insert((77, 3), FileIoEventDef {
        event_id: 77,
        version: 3,
        class_name: "FileIo_DirEnum",
        event_name: "DirNotify",
        description: "Directory notification event (V3)",
    });
    map.insert((79, 3), FileIoEventDef {
        event_id: 79,
        version: 3,
        class_name: "FileIo_PathOperation",
        event_name: "DeletePath",
        description: "File path delete event (V3)",
    });
    map.insert((80, 3), FileIoEventDef {
        event_id: 80,
        version: 3,
        class_name: "FileIo_PathOperation",
        event_name: "RenamePath",
        description: "File path rename event (V3)",
    });
    map.insert((81, 3), FileIoEventDef {
        event_id: 81,
        version: 3,
        class_name: "FileIo_PathOperation",
        event_name: "SetLinkPath",
        description: "File path set link event (V3)",
    });
    map.insert((96, 3), FileIoEventDef {
        event_id: 96,
        version: 3,
        class_name: "FltIoInit",
        event_name: "PreOpInit",
        description: "Minifilter pre-operation init event (V3)",
    });
    map.insert((97, 3), FileIoEventDef {
        event_id: 97,
        version: 3,
        class_name: "FltIoInit",
        event_name: "PostOpInit",
        description: "Minifilter post-operation init event (V3)",
    });
    map.insert((98, 3), FileIoEventDef {
        event_id: 98,
        version: 3,
        class_name: "FltIoCompletion",
        event_name: "PreOpCompletion",
        description: "Minifilter pre-operation completion event (V3)",
    });
    map.insert((99, 3), FileIoEventDef {
        event_id: 99,
        version: 3,
        class_name: "FltIoCompletion",
        event_name: "PostOpCompletion",
        description: "Minifilter post-operation completion event (V3)",
    });
    map.insert((100, 3), FileIoEventDef {
        event_id: 100,
        version: 3,
        class_name: "FltIoFailure",
        event_name: "PreOpFailure",
        description: "Minifilter pre-operation failure event (V3)",
    });
    map.insert((101, 3), FileIoEventDef {
        event_id: 101,
        version: 3,
        class_name: "FltIoFailure",
        event_name: "PostOpFailure",
        description: "Minifilter post-operation failure event (V3)",
    });

    map
});

/// Track unknown event IDs/versions we've already warned about
static WARNED_UNKNOWN: once_cell::sync::Lazy<Mutex<Vec<(u16, u8)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Vec::new()));

/// Log an event, handling unknown events with warn-once semantics
pub fn log_event(event_id: u16, version: u8, _timestamp: u64, process_id: u32, thread_id: u32) {
    let key = (event_id, version);

    if let Some(known) = EVENT_REGISTRY.get(&key) {
        log::trace!(
            "FileIo Event: {} (ID={}, Version={}, Class={}) PID={} TID={}",
            known.event_name, event_id, version, known.class_name, process_id, thread_id
        );
    } else {
        let mut warned = WARNED_UNKNOWN.lock().unwrap();
        if !warned.contains(&key) {
            warned.push(key);
            log::warn!(
                "Unknown FileIo event: ID={}, Version={} (first occurrence, PID={}, TID={})",
                event_id, version, process_id, thread_id
            );
        } else {
            log::debug!(
                "Unknown FileIo event: ID={}, Version={} (PID={}, TID={})",
                event_id, version, process_id, thread_id
            );
        }
    }
}
