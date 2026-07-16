#![allow(dead_code)]

use crate::etw::etw_provider;

pub mod masks {
    pub const KERNEL_FILE_KEYWORD_FILENAME: u64 = 0x10;
    pub const KERNEL_FILE_KEYWORD_FILEIO: u64 = 0x20;
    pub const KERNEL_FILE_KEYWORD_OP_END: u64 = 0x40;
    pub const KERNEL_FILE_KEYWORD_CREATE: u64 = 0x80;
    pub const KERNEL_FILE_KEYWORD_READ: u64 = 0x100;
    pub const KERNEL_FILE_KEYWORD_WRITE: u64 = 0x200;
    pub const KERNEL_FILE_KEYWORD_DELETE_PATH: u64 = 0x400;
    pub const KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH: u64 = 0x800;
    pub const KERNEL_FILE_KEYWORD_CREATE_NEW_FILE: u64 = 0x1000;
}

etw_provider! {
    #[etw_provider(kind = "user", name = "Microsoft-Windows-Kernel-File", guid = "EDD08927-9CC4-4E65-B970-C2560FB5C289")]
    pub enum UserTraceKernelFileEvent {
        // ── Event ID 10 ──────────────────────────────────────
        #[etw_event(id = 10, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameCreateV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 11 ──────────────────────────────────────
        #[etw_event(id = 11, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameDeleteV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Event ID 12 (v0, v1) ─────────────────────────────
        #[etw_event(id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
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

        #[etw_event(id = 12, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
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
        #[etw_event(id = 13, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 13, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 14, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 14, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 15, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
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

        #[etw_event(id = 15, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
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
        #[etw_event(id = 16, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
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

        #[etw_event(id = 16, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
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
        #[etw_event(id = 17, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 17, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 18, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 18, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 19, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 19, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 20, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 20, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 21, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 21, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 22, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 22, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 23, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 23, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 24, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_OP_END)]
        pub struct OperationEndV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── Event ID 25 (v0, v1) ─────────────────────────────
        #[etw_event(id = 25, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 25, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 26, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
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

        #[etw_event(id = 26, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
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
        #[etw_event(id = 27, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
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

        #[etw_event(id = 27, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
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
        #[etw_event(id = 28, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
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

        #[etw_event(id = 28, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
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
        #[etw_event(id = 29, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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

        #[etw_event(id = 29, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 30, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
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

        #[etw_event(id = 30, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
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
        #[etw_event(id = 31, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 32, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 33, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
        #[etw_event(id = 34, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
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
