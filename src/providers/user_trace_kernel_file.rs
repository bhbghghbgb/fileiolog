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
        // ── NameCreateArgs (v=0) ────────────────────────────────
        // XML template: NameCreateArgs
        // Events: NameCreate (id=10), NameDelete (id=11)
        #[etw_event(name = "NameCreateV0", id = 10, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        #[etw_event(name = "NameDeleteV0", id = 11, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameCreateArgs {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── CreateArgsV0 (v=0) ──────────────────────────────────
        // XML template: CreateArgs
        // Events: Create (id=12), CreateNewFile (id=30)
        #[etw_event(name = "CreateV0", id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        #[etw_event(name = "CreateNewFileV0", id = 30, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
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

        // ── CreateArgsV1 (v=1) ──────────────────────────────────
        // XML template: CreateArgs_V1
        // Events: Create_V1 (id=12), CreateNewFile_V1 (id=30)
        #[etw_event(name = "CreateV1", id = 12, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
        #[etw_event(name = "CreateNewFileV1", id = 30, version = 1, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
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

        // ── CleanupArgsV0 (v=0) ─────────────────────────────────
        // XML template: CleanupArgs
        // Events: Cleanup (id=13), Close (id=14), Flush (id=21)
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

        // ── CleanupArgsV1 (v=1) ─────────────────────────────────
        // XML template: CleanupArgs_V1
        // Events: Cleanup_V1 (id=13), Close_V1 (id=14), Flush_V1 (id=21)
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

        // ── ReadArgsV0 (v=0) ────────────────────────────────────
        // XML template: ReadArgs
        // Events: Read (id=15), Write (id=16)
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

        // ── ReadArgsV1 (v=1) ────────────────────────────────────
        // XML template: ReadArgs_V1
        // Events: Read_V1 (id=15), Write_V1 (id=16)
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

        // ── SetInformationArgsV0 (v=0) ──────────────────────────
        // XML template: SetInformationArgs
        // Events: SetInformation (id=17), SetDelete (id=18), Rename (id=19),
        //         QueryInformation (id=22), FSCTL (id=23), Rename29 (id=29)
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

        // ── SetInformationArgsV1 (v=1) ──────────────────────────
        // XML template: SetInformationArgs_V1
        // Events: SetInformation_V1 (id=17), SetDelete_V1 (id=18), Rename_V1 (id=19),
        //         QueryInformation_V1 (id=22), FSCTL_V1 (id=23), Rename29_V1 (id=29),
        //         SetSecurity_V1 (id=31), QuerySecurity_V1 (id=32), SetEA_V1 (id=33), QueryEA_V1 (id=34)
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

        // ── DirEnumArgsV0 (v=0) ────────────────────────────────
        // XML template: DirEnumArgs
        // Events: DirEnum (id=20), DirNotify (id=25)
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

        // ── DirEnumArgsV1 (v=1) ────────────────────────────────
        // XML template: DirEnumArgs_V1
        // Events: DirEnum_V1 (id=20), DirNotify_V1 (id=25)
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

        // ── OperationEndArgs (v=0) ──────────────────────────────
        // XML template: OperationEndArgs
        // Event: OperationEnd (id=24)
        #[etw_event(name = "OperationEndV0", id = 24, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_OP_END)]
        pub struct OperationEndArgs {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
            #[etw_prop(name = "ExtraInformation", parse_as = ferrisetw::parser::Pointer)]
            pub extra_information: usize,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }

        // ── DeletePathArgsV0 (v=0) ──────────────────────────────
        // XML template: DeletePathArgs
        // Events: DeletePath (id=26), RenamePath (id=27), SetLinkPath (id=28)
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

        // ── DeletePathArgsV1 (v=1) ──────────────────────────────
        // XML template: DeletePathArgs_V1
        // Events: DeletePath_V1 (id=26), RenamePath_V1 (id=27), SetLinkPath_V1 (id=28)
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
    }
}
