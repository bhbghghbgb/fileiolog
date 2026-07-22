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

        // ── Template: NameCreateArgs ───────────────────────────
        template NameCreateArgs {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Template: CreateArgs ───────────────────────────────
        template CreateArgs {
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

        // ── Template: CreateArgs_V1 ────────────────────────────
        template CreateArgs_V1 {
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

        // ── Template: CleanupArgs ──────────────────────────────
        template CleanupArgs {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
            pub thread_id: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // ── Template: CleanupArgs_V1 ───────────────────────────
        template CleanupArgs_V1 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IssuingThreadId")]
            pub issuing_thread_id: u32,
        }

        // ── Template: ReadArgs ─────────────────────────────────
        template ReadArgs {
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

        // ── Template: ReadArgs_V1 ──────────────────────────────
        template ReadArgs_V1 {
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

        // ── Template: SetInformationArgs ───────────────────────
        template SetInformationArgs {
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

        // ── Template: SetInformationArgs_V1 ────────────────────
        template SetInformationArgs_V1 {
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

        // ── Template: DirEnumArgs ──────────────────────────────
        template DirEnumArgs {
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

        // ── Template: DirEnumArgs_V1 ───────────────────────────
        template DirEnumArgs_V1 {
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

        // ── Template: OperationEndArgs ─────────────────────────
        template OperationEndArgs {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── Template: DeletePathArgs ───────────────────────────
        template DeletePathArgs {
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

        // ── Template: DeletePathArgs_V1 ────────────────────────
        template DeletePathArgs_V1 {
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

        // ── Events ─────────────────────────────────────────────

        // Event ID 10: NameCreate
        #[etw_event(id = 10, version = 0, name = "NameCreateV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        NameCreateArgs,

        // Event ID 11: NameDelete
        #[etw_event(id = 11, version = 0, name = "NameDeleteV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        NameCreateArgs,

        // Event ID 12: Create
        #[etw_event(id = 12, version = 0, name = "CreateV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        CreateArgs,
        #[etw_event(id = 12, version = 1, name = "CreateV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        CreateArgs_V1,

        // Event ID 13: Cleanup
        #[etw_event(id = 13, version = 0, name = "CleanupV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs,
        #[etw_event(id = 13, version = 1, name = "CleanupV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs_V1,

        // Event ID 14: Close
        #[etw_event(id = 14, version = 0, name = "CloseV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs,
        #[etw_event(id = 14, version = 1, name = "CloseV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs_V1,

        // Event ID 15: Read
        #[etw_event(id = 15, version = 0, name = "ReadV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
        ReadArgs,
        #[etw_event(id = 15, version = 1, name = "ReadV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_READ)]
        ReadArgs_V1,

        // Event ID 16: Write
        #[etw_event(id = 16, version = 0, name = "WriteV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
        ReadArgs,
        #[etw_event(id = 16, version = 1, name = "WriteV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_WRITE)]
        ReadArgs_V1,

        // Event ID 17: SetInformation
        #[etw_event(id = 17, version = 0, name = "SetInformationV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 17, version = 1, name = "SetInformationV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 18: SetDelete
        #[etw_event(id = 18, version = 0, name = "SetDeleteV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 18, version = 1, name = "SetDeleteV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 19: Rename
        #[etw_event(id = 19, version = 0, name = "RenameV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 19, version = 1, name = "RenameV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 20: DirEnum
        #[etw_event(id = 20, version = 0, name = "DirEnumV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        DirEnumArgs,
        #[etw_event(id = 20, version = 1, name = "DirEnumV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        DirEnumArgs_V1,

        // Event ID 21: Flush
        #[etw_event(id = 21, version = 0, name = "FlushV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs,
        #[etw_event(id = 21, version = 1, name = "FlushV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        CleanupArgs_V1,

        // Event ID 22: QueryInformation
        #[etw_event(id = 22, version = 0, name = "QueryInformationV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 22, version = 1, name = "QueryInformationV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 23: FSCTL
        #[etw_event(id = 23, version = 0, name = "FsctlV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 23, version = 1, name = "FsctlV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 24: OperationEnd
        #[etw_event(id = 24, version = 0, name = "OperationEndV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_OP_END)]
        OperationEndArgs,

        // Event ID 25: DirNotify
        #[etw_event(id = 25, version = 0, name = "DirNotifyV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        DirEnumArgs,
        #[etw_event(id = 25, version = 1, name = "DirNotifyV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        DirEnumArgs_V1,

        // Event ID 26: DeletePath
        #[etw_event(id = 26, version = 0, name = "DeletePathV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
        DeletePathArgs,
        #[etw_event(id = 26, version = 1, name = "DeletePathV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_DELETE_PATH)]
        DeletePathArgs_V1,

        // Event ID 27: RenamePath
        #[etw_event(id = 27, version = 0, name = "RenamePathV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        DeletePathArgs,
        #[etw_event(id = 27, version = 1, name = "RenamePathV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        DeletePathArgs_V1,

        // Event ID 28: SetLinkPath
        #[etw_event(id = 28, version = 0, name = "SetLinkPathV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        DeletePathArgs,
        #[etw_event(id = 28, version = 1, name = "SetLinkPathV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_RENAME_SETLINK_PATH)]
        DeletePathArgs_V1,

        // Event ID 29: Rename29 (SetInformationArgs)
        #[etw_event(id = 29, version = 0, name = "SetLinkV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs,
        #[etw_event(id = 29, version = 1, name = "SetLinkV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 30: CreateNewFile
        #[etw_event(id = 30, version = 0, name = "CreateNewFileV0", keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
        CreateArgs,
        #[etw_event(id = 30, version = 1, name = "CreateNewFileV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
        CreateArgs_V1,

        // Event ID 31: SetSecurity (V1 only)
        #[etw_event(id = 31, version = 1, name = "SetSecurityV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 32: QuerySecurity (V1 only)
        #[etw_event(id = 32, version = 1, name = "QuerySecurityV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 33: SetEA (V1 only)
        #[etw_event(id = 33, version = 1, name = "SetEAV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,

        // Event ID 34: QueryEA (V1 only)
        #[etw_event(id = 34, version = 1, name = "QueryEAV1", keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO)]
        SetInformationArgs_V1,
    }
}
