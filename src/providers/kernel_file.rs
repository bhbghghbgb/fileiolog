#![allow(dead_code)]

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;

pub const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-File";
pub const PROVIDER_GUID: &str = "EDD08927-9CC4-4E65-B970-C2560FB5C289";

macro_rules! def_event {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($prop:literal: $field:ident: $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name {
            $(pub $field: $ty),+
        }
        impl $name {
            fn try_from_parser(parser: &Parser<'_, '_>) -> Result<Self, ParserError> {
                Ok(Self {
                    $($field: parser.try_parse($prop)?),+
                })
            }
        }
    };
}

// ── Event ID 10 ──────────────────────────────────────────────
def_event! {
    pub struct NameCreateV0 {
        "FileKey": file_key: u64,
        "FileName": file_name: String,
    }
}

// ── Event ID 11 ──────────────────────────────────────────────
def_event! {
    pub struct NameDeleteV0 {
        "FileKey": file_key: u64,
        "FileName": file_name: String,
    }
}

// ── Event ID 12 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct CreateV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    }
}

def_event! {
    pub struct CreateV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    }
}

// ── Event ID 13 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct CleanupV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    }
}

def_event! {
    pub struct CleanupV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    }
}

// ── Event ID 14 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct CloseV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    }
}

def_event! {
    pub struct CloseV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    }
}

// ── Event ID 15 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct ReadV0 {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
    }
}

def_event! {
    pub struct ReadV1 {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
        "ExtraFlags": extra_flags: u32,
    }
}

// ── Event ID 16 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct WriteV0 {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
    }
}

def_event! {
    pub struct WriteV1 {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
        "ExtraFlags": extra_flags: u32,
    }
}

// ── Event ID 17 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct SetInformationV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct SetInformationV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 18 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct SetDeleteV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct SetDeleteV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 19 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct RenameV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct RenameV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 20 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct DirEnumV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    }
}

def_event! {
    pub struct DirEnumV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    }
}

// ── Event ID 21 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct FlushV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    }
}

def_event! {
    pub struct FlushV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    }
}

// ── Event ID 22 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct QueryInformationV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct QueryInformationV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 23 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct FsctlV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct FsctlV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 24 (v0) ─────────────────────────────────────────
def_event! {
    pub struct OperationEndV0 {
        "Irp": irp: u64,
        "ExtraInformation": extra_information: u64,
        "Status": status: u32,
    }
}

// ── Event ID 25 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct DirNotifyV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    }
}

def_event! {
    pub struct DirNotifyV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    }
}

// ── Event ID 26 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct DeletePathV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

def_event! {
    pub struct DeletePathV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

// ── Event ID 27 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct RenamePathV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

def_event! {
    pub struct RenamePathV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

// ── Event ID 28 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct SetLinkPathV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

def_event! {
    pub struct SetLinkPathV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    }
}

// ── Event ID 29 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct SetLinkV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    }
}

def_event! {
    pub struct SetLinkV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 30 (v0, v1) ─────────────────────────────────────
def_event! {
    pub struct CreateNewFileV0 {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    }
}

def_event! {
    pub struct CreateNewFileV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    }
}

// ── Event ID 31 (v1) ─────────────────────────────────────────
def_event! {
    pub struct SetSecurityV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 32 (v1) ─────────────────────────────────────────
def_event! {
    pub struct QuerySecurityV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 33 (v1) ─────────────────────────────────────────
def_event! {
    pub struct SetEAV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Event ID 34 (v1) ─────────────────────────────────────────
def_event! {
    pub struct QueryEAV1 {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    }
}

// ── Enum of all Kernel-File event variants ───────────────────
#[derive(Debug, Clone)]
pub enum KernelFileEvent {
    NameCreateV0(NameCreateV0),
    NameDeleteV0(NameDeleteV0),
    CreateV0(CreateV0),
    CreateV1(CreateV1),
    CleanupV0(CleanupV0),
    CleanupV1(CleanupV1),
    CloseV0(CloseV0),
    CloseV1(CloseV1),
    ReadV0(ReadV0),
    ReadV1(ReadV1),
    WriteV0(WriteV0),
    WriteV1(WriteV1),
    SetInformationV0(SetInformationV0),
    SetInformationV1(SetInformationV1),
    SetDeleteV0(SetDeleteV0),
    SetDeleteV1(SetDeleteV1),
    RenameV0(RenameV0),
    RenameV1(RenameV1),
    DirEnumV0(DirEnumV0),
    DirEnumV1(DirEnumV1),
    FlushV0(FlushV0),
    FlushV1(FlushV1),
    QueryInformationV0(QueryInformationV0),
    QueryInformationV1(QueryInformationV1),
    FsctlV0(FsctlV0),
    FsctlV1(FsctlV1),
    OperationEndV0(OperationEndV0),
    DirNotifyV0(DirNotifyV0),
    DirNotifyV1(DirNotifyV1),
    DeletePathV0(DeletePathV0),
    DeletePathV1(DeletePathV1),
    RenamePathV0(RenamePathV0),
    RenamePathV1(RenamePathV1),
    SetLinkPathV0(SetLinkPathV0),
    SetLinkPathV1(SetLinkPathV1),
    SetLinkV0(SetLinkV0),
    SetLinkV1(SetLinkV1),
    CreateNewFileV0(CreateNewFileV0),
    CreateNewFileV1(CreateNewFileV1),
    SetSecurityV1(SetSecurityV1),
    QuerySecurityV1(QuerySecurityV1),
    SetEAV1(SetEAV1),
    QueryEAV1(QueryEAV1),
}

impl KernelFileEvent {
    pub fn try_parse(record: &EventRecord, schema_locator: &SchemaLocator) -> Option<Self> {
        let schema = schema_locator.event_schema(record).ok()?;
        let parser = Parser::create(record, &schema);
        match (record.event_id(), record.version()) {
            (10, 0) => Some(Self::NameCreateV0(
                NameCreateV0::try_from_parser(&parser).ok()?,
            )),
            (11, 0) => Some(Self::NameDeleteV0(
                NameDeleteV0::try_from_parser(&parser).ok()?,
            )),
            (12, 0) => Some(Self::CreateV0(CreateV0::try_from_parser(&parser).ok()?)),
            (12, 1) => Some(Self::CreateV1(CreateV1::try_from_parser(&parser).ok()?)),
            (13, 0) => Some(Self::CleanupV0(CleanupV0::try_from_parser(&parser).ok()?)),
            (13, 1) => Some(Self::CleanupV1(CleanupV1::try_from_parser(&parser).ok()?)),
            (14, 0) => Some(Self::CloseV0(CloseV0::try_from_parser(&parser).ok()?)),
            (14, 1) => Some(Self::CloseV1(CloseV1::try_from_parser(&parser).ok()?)),
            (15, 0) => Some(Self::ReadV0(ReadV0::try_from_parser(&parser).ok()?)),
            (15, 1) => Some(Self::ReadV1(ReadV1::try_from_parser(&parser).ok()?)),
            (16, 0) => Some(Self::WriteV0(WriteV0::try_from_parser(&parser).ok()?)),
            (16, 1) => Some(Self::WriteV1(WriteV1::try_from_parser(&parser).ok()?)),
            (17, 0) => Some(Self::SetInformationV0(
                SetInformationV0::try_from_parser(&parser).ok()?,
            )),
            (17, 1) => Some(Self::SetInformationV1(
                SetInformationV1::try_from_parser(&parser).ok()?,
            )),
            (18, 0) => Some(Self::SetDeleteV0(
                SetDeleteV0::try_from_parser(&parser).ok()?,
            )),
            (18, 1) => Some(Self::SetDeleteV1(
                SetDeleteV1::try_from_parser(&parser).ok()?,
            )),
            (19, 0) => Some(Self::RenameV0(RenameV0::try_from_parser(&parser).ok()?)),
            (19, 1) => Some(Self::RenameV1(RenameV1::try_from_parser(&parser).ok()?)),
            (20, 0) => Some(Self::DirEnumV0(DirEnumV0::try_from_parser(&parser).ok()?)),
            (20, 1) => Some(Self::DirEnumV1(DirEnumV1::try_from_parser(&parser).ok()?)),
            (21, 0) => Some(Self::FlushV0(FlushV0::try_from_parser(&parser).ok()?)),
            (21, 1) => Some(Self::FlushV1(FlushV1::try_from_parser(&parser).ok()?)),
            (22, 0) => Some(Self::QueryInformationV0(
                QueryInformationV0::try_from_parser(&parser).ok()?,
            )),
            (22, 1) => Some(Self::QueryInformationV1(
                QueryInformationV1::try_from_parser(&parser).ok()?,
            )),
            (23, 0) => Some(Self::FsctlV0(FsctlV0::try_from_parser(&parser).ok()?)),
            (23, 1) => Some(Self::FsctlV1(FsctlV1::try_from_parser(&parser).ok()?)),
            (24, 0) => Some(Self::OperationEndV0(
                OperationEndV0::try_from_parser(&parser).ok()?,
            )),
            (25, 0) => Some(Self::DirNotifyV0(
                DirNotifyV0::try_from_parser(&parser).ok()?,
            )),
            (25, 1) => Some(Self::DirNotifyV1(
                DirNotifyV1::try_from_parser(&parser).ok()?,
            )),
            (26, 0) => Some(Self::DeletePathV0(
                DeletePathV0::try_from_parser(&parser).ok()?,
            )),
            (26, 1) => Some(Self::DeletePathV1(
                DeletePathV1::try_from_parser(&parser).ok()?,
            )),
            (27, 0) => Some(Self::RenamePathV0(
                RenamePathV0::try_from_parser(&parser).ok()?,
            )),
            (27, 1) => Some(Self::RenamePathV1(
                RenamePathV1::try_from_parser(&parser).ok()?,
            )),
            (28, 0) => Some(Self::SetLinkPathV0(
                SetLinkPathV0::try_from_parser(&parser).ok()?,
            )),
            (28, 1) => Some(Self::SetLinkPathV1(
                SetLinkPathV1::try_from_parser(&parser).ok()?,
            )),
            (29, 0) => Some(Self::SetLinkV0(SetLinkV0::try_from_parser(&parser).ok()?)),
            (29, 1) => Some(Self::SetLinkV1(SetLinkV1::try_from_parser(&parser).ok()?)),
            (30, 0) => Some(Self::CreateNewFileV0(
                CreateNewFileV0::try_from_parser(&parser).ok()?,
            )),
            (30, 1) => Some(Self::CreateNewFileV1(
                CreateNewFileV1::try_from_parser(&parser).ok()?,
            )),
            (31, 1) => Some(Self::SetSecurityV1(
                SetSecurityV1::try_from_parser(&parser).ok()?,
            )),
            (32, 1) => Some(Self::QuerySecurityV1(
                QuerySecurityV1::try_from_parser(&parser).ok()?,
            )),
            (33, 1) => Some(Self::SetEAV1(SetEAV1::try_from_parser(&parser).ok()?)),
            (34, 1) => Some(Self::QueryEAV1(QueryEAV1::try_from_parser(&parser).ok()?)),
            _ => None,
        }
    }

    pub fn print(&self) {
        println!("{:?}", self);
    }
}

pub fn build_provider() -> Provider {
    Provider::by_guid(PROVIDER_GUID)
        .add_callback(|record, locator| {
            if let Some(event) = KernelFileEvent::try_parse(record, locator) {
                event.print();
            }
        })
        .build()
}
