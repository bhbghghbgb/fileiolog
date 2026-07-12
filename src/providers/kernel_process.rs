#![allow(dead_code)]

use crate::etw::etw_provider;

etw_provider! {
    #[etw_provider(name = "Microsoft-Windows-Kernel-Process", guid = "22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716")]
    pub enum KernelProcessEvent {
        // ── Event ID 1 (v0, v1, v2, v3) ──────────────────────────
        #[etw_event(id = 1, version = 0, mask = 0x10)]
        pub struct ProcessStartV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        #[etw_event(id = 1, version = 1, mask = 0x10)]
        pub struct ProcessStartV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
            #[etw_prop(name = "ParentProcessID")]
            pub parent_process_id: u32,
            #[etw_prop(name = "SessionID")]
            pub session_id: u32,
            #[etw_prop(name = "Flags")]
            pub flags: u32,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        #[etw_event(id = 1, version = 2, mask = 0x10)]
        pub struct ProcessStartV2 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
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

        #[etw_event(id = 1, version = 3, mask = 0x10)]
        pub struct ProcessStartV3 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ProcessSequenceNumber")]
            pub process_sequence_number: u64,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
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

        // ── Event ID 2 (v0, v1, v2) ─────────────────────────────
        #[etw_event(id = 2, version = 0, mask = 0x10)]
        pub struct ProcessStopV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::parser::FileTime)]
            pub exit_time: i64,
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

        #[etw_event(id = 2, version = 1, mask = 0x10)]
        pub struct ProcessStopV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::parser::FileTime)]
            pub exit_time: i64,
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

        #[etw_event(id = 2, version = 2, mask = 0x10)]
        pub struct ProcessStopV2 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ProcessSequenceNumber")]
            pub process_sequence_number: u64,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
            #[etw_prop(name = "ExitTime", parse_as = ferrisetw::parser::FileTime)]
            pub exit_time: i64,
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

        // ── Event ID 3 (v0, v1) ─────────────────────────────────
        #[etw_event(id = 3, version = 0, mask = 0x20)]
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

        #[etw_event(id = 3, version = 1, mask = 0x20)]
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

        // ── Event ID 4 (v0, v1) ─────────────────────────────────
        #[etw_event(id = 4, version = 0, mask = 0x20)]
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

        #[etw_event(id = 4, version = 1, mask = 0x20)]
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

        // ── Event ID 5 (v0) ─────────────────────────────────────
        #[etw_event(id = 5, version = 0, mask = 0x40)]
        pub struct ImageLoadV0 {
            #[etw_prop(name = "ImageBase", parse_as = ferrisetw::parser::Pointer)]
            pub image_base: usize,
            #[etw_prop(name = "ImageSize", parse_as = ferrisetw::parser::Pointer)]
            pub image_size: usize,
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ImageCheckSum")]
            pub image_check_sum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "DefaultBase", parse_as = ferrisetw::parser::Pointer)]
            pub default_base: usize,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── Event ID 6 (v0) ─────────────────────────────────────
        #[etw_event(id = 6, version = 0, mask = 0x40)]
        pub struct ImageUnloadV0 {
            #[etw_prop(name = "ImageBase", parse_as = ferrisetw::parser::Pointer)]
            pub image_base: usize,
            #[etw_prop(name = "ImageSize", parse_as = ferrisetw::parser::Pointer)]
            pub image_size: usize,
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ImageCheckSum")]
            pub image_check_sum: u32,
            #[etw_prop(name = "TimeDateStamp")]
            pub time_date_stamp: u32,
            #[etw_prop(name = "DefaultBase", parse_as = ferrisetw::parser::Pointer)]
            pub default_base: usize,
            #[etw_prop(name = "ImageName")]
            pub image_name: String,
        }

        // ── Event ID 7 (v0) ─────────────────────────────────────
        #[etw_event(id = 7, version = 0, mask = 0x80)]
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

        // ── Event ID 8 (v0) ─────────────────────────────────────
        #[etw_event(id = 8, version = 0, mask = 0x80)]
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

        // ── Event ID 9 (v0) ─────────────────────────────────────
        #[etw_event(id = 9, version = 0, mask = 0x100)]
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

        // ── Event ID 10 (v0) ────────────────────────────────────
        #[etw_event(id = 10, version = 0, mask = 0x100)]
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

        // ── Event ID 11 (v0, v1) ────────────────────────────────
        #[etw_event(id = 11, version = 0, mask = 0x200)]
        pub struct ProcessFreezeStartV0 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        #[etw_event(id = 11, version = 1, mask = 0x200)]
        pub struct ProcessFreezeStartV1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
        }

        // ── Event ID 12 (v0, v1) ────────────────────────────────
        #[etw_event(id = 12, version = 0, mask = 0x200)]
        pub struct ProcessFreezeStopV0 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
        }

        #[etw_event(id = 12, version = 1, mask = 0x200)]
        pub struct ProcessFreezeStopV1 {
            #[etw_prop(name = "FrozenProcessID")]
            pub frozen_process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
        }

        // ── Event ID 13 (v0) ────────────────────────────────────
        #[etw_event(id = 13, version = 0, mask = 0x400)]
        pub struct JobStartV0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 14 (v0) ────────────────────────────────────
        #[etw_event(id = 14, version = 0, mask = 0x400)]
        pub struct JobTerminateStopV0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 15 (v0, v1) ────────────────────────────────
        #[etw_event(id = 15, version = 0, mask = 0x10)]
        pub struct ProcessRundownV0 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
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

        #[etw_event(id = 15, version = 1, mask = 0x10)]
        pub struct ProcessRundownV1 {
            #[etw_prop(name = "ProcessID")]
            pub process_id: u32,
            #[etw_prop(name = "ProcessSequenceNumber")]
            pub process_sequence_number: u64,
            #[etw_prop(name = "CreateTime", parse_as = ferrisetw::parser::FileTime)]
            pub create_time: i64,
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

        // ── Event ID 17 (v0) ────────────────────────────────────
        #[etw_event(id = 17, version = 0, mask = 0x1000)]
        pub struct PsDiskIoAttributionStartV0 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 18 (v0) ────────────────────────────────────
        #[etw_event(id = 18, version = 0, mask = 0x1000)]
        pub struct PsDiskIoAttributionStopV0 {
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "DiskIoAttribution", parse_as = ferrisetw::parser::Pointer)]
            pub disk_io_attribution: usize,
            #[etw_prop(name = "StatusCode")]
            pub status_code: u32,
        }

        // ── Event ID 19 (v0, v1, v2) ────────────────────────────
        #[etw_event(id = 19, version = 0, mask = 0x1000)]
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

        #[etw_event(id = 19, version = 1, mask = 0x1000)]
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

        #[etw_event(id = 19, version = 2, mask = 0x1000)]
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

        // ── Event ID 20 (v0, v1, v2) ────────────────────────────
        #[etw_event(id = 20, version = 0, mask = 0x1000)]
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

        #[etw_event(id = 20, version = 1, mask = 0x1000)]
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

        #[etw_event(id = 20, version = 2, mask = 0x1000)]
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

        // ── Event ID 21 (v0) ────────────────────────────────────
        #[etw_event(id = 21, version = 0, mask = 0x2000)]
        pub struct ThreadWorkOnBehalfUpdateV0 {
            #[etw_prop(name = "OldWorkOnBehalfThreadID")]
            pub old_work_on_behalf_thread_id: u32,
            #[etw_prop(name = "NewWorkOnBehalfThreadID")]
            pub new_work_on_behalf_thread_id: u32,
        }

        // ── Event ID 22 (v0) ────────────────────────────────────
        #[etw_event(id = 22, version = 0, mask = 0x4000)]
        pub struct JobServerSiloStartV0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "State")]
            pub state: u16,
        }

        // ── Event ID 23 (v0) ────────────────────────────────────
        #[etw_event(id = 23, version = 0, mask = 0x4000)]
        pub struct JobServerSiloStart23V0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 24 (v0) ────────────────────────────────────
        #[etw_event(id = 24, version = 0, mask = 0x4000)]
        pub struct JobServerSiloStartStopV0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "Status")]
            pub status: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 25 (v0) ────────────────────────────────────
        #[etw_event(id = 25, version = 0, mask = 0x4000)]
        pub struct JobServerSiloStart25V0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }

        // ── Event ID 26 (v0) ────────────────────────────────────
        #[etw_event(id = 26, version = 0, mask = 0x4000)]
        pub struct JobServerSiloStartStop26V0 {
            #[etw_prop(name = "ContainerID")]
            pub container_id: windows::core::GUID,
            #[etw_prop(name = "JobID")]
            pub job_id: u32,
            #[etw_prop(name = "MonitorName")]
            pub monitor_name: String,
        }
    }
}
