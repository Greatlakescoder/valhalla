use core::fmt;
use std::ffi::OsString;

use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use sysinfo::{CpuRefreshKind, Networks, ProcessRefreshKind, ProcessStatus, RefreshKind, System};
use thiserror::Error;

#[derive(Serialize, Deserialize,Debug)]
pub struct OsProcessInformation {
    pid: u32,
    cpu: f32,
    mem: u64,
    start_time: u64,
    name: String,
    // exe: String,
    status: String,
    command: Vec<String>,
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
        Ok(Self {
            pid: process.pid().as_u32(),
            // You could make this fallible if needed
            name,
            command: cmd?,
            // exe: process
            //     .exe()
            //     .map(|p| p.to_string_lossy().to_string())
            //     .expect("Failed to unwrap exe"),
            cpu: process.cpu_usage(),
            mem: process.memory(),
            start_time: process.start_time(),
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

// impl fmt::Display for JoltOutput {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(
//             f,
//             "Effective User Id {} \n
//             Process Id {} \n
//             Cpu Usage {} \n
//             Memory Usage {} \n
//             Cumulative CPU Time {} \n
//             Command {} \n",
//             self.user, self.pid, self.cpu, self.mem, self.time, self.command
//         )
//     }
// }

pub fn get_system_memory() -> SystemMemory {
    let mut sys = System::new_all();

    sys.refresh_all();
    return SystemMemory {
        total_memory: sys.total_memory() / 1024 / 1024,
        used_memory: sys.used_memory() / 1024 / 1024,
        total_swap: sys.total_swap() / 1024 / 1024,
        used_swap: sys.used_swap() / 1024 / 1024,
    };
}

// pub fn get_system_information() -> anyhow::Result<SystemInformation> {
//     let mut sys = System::new_all();
//     sys.refresh_all();
//     Ok(SystemInformation {
//         cpu_arch: psutil::host::info().architecture().to_string(),
//         host_name: psutil::host::info().hostname().to_string(),
//         os_version: psutil::host::info().operating_system().to_string(),
//         name: System::name().unwrap(),
//         uptime: psutil::host::uptime().unwrap().as_secs(),
//         total_cpus: psutil::cpu::cpu_count(),
//         total_memory: psutil::memory::virtual_memory().unwrap().available(),
//     })
// }

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

// pub fn kill_process(pid: u32) -> anyhow::Result<()> {
//     let current_process = psutil::process::Process::new(pid)?;
//     if current_process.is_running() {
//         current_process.kill()?;
//     }
//     Ok(())
// }

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



pub fn scan_running_proccess() -> anyhow::Result<Vec<OsProcessInformation>> {
    let mut output: Vec<OsProcessInformation> = vec![];
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    for (_, process) in sys.processes() {
        let formatted_process: OsProcessInformation = process.try_into().unwrap();
        // println!("{}",formatted_process.status);
        if formatted_process.status == "Sleep" || formatted_process.status == "Dead" {
            tracing::trace!("Ignoring process {}, its {}", formatted_process.name, formatted_process.status)
        }else {
            output.push(process.try_into().unwrap());
        }

       
    }


    Ok(output)
}
