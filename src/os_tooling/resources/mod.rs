use serde::{Deserialize, Serialize};

pub mod cpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod disk;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum MetadataTags {
    TotalCpu,
    TotalMemory,
    TotalFileDescriptors,
    CpuUsage,
    HighCpu,
    HighMemory,
    MemoryUsage,
    ThreatScore,
    ThreatScoreReason,
}
