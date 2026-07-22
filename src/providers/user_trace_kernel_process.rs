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

        // ── Template: ProcessStartArgs ─────────────────────────
        template ProcessStartArgs {
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

        // ── Template: ProcessStartArgs_V1 ──────────────────────
        template ProcessStartArgs_V1 {
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

        // ── Template: ProcessStartArgs_V2 ──────────────────────
        template ProcessStartArgs_V2 {
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

        // ── Template: ProcessRundownArgs_V1 ────────────────────
        template ProcessRundownArgs_V1 {
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

        // ── Template: ProcessStopArgs ──────────────────────────
        template ProcessStopArgs {
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

        // ── Template: ProcessStopArgs_V1 ───────────────────────
        template ProcessStopArgs_V1 {
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

        // ── Template: ProcessStopArgs_V2 ───────────────────────
        template ProcessStopArgs_V2 {
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

        // ── Template: ThreadStartArgs ──────────────────────────
        template ThreadStartArgs {
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

        // ── Template: ThreadStartArgs_V1 ───────────────────────
        template ThreadStartArgs_V1 {
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

        // ── Template: ThreadStopArgs_V1 ────────────────────────
        template ThreadStopArgs_V1 {
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

        // ── Template: ImageLoadArgs ────────────────────────────
        template ImageLoadArgs {
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

        // ── Template: CpuBasePriorityChangeArgs ────────────────
        template CpuBasePriorityChangeArgs {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── Template: ProcessFreezeStartArgs ───────────────────
        template ProcessFreezeStartArgs {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        // ── Template: ProcessFreezeStartArgs_V1 ────────────────
        template ProcessFreezeStartArgs_V1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
        }

        // ── Template: JobStartArgs ─────────────────────────────
        template JobStartArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Template: ProcessRundownArgs ───────────────────────
        template ProcessRundownArgs {
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

        // ── Template: PsDiskIoAttributionStartArgs ─────────────
        template PsDiskIoAttributionStartArgs {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Template: PsIoRateControlStartArgs ─────────────────
        template PsIoRateControlStartArgs {
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

        // ── Template: PsIoRateControlStartArgs_V1 ──────────────
        template PsIoRateControlStartArgs_V1 {
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

        // ── Template: PsIoRateControlStartArgs_V2 ──────────────
        template PsIoRateControlStartArgs_V2 {
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

        // ── Template: ThreadWorkOnBehalfUpdateArgs ─────────────
        template ThreadWorkOnBehalfUpdateArgs {
            #[etw_prop(name = "OldWorkOnBehalfThreadID")]
            pub old_work_on_behalf_thread_id: u32,
            #[etw_prop(name = "NewWorkOnBehalfThreadID")]
            pub new_work_on_behalf_thread_id: u32,
        }

        // ── Template: JobServerSiloStartArgs ───────────────────
        template JobServerSiloStartArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "State")]
            pub state: u16,
        }

        // ── Template: JobServerSiloStart23Args ─────────────────
        template JobServerSiloStart23Args {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Template: JobServerSiloStartStopArgs ───────────────
        template JobServerSiloStartStopArgs {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "Status")]
            pub status: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Template: Empty (no fields) ────────────────────────
        template Empty {}

        // ── Events ─────────────────────────────────────────────

        // Event ID 1: ProcessStart
        #[etw_event(id = 1, version = 0, name = "ProcessStartV0", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStartArgs,
        #[etw_event(id = 1, version = 1, name = "ProcessStartV1", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStartArgs_V1,
        #[etw_event(id = 1, version = 2, name = "ProcessStartV2", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStartArgs_V2,
        #[etw_event(id = 1, version = 3, name = "ProcessStartV3", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessRundownArgs_V1,

        // Event ID 2: ProcessStop
        #[etw_event(id = 2, version = 0, name = "ProcessStopV0", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStopArgs,
        #[etw_event(id = 2, version = 1, name = "ProcessStopV1", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStopArgs_V1,
        #[etw_event(id = 2, version = 2, name = "ProcessStopV2", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessStopArgs_V2,

        // Event ID 3: ThreadStart
        #[etw_event(id = 3, version = 0, name = "ThreadStartV0", keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        ThreadStartArgs,
        #[etw_event(id = 3, version = 1, name = "ThreadStartV1", keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        ThreadStartArgs_V1,

        // Event ID 4: ThreadStop
        #[etw_event(id = 4, version = 0, name = "ThreadStopV0", keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        ThreadStartArgs,
        #[etw_event(id = 4, version = 1, name = "ThreadStopV1", keyword_mask = masks::WINEVENT_KEYWORD_THREAD)]
        ThreadStopArgs_V1,

        // Event ID 5: ImageLoad
        #[etw_event(id = 5, version = 0, name = "ImageLoadV0", keyword_mask = masks::WINEVENT_KEYWORD_IMAGE)]
        ImageLoadArgs,

        // Event ID 6: ImageUnload
        #[etw_event(id = 6, version = 0, name = "ImageUnloadV0", keyword_mask = masks::WINEVENT_KEYWORD_IMAGE)]
        ImageLoadArgs,

        // Event ID 7: CpuBasePriorityChange
        #[etw_event(id = 7, version = 0, name = "CpuBasePriorityChangeV0", keyword_mask = masks::WINEVENT_KEYWORD_CPU_PRIORITY)]
        CpuBasePriorityChangeArgs,

        // Event ID 8: CpuPriorityChange
        #[etw_event(id = 8, version = 0, name = "CpuPriorityChangeV0", keyword_mask = masks::WINEVENT_KEYWORD_CPU_PRIORITY)]
        CpuBasePriorityChangeArgs,

        // Event ID 9: PagePriorityChange
        #[etw_event(id = 9, version = 0, name = "PagePriorityChangeV0", keyword_mask = masks::WINEVENT_KEYWORD_OTHER_PRIORITY)]
        CpuBasePriorityChangeArgs,

        // Event ID 10: IoPriorityChange
        #[etw_event(id = 10, version = 0, name = "IoPriorityChangeV0", keyword_mask = masks::WINEVENT_KEYWORD_OTHER_PRIORITY)]
        CpuBasePriorityChangeArgs,

        // Event ID 11: ProcessFreezeStart
        #[etw_event(id = 11, version = 0, name = "ProcessFreezeStartV0", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        ProcessFreezeStartArgs,
        #[etw_event(id = 11, version = 1, name = "ProcessFreezeStartV1", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        ProcessFreezeStartArgs_V1,

        // Event ID 12: ProcessFreezeStop
        #[etw_event(id = 12, version = 0, name = "ProcessFreezeStopV0", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        ProcessFreezeStartArgs,
        #[etw_event(id = 12, version = 1, name = "ProcessFreezeStopV1", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS_FREEZE)]
        ProcessFreezeStartArgs_V1,

        // Event ID 13: JobStart
        #[etw_event(id = 13, version = 0, name = "JobStartV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB)]
        JobStartArgs,

        // Event ID 14: JobTerminateStop
        #[etw_event(id = 14, version = 0, name = "JobTerminateStopV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB)]
        JobStartArgs,

        // Event ID 15: ProcessRundown
        #[etw_event(id = 15, version = 0, name = "ProcessRundownV0", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessRundownArgs,
        #[etw_event(id = 15, version = 1, name = "ProcessRundownV1", keyword_mask = masks::WINEVENT_KEYWORD_PROCESS)]
        ProcessRundownArgs_V1,

        // Event ID 16: EnableProcessTracingCallbacks
        #[etw_event(id = 16, version = 0, name = "EnableProcessTracingCallbacksV0", keyword_mask = masks::WINEVENT_KEYWORD_ENABLE_PROCESS_TRACING_CALLBACKS)]
        Empty,

        // Event ID 17: PsDiskIoAttributionStart
        #[etw_event(id = 17, version = 0, name = "PsDiskIoAttributionStartV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsDiskIoAttributionStartArgs,

        // Event ID 18: PsDiskIoAttributionStop
        #[etw_event(id = 18, version = 0, name = "PsDiskIoAttributionStopV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsDiskIoAttributionStartArgs,

        // Event ID 19: PsIoRateControlStart
        #[etw_event(id = 19, version = 0, name = "PsIoRateControlStartV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs,
        #[etw_event(id = 19, version = 1, name = "PsIoRateControlStartV1", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs_V1,
        #[etw_event(id = 19, version = 2, name = "PsIoRateControlStartV2", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs_V2,

        // Event ID 20: PsIoRateControlStop
        #[etw_event(id = 20, version = 0, name = "PsIoRateControlStopV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs,
        #[etw_event(id = 20, version = 1, name = "PsIoRateControlStopV1", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs_V1,
        #[etw_event(id = 20, version = 2, name = "PsIoRateControlStopV2", keyword_mask = masks::WINEVENT_KEYWORD_JOB_IO)]
        PsIoRateControlStartArgs_V2,

        // Event ID 21: ThreadWorkOnBehalfUpdate
        #[etw_event(id = 21, version = 0, name = "ThreadWorkOnBehalfUpdateV0", keyword_mask = masks::WINEVENT_KEYWORD_WORK_ON_BEHALF)]
        ThreadWorkOnBehalfUpdateArgs,

        // Event ID 22: JobServerSiloStart
        #[etw_event(id = 22, version = 0, name = "JobServerSiloStartV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        JobServerSiloStartArgs,

        // Event ID 23: JobServerSiloStart23
        #[etw_event(id = 23, version = 0, name = "JobServerSiloStart23V0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        JobServerSiloStart23Args,

        // Event ID 24: JobServerSiloStartStop
        #[etw_event(id = 24, version = 0, name = "JobServerSiloStartStopV0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        JobServerSiloStartStopArgs,

        // Event ID 25: JobServerSiloStart25
        #[etw_event(id = 25, version = 0, name = "JobServerSiloStart25V0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        JobServerSiloStart23Args,

        // Event ID 26: JobServerSiloStartStop26
        #[etw_event(id = 26, version = 0, name = "JobServerSiloStartStop26V0", keyword_mask = masks::WINEVENT_KEYWORD_JOB_SILO)]
        JobServerSiloStart23Args,
    }
}
