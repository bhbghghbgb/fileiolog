#![allow(dead_code)]

use crate::providers::user_trace_kernel_file::UserTraceKernelFileEvent;
use crate::providers::user_trace_kernel_process::UserTraceKernelProcessEvent;

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    KernelFile(UserTraceKernelFileEvent),
    KernelProcess(UserTraceKernelProcessEvent),
}
