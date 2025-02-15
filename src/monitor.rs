use crate::{
    cache::blob::Cache,
    configuration::Settings,
    ollama::{
        get_name_verification_prompt, get_resource_verification_prompt, OllamaAgentOutput,
        OllamaClient, OllamaNameInput, OllamaRequest, OllamaResourceUsageInput,
    },
    os_tooling::{
        cpu::{get_current_cpu_usage, CPUGroup},
        disk::{get_disk_usage, DiskGroup},
        memory::{get_system_memory, SystemMemory},
        network::{get_network_information, NetworkInterfaceGroup},
        process::OsProcessGroup,
        MetadataTags, SystemScanner,
    },
};
use anyhow::Result;
use chrono::Local;
use metrics::counter;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use sysinfo::System;
use tokio::sync::Mutex;

#[derive(Serialize, Default, Deserialize, Debug)]
pub struct MonitorOutput {
    processes: Vec<OsProcessGroup>,
    cpu: CPUGroup,
    memory: SystemMemory,
    disks: DiskGroup,
    network: NetworkInterfaceGroup,
}

impl MonitorOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_processes(mut self, processes: Vec<OsProcessGroup>) -> Self {
        self.processes = processes;
        self
    }

    pub fn with_cpu(mut self, cpus: CPUGroup) -> Self {
        self.cpu = cpus;
        self
    }

    pub fn with_memory(mut self, memory: SystemMemory) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_disks(mut self, disks: DiskGroup) -> Self {
        self.disks = disks;
        self
    }

    pub fn with_networks(mut self, networks: NetworkInterfaceGroup) -> Self {
        self.network = networks;
        self
    }
}

pub struct SystemMonitor {
    ollama_client: OllamaClient,
    storage: Arc<Mutex<Cache<String, Vec<OsProcessGroup>>>>,
    settings: Settings,
    system: System,
}

impl SystemMonitor {
    pub fn new(
        settings: Settings,
        storage_blob: Arc<Mutex<Cache<String, Vec<OsProcessGroup>>>>,
    ) -> Self {
        let ollama_client = OllamaClient::new(settings.clone().monitor.ollama_url);
        let system = System::new_all();
        Self {
            ollama_client,
            settings,
            storage: storage_blob,
            system,
        }
    }

    // This should collect CPU and Memory
    pub async fn monitor_cpu_usage(&mut self) -> CPUGroup {
        get_current_cpu_usage(&mut self.system)
    }

    pub async fn monitor_memory_usage(&mut self) -> SystemMemory {
        get_system_memory(&mut self.system)
    }

    // This should collect disk usage across disks
    pub async fn monitor_disk_usage(&mut self) -> DiskGroup {
        get_disk_usage(&mut self.system)
    }

    // This should collect network usage
    pub async fn monitor_network_usage(&mut self) -> NetworkInterfaceGroup {
        get_network_information(&mut self.system)
    }

    pub async fn monitor_processes(&self) -> Result<Vec<OsProcessGroup>> {
        let scanner = SystemScanner::new();
        let mut results = scanner.scan_running_proccess()?;
        scanner.apply_attributes(&mut results);
        let mut storage_lock = self.storage.lock().await;
        storage_lock.remove_expired();

        let timestamp = Local::now().to_string();
        storage_lock.insert(timestamp, results.clone());
        counter!("scan.done").increment(1);
        Ok(results)
    }

    pub async fn run(&mut self) -> Result<()> {
        let process_scan = self
            .monitor_processes()
            .await
            .expect("Failed to collect system info");

        let cpu_scan = self.monitor_cpu_usage().await;

        let network_scan = self.monitor_network_usage().await;

        let disk_scan = self.monitor_disk_usage().await;

        let memory_scan = self.monitor_memory_usage().await;

        let output = MonitorOutput::new()
            .with_cpu(cpu_scan)
            .with_disks(disk_scan)
            .with_memory(memory_scan)
            .with_processes(process_scan)
            .with_networks(network_scan);

        println!("{}",serde_json::to_string_pretty(&output).unwrap());
        
        Ok(())
    }
}