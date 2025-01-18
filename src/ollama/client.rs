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
        return Ok(resp)
    }
}
