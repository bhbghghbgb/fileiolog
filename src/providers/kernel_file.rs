#![allow(dead_code)]

use crate::etw::{EtwEvent, EtwEventParse, etw_provider};
use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;

pub const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-File";
pub const PROVIDER_GUID: &str = "EDD08927-9CC4-4E65-B970-C2560FB5C289";

etw_provider! {
    pub enum KernelFileEvent {
        // ── Event ID 10 ──────────────────────────────────────
        #[event(id = 10, version = 0)]
        pub struct NameCreateV0 {
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 11 ──────────────────────────────────────
        #[event(id = 11, version = 0)]
        pub struct NameDeleteV0 {
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 12 (v0, v1) ─────────────────────────────
        #[event(id = 12, version = 0)]
        pub struct CreateV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "CreateOptions")]
            pub create_options: u32,
            #[etw(prop = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw(prop = "ShareAccess")]
            pub share_access: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        #[event(id = 12, version = 1)]
        pub struct CreateV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "CreateOptions")]
            pub create_options: u32,
            #[etw(prop = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw(prop = "ShareAccess")]
            pub share_access: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 13 (v0, v1) ─────────────────────────────
        #[event(id = 13, version = 0)]
        pub struct CleanupV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
        }

        #[event(id = 13, version = 1)]
        pub struct CleanupV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 14 (v0, v1) ─────────────────────────────
        #[event(id = 14, version = 0)]
        pub struct CloseV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
        }

        #[event(id = 14, version = 1)]
        pub struct CloseV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 15 (v0, v1) ─────────────────────────────
        #[event(id = 15, version = 0)]
        pub struct ReadV0 {
            #[etw(prop = "ByteOffset")]
            pub byte_offset: u64,
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IOSize")]
            pub io_size: u32,
            #[etw(prop = "IOFlags")]
            pub io_flags: u32,
        }

        #[event(id = 15, version = 1)]
        pub struct ReadV1 {
            #[etw(prop = "ByteOffset")]
            pub byte_offset: u64,
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "IOSize")]
            pub io_size: u32,
            #[etw(prop = "IOFlags")]
            pub io_flags: u32,
            #[etw(prop = "ExtraFlags")]
            pub extra_flags: u32,
        }

        // ── Event ID 16 (v0, v1) ─────────────────────────────
        #[event(id = 16, version = 0)]
        pub struct WriteV0 {
            #[etw(prop = "ByteOffset")]
            pub byte_offset: u64,
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IOSize")]
            pub io_size: u32,
            #[etw(prop = "IOFlags")]
            pub io_flags: u32,
        }

        #[event(id = 16, version = 1)]
        pub struct WriteV1 {
            #[etw(prop = "ByteOffset")]
            pub byte_offset: u64,
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "IOSize")]
            pub io_size: u32,
            #[etw(prop = "IOFlags")]
            pub io_flags: u32,
            #[etw(prop = "ExtraFlags")]
            pub extra_flags: u32,
        }

        // ── Event ID 17 (v0, v1) ─────────────────────────────
        #[event(id = 17, version = 0)]
        pub struct SetInformationV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 17, version = 1)]
        pub struct SetInformationV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 18 (v0, v1) ─────────────────────────────
        #[event(id = 18, version = 0)]
        pub struct SetDeleteV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 18, version = 1)]
        pub struct SetDeleteV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 19 (v0, v1) ─────────────────────────────
        #[event(id = 19, version = 0)]
        pub struct RenameV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 19, version = 1)]
        pub struct RenameV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 20 (v0, v1) ─────────────────────────────
        #[event(id = 20, version = 0)]
        pub struct DirEnumV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "Length")]
            pub length: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FileIndex")]
            pub file_index: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        #[event(id = 20, version = 1)]
        pub struct DirEnumV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "Length")]
            pub length: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FileIndex")]
            pub file_index: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 21 (v0, v1) ─────────────────────────────
        #[event(id = 21, version = 0)]
        pub struct FlushV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
        }

        #[event(id = 21, version = 1)]
        pub struct FlushV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 22 (v0, v1) ─────────────────────────────
        #[event(id = 22, version = 0)]
        pub struct QueryInformationV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 22, version = 1)]
        pub struct QueryInformationV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 23 (v0, v1) ─────────────────────────────
        #[event(id = 23, version = 0)]
        pub struct FsctlV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 23, version = 1)]
        pub struct FsctlV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 24 (v0) ─────────────────────────────────
        #[event(id = 24, version = 0)]
        pub struct OperationEndV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "Status")]
            pub status: u32,
        }

        // ── Event ID 25 (v0, v1) ─────────────────────────────
        #[event(id = 25, version = 0)]
        pub struct DirNotifyV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "Length")]
            pub length: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FileIndex")]
            pub file_index: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        #[event(id = 25, version = 1)]
        pub struct DirNotifyV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "Length")]
            pub length: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FileIndex")]
            pub file_index: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 26 (v0, v1) ─────────────────────────────
        #[event(id = 26, version = 0)]
        pub struct DeletePathV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        #[event(id = 26, version = 1)]
        pub struct DeletePathV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 27 (v0, v1) ─────────────────────────────
        #[event(id = 27, version = 0)]
        pub struct RenamePathV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        #[event(id = 27, version = 1)]
        pub struct RenamePathV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 28 (v0, v1) ─────────────────────────────
        #[event(id = 28, version = 0)]
        pub struct SetLinkPathV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        #[event(id = 28, version = 1)]
        pub struct SetLinkPathV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
            #[etw(prop = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 29 (v0, v1) ─────────────────────────────
        #[event(id = 29, version = 0)]
        pub struct SetLinkV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        #[event(id = 29, version = 1)]
        pub struct SetLinkV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 30 (v0, v1) ─────────────────────────────
        #[event(id = 30, version = 0)]
        pub struct CreateNewFileV0 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "ThreadId")]
            pub thread_id: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "CreateOptions")]
            pub create_options: u32,
            #[etw(prop = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw(prop = "ShareAccess")]
            pub share_access: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        #[event(id = 30, version = 1)]
        pub struct CreateNewFileV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "CreateOptions")]
            pub create_options: u32,
            #[etw(prop = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw(prop = "ShareAccess")]
            pub share_access: u32,
            #[etw(prop = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 31 (v1) ─────────────────────────────────
        #[event(id = 31, version = 1)]
        pub struct SetSecurityV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 32 (v1) ─────────────────────────────────
        #[event(id = 32, version = 1)]
        pub struct QuerySecurityV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 33 (v1) ─────────────────────────────────
        #[event(id = 33, version = 1)]
        pub struct SetEAV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 34 (v1) ─────────────────────────────────
        #[event(id = 34, version = 1)]
        pub struct QueryEAV1 {
            #[etw(prop = "Irp")]
            pub irp: u64,
            #[etw(prop = "FileObject")]
            pub file_object: u64,
            #[etw(prop = "FileKey")]
            pub file_key: u64,
            #[etw(prop = "ExtraInformation")]
            pub extra_information: u64,
            #[etw(prop = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw(prop = "InfoClass")]
            pub info_class: u32,
        }
    }
}

pub fn build_provider<F>(callback: F) -> Provider
where
    F: Fn(KernelFileEvent) + Send + Sync + 'static,
{
    Provider::by_guid(PROVIDER_GUID)
        .add_callback(move |record, locator| {
            if let Some(event) = KernelFileEvent::try_parse(record, locator) {
                callback(event);
            }
        })
        .build()
}
