#![allow(dead_code)]

use crate::providers::user_trace_kernel_file::UserTraceKernelFileEvent;
use crate::providers::user_trace_kernel_process::UserTraceKernelProcessEvent;
use crate::providers::kernel_trace_fileio::KernelTraceFileIoEvent;

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    KernelFile(UserTraceKernelFileEvent),
    KernelProcess(UserTraceKernelProcessEvent),
    /// Kernel trace file IO events (from kernel_trace_fileio provider with group mask support)
    KernelFileIo(KernelTraceFileIoEvent),
}
