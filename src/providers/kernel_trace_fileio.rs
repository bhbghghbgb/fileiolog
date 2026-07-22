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

        // ── Template: FileIo_Name ──────────────────────────────
        template FileIo_Name {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ── Template: FileIo_Create ────────────────────────────
        template FileIo_Create {
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

        // ── Template: FileIo_SimpleOp ──────────────────────────
        template FileIo_SimpleOp {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // ── Template: FileIo_ReadWrite ─────────────────────────
        template FileIo_ReadWrite {
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

        // ── Template: FileIo_Info ──────────────────────────────
        template FileIo_Info {
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

        // ── Template: FileIo_OpEnd ─────────────────────────────
        template FileIo_OpEnd {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "NtStatus")]
            pub nt_status: u32,
        }

        // ── Template: FileIo_DirEnum ───────────────────────────
        template FileIo_DirEnum {
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

        // ── Events ─────────────────────────────────────────────

        // Event type 0: Name (FileIo_Name)
        #[etw_event(id = 0, version = 0, name = "NameV0", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,
        #[etw_event(id = 0, version = 1, name = "NameV1", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,
        #[etw_event(id = 0, version = 2, name = "NameV2", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,

        // Event type 32: FileCreate (FileIo_Name)
        #[etw_event(id = 32, version = 1, name = "FileCreateV1", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,
        #[etw_event(id = 32, version = 2, name = "FileCreateV2", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,

        // Event type 35: FileDelete (FileIo_Name)
        #[etw_event(id = 35, version = 2, name = "FileDeleteV2", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,

        // Event type 36: FileRundown (FileIo_Name)
        #[etw_event(id = 36, version = 2, name = "FileRundownV2", enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        FileIo_Name,

        // Event type 64: Create (FileIo_Create)
        #[etw_event(id = 64, version = 2, name = "CreateV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Create,

        // Event type 65: Cleanup (FileIo_SimpleOp)
        #[etw_event(id = 65, version = 2, name = "CleanupV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_SimpleOp,

        // Event type 66: Close (FileIo_SimpleOp)
        #[etw_event(id = 66, version = 2, name = "CloseV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_SimpleOp,

        // Event type 67: Read (FileIo_ReadWrite)
        #[etw_event(id = 67, version = 2, name = "ReadV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_ReadWrite,

        // Event type 68: Write (FileIo_ReadWrite)
        #[etw_event(id = 68, version = 2, name = "WriteV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_ReadWrite,

        // Event type 69: SetInfo (FileIo_Info)
        #[etw_event(id = 69, version = 2, name = "SetInfoV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Info,

        // Event type 70: Delete (FileIo_Info)
        #[etw_event(id = 70, version = 2, name = "DeleteV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Info,

        // Event type 71: Rename (FileIo_Info)
        #[etw_event(id = 71, version = 2, name = "RenameV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Info,

        // Event type 72: DirEnum (FileIo_DirEnum)
        #[etw_event(id = 72, version = 2, name = "DirEnumV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_DirEnum,

        // Event type 73: Flush (FileIo_SimpleOp)
        #[etw_event(id = 73, version = 2, name = "FlushV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_SimpleOp,

        // Event type 74: QueryInfo (FileIo_Info)
        #[etw_event(id = 74, version = 2, name = "QueryInfoV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Info,

        // Event type 75: FSControl (FileIo_Info)
        #[etw_event(id = 75, version = 2, name = "FSControlV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_Info,

        // Event type 76: OperationEnd (FileIo_OpEnd)
        #[etw_event(id = 76, version = 2, name = "OperationEndV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO)]
        FileIo_OpEnd,

        // Event type 77: DirNotify (FileIo_DirEnum)
        #[etw_event(id = 77, version = 2, name = "DirNotifyV2", enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        FileIo_DirEnum,
    }
}
