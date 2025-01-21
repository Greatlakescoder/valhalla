use crate::{
    configuration::Settings,
    ollama::{get_full_prompt, OllamaClient, OllamaNameInput, OllamaPhase1, OllamaRequest},
    os_tooling::{AgentInput, SystemScanner},
    utils::write_to_json,
};
use anyhow::Result;
use serde_json::json;
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
        let tagged_results = scanner.apply_attributes(&mut results);
        write_to_json(
            &tagged_results,
            "/home/fiz/workbench/valhalla/data/output.json",
        )?;
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
        write_to_json(
            &resp.response,
            "/home/fiz/workbench/valhalla/data/phase_1_output.json",
        )?;
        let formatted_response = &resp.response.replace("...", "");
        let results = serde_json::from_str(formatted_response);
        let results: Vec<OllamaPhase1> = match results {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to deserilize result {} {}", &resp.response, e);
                return Err(e.into());
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
