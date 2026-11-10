use crate::client::{ChatMessage, LocalLlmClient};
use crate::tools::ToolDefinition;
use anyhow::Result;

pub struct SkillRefiner;

impl SkillRefiner {
    /// Skill++ Algorithm:
    /// 1. Identifies overlapping or nearest existing tools.
    /// 2. Performs contrastive analysis using the LLM.
    /// 3. Injects explicit negative boundary constraints ("What NOT to use it for").
    pub async fn refine_tool_description(
        tool_name: &str,
        raw_description: &str,
        existing_tools: &[ToolDefinition],
        llm: &LocalLlmClient,
    ) -> Result<String> {
        if existing_tools.is_empty() {
            return Ok(raw_description.to_string());
        }

        let existing_summary: Vec<String> = existing_tools
            .iter()
            .filter(|t| t.name != tool_name)
            .map(|t| format!("- {}: {}", t.name, t.description))
            .collect();

        if existing_summary.is_empty() {
            return Ok(raw_description.to_string());
        }

        let existing_list = existing_summary.join("\n");

        let prompt = format!(
            r#"You are an AI Agent Skill Optimizer implementing the 'Skill++' algorithm.
Problem: When new skills are added to an agent, routers often confuse them with existing similar skills.

NEW SKILL TO OPTIMIZE:
Name: {tool_name}
Original Description: {raw_description}

EXISTING SKILLS IN THE LIBRARY:
{existing_list}

YOUR TASK:
Rewrite the description for '{tool_name}' in 1-2 concise sentences adhering strictly to the Skill++ principle:
1. State the exact core capability of '{tool_name}'.
2. Clarify the operational boundary from the most similar existing skill.
3. Explicitly state what it does NOT handle (e.g., "Do NOT use this for X; use <other_tool> instead.").

Return ONLY the refined description text, without any conversational preamble or quotes."#
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a precise technical prompt engineer specialized in AI Agent tool routing.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let refined = llm.chat(&messages, 0.2, Some(150)).await?;
        let cleaned = refined.trim().trim_matches('"').trim().to_string();

        Ok(cleaned)
    }
}
