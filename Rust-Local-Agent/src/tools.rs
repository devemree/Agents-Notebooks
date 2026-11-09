use crate::audio::AudioProcessor;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: String,
}

pub struct ToolManager {
    audio_processor: Arc<AudioProcessor>,
}

impl ToolManager {
    pub fn new(audio_processor: Arc<AudioProcessor>) -> Self {
        Self { audio_processor }
    }

    pub fn get_tools_description(&self) -> String {
        r#"- read_file(path): Belirtilen dosyanın içeriğini okur.
- write_file(path, content): Belirtilen dosyaya metin içeriği yazar veya oluşturur.
- list_dir(path): Belirtilen dizindeki dosya ve klasörleri listeler.
- run_command(command): Sistem terminalinde (PowerShell) güvenli bir komut çalıştırır.
- web_search(query): İnternette (DuckDuckGo üzerinden) ücretsiz arama yapar.
- transcribe_audio(path): Yerel ses dosyasını (mp3/wav) dinler ve metne dönüştürür."#.to_string()
    }

    pub async fn execute(&self, tool_name: &str, args: &serde_json::Value) -> Result<String> {
        match tool_name {
            "read_file" => {
                let path = args["path"].as_str().context("path parametresi eksik")?;
                self.read_file(path)
            }
            "write_file" => {
                let path = args["path"].as_str().context("path parametresi eksik")?;
                let content = args["content"].as_str().context("content parametresi eksik")?;
                self.write_file(path, content)
            }
            "list_dir" => {
                let path = args["path"].as_str().unwrap_or(".");
                self.list_dir(path)
            }
            "run_command" => {
                let cmd = args["command"].as_str().context("command parametresi eksik")?;
                self.run_command(cmd)
            }
            "web_search" => {
                let query = args["query"].as_str().context("query parametresi eksik")?;
                self.web_search(query).await
            }
            "transcribe_audio" => {
                let path = args["path"].as_str().context("path parametresi eksik")?;
                self.audio_processor
                    .transcribe_and_analyze(path, "Bu ses kaydında tam olarak ne söyleniyor? Metne dök.")
                    .await
            }
            _ => anyhow::bail!("Bilinmeyen araç: {}", tool_name),
        }
    }

    fn read_file(&self, path_str: &str) -> Result<String> {
        let content = fs::read_to_string(path_str)
            .with_context(|| format!("Dosya okunamadı: {}", path_str))?;
        if content.len() > 4000 {
            Ok(format!("{}...\n[İçerik çok uzun, ilk 4000 karakter gösterildi]", &content[..4000]))
        } else {
            Ok(content)
        }
    }

    fn write_file(&self, path_str: &str, content: &str) -> Result<String> {
        if let Some(parent) = Path::new(path_str).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path_str, content)
            .with_context(|| format!("Dosya yazılamadı: {}", path_str))?;
        Ok(format!("Dosya başarıyla yazıldı: {}", path_str))
    }

    fn list_dir(&self, path_str: &str) -> Result<String> {
        let entries = fs::read_dir(path_str)
            .with_context(|| format!("Dizin okunamadı: {}", path_str))?;
        let mut results = Vec::new();
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                results.push(format!("[DIR]  {}", file_name));
            } else {
                results.push(format!("[FILE] {}", file_name));
            }
        }
        Ok(results.join("\n"))
    }

    fn run_command(&self, cmd: &str) -> Result<String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", cmd])
                .output()?
        } else {
            Command::new("sh")
                .args(["-c", cmd])
                .output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Ok(format!("Çıktı:\n{}\nHata:\n{}", stdout, stderr))
        }
    }

    async fn web_search(&self, query: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        // DuckDuckGo Instant API (Ücretsiz, sınırsız)
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let resp = client.get(&url).send().await;
        if let Ok(res) = resp {
            if let Ok(json_data) = res.json::<serde_json::Value>().await {
                let mut findings = Vec::new();

                if let Some(abstract_text) = json_data["AbstractText"].as_str() {
                    if !abstract_text.is_empty() {
                        findings.push(format!("Özet: {}", abstract_text));
                    }
                }

                if let Some(topics) = json_data["RelatedTopics"].as_array() {
                    for t in topics.iter().take(3) {
                        if let Some(text) = t["Text"].as_str() {
                            findings.push(format!("- {}", text));
                        }
                    }
                }

                if !findings.is_empty() {
                    return Ok(findings.join("\n"));
                }
            }
        }

        // Yedek: HTML Arama
        let html_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        let html_resp = client.get(&html_url).send().await?;
        let text = html_resp.text().await?;
        
        // Basit sonuç çıkarma
        let mut clean_snippets = Vec::new();
        for line in text.lines() {
            if line.contains("result__snippet") {
                let snippet = line
                    .replace("<a class=\"result__snippet\"", "")
                    .replace("</a>", "");
                clean_snippets.push(snippet.trim().to_string());
                if clean_snippets.len() >= 4 {
                    break;
                }
            }
        }

        if clean_snippets.is_empty() {
            Ok(format!("'{}' için doğrudan web sonucu bulunamadı.", query))
        } else {
            Ok(clean_snippets.join("\n---\n"))
        }
    }
}
