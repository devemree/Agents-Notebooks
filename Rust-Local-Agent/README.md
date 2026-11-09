# 🦀 Rust Local AI Agent (Unsloth + ReAct + Audio)

A high-performance autonomous agent written in modern **Rust (2024 Edition)** with **Tokio Async**. Powered 100% locally by **Unsloth Studio** (Qwen 3.5 4B & LFM3-Audio) with zero cloud API keys and near-zero memory footprint (~10 MB RAM).

## 🚀 Key Features
- **⚡ Pure Rust Performance:** Async execution with Tokio runtime, type-safe error handling, and blazing-fast execution with ~10 MB RAM usage.
- **🧠 Local LLM Engine:** Native integration with locally hosted Unsloth Studio (`http://127.0.0.1:8888/v1`), Ollama, vLLM, or LM Studio via OpenAI-compatible endpoints.
- **🎙️ Multimodal Audio Reasoning:** End-to-end local speech recognition and audio analysis (`LiquidAI/LFM3-Audio`) for `.mp3` and `.wav` files without cloud latency.
- **🛠️ Type-Safe ReAct Tools:**
  - `read_file(path)`: Safely reads file contents.
  - `write_file(path, content)`: Creates or updates file content.
  - `list_dir(path)`: Lists directory contents.
  - `run_command(command)`: Executes system terminal commands.
  - `web_search(query)`: Free web search via DuckDuckGo without any API key.
  - `transcribe_audio(path)`: Listens to and transcribes local audio recordings.
- **💬 Dual Execution Modes:** Supports both one-shot CLI task execution and an interactive REPL chat mode.

## 🛠️ Setup & Usage

### 1. Configuration (.env)
Create or verify the `.env` file in the project root:
```env
UNSLOTH_API_URL=http://127.0.0.1:8888/v1
UNSLOTH_API_KEY=sk-unsloth-1cd1858c8973112cc943332dc8d65c9f
UNSLOTH_MODEL=unsloth/Qwen3.5-4B-MTP-GGUF
UNSLOTH_AUDIO_MODEL=LiquidAI/LFM3-Audio-2B-GGUF
```

### 2. Build & Run
```bash
cd Rust-Local-Agent

# 1. One-shot task execution
cargo run -- "List files in the src folder and summarize their purpose"

# 2. Interactive REPL mode
cargo run
```
