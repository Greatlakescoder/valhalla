use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemMemory {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

impl SystemMemory {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn get_system_memory(system: &mut System) -> SystemMemory {
    system.refresh_all();
    SystemMemory {
        total_memory: system.total_memory() / 1024 / 1024,
        used_memory: system.used_memory() / 1024 / 1024,
        total_swap: system.total_swap() / 1024 / 1024,
        used_swap: system.used_swap() / 1024 / 1024,
    }
}
