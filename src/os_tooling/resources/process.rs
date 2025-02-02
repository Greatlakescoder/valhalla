use std::{collections::HashMap, ffi::OsString};

use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use thiserror::Error;

use super::MetadataTags;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OsProcessInformation {
    pub pid: u32,
    #[serde(skip_serializing)]
    pub cpu: f32,
    #[serde(skip_serializing)]
    pub memory_usage: u64,
    #[serde(skip_serializing)]
    pub run_time: u64,
    pub name: String,
    // exe: String,
    #[serde(skip_serializing)]
    status: String,
    #[serde(skip_serializing)]
    pub command: Vec<String>,
    #[serde(skip_serializing)]
    pub user_id: String,
    pub attributes: HashMap<MetadataTags, String>,
    pub fd_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OsProcessGroup {
    pub parent_process: OsProcessInformation,
    pub forked_threads: Vec<OsProcessInformation>,
}

#[derive(Error, Debug)]
pub enum ProcessConversionError {
    #[error("Failed to get process name")]
    NameError,
    #[error("Failed to get process path")]
    PathError,
    #[error("Invalid process state")]
    InvalidState,
    #[error("Failed to convert command string")]
    CommandConversionError,
}

// Helper function to convert OsString to String
fn convert_os_string(os_str: OsString) -> Result<String, ProcessConversionError> {
    os_str
        .into_string()
        .map_err(|_| ProcessConversionError::CommandConversionError)
}

// Using TryFrom which is the basis for TryInto
impl TryFrom<&sysinfo::Process> for OsProcessInformation {
    type Error = ProcessConversionError;

    fn try_from(process: &sysinfo::Process) -> Result<Self, Self::Error> {
        let name = convert_os_string(process.name().into())
            .map_err(|_| ProcessConversionError::NameError)?;

        let cmd: Result<Vec<String>, _> = process
            .cmd()
            .iter()
            .map(|os_str| convert_os_string(os_str.clone()))
            .collect();
        let user_id = process
            .effective_user_id()
            .map(|u| u.to_string())
            .unwrap_or_default();
        Ok(Self {
            pid: process.pid().as_u32(),
            // You could make this fallible if needed
            name,
            command: cmd?,
            user_id,
            cpu: process.cpu_usage(),
            // Convert bytes to MB
            memory_usage: process.memory() / (1024 * 1024),
            run_time: process.run_time(),
            status: format!("{:?}", process.status()),
            attributes: HashMap::new(),
            fd_count: 0,
        })
    }
}

impl OsProcessInformation {
    // This can be useful when you want to handle JSON errors explicitly
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl OsProcessGroup {
    // This can be useful when you want to handle JSON errors explicitly
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}


pub trait ProcessAttribute: Send {
    // &mut interior mutability
    fn tag(&self, process: &mut OsProcessInformation);
    fn untag(&self, process: &mut OsProcessInformation);
}

pub struct ResourceUsageAttribute {
    cpu_threshold: f32,
    memory_threshold: u64,
}
impl ResourceUsageAttribute {
    pub fn new(cpu_threshold: f32, memory_threshold: u64) -> Self {
        Self {
            cpu_threshold,
            memory_threshold,
        }
    }
}
impl ProcessAttribute for ResourceUsageAttribute {
    fn tag(&self, process: &mut OsProcessInformation) {
        if process.cpu > self.cpu_threshold {
            process
                .attributes
                .insert(MetadataTags::HighCpu, process.cpu.to_string());
        } else {
            process
                .attributes
                .insert(MetadataTags::CpuUsage, process.cpu.to_string());
        }
        if process.memory_usage > self.memory_threshold {
            process
                .attributes
                .insert(MetadataTags::HighMemory, process.memory_usage.to_string());
        } else {
            process
                .attributes
                .insert(MetadataTags::MemoryUsage, process.memory_usage.to_string());
        }
    }

    fn untag(&self, process: &mut OsProcessInformation) {
        process.attributes.remove(&MetadataTags::HighCpu);
        process.attributes.remove(&MetadataTags::HighMemory);
    }
}

pub fn is_process_alive(process: &OsProcessInformation) -> bool {
    // TODO we should also put these in the config
    if process.status == "Dead" || process.status == "Idle" {
        tracing::debug!("Ignoring process {}, its {}", process.name, process.status);
        return false;
    }
    true
}
