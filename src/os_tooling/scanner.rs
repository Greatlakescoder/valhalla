use core::fmt;
use std::{collections::HashMap, default, ffi::OsString};

use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use sysinfo::{CpuRefreshKind, Networks, Pid, ProcessRefreshKind, RefreshKind, System};
use thiserror::Error;

use crate::configuration::MonitorSettings;

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
            .unwrap_or_else(String::new);

        Ok(Self {
            pid: process.pid().as_u32(),
            // You could make this fallible if needed
            name,
            command: cmd?,
            user_id: user_id,
            cpu: process.cpu_usage(),
            // Convert bytes to MB
            memory_usage: process.memory() / (1024 * 1024),
            run_time: process.run_time(),
            status: format!("{:?}", process.status()),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaggedProccess {
    pub agent_input: AgentInput,
    pub is_high_cpu: bool,
    pub is_high_mem: bool,
    pub is_forking: bool,
    pub is_long_run_time: bool,
}

impl TaggedProccess {
    pub fn new(agent_input: AgentInput) -> Self {
        Self {
            agent_input: agent_input.clone(),
            is_high_cpu: false,
            is_high_mem: false,
            is_long_run_time: false,
            is_forking: false,
        }
    }

    pub fn tag(&mut self) {
        self.has_high_cpu_usage();
        self.has_high_memory_usage();
        self.has_forked_processes();
        self.has_long_runtime();
    }

    fn has_high_cpu_usage(&mut self) {
        if self.agent_input.parent_process.cpu > 60.0 {
            self.is_high_cpu = true
        }
    }

    fn has_high_memory_usage(&mut self) {
        if self.agent_input.parent_process.memory_usage > 250 {
            self.is_high_mem = true
        }
    }

    fn has_long_runtime(&mut self) {
        if self.agent_input.parent_process.run_time > 7200 {
            self.is_long_run_time = true
        }
    }

    fn has_forked_processes(&mut self) {
        if self.agent_input.forked_threads.len() > 0 {
            self.is_forking = true
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
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

pub struct SystemScanner {}

// TODO determine if we want to do any filtering every
// pub fn is_valid_process_name(process: &OsProcessInformation, filter: &SystemFilter) -> bool {
//     // TODO we should also put these in the config

//     match &filter.process_name_filter {
//         Some(filter) => process.name.contains(filter),
//         None => true,
//     }
// }

// pub fn is_valid_process_pid(process: &OsProcessInformation, filter: &SystemFilter) -> bool {
//     // TODO we should also put these in the config

//     match filter.process_parent_filter {
//         Some(filter) => process.pid == filter,
//         None => true,
//     }
// }
// pub struct SystemFilter {
//     pub process_name_filter: Option<String>,
//     pub process_parent_filter: Option<u32>,
// }

impl SystemScanner {
    pub fn build() -> Self {
        Self {}
    }

    pub fn tag_proccesses(&self, system_proccesses: Vec<AgentInput>) -> Vec<TaggedProccess> {
        let mut tagged_proccesses: Vec<TaggedProccess> = vec![];
        for p in system_proccesses {
            let mut tagged_proccess = TaggedProccess::new(p);
            tagged_proccess.tag();
            tagged_proccesses.push(tagged_proccess);
        }
        return tagged_proccesses;
    }

    pub fn scan_running_proccess(&self) -> anyhow::Result<Vec<AgentInput>> {
        let mut output: Vec<AgentInput> = vec![];
        let mut sys = System::new_all();
        let mut agent_output: HashMap<u32, AgentInput> = HashMap::new();

        // First we update all information of our `System` struct.
        sys.refresh_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );

        for process in sys.processes().values() {
            let formatted_process: OsProcessInformation = process.try_into().unwrap();
            if !is_process_alive(&formatted_process) {
                continue;
            } else {
                // So part of the problem is we need to be able to track forked/spawned processes so the agent will know they are connected
                // otherwise its going to think we have 16 clones of say a Tokio app
                if process.parent().is_some() {
                    let lookup_key = &process.parent().unwrap().as_u32();
                    if agent_output.contains_key(lookup_key) {
                        agent_output
                            .entry(*lookup_key)
                            .and_modify(|x| x.forked_threads.push(formatted_process.clone()))
                            .or_insert(AgentInput {
                                forked_threads: vec![],
                                parent_process: formatted_process,
                            });
                    } else {
                        // If we are adding for first time we need to find the parent process
                        if let Some(parent_process) = sys.process(Pid::from(*lookup_key as usize)) {
                            let formatted_parent_process: OsProcessInformation =
                                parent_process.try_into().unwrap();
                            agent_output.insert(
                                lookup_key.to_owned(),
                                AgentInput {
                                    forked_threads: vec![formatted_process],
                                    parent_process: formatted_parent_process,
                                },
                            );
                        } else {
                            continue;
                        }
                    }
                } else {
                    // If we dont have this proccess from previous grouping of parent-child, add it
                    if !agent_output.contains_key(&formatted_process.pid) {
                        agent_output.insert(
                            formatted_process.pid,
                            AgentInput {
                                forked_threads: vec![],
                                parent_process: formatted_process,
                            },
                        );
                    }
                }
            }
        }
        // We need to use for each here since map does not consume the iter
        agent_output.iter().for_each(|x| output.push(x.1.clone()));
        Ok(output)
    }
}
