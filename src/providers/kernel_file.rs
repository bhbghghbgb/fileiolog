use ferrisetw::provider::Provider;

pub const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-File";
pub const PROVIDER_GUID: &str = "EDD08927-9CC4-4E65-B970-C2560FB5C289";

// ── Convenience macro to define all events at once ──────────
macro_rules! def_events {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $enum_name:ident;

        $(
            $struct_name:ident ($event_id:literal, $event_ver:literal) {
                $($prop:literal : $field:ident : $ty:ty),+ $(,)?
            }
        );+ $(;)?
    ) => {
        // Struct definitions
        $(
            #[derive(Debug, Clone)]
            struct $struct_name {
                $(pub $field: $ty),+
            }
            impl $struct_name {
                fn try_from_parser(
                    parser: &ferrisetw::parser::Parser<'_, '_>,
                ) -> Result<Self, ferrisetw::parser::ParserError> {
                    Ok(Self {
                        $($field: parser.try_parse($prop)?),+
                    })
                }
            }
        )+

        // Enum
        $(#[$enum_meta])*
        #[derive(Debug, Clone)]
        $vis enum $enum_name {
            $($struct_name($struct_name)),+
        }

        impl $enum_name {
            pub fn try_parse(
                record: &ferrisetw::EventRecord,
                schema_locator: &ferrisetw::schema_locator::SchemaLocator,
            ) -> Option<Self> {
                let schema = schema_locator.event_schema(record).ok()?;
                let parser = ferrisetw::parser::Parser::create(record, &schema);
                match (record.event_id(), record.version()) {
                    $(($event_id, $event_ver) => {
                        Some(Self::$struct_name($struct_name::try_from_parser(&parser).ok()?))
                    }),+
                    _ => None,
                }
            }

            pub fn print(&self) {
                println!("{:?}", self);
            }
        }
    };
}

// ── All Kernel-File events ──────────────────────────────────
def_events! {
    pub enum KernelFileEvent;

    // Event ID 10
    NameCreateV0 (10, 0) {
        "FileKey": file_key: u64,
        "FileName": file_name: String,
    };
    // Event ID 11
    NameDeleteV0 (11, 0) {
        "FileKey": file_key: u64,
        "FileName": file_name: String,
    };
    // Event ID 12 v0
    CreateV0 (12, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    };
    // Event ID 12 v1
    CreateV1 (12, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    };
    // Event ID 13 v0
    CleanupV0 (13, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    };
    // Event ID 13 v1
    CleanupV1 (13, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    };
    // Event ID 14 v0
    CloseV0 (14, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    };
    // Event ID 14 v1
    CloseV1 (14, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    };
    // Event ID 15 v0
    ReadV0 (15, 0) {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
    };
    // Event ID 15 v1
    ReadV1 (15, 1) {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
        "ExtraFlags": extra_flags: u32,
    };
    // Event ID 16 v0
    WriteV0 (16, 0) {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
    };
    // Event ID 16 v1
    WriteV1 (16, 1) {
        "ByteOffset": byte_offset: u64,
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "IOSize": io_size: u32,
        "IOFlags": io_flags: u32,
        "ExtraFlags": extra_flags: u32,
    };
    // Event ID 17 v0
    SetInformationV0 (17, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 17 v1
    SetInformationV1 (17, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 18 v0
    SetDeleteV0 (18, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 18 v1
    SetDeleteV1 (18, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 19 v0
    RenameV0 (19, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 19 v1
    RenameV1 (19, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 20 v0
    DirEnumV0 (20, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    };
    // Event ID 20 v1
    DirEnumV1 (20, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    };
    // Event ID 21 v0
    FlushV0 (21, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
    };
    // Event ID 21 v1
    FlushV1 (21, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
    };
    // Event ID 22 v0
    QueryInformationV0 (22, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 22 v1
    QueryInformationV1 (22, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 23 v0
    FsctlV0 (23, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 23 v1
    FsctlV1 (23, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 24 v0
    OperationEndV0 (24, 0) {
        "Irp": irp: u64,
        "ExtraInformation": extra_information: u64,
        "Status": status: u32,
    };
    // Event ID 25 v0
    DirNotifyV0 (25, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    };
    // Event ID 25 v1
    DirNotifyV1 (25, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "Length": length: u32,
        "InfoClass": info_class: u32,
        "FileIndex": file_index: u32,
        "FileName": file_name: String,
    };
    // Event ID 26 v0
    DeletePathV0 (26, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 26 v1
    DeletePathV1 (26, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 27 v0
    RenamePathV0 (27, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 27 v1
    RenamePathV1 (27, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 28 v0
    SetLinkPathV0 (28, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 28 v1
    SetLinkPathV1 (28, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
        "FilePath": file_path: String,
    };
    // Event ID 29 v0
    SetLinkV0 (29, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "InfoClass": info_class: u32,
    };
    // Event ID 29 v1
    SetLinkV1 (29, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 30 v0
    CreateNewFileV0 (30, 0) {
        "Irp": irp: u64,
        "ThreadId": thread_id: u64,
        "FileObject": file_object: u64,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    };
    // Event ID 30 v1
    CreateNewFileV1 (30, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "CreateOptions": create_options: u32,
        "CreateAttributes": create_attributes: u32,
        "ShareAccess": share_access: u32,
        "FileName": file_name: String,
    };
    // Event ID 31 v1
    SetSecurityV1 (31, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 32 v1
    QuerySecurityV1 (32, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 33 v1
    SetEAV1 (33, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
    // Event ID 34 v1
    QueryEAV1 (34, 1) {
        "Irp": irp: u64,
        "FileObject": file_object: u64,
        "FileKey": file_key: u64,
        "ExtraInformation": extra_information: u64,
        "IssuingThreadId": issuing_thread_id: u32,
        "InfoClass": info_class: u32,
    };
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
