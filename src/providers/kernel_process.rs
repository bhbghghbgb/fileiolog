#![allow(dead_code)]

use crate::etw::etw_provider;
use etw_macros::guid;

pub const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-Process";
pub const PROVIDER_GUID: ::windows::core::GUID =
    guid!("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716");

pub const PROC_PROCESS_MASK: u64 = 0x10;
pub const PROC_THREAD_MASK: u64 = 0x20;
pub const PROC_IMAGE_MASK: u64 = 0x40;
pub const PROC_PRIORITY_MASK: u64 = 0x80;
pub const PROC_PAGE_MASK: u64 = 0x100;
pub const PROC_FREEZE_MASK: u64 = 0x200;
pub const PROC_JOB_MASK: u64 = 0x400;
pub const PROC_CALLBACK_MASK: u64 = 0x800;
pub const PROC_IO_MASK: u64 = 0x1000;
pub const PROC_THREAD_WORK_MASK: u64 = 0x2000;
pub const PROC_SILO_MASK: u64 = 0x4000;

etw_provider! {
    #[etw_provider(name = PROVIDER_NAME, guid = PROVIDER_GUID)]
    pub enum KernelProcessEvent {
        // ── Event ID 1 (ProcessStart) ──────────────────────────
        #[etw_event(id = 1, version = 0, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStartV0 {
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

        #[etw_event(id = 1, version = 1, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStartV1 {
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

        #[etw_event(id = 1, version = 2, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStartV2 {
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

        #[etw_event(id = 1, version = 3, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStartV3 {
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

        #[etw_event(id = 2, version = 0, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStopV0 {
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

        #[etw_event(id = 2, version = 1, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStopV1 {
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

        #[etw_event(id = 2, version = 2, mask = PROC_PROCESS_MASK)]
        pub struct ProcessStopV2 {
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

        // ── Event ID 3 (ThreadStart) ───────────────────────────
        #[etw_event(id = 3, version = 0, mask = PROC_THREAD_MASK)]
        pub struct ThreadStartV0 {
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

        #[etw_event(id = 3, version = 1, mask = PROC_THREAD_MASK)]
        pub struct ThreadStartV1 {
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

        // ── Event ID 4 (ThreadStop) ────────────────────────────
        #[etw_event(id = 4, version = 0, mask = PROC_THREAD_MASK)]
        pub struct ThreadStopV0 {
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

        #[etw_event(id = 4, version = 1, mask = PROC_THREAD_MASK)]
        pub struct ThreadStopV1 {
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

        // ── Event ID 5 (ImageLoad) ─────────────────────────────
        #[etw_event(id = 5, version = 0, mask = PROC_IMAGE_MASK)]
        pub struct ImageLoadV0 {
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

        // ── Event ID 6 (ImageUnload) ───────────────────────────
        #[etw_event(id = 6, version = 0, mask = PROC_IMAGE_MASK)]
        pub struct ImageUnloadV0 {
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

        // ── Event ID 7 (CpuBasePriorityChange) ─────────────────
        #[etw_event(id = 7, version = 0, mask = PROC_PRIORITY_MASK)]
        pub struct CpuBasePriorityChangeV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── Event ID 8 (CpuPriorityChange) ─────────────────────
        #[etw_event(id = 8, version = 0, mask = PROC_PRIORITY_MASK)]
        pub struct CpuPriorityChangeV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── Event ID 9 (PagePriorityChange) ────────────────────
        #[etw_event(id = 9, version = 0, mask = PROC_PAGE_MASK)]
        pub struct PagePriorityChangeV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── Event ID 10 (IoPriorityChange) ─────────────────────
        #[etw_event(id = 10, version = 0, mask = PROC_PAGE_MASK)]
        pub struct IoPriorityChangeV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ThreadID")]
            pub thread_id: u32,
            #[etw_prop(name = "OldPriority")]
            pub old_priority: u8,
            #[etw_prop(name = "NewPriority")]
            pub new_priority: u8,
        }

        // ── Event ID 11 (ProcessFreezeStart) ───────────────────
        #[etw_event(id = 11, version = 0, mask = PROC_FREEZE_MASK)]
        pub struct ProcessFreezeStartV0 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        #[etw_event(id = 11, version = 1, mask = PROC_FREEZE_MASK)]
        pub struct ProcessFreezeStartV1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
        }

        // ── Event ID 12 (ProcessFreezeStop) ────────────────────
        #[etw_event(id = 12, version = 0, mask = PROC_FREEZE_MASK)]
        pub struct ProcessFreezeStopV0 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        #[etw_event(id = 12, version = 1, mask = PROC_FREEZE_MASK)]
        pub struct ProcessFreezeStopV1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
            pub create_time: time::OffsetDateTime,
        }

        // ── Event ID 13 (JobStart) ─────────────────────────────
        #[etw_event(id = 13, version = 0, mask = PROC_JOB_MASK)]
        pub struct JobStartV0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 14 (JobTerminateStop) ─────────────────────
        #[etw_event(id = 14, version = 0, mask = PROC_JOB_MASK)]
        pub struct JobTerminateStopV0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 15 (ProcessRundown) ───────────────────────
        #[etw_event(id = 15, version = 0, mask = PROC_PROCESS_MASK)]
        pub struct ProcessRundownV0 {
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

        #[etw_event(id = 15, version = 1, mask = PROC_PROCESS_MASK)]
        pub struct ProcessRundownV1 {
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

        // ── Event ID 16 (task_0) ───────────────────────────────
        #[etw_event(id = 16, version = 0, mask = PROC_CALLBACK_MASK)]
        pub struct EnableProcessTracingCallbacksV0 {}

        // ── Event ID 17 (PsDiskIoAttributionStart) ─────────────
        #[etw_event(id = 17, version = 0, mask = PROC_IO_MASK)]
        pub struct PsDiskIoAttributionStartV0 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 18 (PsDiskIoAttributionStop) ──────────────
        #[etw_event(id = 18, version = 0, mask = PROC_IO_MASK)]
        pub struct PsDiskIoAttributionStopV0 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 19 (PsIoRateControlStart) ─────────────────
        #[etw_event(id = 19, version = 0, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStartV0 {
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

        #[etw_event(id = 19, version = 1, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStartV1 {
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

        #[etw_event(id = 19, version = 2, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStartV2 {
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

        // ── Event ID 20 (PsIoRateControlStop) ──────────────────
        #[etw_event(id = 20, version = 0, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStopV0 {
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

        #[etw_event(id = 20, version = 1, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStopV1 {
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

        #[etw_event(id = 20, version = 2, mask = PROC_IO_MASK)]
        pub struct PsIoRateControlStopV2 {
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

        // ── Event ID 21 (ThreadWorkOnBehalfUpdate) ──────────────
        #[etw_event(id = 21, version = 0, mask = PROC_THREAD_WORK_MASK)]
        pub struct ThreadWorkOnBehalfUpdateV0 {
            #[etw_prop(name = "OldWorkOnBehalfThreadID")]
            pub old_work_on_behalf_thread_id: u32,
            #[etw_prop(name = "NewWorkOnBehalfThreadID")]
            pub new_work_on_behalf_thread_id: u32,
        }

        // ── Event ID 22 (JobServerSiloStart) ───────────────────
        #[etw_event(id = 22, version = 0, mask = PROC_SILO_MASK)]
        pub struct JobServerSiloStartV0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "State")]
            pub state: u16,
        }

        // ── Event ID 23 (JobServerSiloStart23) ──────────────────
        #[etw_event(id = 23, version = 0, mask = PROC_SILO_MASK)]
        pub struct JobServerSiloStart23V0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 24 (JobServerSiloStartStop) ────────────────
        #[etw_event(id = 24, version = 0, mask = PROC_SILO_MASK)]
        pub struct JobServerSiloStartStopV0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "Status")]
            pub status: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 25 (JobServerSiloStart25) ──────────────────
        #[etw_event(id = 25, version = 0, mask = PROC_SILO_MASK)]
        pub struct JobServerSiloStart25V0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 26 (JobServerSiloStartStop26) ──────────────
        #[etw_event(id = 26, version = 0, mask = PROC_SILO_MASK)]
        pub struct JobServerSiloStartStop26V0 {
            #[etw_prop(name = "ContainerID", )]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }
    }
}
