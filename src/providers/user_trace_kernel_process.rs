#![allow(dead_code)]

use crate::etw::etw_provider;

pub mod masks {
    pub const WINEVENT_KEYWORD_PROCESS: u64 = 0x10;
    pub const WINEVENT_KEYWORD_THREAD: u64 = 0x20;
    pub const WINEVENT_KEYWORD_IMAGE: u64 = 0x40;
    pub const WINEVENT_KEYWORD_CPU_PRIORITY: u64 = 0x80;
    pub const WINEVENT_KEYWORD_OTHER_PRIORITY: u64 = 0x100;
    pub const WINEVENT_KEYWORD_PROCESS_FREEZE: u64 = 0x200;
    pub const WINEVENT_KEYWORD_JOB: u64 = 0x400;
    pub const WINEVENT_KEYWORD_ENABLE_PROCESS_TRACING_CALLBACKS: u64 = 0x800;
    pub const WINEVENT_KEYWORD_JOB_IO: u64 = 0x1000;
    pub const WINEVENT_KEYWORD_WORK_ON_BEHALF: u64 = 0x2000;
    pub const WINEVENT_KEYWORD_JOB_SILO: u64 = 0x4000;
}

etw_provider! {
    #[etw_provider(kind = "user", name = "Microsoft-Windows-Kernel-Process", guid = "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")]
    pub enum UserTraceKernelProcessEvent {
        // ── ProcessStartArgsV0 (v=0, tid=ProcessStartArgs) ─────
        #[etw_event(name = "ProcessStartV0", id = 1, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStartArgsV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── ProcessStartArgsV1 (v=1, tid=ProcessStartArgs_V1) ──
        #[etw_event(name = "ProcessStartV1", id = 1, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStartArgsV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "Flags")]
            pub flags: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── ProcessStartArgsV2 (v=2, tid=ProcessStartArgs_V2) ──
        #[etw_event(name = "ProcessStartV2", id = 1, version = 2, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStartArgsV2 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "Flags")]
            pub flags: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
            #[etw_prop(name = "ImageChecksum")]
            pub image_checksum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "PackageFullName")]
            pub package_full_name: String,
            #[etw_prop(name = "PackageRelativeAppId")]
            pub package_relative_app_id: String,
        }

        // ── ProcessRundownArgsV1 (tid=ProcessRundownArgs_V1) ───
        // Shared by ProcessStartV3 (id=1, v=3) and ProcessRundownV1 (id=15, v=1)
        #[etw_event(name = "ProcessStartV3", id = 1, version = 3, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        #[etw_event(name = "ProcessRundownV1", id = 15, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessRundownArgsV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ProcessSequenceNumber")]
            pub process_sequence_number: u64,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "ParentProcessSequenceNumber")]
            pub parent_process_sequence_number: u64,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "Flags")]
            pub flags: u32,
            #[etw_prop(name = "ProcessTokenElevationType")]
            pub process_token_elevation_type: u32,
            #[etw_prop(name = "ProcessTokenIsElevated")]
            pub process_token_is_elevated: u32,
            #[etw_prop(name = "MandatoryLabel")]
            pub mandatory_label: String,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
            #[etw_prop(name = "ImageChecksum")]
            pub image_checksum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "PackageFullName")]
            pub package_full_name: String,
            #[etw_prop(name = "PackageRelativeAppId")]
            pub package_relative_app_id: String,
        }

        // ── ProcessStopArgsV0 (v=0, tid=ProcessStopArgs) ───────
        #[etw_event(name = "ProcessStopV0", id = 2, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStopArgsV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::native::time::FileTime)]
            pub exit_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitCode")]
            pub exit_code: u32,
            #[etw_prop(name = "TokenElevationType")]
            pub token_elevation_type: u32,
            #[etw_prop(name = "HandleCount")]
            pub handle_count: u32,
            #[etw_prop(name = "CommitCharge")]
            pub commit_charge: u64,
            #[etw_prop(name = "CommitPeak")]
            pub commit_peak: u64,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── ProcessStopArgsV1 (v=1, tid=ProcessStopArgs_V1) ────
        #[etw_event(name = "ProcessStopV1", id = 2, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStopArgsV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::native::time::FileTime)]
            pub exit_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitCode")]
            pub exit_code: u32,
            #[etw_prop(name = "TokenElevationType")]
            pub token_elevation_type: u32,
            #[etw_prop(name = "HandleCount")]
            pub handle_count: u32,
            #[etw_prop(name = "CommitCharge")]
            pub commit_charge: u64,
            #[etw_prop(name = "CommitPeak")]
            pub commit_peak: u64,
            #[etw_prop(name = "CPUCycleCount")]
            pub cpu_cycle_count: u64,
            #[etw_prop(name = "ReadOperationCount")]
            pub read_operation_count: u32,
            #[etw_prop(name = "WriteOperationCount")]
            pub write_operation_count: u32,
            #[etw_prop(name = "ReadTransferKiloBytes")]
            pub read_transfer_kilo_bytes: u32,
            #[etw_prop(name = "WriteTransferKiloBytes")]
            pub write_transfer_kilo_bytes: u32,
            #[etw_prop(name = "HardFaultCount")]
            pub hard_fault_count: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── ProcessStopArgsV2 (v=2, tid=ProcessStopArgs_V2) ────
        #[etw_event(name = "ProcessStopV2", id = 2, version = 2, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessStopArgsV2 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ProcessSequenceNumber")]
            pub process_sequence_number: u64,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::native::time::FileTime)]
            pub exit_time: time::OffsetDateTime,
            #[etw_prop(name = "ExitCode")]
            pub exit_code: u32,
            #[etw_prop(name = "TokenElevationType")]
            pub token_elevation_type: u32,
            #[etw_prop(name = "HandleCount")]
            pub handle_count: u32,
            #[etw_prop(name = "CommitCharge")]
            pub commit_charge: u64,
            #[etw_prop(name = "CommitPeak")]
            pub commit_peak: u64,
            #[etw_prop(name = "CPUCycleCount")]
            pub cpu_cycle_count: u64,
            #[etw_prop(name = "ReadOperationCount")]
            pub read_operation_count: u32,
            #[etw_prop(name = "WriteOperationCount")]
            pub write_operation_count: u32,
            #[etw_prop(name = "ReadTransferKiloBytes")]
            pub read_transfer_kilo_bytes: u32,
            #[etw_prop(name = "WriteTransferKiloBytes")]
            pub write_transfer_kilo_bytes: u32,
            #[etw_prop(name = "HardFaultCount")]
            pub hard_fault_count: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── ThreadStartArgsV0 (v=0, tid=ThreadStartArgs) ───────
        // Shared by ThreadStart (id=3) and ThreadStop (id=4) at v=0
        #[etw_event(name = "ThreadStartV0", id = 3, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        #[etw_event(name = "ThreadStopV0", id = 4, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        pub struct ThreadStartArgsV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "StackBase", parse_as = ferrisetw::parser::Pointer)]
            pub stack_base: usize,
            #[etw_prop(name = "StackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub stack_limit: usize,
            #[etw_prop(name = "UserStackBase", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_base: usize,
            #[etw_prop(name = "UserStackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_limit: usize,
            #[etw_prop(name = "StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub start_addr: usize,
            #[etw_prop(name = "Win32StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub win32_start_addr: usize,
            #[etw_prop(name = "TebBase", parse_as = ferrisetw::parser::Pointer)]
            pub teb_base: usize,
        }

        // ── ThreadStartArgsV1 (v=1, tid=ThreadStartArgs_V1) ────
        #[etw_event(name = "ThreadStartV1", id = 3, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        pub struct ThreadStartArgsV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "StackBase", parse_as = ferrisetw::parser::Pointer)]
            pub stack_base: usize,
            #[etw_prop(name = "StackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub stack_limit: usize,
            #[etw_prop(name = "UserStackBase", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_base: usize,
            #[etw_prop(name = "UserStackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_limit: usize,
            #[etw_prop(name = "StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub start_addr: usize,
            #[etw_prop(name = "Win32StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub win32_start_addr: usize,
            #[etw_prop(name = "TebBase", parse_as = ferrisetw::parser::Pointer)]
            pub teb_base: usize,
            #[etw_prop(name = "SubProcessTag")]
            pub sub_process_tag: u32,
        }

        // ── ThreadStopArgsV1 (v=1, tid=ThreadStopArgs_V1) ──────
        #[etw_event(name = "ThreadStopV1", id = 4, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        pub struct ThreadStopArgsV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "StackBase", parse_as = ferrisetw::parser::Pointer)]
            pub stack_base: usize,
            #[etw_prop(name = "StackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub stack_limit: usize,
            #[etw_prop(name = "UserStackBase", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_base: usize,
            #[etw_prop(name = "UserStackLimit", parse_as = ferrisetw::parser::Pointer)]
            pub user_stack_limit: usize,
            #[etw_prop(name = "StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub start_addr: usize,
            #[etw_prop(name = "Win32StartAddr", parse_as = ferrisetw::parser::Pointer)]
            pub win32_start_addr: usize,
            #[etw_prop(name = "TebBase", parse_as = ferrisetw::parser::Pointer)]
            pub teb_base: usize,
            #[etw_prop(name = "SubProcessTag")]
            pub sub_process_tag: u32,
            #[etw_prop(name = "CycleTime")]
            pub cycle_time: u64,
        }

        // ── ImageLoadArgs (v=0, tid=ImageLoadArgs) ─────────────
        // Shared by ImageLoad (id=5) and ImageUnload (id=6)
        #[etw_event(name = "ImageLoadV0", id = 5, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_IMAGE)]
        #[etw_event(name = "ImageUnloadV0", id = 6, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_IMAGE)]
        pub struct ImageLoadArgs {
            #[etw_prop(name = "ImageBase", parse_as = ferrisetw::parser::Pointer)]
            pub image_base: usize,
            #[etw_prop(name = "ImageSize", parse_as = ferrisetw::parser::Pointer)]
            pub image_size: usize,
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ImageCheckSum")]
            pub image_checksum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "DefaultBase", parse_as = ferrisetw::parser::Pointer)]
            pub default_base: usize,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── CpuBasePriorityChangeArgs (v=0, tid=CpuBasePriorityChangeArgs)
        // Shared by CpuBasePriorityChange (id=7), CpuPriorityChange (id=8),
        // PagePriorityChange (id=9), IoPriorityChange (id=10)
        #[etw_event(name = "CpuBasePriorityChangeV0", id = 7, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_CPU_PRIORITY)]
        #[etw_event(name = "CpuPriorityChangeV0", id = 8, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_CPU_PRIORITY)]
        #[etw_event(name = "PagePriorityChangeV0", id = 9, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_OTHER_PRIORITY)]
        #[etw_event(name = "IoPriorityChangeV0", id = 10, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_OTHER_PRIORITY)]
        pub struct CpuBasePriorityChangeArgs {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── ProcessFreezeStartArgsV0 (v=0, tid=ProcessFreezeStartArgs)
        // Shared by ProcessFreezeStart (id=11) and ProcessFreezeStop (id=12)
        #[etw_event(name = "ProcessFreezeStartV0", id = 11, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        #[etw_event(name = "ProcessFreezeStopV0", id = 12, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        pub struct ProcessFreezeStartArgsV0 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        // ── ProcessFreezeStartArgsV1 (v=1, tid=ProcessFreezeStartArgs_V1)
        // Shared by ProcessFreezeStart_V1 (id=11) and ProcessFreezeStop_V1 (id=12)
        #[etw_event(name = "ProcessFreezeStartV1", id = 11, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        #[etw_event(name = "ProcessFreezeStopV1", id = 12, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        pub struct ProcessFreezeStartArgsV1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
        }

        // ── JobStartArgs (v=0, tid=JobStartArgs) ───────────────
        // Shared by JobStart (id=13) and JobTerminateStop (id=14)
        #[etw_event(name = "JobStartV0", id = 13, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB)]
        #[etw_event(name = "JobTerminateStopV0", id = 14, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB)]
        pub struct JobStartArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── ProcessRundownArgsV0 (v=0, tid=ProcessRundownArgs) ─
        #[etw_event(name = "ProcessRundownV0", id = 15, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        pub struct ProcessRundownArgsV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "Flags")]
            pub flags: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
            #[etw_prop(name = "ImageChecksum")]
            pub image_checksum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "PackageFullName")]
            pub package_full_name: String,
            #[etw_prop(name = "PackageRelativeAppId")]
            pub package_relative_app_id: String,
        }

        // ── Event ID 16: task_0 (v=0, no template) ─────────────
        #[etw_event(name = "EnableProcessTracingCallbacksV0", id = 16, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_ENABLE_PROCESS_TRACING_CALLBACKS)]
        pub struct EnableProcessTracingCallbacksV0 {}

        // ── PsDiskIoAttributionStartArgs (v=0, tid=PsDiskIoAttributionStartArgs)
        // Shared by PsDiskIoAttributionStart (id=17) and PsDiskIoAttributionStop (id=18)
        #[etw_event(name = "PsDiskIoAttributionStartV0", id = 17, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        #[etw_event(name = "PsDiskIoAttributionStopV0", id = 18, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        pub struct PsDiskIoAttributionStartArgs {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── PsIoRateControlStartArgsV0 (v=0, tid=PsIoRateControlStartArgs)
        // Shared by PsIoRateControlStart (id=19) and PsIoRateControlStop (id=20)
        #[etw_event(name = "PsIoRateControlStartV0", id = 19, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        #[etw_event(name = "PsIoRateControlStopV0", id = 20, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        pub struct PsIoRateControlStartArgsV0 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "IoRateControl", parse_as = ferrisetw::parser::Pointer)]
            pub io_rate_control: usize,
            #[etw_prop(name = "ControlType")]
            pub control_type: u32,
            #[etw_prop(name = "RateType")]
            pub rate_type: u32,
            #[etw_prop(name = "RateAmount")]
            pub rate_amount: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── PsIoRateControlStartArgsV1 (v=1, tid=PsIoRateControlStartArgs_V1)
        // Shared by PsIoRateControlStart_V1 (id=19) and PsIoRateControlStop_V1 (id=20)
        #[etw_event(name = "PsIoRateControlStartV1", id = 19, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        #[etw_event(name = "PsIoRateControlStopV1", id = 20, version = 1, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        pub struct PsIoRateControlStartArgsV1 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "IoRateControl", parse_as = ferrisetw::parser::Pointer)]
            pub io_rate_control: usize,
            #[etw_prop(name = "MaxIops")]
            pub max_iops: u64,
            #[etw_prop(name = "MaxBandwidth")]
            pub max_bandwidth: u64,
            #[etw_prop(name = "MaxTimePercent")]
            pub max_time_percent: u64,
            #[etw_prop(name = "ReservationIops")]
            pub reservation_iops: u64,
            #[etw_prop(name = "ReservationBandwidth")]
            pub reservation_bandwidth: u64,
            #[etw_prop(name = "ReservationTimePercent")]
            pub reservation_time_percent: u64,
            #[etw_prop(name = "CriticalReservationIops")]
            pub critical_reservation_iops: u64,
            #[etw_prop(name = "CriticalReservationBandwidth")]
            pub critical_reservation_bandwidth: u64,
            #[etw_prop(name = "CriticalReservationTimePercent")]
            pub critical_reservation_time_percent: u64,
            #[etw_prop(name = "ControlFlags")]
            pub control_flags: u32,
            #[etw_prop(name = "VolumeName")]
            pub volume_name: String,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── PsIoRateControlStartArgsV2 (v=2, tid=PsIoRateControlStartArgs_V2)
        // Shared by PsIoRateControlStart_V2 (id=19) and PsIoRateControlStop_V2 (id=20)
        #[etw_event(name = "PsIoRateControlStartV2", id = 19, version = 2, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        #[etw_event(name = "PsIoRateControlStopV2", id = 20, version = 2, keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        pub struct PsIoRateControlStartArgsV2 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "IoRateControl", parse_as = ferrisetw::parser::Pointer)]
            pub io_rate_control: usize,
            #[etw_prop(name = "MaxIops")]
            pub max_iops: u64,
            #[etw_prop(name = "MaxBandwidth")]
            pub max_bandwidth: u64,
            #[etw_prop(name = "MaxTimePercent")]
            pub max_time_percent: u64,
            #[etw_prop(name = "ReservationIops")]
            pub reservation_iops: u64,
            #[etw_prop(name = "ReservationBandwidth")]
            pub reservation_bandwidth: u64,
            #[etw_prop(name = "ReservationTimePercent")]
            pub reservation_time_percent: u64,
            #[etw_prop(name = "CriticalReservationIops")]
            pub critical_reservation_iops: u64,
            #[etw_prop(name = "CriticalReservationBandwidth")]
            pub critical_reservation_bandwidth: u64,
            #[etw_prop(name = "CriticalReservationTimePercent")]
            pub critical_reservation_time_percent: u64,
            #[etw_prop(name = "SoftMaxIops")]
            pub soft_max_iops: u64,
            #[etw_prop(name = "SoftMaxBandwidth")]
            pub soft_max_bandwidth: u64,
            #[etw_prop(name = "SoftMaxTimePercent")]
            pub soft_max_time_percent: u64,
            #[etw_prop(name = "ControlFlags")]
            pub control_flags: u32,
            #[etw_prop(name = "VolumeName")]
            pub volume_name: String,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── ThreadWorkOnBehalfUpdateArgs (v=0, tid=ThreadWorkOnBehalfUpdateArgs)
        #[etw_event(name = "ThreadWorkOnBehalfUpdateV0", id = 21, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_WORK_ON_BEHALF)]
        pub struct ThreadWorkOnBehalfUpdateArgs {
            #[etw_prop(name = "OldWorkOnBehalfThreadID")]
            pub old_work_on_behalf_thread_id: u32,
            #[etw_prop(name = "NewWorkOnBehalfThreadID")]
            pub new_work_on_behalf_thread_id: u32,
        }

        // ── JobServerSiloStartArgs (v=0, tid=JobServerSiloStartArgs)
        #[etw_event(name = "JobServerSiloStartV0", id = 22, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        pub struct JobServerSiloStartArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "State")]
            pub state: u16,
        }

        // ── JobServerSiloStart23Args (v=0, tid=JobServerSiloStart23Args)
        // Shared by JobServerSiloStart23 (id=23), JobServerSiloStart25 (id=25), JobServerSiloStartStop26 (id=26)
        #[etw_event(name = "JobServerSiloStart23V0", id = 23, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        #[etw_event(name = "JobServerSiloStart25V0", id = 25, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        #[etw_event(name = "JobServerSiloStartStop26V0", id = 26, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        pub struct JobServerSiloStart23Args {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── JobServerSiloStartStopArgs (v=0, tid=JobServerSiloStartStopArgs)
        #[etw_event(name = "JobServerSiloStartStopV0", id = 24, version = 0, keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        pub struct JobServerSiloStartStopArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "Status")]
            pub status: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }
    }
}
