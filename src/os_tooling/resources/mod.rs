use serde::{Deserialize, Serialize};

pub mod cpu;
pub mod memory;
pub mod network;
pub mod process;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum MetadataTags {
    TotalCpu,
    TotalMemory,
    TotalFileDescriptors,
    CpuUsage,
    HighCpu,
    HighMemory,
    MemoryUsage,
}
