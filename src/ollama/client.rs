use reqwest::Client;

use super::{OllamaRequest, OllamaResponse};
use anyhow::{anyhow, Result};

pub struct OllamaClient {
    pub client: Client,
    pub url: String,
}
impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        let ollama_client = Client::new();
        OllamaClient {
            client: ollama_client,
            url: base_url,
        }
    }

    pub async fn make_generate_request(&self, request: OllamaRequest<'_>) -> Result<OllamaResponse> {
        //http://ai-ollama.tail8c6aba.ts.net:11434/api/generate
        let generate_url: String = format!("{}/api/generate",self.url);
        let resp = match self.client.post(&generate_url).json(&request).send().await {
            Ok(resp) => OllamaResponse::from_response(resp)
                .await
                .expect("Failed to talk to Ollama"),
            Err(err) => return Err(anyhow!("Failed to send to request {err}")),
        };

        tracing::debug!("Response: {}", &resp.response);
        Ok(resp)
    }


    // //     // resource usage
    // //     // Phase 3
    // //     // Generate report
    // async fn call_ollama_name_verification(
    //     &self,
    //     system_info: Vec<OsProcessGroup>,
    // ) -> Result<Vec<OllamaAgentOutput>> {
    //     let system_prompt = get_name_verification_prompt();

    //     // Need to get just a list of names
    //     let names: Vec<OllamaNameInput> = system_info
    //         .into_iter()
    //         .map(|input| OllamaNameInput {
    //             pid: input.parent_process.pid,
    //             name: input.parent_process.name,
    //         })
    //         .collect();
    //     // Get set of input PIDs
    //     let input_pids: HashSet<u64> = names.iter().map(|input| input.pid as u64).collect();

    //     let input = json!(names);

    //     let initial_prompt = format!("{},{}", system_prompt, input);

    //     let request_body = OllamaRequest {
    //         model: &self.settings.monitor.model,
    //         prompt: initial_prompt,
    //         stream: false,
    //         options: {
    //             crate::ollama::Options {
    //                 num_ctx: self.settings.monitor.context_size,
    //             }
    //         },
    //     };
    //     tracing::debug!("Sending Request {}", request_body);
    //     let resp = self
    //         .ollama_client
    //         .make_generate_request(request_body)
    //         .await?;
    //     tracing::debug!("Got Response {}", &resp.response);

    //     let results: Vec<OllamaAgentOutput> =
    //         match serde_json::from_str::<Vec<OllamaAgentOutput>>(&resp.response) {
    //             Ok(v) => {
    //                 // Get set of output PIDs
    //                 let output_pids: HashSet<u64> = v.iter().map(|result| result.pid).collect();
    //                 assert!(output_pids.is_subset(&input_pids));
    //                 v
    //             }
    //             Err(e) => {
    //                 // Log the error details for debugging
    //                 tracing::error!("JSON parsing error: {}", e);
    //                 tracing::debug!("Raw response: {}", &resp.response);

    //                 // Attempt to clean/fix common JSON issues
    //                 let cleaned_response = attempt_json_cleanup(&resp.response);
    //                 match serde_json::from_str::<Vec<OllamaAgentOutput>>(&cleaned_response) {
    //                     Ok(v) => v,
    //                     Err(e) => {
    //                         tracing::error!("Failed to parse even after cleanup: {}", e);
    //                         return Err(e.into());
    //                     }
    //                 }
    //             }
    //         };

    //     let filtered_results: Vec<OllamaAgentOutput> =
    //         results.into_iter().filter(|x| x.is_malicious).collect();

    //     Ok(filtered_results)
    // }

    // async fn call_ollama_resource_verification(
    //     &self,
    //     system_info: Vec<OsProcessGroup>,
    //     name_verification_results: Vec<OllamaAgentOutput>,
    // ) -> Result<Vec<OllamaAgentOutput>> {
    //     let system_prompt = get_resource_verification_prompt();

    //     // Need to get just a list of pids from first agent and map to original input to get the resource usage

    //     // Need to map si to name_verification_results
    //     let mut resource_usage: Vec<OllamaResourceUsageInput> = Vec::new();
    //     for si in system_info {
    //         let name_verification = name_verification_results
    //             .iter()
    //             .find(|x| x.pid == si.parent_process.pid as u64);
    //         if let Some(nv) = name_verification {
    //             let cpu_usage = si
    //                 .parent_process
    //                 .attributes
    //                 .get(&MetadataTags::HighCpu)
    //                 .ok_or_else(|| anyhow::anyhow!("CPU usage attribute not found"))?
    //                 .parse::<u32>()
    //                 .map_err(|e| anyhow::anyhow!("Failed to parse CPU usage: {}", e))?;

    //             let memory_usage = si
    //                 .parent_process
    //                 .attributes
    //                 .get(&MetadataTags::HighMemory)
    //                 .ok_or_else(|| anyhow::anyhow!("Memory usage attribute not found"))?
    //                 .parse::<u32>()
    //                 .map_err(|e| anyhow::anyhow!("Failed to parse memory usage: {}", e))?;

    //             resource_usage.push(OllamaResourceUsageInput {
    //                 pid: nv.pid as u32,
    //                 name: nv.name.clone(),
    //                 cpu_usage,
    //                 memory_usage,
    //             });
    //         }
    //     }
    //     // Get set of input PIDs
    //     let input_pids: HashSet<u64> = resource_usage
    //         .iter()
    //         .map(|input| input.pid as u64)
    //         .collect();

    //     let input = json!(resource_usage);

    //     let initial_prompt = format!("{},{}", system_prompt, input);

    //     let request_body = OllamaRequest {
    //         model: &self.settings.monitor.model,
    //         prompt: initial_prompt,
    //         stream: false,
    //         options: {
    //             crate::ollama::Options {
    //                 num_ctx: self.settings.monitor.context_size,
    //             }
    //         },
    //     };
    //     tracing::debug!("Sending Request {}", request_body);
    //     let resp = self
    //         .ollama_client
    //         .make_generate_request(request_body)
    //         .await?;
    //     tracing::debug!("Got Response {}", &resp.response);

    //     let results: Vec<OllamaAgentOutput> =
    //         match serde_json::from_str::<Vec<OllamaAgentOutput>>(&resp.response) {
    //             Ok(v) => {
    //                 // Get set of output PIDs
    //                 let output_pids: HashSet<u64> = v.iter().map(|result| result.pid).collect();
    //                 assert!(output_pids.is_subset(&input_pids));
    //                 v
    //             }
    //             Err(e) => {
    //                 // Log the error details for debugging
    //                 tracing::error!("JSON parsing error: {}", e);
    //                 tracing::debug!("Raw response: {}", &resp.response);

    //                 // Attempt to clean/fix common JSON issues
    //                 let cleaned_response = attempt_json_cleanup(&resp.response);
    //                 match serde_json::from_str::<Vec<OllamaAgentOutput>>(&cleaned_response) {
    //                     Ok(v) => v,
    //                     Err(e) => {
    //                         tracing::error!("Failed to parse even after cleanup: {}", e);
    //                         return Err(e.into());
    //                     }
    //                 }
    //             }
    //         };

    //     let filtered_results: Vec<OllamaAgentOutput> =
    //         results.into_iter().filter(|x| x.is_malicious).collect();

    //     Ok(filtered_results)
    // }

}
