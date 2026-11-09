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
            r#"Sen Rust ile yazılmış, hızlı ve yetenekli bir otonom yapay zeka ajanısın.
Hedefin kullanıcının isteğini yerine getirmektir.

KULLANABİLECEĞİN ARAÇLAR:
{tools_desc}

DAVRANIŞ KURALLARI (ReAct Döngüsü):
1. Önce ne yapacağını düşün: "THOUGHT: <düşüncen>"
2. Bir araca ihtiyacın varsa tam olarak şu formatta çağır:
ACTION: {{"tool": "<araç_adı>", "args": {{<parametreler>}}}}
3. Araç çalıştırılacak ve sana sonucu "OBSERVATION: <sonuç>" olarak verilecektir.
4. Hedefe ulaştığında veya araca ihtiyacın kalmadığında doğrudan şu formatta bitir:
FINAL_ANSWER: <kullanıcıya cevabın>

ÖNEMLİ:
- Her adımda SADECE BİR ACTION çağır.
- ACTION formatını bozma, geçerli bir JSON olmalıdır.
- Yanıtlarını net, Türkçe ve açıklayıcı ver."#
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
        println!("{} {}", "🎯 Hedef:".bold().cyan(), user_goal.white().bold());
        println!("{} {}", "🧠 Model:".bold().cyan(), self.llm.model_name().yellow());
        println!("{}\n", "=".repeat(60).cyan());

        for step in 1..=self.max_steps {
            println!("{} {}/{}", "🔄 Adım".bold().blue(), step, self.max_steps);

            let response = self.llm.chat(&conversation, 0.2, Some(512)).await?;
            let trimmed = response.trim();

            // Asistan yanıtını ekle
            conversation.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            });

            // FINAL_ANSWER kontrolü
            if let Some(final_idx) = trimmed.find("FINAL_ANSWER:") {
                let answer = trimmed[final_idx + "FINAL_ANSWER:".len()..].trim();
                println!("\n{}", "✅ Nihai Yanıt:".bold().green());
                println!("{}\n", answer.white());
                return Ok(answer.to_string());
            }

            // ACTION kontrolü
            if let Some(action_idx) = trimmed.find("ACTION:") {
                let action_part = trimmed[action_idx + "ACTION:".len()..].trim();
                let json_start = action_part.find('{').unwrap_or(0);
                let json_candidate = &action_part[json_start..];

                // JSON blok sonunu bul
                if let Ok(action_json) = serde_json::from_str::<serde_json::Value>(json_candidate) {
                    let tool_name = action_json["tool"].as_str().unwrap_or("");
                    let tool_args = &action_json["args"];

                    println!(
                        "  {} {}({})",
                        "⚡ Araç Çağrısı:".bold().magenta(),
                        tool_name.bold().yellow(),
                        tool_args.to_string().bright_black()
                    );

                    let observation = match self.tools.execute(tool_name, tool_args).await {
                        Ok(obs) => obs,
                        Err(e) => format!("Araç hatası: {}", e),
                    };

                    println!("  {} {}", "👁️ Gözlem:".bold().blue(), observation.trim());

                    conversation.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("OBSERVATION: {}", observation),
                    });
                    continue;
                }
            }

            // Eğer özel format yoksa ama yanıt verildiyse
            if !trimmed.contains("ACTION:") && !trimmed.contains("THOUGHT:") {
                println!("\n{}", "✅ Yanıt:".bold().green());
                println!("{}\n", trimmed.white());
                return Ok(trimmed.to_string());
            }
        }

        println!("{}", "⚠️ Maksimum adım sayısına ulaşıldı.".yellow());
        Ok("Maksimum adım sınırına ulaşıldı.".to_string())
    }
}
