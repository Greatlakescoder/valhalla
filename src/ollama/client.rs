use std::collections::HashSet;

use reqwest::Client;
use serde_json::json;

use crate::{
    configuration::Settings,
    monitor::{MonitorOutput, SystemMonitor},
    os_tooling::process::OsProcessGroup,
};

use super::{
    OllamaNameInput, OllamaRequest, OllamaResponse, ProcessScore, PROCESS_ANALYSIS_PROMPT,
};
use anyhow::{anyhow, Context, Result};

pub struct OllamaClient {
    pub client: Client,
    pub settings: Settings,
}
impl OllamaClient {
    pub fn new(settings: Settings) -> Self {
        let ollama_client = Client::new();
        OllamaClient {
            client: ollama_client,
            settings,
        }
    }

    pub async fn make_generate_request(
        &self,
        request: OllamaRequest<'_>,
    ) -> Result<OllamaResponse> {
        //http://ai-ollama.tail8c6aba.ts.net:11434/api/generate
        let generate_url: String = format!("{}/api/generate", self.settings.monitor.ollama_url);
        let resp = self
            .client
            .post(&generate_url)
            .json(&request)
            .send()
            .await?;
        let resp = match OllamaResponse::from_response(resp).await {
            Ok(ollama_response) => ollama_response,
            Err(err) => return Err(anyhow!("Invalid request, ollama responded with {err}")),
        };

        Ok(resp)
    }

    pub async fn analyze_system_monitor_output(
        &self,
        output: &MonitorOutput,
    ) -> Result<Vec<ProcessScore>> {
        self.analyze_process_names(&output.processes).await
    }

    async fn analyze_process_names(
        &self,
        processess: &Vec<OsProcessGroup>,
    ) -> Result<Vec<ProcessScore>> {
        // We probably only need to look at parent threads for now
        // Input object should prob
        let names: Vec<OllamaNameInput> = processess
            .into_iter()
            .map(|input| OllamaNameInput {
                pid: input.parent_process.pid,
                name: input.parent_process.name.clone(),
            })
            .collect();

        // Need the pids to make sure response doesnt make shit up and for mapping back
        let input_pids: HashSet<u64> = names.iter().map(|input| input.pid as u64).collect();

        let input = json!(names);
        let prompt = format!(
            "{}\n\nInput processes to analyze:\n{}",
            PROCESS_ANALYSIS_PROMPT, input
        );

        let request_body = OllamaRequest {
            model: &self.settings.monitor.model,
            prompt,
            stream: false,
            options: {
                crate::ollama::Options {
                    num_ctx: self.settings.monitor.context_size,
                }
            },
        };
        let resp = self.make_generate_request(request_body).await?;

        let scores: Vec<ProcessScore> = serde_json::from_str(&resp.response)
            .context("Failed to parse LLM response as ProcessScore array")?;
        Ok(scores)
    }
}
