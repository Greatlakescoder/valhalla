use crate::{
    configuration::Settings,
    memory::blob::Cache,
    ollama::{
        get_name_verification_prompt, get_resource_verification_prompt, OllamaAgentOutput,
        OllamaClient, OllamaNameInput, OllamaRequest, OllamaResourceUsageInput,
    },
    os_tooling::{AgentInput, MetadataTags, SystemScanner},
};
use anyhow::Result;
use chrono::Local;
use metrics::counter;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;
pub struct SystemMonitor {
    ollama_client: OllamaClient,
    storage: Arc<Mutex<Cache<String, Vec<AgentInput>>>>,
    settings: Settings,
}

impl SystemMonitor {
    pub fn new(
        settings: Settings,
        storage_blob: Arc<Mutex<Cache<String, Vec<AgentInput>>>>,
    ) -> Self {
        let ollama_client = OllamaClient::new(settings.clone().monitor.ollama_url);

        Self {
            ollama_client,
            settings,
            storage: storage_blob,
        }
    }

    pub async fn collect_info(&self) -> Result<Vec<AgentInput>> {
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
        system_info: Vec<AgentInput>,
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

        return Ok(filtered_results);
    }

    async fn call_ollama_resource_verification(
        &self,
        system_info: Vec<AgentInput>,
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

        return Ok(filtered_results);
    }

    pub async fn run(&self) -> Result<()> {
        let input = self
            .collect_info()
            .await
            .expect("Failed to collect system info");
        if !self.settings.monitor.offline {
            let results = self.call_ollama_name_verification(input.clone()).await?;
            let _ = self
                .call_ollama_resource_verification(input, results)
                .await?;
        }

        Ok(())
    }
}

fn attempt_json_cleanup(response: &str) -> String {
    // Add basic JSON cleanup logic here
    response.trim().replace("...", "")
}
