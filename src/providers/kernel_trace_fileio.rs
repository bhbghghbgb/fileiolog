#![allow(dead_code)]

use crate::etw::etw_provider;

pub mod flags {
    pub const EVENT_TRACE_FLAG_DISK_FILE_IO: u32 = 0x00000200;
    pub const EVENT_TRACE_FLAG_FILE_IO: u32 = 0x02000000;
    pub const EVENT_TRACE_FLAG_FILE_IO_INIT: u32 = 0x04000000;
}

etw_provider! {
    #[etw_provider(
        kind = "kernel",
        guid = "90cbdc39-4a3e-11d1-84f4-0000f80464e3"
    )]
    pub enum KernelTraceFileIoEvent {
        // ── FileIo_Name MOF class ────────────────────────────
        // EventType {0, 32, 35, 36} — Name, FileCreate, FileDelete, FileRundown
        // All V0 field layout: FileObject + FileName
        #[etw_event(name = "NameV0", id = 0, version = 0, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "NameV1", id = 0, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "NameV2", id = 0, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV1", id = 32, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV2", id = 32, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileDeleteV2", id = 35, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileRundownV2", id = 36, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV0 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_Create MOF class ──────────────────────────
        // EventType {64} — Create
        #[etw_event(name = "CreateV2", id = 64, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoCreateV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "FileAttributes")]
            pub file_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "OpenPath")]
            pub open_path: String,
        }

        // ── FileIo_SimpleOp MOF class ────────────────────────
        // EventType {65, 66, 73} — Cleanup, Close, Flush
        #[etw_event(name = "CleanupV2", id = 65, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "CloseV2", id = 66, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "FlushV2", id = 73, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoSimpleOpV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // ── FileIo_ReadWrite MOF class ───────────────────────
        // EventType {67, 68} — Read, Write
        #[etw_event(name = "ReadV2", id = 67, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "WriteV2", id = 68, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoReadWriteV2 {
            #[etw_prop(name = "Offset")]
            pub offset: u64,
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "IoSize")]
            pub io_size: u32,
            #[etw_prop(name = "IoFlags")]
            pub io_flags: u32,
        }

        // ── FileIo_Info MOF class ────────────────────────────
        // EventType {69, 70, 71, 74, 75} — SetInfo, Delete, Rename, QueryInfo, FSControl
        #[etw_event(name = "SetInfoV2", id = 69, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "DeleteV2", id = 70, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "RenameV2", id = 71, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "QueryInfoV2", id = 74, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "FSControlV2", id = 75, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoInfoV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── FileIo_DirEnum MOF class ─────────────────────────
        // EventType {72, 77} — DirEnum, DirNotify
        #[etw_event(name = "DirEnumV2", id = 72, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "DirNotifyV2", id = 77, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoDirEnumV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
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

        // ── FileIo_OpEnd MOF class ───────────────────────────
        // EventType {76} — OperationEnd
        #[etw_event(name = "OperationEndV2", id = 76, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO)]
        pub struct FileIoOpEndV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "NtStatus")]
            pub nt_status: u32,
        }
    }
}
