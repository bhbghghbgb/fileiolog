#![allow(dead_code)]

use crate::etw::{EtwEvent, EtwEventParse, etw_provider};
use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;

etw_provider! {
    #[etw_provider(name = "Microsoft-Windows-Kernel-File", guid = "EDD08927-9CC4-4E65-B970-C2560FB5C289")]
    pub enum KernelFileEvent {
        // ── Event ID 10 ──────────────────────────────────────
        #[etw_event(id = 10, version = 0, mask = 0x10)]
        pub struct NameCreateV0 {
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 11 ──────────────────────────────────────
        #[etw_event(id = 11, version = 0, mask = 0x10)]
        pub struct NameDeleteV0 {
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 12 (v0, v1) ─────────────────────────────
        #[etw_event(id = 12, version = 0, mask = 0xa0)]
        pub struct CreateV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 12, version = 1, mask = 0xa0)]
        pub struct CreateV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 13 (v0, v1) ─────────────────────────────
        #[etw_event(id = 13, version = 0, mask = 0x20)]
        pub struct CleanupV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
        }

        #[etw_event(id = 13, version = 1, mask = 0x20)]
        pub struct CleanupV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 14 (v0, v1) ─────────────────────────────
        #[etw_event(id = 14, version = 0, mask = 0x20)]
        pub struct CloseV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
        }

        #[etw_event(id = 14, version = 1, mask = 0x20)]
        pub struct CloseV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 15 (v0, v1) ─────────────────────────────
        #[etw_event(id = 15, version = 0, mask = 0x120)]
        pub struct ReadV0 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
        }

        #[etw_event(id = 15, version = 1, mask = 0x120)]
        pub struct ReadV1 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
            #[etw_prop(name = "ExtraFlags")]
            pub extra_flags: u32,
        }

        // ── Event ID 16 (v0, v1) ─────────────────────────────
        #[etw_event(id = 16, version = 0, mask = 0x220)]
        pub struct WriteV0 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
        }

        #[etw_event(id = 16, version = 1, mask = 0x220)]
        pub struct WriteV1 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
            #[etw_prop(name = "ExtraFlags")]
            pub extra_flags: u32,
        }

        // ── Event ID 17 (v0, v1) ─────────────────────────────
        #[etw_event(id = 17, version = 0, mask = 0x20)]
        pub struct SetInformationV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 17, version = 1, mask = 0x20)]
        pub struct SetInformationV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 18 (v0, v1) ─────────────────────────────
        #[etw_event(id = 18, version = 0, mask = 0x20)]
        pub struct SetDeleteV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 18, version = 1, mask = 0x20)]
        pub struct SetDeleteV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 19 (v0, v1) ─────────────────────────────
        #[etw_event(id = 19, version = 0, mask = 0x20)]
        pub struct RenameV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 19, version = 1, mask = 0x20)]
        pub struct RenameV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 20 (v0, v1) ─────────────────────────────
        #[etw_event(id = 20, version = 0, mask = 0x20)]
        pub struct DirEnumV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 20, version = 1, mask = 0x20)]
        pub struct DirEnumV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 21 (v0, v1) ─────────────────────────────
        #[etw_event(id = 21, version = 0, mask = 0x20)]
        pub struct FlushV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
        }

        #[etw_event(id = 21, version = 1, mask = 0x20)]
        pub struct FlushV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 22 (v0, v1) ─────────────────────────────
        #[etw_event(id = 22, version = 0, mask = 0x20)]
        pub struct QueryInformationV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 22, version = 1, mask = 0x20)]
        pub struct QueryInformationV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 23 (v0, v1) ─────────────────────────────
        #[etw_event(id = 23, version = 0, mask = 0x20)]
        pub struct FsctlV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 23, version = 1, mask = 0x20)]
        pub struct FsctlV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 24 (v0) ─────────────────────────────────
        #[etw_event(id = 24, version = 0, mask = 0x60)]
        pub struct OperationEndV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── Event ID 25 (v0, v1) ─────────────────────────────
        #[etw_event(id = 25, version = 0, mask = 0x20)]
        pub struct DirNotifyV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 25, version = 1, mask = 0x20)]
        pub struct DirNotifyV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 26 (v0, v1) ─────────────────────────────
        #[etw_event(id = 26, version = 0, mask = 0x400)]
        pub struct DeletePathV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 26, version = 1, mask = 0x400)]
        pub struct DeletePathV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 27 (v0, v1) ─────────────────────────────
        #[etw_event(id = 27, version = 0, mask = 0x800)]
        pub struct RenamePathV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 27, version = 1, mask = 0x800)]
        pub struct RenamePathV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 28 (v0, v1) ─────────────────────────────
        #[etw_event(id = 28, version = 0, mask = 0x800)]
        pub struct SetLinkPathV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 28, version = 1, mask = 0x800)]
        pub struct SetLinkPathV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 29 (v0, v1) ─────────────────────────────
        #[etw_event(id = 29, version = 0, mask = 0x20)]
        pub struct SetLinkV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 29, version = 1, mask = 0x20)]
        pub struct SetLinkV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 30 (v0, v1) ─────────────────────────────
        #[etw_event(id = 30, version = 0, mask = 0x1000)]
        pub struct CreateNewFileV0 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "ThreadId")]
            pub thread_id: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 30, version = 1, mask = 0x1000)]
        pub struct CreateNewFileV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 31 (v1) ─────────────────────────────────
        #[etw_event(id = 31, version = 1, mask = 0x20)]
        pub struct SetSecurityV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 32 (v1) ─────────────────────────────────
        #[etw_event(id = 32, version = 1, mask = 0x20)]
        pub struct QuerySecurityV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 33 (v1) ─────────────────────────────────
        #[etw_event(id = 33, version = 1, mask = 0x20)]
        pub struct SetEAV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 34 (v1) ─────────────────────────────────
        #[etw_event(id = 34, version = 1, mask = 0x20)]
        pub struct QueryEAV1 {
            #[etw_prop(name = "Irp")]
            pub irp: u64,
            #[etw_prop(name = "FileObject")]
            pub file_object: u64,
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "ExtraInformation")]
            pub extra_information: u64,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }
    }
}
