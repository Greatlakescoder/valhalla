use std::fmt::Display;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaResponse {
    pub model: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub context: Vec<i64>,
    #[serde(rename = "total_duration")]
    pub total_duration: i64,
    #[serde(rename = "load_duration")]
    pub load_duration: i64,
    #[serde(rename = "prompt_eval_count")]
    pub prompt_eval_count: i64,
    #[serde(rename = "prompt_eval_duration")]
    pub prompt_eval_duration: i64,
    #[serde(rename = "eval_count")]
    pub eval_count: i64,
    #[serde(rename = "eval_duration")]
    pub eval_duration: i64,
}

impl OllamaResponse {
    pub async fn from_response(response: reqwest::Response) -> Result<Self, reqwest::Error> {
        response.json().await
    }
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaRequest<'a> {
    pub model: &'a str,
    pub prompt: String,
    pub stream: bool,
    pub options: Options
}

impl Display for OllamaRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resp = serde_json::to_string_pretty(self).expect("Failed to format");
        write!(f,"{}",resp)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Options {
    pub num_ctx: u32
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OllamaNameInput{
    pub pid: u32,
    pub name: String
}

impl OllamaNameInput {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPhase1{
    pub pid: u64,
    pub name: String,
    pub is_malicious: bool,
    pub reason: String
}

impl OllamaPhase1 {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}


