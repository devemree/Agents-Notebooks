use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::Client;
use serde_json::json;
use std::path::Path;

pub struct AudioProcessor {
    client: Client,
    base_url: String,
    api_key: String,
    audio_model: String,
}

impl AudioProcessor {
    pub fn new(base_url: String, api_key: String, audio_model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            audio_model,
        }
    }

    pub async fn transcribe_and_analyze(&self, file_path: &str, prompt: &str) -> Result<String> {
        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("Ses dosyası bulunamadı: {}", file_path);
        }

        let audio_bytes = std::fs::read(path)
            .with_context(|| format!("Ses dosyası okunamadı: {}", file_path))?;
        let audio_b64 = BASE64.encode(&audio_bytes);

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3")
            .to_lowercase();
        let format = match ext.as_str() {
            "wav" => "wav",
            "ogg" => "ogg",
            "m4a" => "m4a",
            _ => "mp3",
        };

        let endpoint = format!("{}/chat/completions", self.base_url);
        let payload = json!({
            "model": self.audio_model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        { "type": "text", "text": prompt },
                        {
                            "type": "input_audio",
                            "input_audio": {
                                "data": audio_b64,
                                "format": format
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 256,
            "temperature": 0.1
        });

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Ses analizi isteği gönderilemedi")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ses modeli API hatası [{}]: {}", status, body);
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .context("Ses analizi yanıtı ayrıştırılamadı")?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Metin ayrıştırılamadı")
            .to_string();

        Ok(content)
    }
}
