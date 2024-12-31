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
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool
}