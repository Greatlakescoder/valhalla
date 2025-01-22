use crate::{
    configuration::Settings,
    ollama::{get_full_prompt, OllamaClient, OllamaNameInput, OllamaPhase1, OllamaRequest},
    os_tooling::{AgentInput, SystemScanner},
    utils::write_to_json,
};
use anyhow::Result;
use serde_json::json;
use std::collections::HashSet;
pub struct SystemMonitor {
    ollama_client: OllamaClient,
    settings: Settings,
}

impl SystemMonitor {
    pub fn new(settings: Settings) -> Self {
        let ollama_client = OllamaClient::new(settings.clone().monitor.ollama_url);

        Self {
            ollama_client,
            settings,
        }
    }

    pub fn collect_info(&self) -> Result<Vec<AgentInput>> {
        let scanner = SystemScanner::new();
        let mut results = scanner.scan_running_proccess()?;
        scanner.apply_attributes(&mut results);
        write_to_json(&results, "/home/fiz/workbench/valhalla/data/output.json")?;
        Ok(results)
    }

    //     // Chain of Thought
    //     // Phase 1
    //     // Collect all the names of the parent proccesses and create list
    //     // Send to agent to determine which ones are safe
    //     // Phase 2
    //     // Parse response of agent and apply metadata tags to proccesses that come back to add
    //     // more context
    //     // metadata includes forked proccesses names
    //     // resource usage
    //     // Phase 3
    //     // Generate report
    async fn call_ollama_name_verification(
        &self,
        system_info: Vec<AgentInput>,
    ) -> Result<Vec<OllamaPhase1>> {
        let system_prompt = get_full_prompt();

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

        let results: Vec<OllamaPhase1> =
            match serde_json::from_str::<Vec<OllamaPhase1>>(&resp.response) {
                Ok(v) => {
                    write_to_json(&v, "/home/fiz/workbench/valhalla/data/phase_1_output.json")?;
                    // Get set of output PIDs
                    let output_pids: HashSet<u64> = v.iter().map(|result| result.pid).collect();
                    assert!(output_pids.is_subset(&input_pids));
                    v
                }
                Err(e) => {
                    // Log the error details for debugging
                    tracing::error!("JSON parsing error: {}", e);
                    tracing::debug!("Raw response: {}", &resp.response);
                    write_to_json(
                        &resp.response,
                        "/home/fiz/workbench/valhalla/data/phase_1_output.json",
                    )?;

                    // Attempt to clean/fix common JSON issues
                    let cleaned_response = attempt_json_cleanup(&resp.response);
                    match serde_json::from_str::<Vec<OllamaPhase1>>(&cleaned_response) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("Failed to parse even after cleanup: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            };

        let filtered_results: Vec<OllamaPhase1> =
            results.into_iter().filter(|x| x.is_malicious).collect();
        if filtered_results.len() > 0 {
            for f in &filtered_results {
                println!("{:?}", f.to_json_string())
            }
        } else {
            println!("No bad proccess found")
        }

        return Ok(filtered_results);
    }

    pub async fn run(&self) -> Result<()> {
        let input = self.collect_info().expect("Failed to collect system info");
        let _ = self.call_ollama_name_verification(input).await;

        Ok(())
    }
}

fn attempt_json_cleanup(response: &str) -> String {
    // Add basic JSON cleanup logic here
    response.trim().replace("...", "")
}
