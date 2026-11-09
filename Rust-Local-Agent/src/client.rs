use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Clone)]
pub struct LocalLlmClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl LocalLlmClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    pub async fn chat(&self, messages: &[ChatMessage], temperature: f32, max_tokens: Option<u32>) -> Result<String> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let payload = ChatCompletionRequest {
            model: &self.model,
            messages,
            temperature,
            max_tokens,
        };

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Yerel LLM sunucusuna bağlantı başarısız oldu")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API Hatası [{}]: {}", status, body);
        }

        let resp_json: ChatCompletionResponse = response
            .json()
            .await
            .context("LLM yanıtı JSON olarak ayrıştırılamadı")?;

        let reply = resp_json
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(reply)
    }
}
