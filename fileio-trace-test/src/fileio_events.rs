use fileio_events_macro::fileio_events;

fileio_events! {
    pub enum FileIoEvent {
        // ── FileIo V0 (EventVersion 0) ──────────────────────────────

        NameV0 = (0, 0) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },

        // ── FileIo V1 (EventVersion 1) ──────────────────────────────

        NameV1 = (0, 1) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileCreateV1 = (32, 1) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },

        // ── FileIo V2 (EventVersion 2) ──────────────────────────────

        NameV2 = (0, 2) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileCreateV2 = (32, 2) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileDeleteV2 = (35, 2) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileRundownV2 = (36, 2) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },

        MapFileV2 = (37, 2) {
            ViewBase: pointer "ViewBase",
            FileObject: pointer "FileObject",
            MiscInfo: u64 "MiscInfo",
            ViewSize: u64 "ViewSize",
            ProcessId: u32 "ProcessId",
        },
        UnmapFileV2 = (38, 2) {
            ViewBase: pointer "ViewBase",
            FileObject: pointer "FileObject",
            MiscInfo: u64 "MiscInfo",
            ViewSize: u64 "ViewSize",
            ProcessId: u32 "ProcessId",
        },
        MapFileDCStartV2 = (39, 2) {
            ViewBase: pointer "ViewBase",
            FileObject: pointer "FileObject",
            MiscInfo: u64 "MiscInfo",
            ViewSize: u64 "ViewSize",
            ProcessId: u32 "ProcessId",
        },
        MapFileDCEndV2 = (40, 2) {
            ViewBase: pointer "ViewBase",
            FileObject: pointer "FileObject",
            MiscInfo: u64 "MiscInfo",
            ViewSize: u64 "ViewSize",
            ProcessId: u32 "ProcessId",
        },

        CreateV2 = (64, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            CreateOptions: u32 "CreateOptions",
            FileAttributes: u32 "FileAttributes",
            ShareAccess: u32 "ShareAccess",
            OpenPath: string "OpenPath",
        },

        CleanupV2 = (65, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
        },
        CloseV2 = (66, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
        },
        FlushV2 = (73, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
        },

        ReadV2 = (67, 2) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },
        WriteV2 = (68, 2) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },

        SetInfoV2 = (69, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            InfoClass: u32 "InfoClass",
        },
        DeleteV2 = (70, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            InfoClass: u32 "InfoClass",
        },
        RenameV2 = (71, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            InfoClass: u32 "InfoClass",
        },
        QueryInfoV2 = (74, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            InfoClass: u32 "InfoClass",
        },
        FSControlV2 = (75, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            InfoClass: u32 "InfoClass",
        },

        DirEnumV2 = (72, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },
        DirNotifyV2 = (77, 2) {
            IrpPtr: pointer "IrpPtr",
            TTID: pointer "TTID",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },

        OperationEndV2 = (76, 2) {
            IrpPtr: pointer "IrpPtr",
            ExtraInfo: pointer "ExtraInfo",
            NtStatus: u32 "NtStatus",
        },

        // ── FileIo V3 (EventVersion 3) ──────────────────────────────

        NameV3 = (0, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileCreateV3 = (32, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileDeleteV3 = (35, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },
        FileRundownV3 = (36, 3) {
            FileObject: pointer "FileObject",
            FileName: string "FileName",
        },

        CreateV3 = (64, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            TTID: u32 "TTID",
            CreateOptions: u32 "CreateOptions",
            FileAttributes: u32 "FileAttributes",
            ShareAccess: u32 "ShareAccess",
            OpenPath: string "OpenPath",
        },

        CleanupV3 = (65, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },
        CloseV3 = (66, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },
        FlushV3 = (73, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
        },

        ReadV3 = (67, 3) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },
        WriteV3 = (68, 3) {
            Offset: u64 "Offset",
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            IoSize: u32 "IoSize",
            IoFlags: u32 "IoFlags",
        },

        SetInfoV3 = (69, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        DeleteInfoV3 = (70, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        RenameV3 = (71, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        QueryInfoV3 = (74, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },
        FSControlV3 = (75, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
        },

        DirEnumV3 = (72, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },
        DirNotifyV3 = (77, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            TTID: u32 "TTID",
            Length: u32 "Length",
            InfoClass: u32 "InfoClass",
            FileIndex: u32 "FileIndex",
            FileName: string "FileName",
        },

        DeletePathV3 = (79, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },
        RenamePathV3 = (80, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },
        SetLinkPathV3 = (81, 3) {
            IrpPtr: pointer "IrpPtr",
            FileObject: pointer "FileObject",
            FileKey: pointer "FileKey",
            ExtraInfo: pointer "ExtraInfo",
            TTID: u32 "TTID",
            InfoClass: u32 "InfoClass",
            FileName: string "FileName",
        },

        OperationEndV3 = (76, 3) {
            IrpPtr: pointer "IrpPtr",
            ExtraInfo: pointer "ExtraInfo",
            NtStatus: u32 "NtStatus",
        },

        PreOpInitV3 = (96, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpInitV3 = (97, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        PreOpCompletionV3 = (98, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpCompletionV3 = (99, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        PreOpFailureV3 = (100, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
            Status: u32 "Status",
        },
        PostOpFailureV3 = (101, 3) {
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
