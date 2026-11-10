use crate::audio::AudioProcessor;
use crate::client::LocalLlmClient;
use crate::skill_refiner::SkillRefiner;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: String,
}

pub struct ToolManager {
    audio_processor: Arc<AudioProcessor>,
    tools: Vec<ToolDefinition>,
}

impl ToolManager {
    pub fn new(audio_processor: Arc<AudioProcessor>) -> Self {
        let default_tools = vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Reads the full text content of a local file.".to_string(),
                parameters: "{\"path\": \"string\"}".to_string(),
            },
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Writes or creates a local file with the specified text content.".to_string(),
                parameters: "{\"path\": \"string\", \"content\": \"string\"}".to_string(),
            },
            ToolDefinition {
                name: "list_dir".to_string(),
                description: "Lists all files and subdirectories located inside a directory path.".to_string(),
                parameters: "{\"path\": \"string\"}".to_string(),
            },
            ToolDefinition {
                name: "run_command".to_string(),
                description: "Executes a system shell or PowerShell terminal command.".to_string(),
                parameters: "{\"command\": \"string\"}".to_string(),
            },
            ToolDefinition {
                name: "web_search".to_string(),
                description: "Searches the live public internet using DuckDuckGo.".to_string(),
                parameters: "{\"query\": \"string\"}".to_string(),
            },
            ToolDefinition {
                name: "transcribe_audio".to_string(),
                description: "Listens to and transcribes local audio recordings (mp3, wav).".to_string(),
                parameters: "{\"path\": \"string\"}".to_string(),
            },
        ];

        Self {
            audio_processor,
            tools: default_tools,
        }
    }

    /// Skill++ Optimizer:
    /// Re-evaluates each tool against neighbors to add negative boundary definitions and prevent misrouting.
    pub async fn optimize_with_skill_plus_plus(&mut self, llm: &LocalLlmClient) -> Result<()> {
        println!("{}", "⚡ [Skill++] Optimizing tool descriptions to eliminate routing ambiguity...".bold().magenta());

        let snapshot = self.tools.clone();
        for tool in &mut self.tools {
            let original_desc = tool.description.clone();
            match SkillRefiner::refine_tool_description(&tool.name, &original_desc, &snapshot, llm).await {
                Ok(refined) => {
                    println!("  {} {}:", "✓".green(), tool.name.yellow().bold());
                    println!("    {} {}", "Original:".bright_black(), original_desc.bright_black());
                    println!("    {} {}", "Skill++: ".cyan().bold(), refined.white());
                    tool.description = refined;
                }
                Err(e) => {
                    eprintln!("  {} Failed to refine {}: {}", "✗".red(), tool.name, e);
                }
            }
        }
        println!("{}", "✨ [Skill++] Optimization complete! Negative routing boundaries active.\n".bold().green());
        Ok(())
    }

    pub fn get_tools_description(&self) -> String {
        self.tools
            .iter()
            .map(|t| format!("- {}({}): {}", t.name, t.parameters, t.description))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub async fn execute(&self, tool_name: &str, args: &serde_json::Value) -> Result<String> {
        match tool_name {
            "read_file" => {
                let path = args["path"].as_str().context("Missing required parameter: path")?;
                self.read_file(path)
            }
            "write_file" => {
                let path = args["path"].as_str().context("Missing required parameter: path")?;
                let content = args["content"].as_str().context("Missing required parameter: content")?;
                self.write_file(path, content)
            }
            "list_dir" => {
                let path = args["path"].as_str().unwrap_or(".");
                self.list_dir(path)
            }
            "run_command" => {
                let cmd = args["command"].as_str().context("Missing required parameter: command")?;
                self.run_command(cmd)
            }
            "web_search" => {
                let query = args["query"].as_str().context("Missing required parameter: query")?;
                self.web_search(query).await
            }
            "transcribe_audio" => {
                let path = args["path"].as_str().context("Missing required parameter: path")?;
                self.audio_processor
                    .transcribe_and_analyze(path, "Transcribe this audio recording accurately.")
                    .await
            }
            _ => anyhow::bail!("Unknown tool: {}", tool_name),
        }
    }

    fn read_file(&self, path_str: &str) -> Result<String> {
        let content = fs::read_to_string(path_str)
            .with_context(|| format!("Failed to read file: {}", path_str))?;
        if content.len() > 4000 {
            Ok(format!("{}...\n[Content truncated: showing first 4000 characters]", &content[..4000]))
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
            .with_context(|| format!("Failed to write file: {}", path_str))?;
        Ok(format!("File successfully written: {}", path_str))
    }

    fn list_dir(&self, path_str: &str) -> Result<String> {
        let entries = fs::read_dir(path_str)
            .with_context(|| format!("Failed to read directory: {}", path_str))?;
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
            Ok(format!("Output:\n{}\nError:\n{}", stdout, stderr))
        }
    }

    async fn web_search(&self, query: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

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
                        findings.push(format!("Abstract: {}", abstract_text));
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

        let html_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        let html_resp = client.get(&html_url).send().await?;
        let text = html_resp.text().await?;
        
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
            Ok(format!("No direct web search results found for '{}'.", query))
        } else {
            Ok(clean_snippets.join("\n---\n"))
        }
    }
}
