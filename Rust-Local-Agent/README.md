# 🦀 Rust Local AI Agent (Unsloth + ReAct + Audio)

2026 standartlarında, **Rust 2024 Edition** ve **Tokio** asenkron altyapısıyla geliştirilmiş, **%100 yerel (local)** ve sıfır API maliyetli otonom yapay zeka ajanı.

## 🚀 Özellikler
- **⚡ Rust Performansı:** Python'a kıyasla mikrosaniyeler mertebesinde çalışma, ~10 MB RAM tüketimi.
- **🧠 Local LLM:** Yerel çalışan Unsloth Studio (`http://127.0.0.1:8888/v1`) veya Ollama / vLLM / LM Studio ile tam OpenAI-uyumluluğu.
- **🎙️ Yerel Multimodal Ses (Audio In):** `LiquidAI/LFM3-Audio` ve benzeri modellerle `.mp3` / `.wav` dosyalarını yerel olarak dinleme ve metne dökme.
- **🛠️ Tip Güvenli Araçlar (ReAct Loop):**
  - `read_file(path)`: Dosya okuma.
  - `write_file(path, content)`: Dosya yazma / oluşturma.
  - `list_dir(path)`: Dizin listeleme.
  - `run_command(command)`: PowerShell komutları çalıştırma.
  - `web_search(query)`: DuckDuckGo üzerinden ücretsiz web araması.
  - `transcribe_audio(path)`: Yerel ses dosyasını dinleyip analiz etme.
- **💬 Çift Mod:** Hem doğrudan CLI argümanıyla tek seferlik görev çalıştırma hem de interaktif REPL sohbet modu.

## 🛠️ Kurulum & Çalıştırma

### 1. Ayarlar (.env)
`.env` dosyasını düzenleyin veya varsayılan yerel Unsloth Studio ayarlarıyla bırakın:
```env
UNSLOTH_API_URL=http://127.0.0.1:8888/v1
UNSLOTH_API_KEY=sk-unsloth-1cd1858c8973112cc943332dc8d65c9f
UNSLOTH_MODEL=LiquidAI/LFM3-Audio-2B-GGUF
UNSLOTH_AUDIO_MODEL=LiquidAI/LFM3-Audio-2B-GGUF
```

### 2. Derleme ve Çalıştırma
```bash
cd Rust-Local-Agent

# Tek seferlik görev çalıştırma
cargo run -- "Mevcut dizindeki dosyaları listele"

# İnteraktif mod
cargo run
```
