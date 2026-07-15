#![allow(dead_code)]

use crate::etw::etw_provider;
use etw_macros::guid;

pub const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-File";
pub const PROVIDER_GUID: ::windows::core::GUID =
    guid!("EDD08927-9CC4-4E65-B970-C2560FB5C289");

pub const FILE_NAME_MASK: u64 = 0x10;
pub const FILE_CREATE_MASK: u64 = 0xa0;
pub const FILE_GENERIC_MASK: u64 = 0x20;
pub const FILE_READ_MASK: u64 = 0x120;
pub const FILE_WRITE_MASK: u64 = 0x220;
pub const FILE_OP_END_MASK: u64 = 0x60;
pub const FILE_DELETE_PATH_MASK: u64 = 0x400;
pub const FILE_RENAME_PATH_MASK: u64 = 0x800;
pub const FILE_CREATE_NEW_MASK: u64 = 0x1000;

etw_provider! {
    #[etw_provider(name = PROVIDER_NAME, guid = PROVIDER_GUID)]
    pub enum KernelFileEvent {
        // ── Event ID 10 ──────────────────────────────────────
        #[etw_event(id = 10, version = 0, mask = FILE_NAME_MASK)]
        pub struct NameCreateV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 11 ──────────────────────────────────────
        #[etw_event(id = 11, version = 0, mask = FILE_NAME_MASK)]
        pub struct NameDeleteV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 12 (v0, v1) ─────────────────────────────
        #[etw_event(id = 12, version = 0, mask = FILE_CREATE_MASK)]
        pub struct CreateV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 12, version = 1, mask = FILE_CREATE_MASK)]
        pub struct CreateV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
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
        #[etw_event(id = 13, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct CleanupV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        #[etw_event(id = 13, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct CleanupV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 14 (v0, v1) ─────────────────────────────
        #[etw_event(id = 14, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct CloseV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        #[etw_event(id = 14, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct CloseV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 15 (v0, v1) ─────────────────────────────
        #[etw_event(id = 15, version = 0, mask = FILE_READ_MASK)]
        pub struct ReadV0 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
        }

        #[etw_event(id = 15, version = 1, mask = FILE_READ_MASK)]
        pub struct ReadV1 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
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
        #[etw_event(id = 16, version = 0, mask = FILE_WRITE_MASK)]
        pub struct WriteV0 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IOSize")]
            pub io_size: u32,
            #[etw_prop(name = "IOFlags")]
            pub io_flags: u32,
        }

        #[etw_event(id = 16, version = 1, mask = FILE_WRITE_MASK)]
        pub struct WriteV1 {
            #[etw_prop(name = "ByteOffset")]
            pub byte_offset: u64,
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
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
        #[etw_event(id = 17, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct SetInformationV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 17, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct SetInformationV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 18 (v0, v1) ─────────────────────────────
        #[etw_event(id = 18, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct SetDeleteV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 18, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct SetDeleteV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 19 (v0, v1) ─────────────────────────────
        #[etw_event(id = 19, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct RenameV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 19, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct RenameV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 20 (v0, v1) ─────────────────────────────
        #[etw_event(id = 20, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct DirEnumV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 20, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct DirEnumV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
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
        #[etw_event(id = 21, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct FlushV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        #[etw_event(id = 21, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct FlushV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Event ID 22 (v0, v1) ─────────────────────────────
        #[etw_event(id = 22, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct QueryInformationV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 22, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct QueryInformationV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 23 (v0, v1) ─────────────────────────────
        #[etw_event(id = 23, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct FsctlV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 23, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct FsctlV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 24 (v0) ─────────────────────────────────
        #[etw_event(id = 24, version = 0, mask = FILE_OP_END_MASK)]
        pub struct OperationEndV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── Event ID 25 (v0, v1) ─────────────────────────────
        #[etw_event(id = 25, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct DirNotifyV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 25, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct DirNotifyV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
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
        #[etw_event(id = 26, version = 0, mask = FILE_DELETE_PATH_MASK)]
        pub struct DeletePathV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 26, version = 1, mask = FILE_DELETE_PATH_MASK)]
        pub struct DeletePathV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 27 (v0, v1) ─────────────────────────────
        #[etw_event(id = 27, version = 0, mask = FILE_RENAME_PATH_MASK)]
        pub struct RenamePathV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 27, version = 1, mask = FILE_RENAME_PATH_MASK)]
        pub struct RenamePathV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 28 (v0, v1) ─────────────────────────────
        #[etw_event(id = 28, version = 0, mask = FILE_RENAME_PATH_MASK)]
        pub struct SetLinkPathV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        #[etw_event(id = 28, version = 1, mask = FILE_RENAME_PATH_MASK)]
        pub struct SetLinkPathV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FilePath")]
            pub file_path: String,
        }

        // ── Event ID 29 (v0, v1) ─────────────────────────────
        #[etw_event(id = 29, version = 0, mask = FILE_GENERIC_MASK)]
        pub struct SetLinkV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        #[etw_event(id = 29, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct SetLinkV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 30 (v0, v1) ─────────────────────────────
        #[etw_event(id = 30, version = 0, mask = FILE_CREATE_NEW_MASK)]
        pub struct CreateNewFileV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "CreateAttributes")]
            pub create_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 30, version = 1, mask = FILE_CREATE_NEW_MASK)]
        pub struct CreateNewFileV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
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
        #[etw_event(id = 31, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct SetSecurityV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 32 (v1) ─────────────────────────────────
        #[etw_event(id = 32, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct QuerySecurityV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 33 (v1) ─────────────────────────────────
        #[etw_event(id = 33, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct SetEAV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── Event ID 34 (v1) ─────────────────────────────────
        #[etw_event(id = 34, version = 1, mask = FILE_GENERIC_MASK)]
        pub struct QueryEAV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }
    }
}
