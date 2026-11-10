mod agent;
mod audio;
mod client;
mod skill_refiner;
mod tools;

use agent::ReactAgent;
use audio::AudioProcessor;
use client::LocalLlmClient;
use colored::Colorize;
use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use tools::ToolManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env configuration
    dotenvy::dotenv().ok();

    let api_url = env::var("UNSLOTH_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8888/v1".to_string());
    let api_key = env::var("UNSLOTH_API_KEY")
        .unwrap_or_else(|_| "sk-unsloth-2ab2d4bac872bae08d4e5b48560edc8c".to_string());
    let text_model = env::var("UNSLOTH_MODEL")
        .unwrap_or_else(|_| "unsloth/Qwen3.5-4B-MTP-GGUF".to_string());
    let audio_model = env::var("UNSLOTH_AUDIO_MODEL")
        .unwrap_or_else(|_| "LiquidAI/LFM3-Audio-2B-GGUF".to_string());

    println!("{}", "\n🦀 =============================================== 🦀".bright_yellow());
    println!("{}", "   RUST LOCAL AI AGENT (Unsloth + ReAct + Skill++) ".bold().bright_white());
    println!("{}", "🦀 =============================================== 🦀\n".bright_yellow());
    println!("📡 API URL:     {}", api_url.cyan());
    println!("🧠 LLM Model:   {}", text_model.yellow());
    println!("🎙️ Audio Model: {}", audio_model.green());
    println!("🔬 Algorithm:   {} (Routing-Aware Boundary Refinement)", "Skill++".bright_magenta().bold());
    println!("⚡ Engine:      {} (Rust 2024 Edition)\n", "Tokio Async".magenta());

    // Initialize services
    let llm_client = LocalLlmClient::new(api_url.clone(), api_key.clone(), text_model);
    let audio_proc = Arc::new(AudioProcessor::new(api_url, api_key, audio_model));
    let mut tool_mgr = ToolManager::new(audio_proc);

    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut enable_skill_plus_plus = false;

    // Check if --skill++ flag is passed
    if let Some(pos) = args.iter().position(|a| a == "--skill++" || a == "--refine") {
        args.remove(pos);
        enable_skill_plus_plus = true;
    }

    if enable_skill_plus_plus {
        tool_mgr.optimize_with_skill_plus_plus(&llm_client).await?;
    }

    let agent = ReactAgent::new(llm_client.clone(), tool_mgr, 8);

    // CLI Arguments check (one-shot mode)
    if !args.is_empty() {
        let task = args.join(" ");
        agent.run(&task).await?;
        return Ok(());
    }

    // Interactive REPL Mode
    println!("{}", "💬 Switched to Interactive Mode. Commands:".italic().bright_black());
    println!("{}", "   • Type '/skill++' to optimize tool descriptions dynamically".italic().bright_black());
    println!("{}", "   • Type 'q' or 'exit' to quit\n".italic().bright_black());

    loop {
        print!("{}", "Agent > ".bold().green());
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "exit" || trimmed == "q" {
            println!("{}", "Goodbye! 👋".yellow());
            break;
        }

        if trimmed == "/skill++" || trimmed == "/refine" {
            println!("\n{}", "🔄 Triggering Skill++ Tool Optimization...".magenta().bold());
            let mut fresh_tools = ToolManager::new(Arc::new(AudioProcessor::new(
                env::var("UNSLOTH_API_URL").unwrap_or_default(),
                env::var("UNSLOTH_API_KEY").unwrap_or_default(),
                env::var("UNSLOTH_AUDIO_MODEL").unwrap_or_default(),
            )));
            if let Err(e) = fresh_tools.optimize_with_skill_plus_plus(&llm_client).await {
                eprintln!("❌ Failed to optimize tools: {}", e);
            }
            continue;
        }

        if let Err(e) = agent.run(trimmed).await {
            eprintln!("{} {}", "❌ Error:".bold().red(), e);
        }
    }

    Ok(())
}
