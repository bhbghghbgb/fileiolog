#![allow(dead_code)]

use crate::providers::kernel_file::KernelFileEvent;

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    KernelFile(KernelFileEvent),
}
