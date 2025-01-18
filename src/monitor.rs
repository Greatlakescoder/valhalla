use crate::{
    configuration::Settings,
    ollama::{
        create_system_prompt, create_system_prompt_name_verifier, OllamaClient, OllamaNameInput,
        OllamaPhase1, OllamaRequest, OllamaResponse,
    },
    os_tooling::{SystemScanner, TaggedProccess},
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

    pub fn collect_info(&self) -> Result<Vec<TaggedProccess>> {
        let scanner = SystemScanner::build();
        let results = scanner.scan_running_proccess()?;
        let tagged_results = scanner.tag_proccesses(results);
        write_to_json(
            &tagged_results,
            "/home/fiz/workbench/valhalla/data/output.json",
        )?;
        Ok(tagged_results)
    }

    async fn call_ollama_name_verification(
        &self,
        system_info: Vec<TaggedProccess>,
    ) -> Result<Vec<OllamaPhase1>> {
        let system_prompt = create_system_prompt_name_verifier();

        // Need to get just a list of names
        let names: Vec<OllamaNameInput> = system_info
            .into_iter()
            .map(|input| OllamaNameInput {
                pid: input.agent_input.parent_process.pid,
                name: input.agent_input.parent_process.name,
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
        let json_str = &resp.response.replace("...", "");
        let results = serde_json::from_str(json_str);
        let results: Vec<OllamaPhase1> = match results {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to deserilize result {} {}", json_str, e);
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

    fn call_ollama(&self, system_info: Vec<TaggedProccess>) {
        // Start Chain of Thought

        // Phase 1
        // Collect all the names of the parent proccesses and create list
        // Send to agent to determine which ones are safe
        // Phase 2
        // Parse response of agent and apply metadata tags to proccesses that come back to add
        // more context
        // metadata includes forked proccesses names
        // resource usage
        // Phase 3
        // Generate report

        let system_prompt = create_system_prompt_name_verifier();

        for tp in system_info {
            let mut initial_prompt_input: String = String::from("");

            match tp.to_json_string() {
                Ok(json) => {
                    tracing::debug!("{}", json);
                    initial_prompt_input.push_str(&json)
                }
                Err(e) => eprintln!("Failed to serialize: {}", e),
            }

            let initial_prompt = format!("{},{}", system_prompt, initial_prompt_input);

            let request_body = OllamaRequest {
                model: "mistral".into(),
                prompt: initial_prompt,
                stream: false,
                options: { crate::ollama::Options { num_ctx: 20000 } },
            };
        }
    }
    pub async fn run(&self) -> Result<()> {
        let input = self.collect_info().expect("Failed to collect system info");
        self.call_ollama_name_verification(input).await;
        // self.call_ollama(input);
        Ok(())
    }
}
