#![allow(dead_code)]

use std::fmt;

/// A unique identifier for a FileIo event type, combining id and version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventKey {
    pub id: u16,
    pub version: u8,
}

impl EventKey {
    pub const fn new(id: u16, version: u8) -> Self {
        Self { id, version }
    }
}

impl fmt::Display for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id={} ver={}", self.id, self.version)
    }
}

/// Metadata for a known FileIo event type.
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub key: EventKey,
    pub name: &'static str,
    pub mof_class: &'static str,
}

/// All known FileIo event definitions, grouped by version.
pub struct EventDefs;

impl EventDefs {
    /// Returns all known event definitions across all versions.
    pub fn all() -> Vec<EventInfo> {
        let mut v = Vec::new();
        v.extend(Self::v0());
        v.extend(Self::v1());
        v.extend(Self::v2());
        v.extend(Self::v3());
        v
    }

    /// FileIo V0 (EventVersion 0) - Windows 2000
    pub fn v0() -> Vec<EventInfo> {
        vec![EventInfo {
            key: EventKey::new(0, 0),
            name: "Name",
            mof_class: "FileIo_V0_Name",
        }]
    }

    /// FileIo V1 (EventVersion 1) - Windows XP
    pub fn v1() -> Vec<EventInfo> {
        vec![
            EventInfo {
                key: EventKey::new(0, 1),
                name: "Name",
                mof_class: "FileIo_V1_Name",
            },
            EventInfo {
                key: EventKey::new(32, 1),
                name: "FileCreate",
                mof_class: "FileIo_V1_Name",
            },
        ]
    }

    /// FileIo V2 (EventVersion 2) - Windows Vista+
    pub fn v2() -> Vec<EventInfo> {
        vec![
            // FileIo_Name
            EventInfo { key: EventKey::new(0, 2), name: "Name", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(32, 2), name: "FileCreate", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(35, 2), name: "FileDelete", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(36, 2), name: "FileRundown", mof_class: "FileIo_Name" },
            // FileIo_V2_MapFile (undocumented in official V2 docs)
            EventInfo { key: EventKey::new(37, 2), name: "MapFile", mof_class: "FileIo_V2_MapFile" },
            EventInfo { key: EventKey::new(38, 2), name: "UnmapFile", mof_class: "FileIo_V2_MapFile" },
            EventInfo { key: EventKey::new(39, 2), name: "MapFileDCStart", mof_class: "FileIo_V2_MapFile" },
            EventInfo { key: EventKey::new(40, 2), name: "MapFileDCEnd", mof_class: "FileIo_V2_MapFile" },
            // FileIo_Create
            EventInfo { key: EventKey::new(64, 2), name: "Create", mof_class: "FileIo_Create" },
            // FileIo_SimpleOp
            EventInfo { key: EventKey::new(65, 2), name: "Cleanup", mof_class: "FileIo_SimpleOp" },
            EventInfo { key: EventKey::new(66, 2), name: "Close", mof_class: "FileIo_SimpleOp" },
            EventInfo { key: EventKey::new(73, 2), name: "Flush", mof_class: "FileIo_SimpleOp" },
            // FileIo_ReadWrite
            EventInfo { key: EventKey::new(67, 2), name: "Read", mof_class: "FileIo_ReadWrite" },
            EventInfo { key: EventKey::new(68, 2), name: "Write", mof_class: "FileIo_ReadWrite" },
            // FileIo_Info
            EventInfo { key: EventKey::new(69, 2), name: "SetInfo", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(70, 2), name: "Delete", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(71, 2), name: "Rename", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(74, 2), name: "QueryInfo", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(75, 2), name: "FSControl", mof_class: "FileIo_Info" },
            // FileIo_DirEnum
            EventInfo { key: EventKey::new(72, 2), name: "DirEnum", mof_class: "FileIo_DirEnum" },
            EventInfo { key: EventKey::new(77, 2), name: "DirNotify", mof_class: "FileIo_DirEnum" },
            // FileIo_OpEnd
            EventInfo { key: EventKey::new(76, 2), name: "OperationEnd", mof_class: "FileIo_OpEnd" },
            // FileIo_PathOperation (V3 MOF only, but may appear in V2 on newer Windows)
            EventInfo { key: EventKey::new(79, 2), name: "DeletePath", mof_class: "FileIo_PathOperation" },
            EventInfo { key: EventKey::new(80, 2), name: "RenamePath", mof_class: "FileIo_PathOperation" },
            EventInfo { key: EventKey::new(81, 2), name: "SetLinkPath", mof_class: "FileIo_PathOperation" },
        ]
    }

    /// FileIo V3 (EventVersion 3) - Current/latest, adds minifilter events
    pub fn v3() -> Vec<EventInfo> {
        vec![
            // FileIo_Name
            EventInfo { key: EventKey::new(0, 3), name: "Name", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(32, 3), name: "FileCreate", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(35, 3), name: "FileDelete", mof_class: "FileIo_Name" },
            EventInfo { key: EventKey::new(36, 3), name: "FileRundown", mof_class: "FileIo_Name" },
            // FileIo_Create
            EventInfo { key: EventKey::new(64, 3), name: "Create", mof_class: "FileIo_Create" },
            // FileIo_SimpleOp
            EventInfo { key: EventKey::new(65, 3), name: "Cleanup", mof_class: "FileIo_SimpleOp" },
            EventInfo { key: EventKey::new(66, 3), name: "Close", mof_class: "FileIo_SimpleOp" },
            EventInfo { key: EventKey::new(73, 3), name: "Flush", mof_class: "FileIo_SimpleOp" },
            // FileIo_ReadWrite
            EventInfo { key: EventKey::new(67, 3), name: "Read", mof_class: "FileIo_ReadWrite" },
            EventInfo { key: EventKey::new(68, 3), name: "Write", mof_class: "FileIo_ReadWrite" },
            // FileIo_Info
            EventInfo { key: EventKey::new(69, 3), name: "SetInfo", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(70, 3), name: "Delete", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(71, 3), name: "Rename", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(74, 3), name: "QueryInfo", mof_class: "FileIo_Info" },
            EventInfo { key: EventKey::new(75, 3), name: "FSControl", mof_class: "FileIo_Info" },
            // FileIo_DirEnum
            EventInfo { key: EventKey::new(72, 3), name: "DirEnum", mof_class: "FileIo_DirEnum" },
            EventInfo { key: EventKey::new(77, 3), name: "DirNotify", mof_class: "FileIo_DirEnum" },
            // FileIo_OpEnd
            EventInfo { key: EventKey::new(76, 3), name: "OperationEnd", mof_class: "FileIo_OpEnd" },
            // FileIo_PathOperation
            EventInfo { key: EventKey::new(79, 3), name: "DeletePath", mof_class: "FileIo_PathOperation" },
            EventInfo { key: EventKey::new(80, 3), name: "RenamePath", mof_class: "FileIo_PathOperation" },
            EventInfo { key: EventKey::new(81, 3), name: "SetLinkPath", mof_class: "FileIo_PathOperation" },
            // FltIoInit (V3 only - minifilter)
            EventInfo { key: EventKey::new(96, 3), name: "FltPreOpInit", mof_class: "FltIoInit" },
            EventInfo { key: EventKey::new(97, 3), name: "FltPostOpInit", mof_class: "FltIoInit" },
            // FltIoCompletion (V3 only - minifilter)
            EventInfo { key: EventKey::new(98, 3), name: "FltPreOpCompletion", mof_class: "FltIoCompletion" },
            EventInfo { key: EventKey::new(99, 3), name: "FltPostOpCompletion", mof_class: "FltIoCompletion" },
            // FltIoFailure (V3 only - minifilter)
            EventInfo { key: EventKey::new(100, 3), name: "FltPreOpFailure", mof_class: "FltIoFailure" },
            EventInfo { key: EventKey::new(101, 3), name: "FltPostOpFailure", mof_class: "FltIoFailure" },
        ]
    }
}

/// Static list of all known events, built at compile time via const initialization.
static ALL_EVENTS_DATA: &[(&str, u16, u8, &str)] = &[
    // V0
    ("Name", 0, 0, "FileIo_V0_Name"),
    // V1
    ("Name", 0, 1, "FileIo_V1_Name"),
    ("FileCreate", 32, 1, "FileIo_V1_Name"),
    // V2
    ("Name", 0, 2, "FileIo_Name"),
    ("FileCreate", 32, 2, "FileIo_Name"),
    ("FileDelete", 35, 2, "FileIo_Name"),
    ("FileRundown", 36, 2, "FileIo_Name"),
    ("MapFile", 37, 2, "FileIo_V2_MapFile"),
    ("UnmapFile", 38, 2, "FileIo_V2_MapFile"),
    ("MapFileDCStart", 39, 2, "FileIo_V2_MapFile"),
    ("MapFileDCEnd", 40, 2, "FileIo_V2_MapFile"),
    ("Create", 64, 2, "FileIo_Create"),
    ("Cleanup", 65, 2, "FileIo_SimpleOp"),
    ("Close", 66, 2, "FileIo_SimpleOp"),
    ("Read", 67, 2, "FileIo_ReadWrite"),
    ("Write", 68, 2, "FileIo_ReadWrite"),
    ("SetInfo", 69, 2, "FileIo_Info"),
    ("Delete", 70, 2, "FileIo_Info"),
    ("Rename", 71, 2, "FileIo_Info"),
    ("DirEnum", 72, 2, "FileIo_DirEnum"),
    ("Flush", 73, 2, "FileIo_SimpleOp"),
    ("QueryInfo", 74, 2, "FileIo_Info"),
    ("FSControl", 75, 2, "FileIo_Info"),
    ("OperationEnd", 76, 2, "FileIo_OpEnd"),
    ("DirNotify", 77, 2, "FileIo_DirEnum"),
    ("DeletePath", 79, 2, "FileIo_PathOperation"),
    ("RenamePath", 80, 2, "FileIo_PathOperation"),
    ("SetLinkPath", 81, 2, "FileIo_PathOperation"),
    // V3 (same as V2 plus minifilter events)
    ("Name", 0, 3, "FileIo_Name"),
    ("FileCreate", 32, 3, "FileIo_Name"),
    ("FileDelete", 35, 3, "FileIo_Name"),
    ("FileRundown", 36, 3, "FileIo_Name"),
    ("Create", 64, 3, "FileIo_Create"),
    ("Cleanup", 65, 3, "FileIo_SimpleOp"),
    ("Close", 66, 3, "FileIo_SimpleOp"),
    ("Read", 67, 3, "FileIo_ReadWrite"),
    ("Write", 68, 3, "FileIo_ReadWrite"),
    ("SetInfo", 69, 3, "FileIo_Info"),
    ("Delete", 70, 3, "FileIo_Info"),
    ("Rename", 71, 3, "FileIo_Info"),
    ("DirEnum", 72, 3, "FileIo_DirEnum"),
    ("Flush", 73, 3, "FileIo_SimpleOp"),
    ("QueryInfo", 74, 3, "FileIo_Info"),
    ("FSControl", 75, 3, "FileIo_Info"),
    ("OperationEnd", 76, 3, "FileIo_OpEnd"),
    ("DirNotify", 77, 3, "FileIo_DirEnum"),
    ("DeletePath", 79, 3, "FileIo_PathOperation"),
    ("RenamePath", 80, 3, "FileIo_PathOperation"),
    ("SetLinkPath", 81, 3, "FileIo_PathOperation"),
    ("FltPreOpInit", 96, 3, "FltIoInit"),
    ("FltPostOpInit", 97, 3, "FltIoInit"),
    ("FltPreOpCompletion", 98, 3, "FltIoCompletion"),
    ("FltPostOpCompletion", 99, 3, "FltIoCompletion"),
    ("FltPreOpFailure", 100, 3, "FltIoFailure"),
    ("FltPostOpFailure", 101, 3, "FltIoFailure"),
];

/// Lazily-initialized static list of all event infos.
use std::sync::OnceLock;
static ALL_EVENTS: OnceLock<Vec<EventInfo>> = OnceLock::new();

fn get_all_events() -> &'static Vec<EventInfo> {
    ALL_EVENTS.get_or_init(|| {
        ALL_EVENTS_DATA
            .iter()
            .map(|&(name, id, ver, mof)| EventInfo {
                key: EventKey::new(id, ver),
                name,
                mof_class: mof,
            })
            .collect()
    })
}

impl EventDefs {
    /// Lookup an event by id and version. Returns None if unknown.
    pub fn lookup(id: u16, version: u8) -> Option<&'static EventInfo> {
        get_all_events().iter().find(|e| e.key.id == id && e.key.version == version)
    }
}
