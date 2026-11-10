# 🦀 Rust Local AI Agent (Unsloth + ReAct + Skill++)

A high-performance autonomous agent written in modern **Rust (2024 Edition)** with **Tokio Async**. Powered 100% locally by **Unsloth Studio** (Qwen 3.5 4B & LFM3-Audio) with zero cloud API keys and near-zero memory footprint (~10 MB RAM).

## 🚀 Key Features
- **⚡ Pure Rust Performance:** Async execution with Tokio runtime, type-safe error handling, and blazing-fast execution with ~10 MB RAM usage.
- **🧠 Local LLM Engine:** Native integration with locally hosted Unsloth Studio (`http://127.0.0.1:8888/v1`), Ollama, vLLM, or LM Studio via OpenAI-compatible endpoints.
- **🎙️ Multimodal Audio Reasoning:** End-to-end local speech recognition and audio analysis (`LiquidAI/LFM3-Audio`) for `.mp3` and `.wav` files without cloud latency.
- **🔬 Skill++ Routing-Aware Optimization:**
  - Implements the SOTA **Skill++** algorithm (*"Towards Lifelong Agents via Routing-Aware Skill Refinement"*).
  - Automatically compares each skill against neighbors in the library.
  - Dynamically rewrites tool descriptions with **Contrastive Differentiation** and explicit **Negative Exclusion Boundaries** (e.g., *"Do NOT use this for X; use <other_tool> instead"*), completely eliminating router confusion and skill substitution.
- **🛠️ Type-Safe ReAct Tools:**
  - `read_file(path)`: Safely reads file contents.
  - `write_file(path, content)`: Creates or updates file content.
  - `list_dir(path)`: Lists directory contents.
  - `run_command(command)`: Executes system terminal commands.
  - `web_search(query)`: Free web search via DuckDuckGo without any API key.
  - `transcribe_audio(path)`: Listens to and transcribes local audio recordings.
- **💬 Dual Execution Modes:** Supports both one-shot CLI task execution and an interactive REPL chat mode with dynamic `/skill++` triggers.

## 🛠️ Setup & Usage

### 1. Configuration (.env)
Create or verify the `.env` file in the project root:
```env
UNSLOTH_API_URL=http://127.0.0.1:8888/v1
UNSLOTH_API_KEY=sk-unsloth-2ab2d4bac872bae08d4e5b48560edc8c
UNSLOTH_MODEL=unsloth/Qwen3.5-4B-MTP-GGUF
UNSLOTH_AUDIO_MODEL=LiquidAI/LFM3-Audio-2B-GGUF
```

### 2. Build & Run
```bash
cd Rust-Local-Agent

# 1. Run with Skill++ optimization enabled
cargo run -- --skill++ "List files in the src folder"

# 2. Standard execution
cargo run -- "List files in the src folder"

# 3. Interactive REPL mode (Type '/skill++' to optimize dynamically)
cargo run
```
