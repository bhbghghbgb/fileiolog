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
        // ================================================================
        //  FileIo_V0  (EventVersion 0)
        //  Only event type 0 - Name
        // ================================================================

        // -- Event type 0: Name (FileIo_V0_Name) v0 -------------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 0, version = 0, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct V0Name {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ================================================================
        //  FileIo_V1  (EventVersion 1)
        //  Event types 0 (Name) and 32 (FileCreate)
        // ================================================================

        // -- Event type 0: Name (FileIo_V1_Name) v1 -------------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 0, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct V1Name {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // -- Event type 32: FileCreate (FileIo_V1_Name) v1 -------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 32, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct V1FileCreate {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // ================================================================
        //  FileIo_V2  (EventVersion 2)
        //  All event types - the current / latest version
        // ================================================================

        // -- Event type 0: Name (FileIo_Name) v2 ----------------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 0, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct Name {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // -- Event type 32: FileCreate (FileIo_Name) v2 ----------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 32, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileCreate {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // -- Event type 35: FileDelete (FileIo_Name) v2 ----------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 35, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileDelete {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // -- Event type 36: FileRundown (FileIo_Name) v2 ---------------
        // EVENT_TRACE_FLAG_DISK_FILE_IO
        #[etw_event(id = 36, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileRundown {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // -- Event type 64: Create (FileIo_Create) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 64, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Create {
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

        // -- Event type 65: Cleanup (FileIo_SimpleOp) v2 ---------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 65, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Cleanup {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // -- Event type 66: Close (FileIo_SimpleOp) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 66, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Close {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // -- Event type 67: Read (FileIo_ReadWrite) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 67, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Read {
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

        // -- Event type 68: Write (FileIo_ReadWrite) v2 ----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 68, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Write {
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

        // -- Event type 69: SetInfo (FileIo_Info) v2 -------------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 69, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct SetInfo {
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

        // -- Event type 70: Delete (FileIo_Info) v2 --------------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 70, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Delete {
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

        // -- Event type 71: Rename (FileIo_Info) v2 --------------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 71, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Rename {
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

        // -- Event type 72: DirEnum (FileIo_DirEnum) v2 ----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 72, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct DirEnum {
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

        // -- Event type 73: Flush (FileIo_SimpleOp) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 73, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct Flush {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "TTID", parse_as = ferrisetw::parser::Pointer)]
            pub ttid: usize,
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
        }

        // -- Event type 74: QueryInfo (FileIo_Info) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 74, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct QueryInfo {
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

        // -- Event type 75: FSControl (FileIo_Info) v2 -----------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 75, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct FSControl {
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

        // -- Event type 76: OperationEnd (FileIo_OpEnd) v2 --------------
        // EVENT_TRACE_FLAG_FILE_IO
        #[etw_event(id = 76, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO)]
        pub struct OperationEnd {
            #[etw_prop(name = "IrpPtr", parse_as = ferrisetw::parser::Pointer)]
            pub irp_ptr: usize,
            #[etw_prop(name = "ExtraInfo", parse_as = ferrisetw::parser::Pointer)]
            pub extra_info: usize,
            #[etw_prop(name = "NtStatus")]
            pub nt_status: u32,
        }

        // -- Event type 77: DirNotify (FileIo_DirEnum) v2 --------------
        // EVENT_TRACE_FLAG_FILE_IO_INIT
        #[etw_event(id = 77, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_FILE_IO_INIT)]
        pub struct DirNotify {
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
    }
}
