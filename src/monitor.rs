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

    //     // resource usage
    //     // Phase 3
    //     // Generate report
    async fn call_ollama_name_verification(
        &self,
        system_info: Vec<OsProcessGroup>,
    ) -> Result<Vec<OllamaAgentOutput>> {
        let system_prompt = get_name_verification_prompt();

        // Need to get just a list of names
        let names: Vec<OllamaNameInput> = system_info
            .into_iter()
            .map(|input| OllamaNameInput {
                pid: input.parent_process.pid,
                name: input.parent_process.name,
            })
            .collect();
        // Get set of input PIDs
        let input_pids: HashSet<u64> = names.iter().map(|input| input.pid as u64).collect();

        let input = json!(names);

        let initial_prompt = format!("{},{}", system_prompt, input);

        let request_body = OllamaRequest {
            model: &self.settings.monitor.model,
            prompt: initial_prompt,
            stream: false,
            options: {
                crate::ollama::Options {
                    num_ctx: self.settings.monitor.context_size,
                }
            },
        };
        tracing::debug!("Sending Request {}", request_body);
        let resp = self
            .ollama_client
            .make_generate_request(request_body)
            .await?;
        tracing::debug!("Got Response {}", &resp.response);

        let results: Vec<OllamaAgentOutput> =
            match serde_json::from_str::<Vec<OllamaAgentOutput>>(&resp.response) {
                Ok(v) => {
                    // Get set of output PIDs
                    let output_pids: HashSet<u64> = v.iter().map(|result| result.pid).collect();
                    assert!(output_pids.is_subset(&input_pids));
                    v
                }
                Err(e) => {
                    // Log the error details for debugging
                    tracing::error!("JSON parsing error: {}", e);
                    tracing::debug!("Raw response: {}", &resp.response);

                    // Attempt to clean/fix common JSON issues
                    let cleaned_response = attempt_json_cleanup(&resp.response);
                    match serde_json::from_str::<Vec<OllamaAgentOutput>>(&cleaned_response) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("Failed to parse even after cleanup: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            };

        let filtered_results: Vec<OllamaAgentOutput> =
            results.into_iter().filter(|x| x.is_malicious).collect();

        Ok(filtered_results)
    }

    async fn call_ollama_resource_verification(
        &self,
        system_info: Vec<OsProcessGroup>,
        name_verification_results: Vec<OllamaAgentOutput>,
    ) -> Result<Vec<OllamaAgentOutput>> {
        let system_prompt = get_resource_verification_prompt();

        // Need to get just a list of pids from first agent and map to original input to get the resource usage

        // Need to map si to name_verification_results
        let mut resource_usage: Vec<OllamaResourceUsageInput> = Vec::new();
        for si in system_info {
            let name_verification = name_verification_results
                .iter()
                .find(|x| x.pid == si.parent_process.pid as u64);
            if let Some(nv) = name_verification {
                let cpu_usage = si
                    .parent_process
                    .attributes
                    .get(&MetadataTags::HighCpu)
                    .ok_or_else(|| anyhow::anyhow!("CPU usage attribute not found"))?
                    .parse::<u32>()
                    .map_err(|e| anyhow::anyhow!("Failed to parse CPU usage: {}", e))?;

                let memory_usage = si
                    .parent_process
                    .attributes
                    .get(&MetadataTags::HighMemory)
                    .ok_or_else(|| anyhow::anyhow!("Memory usage attribute not found"))?
                    .parse::<u32>()
                    .map_err(|e| anyhow::anyhow!("Failed to parse memory usage: {}", e))?;

                resource_usage.push(OllamaResourceUsageInput {
                    pid: nv.pid as u32,
                    name: nv.name.clone(),
                    cpu_usage,
                    memory_usage,
                });
            }
        }
        // Get set of input PIDs
        let input_pids: HashSet<u64> = resource_usage
            .iter()
            .map(|input| input.pid as u64)
            .collect();

        let input = json!(resource_usage);

        let initial_prompt = format!("{},{}", system_prompt, input);

        let request_body = OllamaRequest {
            model: &self.settings.monitor.model,
            prompt: initial_prompt,
            stream: false,
            options: {
                crate::ollama::Options {
                    num_ctx: self.settings.monitor.context_size,
                }
            },
        };
        tracing::debug!("Sending Request {}", request_body);
        let resp = self
            .ollama_client
            .make_generate_request(request_body)
            .await?;
        tracing::debug!("Got Response {}", &resp.response);

        let results: Vec<OllamaAgentOutput> =
            match serde_json::from_str::<Vec<OllamaAgentOutput>>(&resp.response) {
                Ok(v) => {
                    // Get set of output PIDs
                    let output_pids: HashSet<u64> = v.iter().map(|result| result.pid).collect();
                    assert!(output_pids.is_subset(&input_pids));
                    v
                }
                Err(e) => {
                    // Log the error details for debugging
                    tracing::error!("JSON parsing error: {}", e);
                    tracing::debug!("Raw response: {}", &resp.response);

                    // Attempt to clean/fix common JSON issues
                    let cleaned_response = attempt_json_cleanup(&resp.response);
                    match serde_json::from_str::<Vec<OllamaAgentOutput>>(&cleaned_response) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("Failed to parse even after cleanup: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            };

        let filtered_results: Vec<OllamaAgentOutput> =
            results.into_iter().filter(|x| x.is_malicious).collect();

        Ok(filtered_results)
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

        // if !self.settings.monitor.offline {
        //     let results = self
        //         .call_ollama_name_verification(process_scan.clone())
        //         .await?;
        //     let _ = self
        //         .call_ollama_resource_verification(process_scan, results)
        //         .await?;
        // }

        Ok(())
    }
}

fn attempt_json_cleanup(response: &str) -> String {
    // Add basic JSON cleanup logic here
    response.trim().replace("...", "")
}
