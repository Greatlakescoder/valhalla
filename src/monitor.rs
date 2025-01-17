use crate::{
    configuration::Settings,
    ollama::{create_system_prompt, OllamaClient, OllamaRequest},
    os_tooling::{SystemScanner, TaggedProccess},
    utils::write_to_json,
};
use anyhow::Result;
pub struct SystemMonitor {
    ollama_client: OllamaClient,
}

pub struct ClientSettings {
    url: String,
}

impl SystemMonitor {
    pub fn new(settings: Settings) -> Self {
        let ollama_client = OllamaClient::new(settings.monitor.ollama_url);
        Self { ollama_client }
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

        let system_prompt = create_system_prompt();

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
    pub fn run(&self) -> Result<()>{
        let input = self.collect_info().expect("Failed to collect system info");
        self.call_ollama(input);
        Ok(())
    }
}
