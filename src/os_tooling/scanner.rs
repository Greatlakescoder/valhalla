use core::fmt;
use std::{collections::HashMap, ffi::OsString, time::Duration};

use metrics::counter;
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use sysinfo::{CpuRefreshKind, Networks, Pid, ProcessRefreshKind, RefreshKind, System};
use thiserror::Error;

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
pub struct AgentInput {
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemInformation {
    pub name: String,
    pub os_version: String,
    pub host_name: String,
    pub uptime: u64,
    pub total_cpus: u64,
    pub total_memory: u64,
    pub cpu_arch: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemMemory {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

impl fmt::Display for SystemInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Name {} \n 
            Os Version {} \n
            Hostname {} \n
            uptime {} \n
            total cpus {} \n
            total memory {} \n
            cpu arch {} \n",
            self.name,
            self.os_version,
            self.host_name,
            self.uptime,
            self.total_cpus,
            self.total_memory,
            self.cpu_arch
        )
    }
}

pub fn get_system_memory() -> SystemMemory {
    let mut sys = System::new_all();

    sys.refresh_all();
    SystemMemory {
        total_memory: sys.total_memory() / 1024 / 1024,
        used_memory: sys.used_memory() / 1024 / 1024,
        total_swap: sys.total_swap() / 1024 / 1024,
        used_swap: sys.used_swap() / 1024 / 1024,
    }
}

pub fn get_network_information() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let networks = Networks::new_with_refreshed_list();
    let users = sysinfo::Users::new();
    for user in users.list() {
        println!("{} is in {} groups", user.name(), user.groups().len());
    }
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {}/{} B  Mac Address: {}, Packets {}/{}",
            data.received(),
            data.transmitted(),
            data.mac_address(),
            data.packets_received(),
            data.packets_transmitted()
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct CpuUsageResponse {
    pub cpus: Vec<CpuUsage>,
}
#[derive(Serialize, Deserialize)]
pub struct CpuUsage {
    pub name: String,
    pub brand: String,
    pub frequency: u64,
    pub usage: f32,
}

pub fn get_current_cpu_usage() -> CpuUsageResponse {
    println!("Get Current CPU Start");
    let mut s = System::new_with_specifics(
        sysinfo::RefreshKind::everything().with_cpu(CpuRefreshKind::everything()),
    );

    // Wait a bit because CPU usage is based on diff.
    println!("Get Current CPU Sleep");
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    println!("Get Current CPU Refresh");
    s.refresh_cpu_all();
    println!("Get Current CPU Logic");

    let mut resp: Vec<CpuUsage> = vec![];
    for cpu in s.cpus() {
        resp.push(CpuUsage {
            name: cpu.name().to_string(),
            brand: cpu.brand().to_string(),
            frequency: cpu.frequency(),
            usage: cpu.cpu_usage(),
        })
    }
    CpuUsageResponse { cpus: resp }
}

#[derive(Serialize, Deserialize)]
pub struct MemoryResponse {
    pub free_memory: u64,
    pub total_memory: u64,
}

pub fn get_memory_cpu_usage() -> MemoryResponse {
    let mut s = System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );

    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh Memory
    s.refresh_memory();

    MemoryResponse {
        free_memory: s.free_memory(),
        total_memory: s.total_memory(),
    }
}

impl OsProcessInformation {
    // This can be useful when you want to handle JSON errors explicitly
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl AgentInput {
    // This can be useful when you want to handle JSON errors explicitly
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum MetadataTags {
    TotalCpu,
    TotalMemory,
    TotalFileDescriptors,
    NormalCpu,
    HighCpu,
    HighMemory,
    NormalMemory,
    TooManyFileDescriptors,
}

trait ProcessAttribute {
    // &mut interior mutability
    fn tag(&self, process: &mut OsProcessInformation);
    fn untag(&self, process: &mut OsProcessInformation);
}

struct ResourceUsageAttribute {
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
                .insert(MetadataTags::NormalCpu, process.cpu.to_string());
        }
        if process.memory_usage > self.memory_threshold {
            process
                .attributes
                .insert(MetadataTags::HighMemory, process.memory_usage.to_string());
        } else {
            process
                .attributes
                .insert(MetadataTags::NormalMemory, process.memory_usage.to_string());
        }
    }

    fn untag(&self, process: &mut OsProcessInformation) {
        process.attributes.remove(&MetadataTags::HighCpu);
        process.attributes.remove(&MetadataTags::HighMemory);
    }
}

struct FileDescriptorAttribute {
    fd_threshold: u64,
}

impl ProcessAttribute for FileDescriptorAttribute {
    fn tag(&self, process: &mut OsProcessInformation) {
        if process.fd_count > self.fd_threshold {
            process.attributes.insert(
                MetadataTags::TooManyFileDescriptors,
                process.fd_count.to_string(),
            );
        }
    }

    fn untag(&self, process: &mut OsProcessInformation) {
        process
            .attributes
            .remove(&MetadataTags::TooManyFileDescriptors);
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

///
/// I think in order to get the model to act how we want, we need to label things and send it to multiple agents
/// We can implement this with a tagging methodolgy similar to how EC2 does it or how you would label data in a csv
/// Tags Available to use
/// - High CPU
/// - High Memory
/// - High Runtime
/// - Has Forked/Spawned Processes
/// We want to store this data in a lookup table so that we can do the post processing of the model out here
/// instead of wasting context windows
///

pub struct SystemScanner {
    attributes: Vec<Box<dyn ProcessAttribute>>,
}

impl SystemScanner {
    pub fn new() -> Self {
        let attributes: Vec<Box<dyn ProcessAttribute>> =
            vec![Box::new(ResourceUsageAttribute::new(60.0, 7200))];
        Self { attributes }
    }

    fn calculate_total_resource_usage(&self, input: &mut AgentInput) {
        let mut total_cpu = input.parent_process.cpu;
        let mut total_memory = input.parent_process.memory_usage;
        let mut total_fd = input.parent_process.fd_count;

        // Sum up resources from all child processes
        for child in &input.forked_threads {
            total_cpu += child.cpu;
            total_memory += child.memory_usage;
            total_fd += child.fd_count;
        }

        // Store totals in parent's attributes
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalCpu, total_cpu.to_string());
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalMemory, total_memory.to_string());
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalFileDescriptors, total_fd.to_string());
    }

    pub fn apply_attributes(&self, input: &mut Vec<AgentInput>) {
        for p in input.iter_mut() {
            self.calculate_total_resource_usage(p);
            for attribute in &self.attributes {
                for sp in p.forked_threads.iter_mut() {
                    attribute.tag(sp);
                }
                attribute.tag(&mut p.parent_process);
            }
        }
    }
    /**
    Scans all running processes on the system and groups them by parent-child relationships.
    Returns a Vec of AgentInput where each entry contains a parent process and its forked threads.
    This helps track process hierarchies and identify related processes.
    */
    pub fn scan_running_proccess(&self) -> anyhow::Result<Vec<AgentInput>> {
        tracing::info!("System Monitor scanning");

        let mut agent_output: HashMap<u32, AgentInput> = HashMap::new();
        counter!("scan.run").increment(1);

        let mut sys = System::new_all();
        // Wait a bit because CPU usage is based on diff.
        std::thread::sleep(Duration::from_secs(2));
        // Refresh CPU usage to get actual value.
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );

        for process in sys.processes().values() {
            let formatted_process: OsProcessInformation = process.try_into()?;
            if !is_process_alive(&formatted_process) {
                continue;
            }

            // Handle process based on whether it has a parent
            if let Some(parent_pid) = process.parent() {
                let lookup_key = parent_pid.as_u32();

                match sys.process(Pid::from(lookup_key as usize)) {
                    Some(parent_process) if !agent_output.contains_key(&lookup_key) => {
                        // First time seeing this parent, create new entry
                        let formatted_parent: OsProcessInformation = parent_process.try_into()?;
                        agent_output.insert(
                            lookup_key,
                            AgentInput {
                                forked_threads: vec![formatted_process],
                                parent_process: formatted_parent,
                            },
                        );
                    }
                    Some(_) => {
                        // Parent exists in map, update forked threads
                        agent_output
                            .entry(lookup_key)
                            .and_modify(|x| x.forked_threads.push(formatted_process.clone()))
                            .or_insert(AgentInput {
                                forked_threads: vec![],
                                parent_process: formatted_process,
                            });
                    }
                    None => continue,
                }
            } else {
                // No parent, add as standalone process
                agent_output
                    .entry(formatted_process.pid)
                    .or_insert(AgentInput {
                        forked_threads: vec![],
                        parent_process: formatted_process,
                    });
            }
        }
        tracing::info!("System Monitor scanning done");
        // Convert to Vec more efficiently
        Ok(agent_output.into_values().collect())
    }
}

impl Default for SystemScanner {
    fn default() -> Self {
        Self::new()
    }
}
