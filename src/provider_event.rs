#![allow(dead_code)]

use crate::providers::kernel_file::KernelFileEvent;
use crate::providers::kernel_process::KernelProcessEvent;

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    KernelFile(KernelFileEvent),
    KernelProcess(KernelProcessEvent),
}
