use crate::client::{ChatMessage, LocalLlmClient};
use crate::tools::ToolManager;
use anyhow::Result;
use colored::Colorize;

pub struct ReactAgent {
    llm: LocalLlmClient,
    tools: ToolManager,
    max_steps: usize,
}

impl ReactAgent {
    pub fn new(llm: LocalLlmClient, tools: ToolManager, max_steps: usize) -> Self {
        Self {
            llm,
            tools,
            max_steps,
        }
    }

    pub async fn run(&self, user_goal: &str) -> Result<String> {
        let tools_desc = self.tools.get_tools_description();

        let system_prompt = format!(
            r#"You are a high-performance autonomous AI agent built in Rust.
Your objective is to accomplish the user's task using available tools.

AVAILABLE TOOLS:
{tools_desc}

BEHAVIORAL RULES (ReAct Loop):
1. Think before acting: "THOUGHT: <your reasoning>"
2. When a tool is needed, call it strictly in this format:
ACTION: {{"tool": "<tool_name>", "args": {{<parameters>}}}}
3. The system executes the tool and returns the result as "OBSERVATION: <result>".
4. When the goal is completed or no further tools are needed, terminate with:
FINAL_ANSWER: <your response to the user>

IMPORTANT:
- Call only ONE ACTION per step.
- Ensure the ACTION payload is strictly valid JSON.
- Provide clear, well-structured, and helpful answers in English."#
        );

        let mut conversation: Vec<ChatMessage> = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_goal.to_string(),
            },
        ];

        println!("\n{}", "=".repeat(60).cyan());
        println!("{} {}", "🎯 Goal: ".bold().cyan(), user_goal.white().bold());
        println!("{} {}", "🧠 Model:".bold().cyan(), self.llm.model_name().yellow());
        println!("{}\n", "=".repeat(60).cyan());

        for step in 1..=self.max_steps {
            println!("{} {}/{}", "🔄 Step".bold().blue(), step, self.max_steps);

            let response = self.llm.chat(&conversation, 0.2, Some(512)).await?;
            let trimmed = response.trim();

            conversation.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            });

            // FINAL_ANSWER detection
            if let Some(final_idx) = trimmed.find("FINAL_ANSWER:") {
                let answer = trimmed[final_idx + "FINAL_ANSWER:".len()..].trim();
                println!("\n{}", "✅ Final Answer:".bold().green());
                println!("{}\n", answer.white());
                return Ok(answer.to_string());
            }

            // ACTION detection
            if let Some(action_idx) = trimmed.find("ACTION:") {
                let action_part = trimmed[action_idx + "ACTION:".len()..].trim();
                let json_start = action_part.find('{').unwrap_or(0);
                let json_candidate = &action_part[json_start..];

                if let Ok(action_json) = serde_json::from_str::<serde_json::Value>(json_candidate) {
                    let tool_name = action_json["tool"].as_str().unwrap_or("");
                    let tool_args = &action_json["args"];

                    println!(
                        "  {} {}({})",
                        "⚡ Tool Call:".bold().magenta(),
                        tool_name.bold().yellow(),
                        tool_args.to_string().bright_black()
                    );

                    let observation = match self.tools.execute(tool_name, tool_args).await {
                        Ok(obs) => obs,
                        Err(e) => format!("Tool execution error: {}", e),
                    };

                    println!("  {} {}", "👁️ Observation:".bold().blue(), observation.trim());

                    conversation.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("OBSERVATION: {}", observation),
                    });
                    continue;
                }
            }

            // Fallback when model answers directly without special tags
            if !trimmed.contains("ACTION:") && !trimmed.contains("THOUGHT:") {
                println!("\n{}", "✅ Response:".bold().green());
                println!("{}\n", trimmed.white());
                return Ok(trimmed.to_string());
            }
        }

        println!("{}", "⚠️ Maximum step limit reached.".yellow());
        Ok("Maximum steps limit reached.".to_string())
    }
}
