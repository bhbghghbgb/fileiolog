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
        // ── NameCreateArgs template ──────────────────────────
        // Events 10 (NameCreate) and 11 (NameDelete) share this template
        #[etw_event(name = "NameCreateV0", id = 10, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        #[etw_event(name = "NameDeleteV0", id = 11, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameCreateArgs {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── CreateArgs template (V0) ────────────────────────
        #[etw_event(name = "CreateV0", id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        pub struct CreateArgsV0 {
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

        // ── CreateArgs template (V1) ────────────────────────
        #[etw_event(name = "CreateV1", id = 12, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        pub struct CreateArgsV1 {
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

        // ── CleanupArgs template (V0) ───────────────────────
        // Events 13 (Cleanup), 14 (Close), 21 (Flush) share this template
        #[etw_event(name = "CleanupV0", id = 13, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "CloseV0", id = 14, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "FlushV0", id = 21, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct CleanupArgsV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // ── CleanupArgs template (V1) ───────────────────────
        // Events 13 (Cleanup), 14 (Close), 21 (Flush) share this template
        #[etw_event(name = "CleanupV1", id = 13, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "CloseV1", id = 14, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "FlushV1", id = 21, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct CleanupArgsV1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── ReadArgs template (V0) ──────────────────────────
        // Events 15 (Read), 16 (Write) share this template
        #[etw_event(name = "ReadV0", id = 15, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
        #[etw_event(name = "WriteV0", id = 16, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
        pub struct ReadArgsV0 {
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

        // ── ReadArgs template (V1) ──────────────────────────
        // Events 15 (Read), 16 (Write) share this template
        #[etw_event(name = "ReadV1", id = 15, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
        #[etw_event(name = "WriteV1", id = 16, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
        pub struct ReadArgsV1 {
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

        // ── SetInformationArgs template (V0) ─────────────────
        // Events 17 (SetInformation), 18 (SetDelete), 19 (Rename),
        // 22 (QueryInformation), 23 (FSCTL) share this template
        #[etw_event(name = "SetInformationV0", id = 17, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetDeleteV0", id = 18, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "RenameV0", id = 19, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "QueryInformationV0", id = 22, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "FsctlV0", id = 23, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetLinkV0", id = 29, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct SetInformationArgsV0 {
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

        // ── SetInformationArgs template (V1) ─────────────────
        // Events 17 (SetInformation), 18 (SetDelete), 19 (Rename),
        // 22 (QueryInformation), 23 (FSCTL), 29 (Rename29),
        // 31 (SetSecurity), 32 (QuerySecurity), 33 (SetEA), 34 (QueryEA)
        #[etw_event(name = "SetInformationV1", id = 17, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetDeleteV1", id = 18, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "RenameV1", id = 19, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "QueryInformationV1", id = 22, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "FsctlV1", id = 23, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetLinkV1", id = 29, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetSecurityV1", id = 31, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "QuerySecurityV1", id = 32, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "SetEAV1", id = 33, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "QueryEAV1", id = 34, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct SetInformationArgsV1 {
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

        // ── DirEnumArgs template (V0) ────────────────────────
        // Events 20 (DirEnum), 25 (DirNotify) share this template
        #[etw_event(name = "DirEnumV0", id = 20, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "DirNotifyV0", id = 25, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct DirEnumArgsV0 {
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

        // ── DirEnumArgs template (V1) ────────────────────────
        // Events 20 (DirEnum), 25 (DirNotify) share this template
        #[etw_event(name = "DirEnumV1", id = 20, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        #[etw_event(name = "DirNotifyV1", id = 25, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        pub struct DirEnumArgsV1 {
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

        // ── OperationEndArgs template ────────────────────────
        #[etw_event(name = "OperationEndV0", id = 24, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_OP_END)]
        pub struct OperationEndArgsV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── DeletePathArgs template (V0) ─────────────────────
        // Events 26 (DeletePath), 27 (RenamePath), 28 (SetLinkPath) share this template
        #[etw_event(name = "DeletePathV0", id = 26, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
        #[etw_event(name = "RenamePathV0", id = 27, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        #[etw_event(name = "SetLinkPathV0", id = 28, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        pub struct DeletePathArgsV0 {
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

        // ── DeletePathArgs template (V1) ─────────────────────
        // Events 26 (DeletePath), 27 (RenamePath), 28 (SetLinkPath) share this template
        #[etw_event(name = "DeletePathV1", id = 26, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
        #[etw_event(name = "RenamePathV1", id = 27, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        #[etw_event(name = "SetLinkPathV1", id = 28, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        pub struct DeletePathArgsV1 {
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

        // ── CreateNewFileArgs template (V0) ──────────────────
        #[etw_event(name = "CreateNewFileV0", id = 30, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
        pub struct CreateNewFileArgsV0 {
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

        // ── CreateNewFileArgs template (V1) ──────────────────
        #[etw_event(name = "CreateNewFileV1", id = 30, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
        pub struct CreateNewFileArgsV1 {
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
    }
}
