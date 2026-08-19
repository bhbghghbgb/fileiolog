#![allow(dead_code)]

use crate::etw::etw_provider;

pub mod flags {
    pub const EVENT_TRACE_FLAG_DISK_FILE_IO: u32 = 0x00000200;
    pub const EVENT_TRACE_FLAG_FILE_IO: u32 = 0x02000000;
    pub const EVENT_TRACE_FLAG_FILE_IO_INIT: u32 = 0x04000000;
    pub const EVENT_TRACE_FLAG_VAMAP: u32 = 0x00008000;

    // Extended PERFINFO_GROUPMASK bits (group 4) for minifilter events.
    // These cannot be set via EnableFlags alone; use TraceSetInformation
    // with PERFINFO_GROUPMASK to activate them.
    pub const PERF_FLT_IO_INIT: u32 = 0x80080000;
    pub const PERF_FLT_IO: u32 = 0x80100000;
    pub const PERF_FLT_IO_FAILURE: u32 = 0x80400000;
}

etw_provider! {
    #[etw_provider(
        kind = "kernel",
        guid = "90cbdc39-4a3e-11d1-84f4-0000f80464e3"
    )]
    pub enum KernelTraceFileIoEvent {
        // ── FileIo_Name V0 ────────────────────────────────────
        // Class: FileIo_V0_Name (EventVersion(0), EventType(0))
        #[etw_event(name = "NameV0", id = 0, version = 0, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV0 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_Name V1 ────────────────────────────────────
        // Class: FileIo_V1_Name (EventVersion(1), EventType{0, 32})
        #[etw_event(name = "NameV1", id = 0, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV1", id = 32, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV1 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_Name V2 ────────────────────────────────────
        // Class: FileIo_Name (EventVersion(2), EventType{0, 32, 35, 36})
        #[etw_event(name = "NameV2", id = 0, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV2", id = 32, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileDeleteV2", id = 35, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileRundownV2", id = 36, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV2 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_MapFile V2 ─────────────────────────────────
        // Class: FileIo_V2_MapFile (EventVersion(2), EventType{37, 38, 39, 40})
        #[etw_event(name = "MapFileV2", id = 37, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_VAMAP)]
        #[etw_event(name = "UnmapFileV2", id = 38, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_VAMAP)]
        #[etw_event(name = "MapFileDCStartV2", id = 39, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_VAMAP)]
        #[etw_event(name = "MapFileDCEndV2", id = 40, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_VAMAP)]
        pub struct FileIoMapFileV2 {
            #[etw_prop(name = "ViewBase", parse_as = ferrisetw::parser::Pointer)]
            pub view_base: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "MiscInfo")]
            pub misc_info: u64,
            #[etw_prop(name = "ViewSize", parse_as = ferrisetw::parser::Pointer)]
            pub view_size: usize,
            #[etw_prop(name = "ProcessId")]
            pub process_id: u32,
        }

        // ── FileIo_Create V2 ─────────────────────────────────
        // Class: FileIo_Create (EventVersion(2), EventType{64})
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

        // ── FileIo_SimpleOp V2 ───────────────────────────────
        // Class: FileIo_SimpleOp (EventVersion(2), EventType{65, 66, 73})
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

        // ── FileIo_ReadWrite V2 ──────────────────────────────
        // Class: FileIo_ReadWrite (EventVersion(2), EventType{67, 68})
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

        // ── FileIo_Info V2 ───────────────────────────────────
        // Class: FileIo_Info (EventVersion(2), EventType{69, 70, 71, 74, 75})
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

        // ── FileIo_DirEnum V2 ────────────────────────────────
        // Class: FileIo_DirEnum (EventVersion(2), EventType{72, 77})
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

        // ── FileIo_OpEnd V2 ──────────────────────────────────
        // Class: FileIo_OpEnd (EventVersion(2), EventType{76})
        #[etw_event(name = "OperationEndV2", id = 76, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO)]
        pub struct FileIoOpEndV2 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "NtStatus")]
            pub nt_status: u32,
        }

        // ══════════════════════════════════════════════════════
        // FileIo V3 (EventVersion 3) — newer Windows versions
        // ══════════════════════════════════════════════════════

        // ── FileIo_Name V3 ────────────────────────────────────
        // Class: FileIo_Name (EventVersion(3), EventType{0, 32, 35, 36})
        #[etw_event(name = "NameV3", id = 0, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV3", id = 32, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileDeleteV3", id = 35, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileRundownV3", id = 36, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV3 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_Create V3 ──────────────────────────────────
        // Class: FileIo_Create (EventVersion(3), EventType{64})
        // V3: TTID is a plain u32 (not a pointer), field order changed
        #[etw_event(name = "CreateV3", id = 64, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoCreateV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
            #[etw_prop(name = "CreateOptions")]
            pub create_options: u32,
            #[etw_prop(name = "FileAttributes")]
            pub file_attributes: u32,
            #[etw_prop(name = "ShareAccess")]
            pub share_access: u32,
            #[etw_prop(name = "OpenPath")]
            pub open_path: String,
        }

        // ── FileIo_SimpleOp V3 ────────────────────────────────
        // Class: FileIo_SimpleOp (EventVersion(3), EventType{65, 66, 73})
        // V3: TTID moved after FileKey, changed to plain u32
        #[etw_event(name = "CleanupV3", id = 65, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "CloseV3", id = 66, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "FlushV3", id = 73, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoSimpleOpV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
        }

        // ── FileIo_ReadWrite V3 ───────────────────────────────
        // Class: FileIo_ReadWrite (EventVersion(3), EventType{67, 68})
        // V3: TTID moved after FileKey, changed to plain u32
        #[etw_event(name = "ReadV3", id = 67, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "WriteV3", id = 68, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoReadWriteV3 {
            #[etw_prop(name = "Offset")]
            pub offset: u64,
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
            #[etw_prop(name = "IoSize")]
            pub io_size: u32,
            #[etw_prop(name = "IoFlags")]
            pub io_flags: u32,
        }

        // ── FileIo_Info V3 ────────────────────────────────────
        // Class: FileIo_Info (EventVersion(3), EventType{69, 70, 71, 74, 75})
        // V3: TTID moved after ExtraInfo, changed to plain u32
        #[etw_event(name = "SetInfoV3", id = 69, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "DeleteInfoV3", id = 70, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "RenameV3", id = 71, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "QueryInfoV3", id = 74, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "FSControlV3", id = 75, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoInfoV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
        }

        // ── FileIo_DirEnum V3 ─────────────────────────────────
        // Class: FileIo_DirEnum (EventVersion(3), EventType{72, 77})
        // V3: TTID moved after FileKey, changed to plain u32
        #[etw_event(name = "DirEnumV3", id = 72, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "DirNotifyV3", id = 77, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoDirEnumV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
            #[etw_prop(name = "Length")]
            pub length: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileIndex")]
            pub file_index: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FileIo_OpEnd V3 ───────────────────────────────────
        // Class: FileIo_OpEnd (EventVersion(3), EventType{76})
        #[etw_event(name = "OperationEndV3", id = 76, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO)]
        pub struct FileIoOpEndV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "NtStatus")]
            pub nt_status: u32,
        }

        // ── FileIo_PathOperation V3 ───────────────────────────
        // Class: FileIo_PathOperation (EventVersion(3), EventType{79, 80, 81})
        // New in V3: path-level delete/rename/setlink with FileName
        #[etw_event(name = "DeletePathV3", id = 79, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "RenamePathV3", id = 80, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        #[etw_event(name = "SetLinkPathV3", id = 81, version = 3, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FileIoPathOperationV3 {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "TTID")]
            pub ttid: u32,
            #[etw_prop(name = "InfoClass")]
            pub info_class: u32,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── FltIoInit V3 ──────────────────────────────────────
        // Class: FltIoInit (EventVersion(3), EventType{96, 97})
        // New in V3: minifilter IO init events
        // Only accessible via PERFINFO_GROUPMASK (no EnableFlags equivalent)
        #[etw_event(name = "PreOpInitV3", id = 96, version = 3, group_mask = flags::PERF_FLT_IO_INIT)]
        #[etw_event(name = "PostOpInitV3", id = 97, version = 3, group_mask = flags::PERF_FLT_IO_INIT)]
        pub struct FltIoInitV3 {
            #[etw_prop(name = "RoutineAddr", parse_as = ferrisetw::parser::Pointer)]
            pub routine_addr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileContext", parse_as = ferrisetw::parser::Pointer)]
            pub file_context: usize,
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "CallbackDataPtr", parse_as = ferrisetw::parser::Pointer)]
            pub callback_data_ptr: usize,
            #[etw_prop(name = "MajorFunction")]
            pub major_function: u32,
        }

        // ── FltIoCompletion V3 ────────────────────────────────
        // Class: FltIoCompletion (EventVersion(3), EventType{98, 99})
        // New in V3: minifilter IO completion events with InitialTime
        // Only accessible via PERFINFO_GROUPMASK (no EnableFlags equivalent)
        #[etw_event(name = "PreOpCompletionV3", id = 98, version = 3, group_mask = flags::PERF_FLT_IO)]
        #[etw_event(name = "PostOpCompletionV3", id = 99, version = 3, group_mask = flags::PERF_FLT_IO)]
        pub struct FltIoCompletionV3 {
            #[etw_prop(name = "InitialTime")]
            pub initial_time: u64,
            #[etw_prop(name = "RoutineAddr", parse_as = ferrisetw::parser::Pointer)]
            pub routine_addr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileContext", parse_as = ferrisetw::parser::Pointer)]
            pub file_context: usize,
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "CallbackDataPtr", parse_as = ferrisetw::parser::Pointer)]
            pub callback_data_ptr: usize,
            #[etw_prop(name = "MajorFunction")]
            pub major_function: u32,
        }

        // ── FltIoFailure V3 ───────────────────────────────────
        // Class: FltIoFailure (EventVersion(3), EventType{100, 101})
        // New in V3: minifilter IO failure events with Status
        // Only accessible via PERFINFO_GROUPMASK (no EnableFlags equivalent)
        #[etw_event(name = "PreOpFailureV3", id = 100, version = 3, group_mask = flags::PERF_FLT_IO_FAILURE)]
        #[etw_event(name = "PostOpFailureV3", id = 101, version = 3, group_mask = flags::PERF_FLT_IO_FAILURE)]
        pub struct FltIoFailureV3 {
            #[etw_prop(name = "RoutineAddr", parse_as = ferrisetw::parser::Pointer)]
            pub routine_addr: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileContext", parse_as = ferrisetw::parser::Pointer)]
            pub file_context: usize,
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "CallbackDataPtr", parse_as = ferrisetw::parser::Pointer)]
            pub callback_data_ptr: usize,
            #[etw_prop(name = "MajorFunction")]
            pub major_function: u32,
            #[etw_prop(name = "Status")]
            pub status: u32,
        }
    }
}
