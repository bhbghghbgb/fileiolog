use fileio_events_macro::fileio_events;

fileio_events! {
    pub enum FileIoEvent {
        // ── FileIo V3 (EventVersion 3) ──────────────────────────────

        // FileIo_Name events (EventType 0, 32, 35, 36)
        Name = (0, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileCreate = (32, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileDelete = (35, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileRundown = (36, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },

        // FileIo_Create (EventType 64)
        Create = (64, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            TTID: u32 "TTID",
            CreateOptions: u32 "CreateOptions",
            FileAttributes: u32 "FileAttributes",
            ShareAccess: u32 "ShareAccess",
            OpenPath: string "OpenPath",
        },

        // FileIo_SimpleOp events (EventType 65, 66, 73)
        Cleanup = (65, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },
        Close = (66, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },
        Flush = (73, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },

        // FileIo_ReadWrite events (EventType 67, 68)
        Read = (67, 3) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },
        Write = (68, 3) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },

        // FileIo_Info events (EventType 69, 70, 71, 74, 75)
        SetInfo = (69, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        DeleteInfo = (70, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        RenameInfo = (71, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        QueryInfo = (74, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        FSControl = (75, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },

        // FileIo_DirEnum events (EventType 72, 77)
        DirEnum = (72, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },
        DirNotify = (77, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },

        // FileIo_PathOperation events (EventType 79, 80, 81)
        DeletePath = (79, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },
        RenamePath = (80, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },
        SetLinkPath = (81, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },

        // FileIo_OpEnd (EventType 76)
        OperationEnd = (76, 3) {
            IrpPtr: pointer "IrpPtr",
            ExtraInfo: pointer "ExtraInfo",
            NtStatus: u32 "NtStatus",
        },

        // FltIoInit events (EventType 96, 97)
        PreOpInit = (96, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpInit = (97, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        // FltIoCompletion events (EventType 98, 99)
        PreOpCompletion = (98, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpCompletion = (99, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        // FltIoFailure events (EventType 100, 101)
        PreOpFailure = (100, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
            Status: u32 "Status",
        },
        PostOpFailure = (101, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
            Status: u32 "Status",
        },
    }
}
