# Tech Stack

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Languages

| Language | Version | Usage |
|----------|---------|-------|
| Python | 3.10+ | Primary language, entire codebase |

## Runtime & Framework

| Component | Technology | Version |
|-----------|-----------|---------|
| Web Framework | Streamlit | >=1.45.0 |
| ASGI | N/A (Streamlit handles) | — |
| Package Manager | pip (requirements.txt) | — |

## Core Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| openai | >=1.77.0 | LLM API client (OpenAI-compatible protocol) |
| google-generativeai | >=0.8.5 | Native Gemini integration (legacy) |
| edge-tts | 7.2.7 | Microsoft Edge TTS engine |
| azure-cognitiveservices-speech | >=1.37.0 | Azure Speech TTS |
| tencentcloud-sdk-python | >=3.0.1200 | Tencent Cloud TTS |
| dashscope | >=1.24.6 | Alibaba Qwen TTS |
| moviepy | 2.1.1 | Video editing/composition |
| pydub | 0.25.1 | Audio processing |
| pysrt | 1.1.2 | SRT subtitle parsing/generation |
| Pillow | >=10.3.0 | Image processing |
| pydantic | (transitive) | Data models/schema validation |
| loguru | >=0.7.3 | Structured logging |
| toml / tomli / tomli-w | >=2.2.1 | Configuration file parsing |
| streamlit | >=1.45.0 | Web UI framework |
| watchdog | 6.0.0 | File system monitoring |
| tqdm | >=4.66.6 | Progress bars |
| tenacity | >=9.0.0 | Retry logic |
| requests | >=2.32.0 | HTTP client |
| pyJianYingDraft | >=0.1.0 | JianYing (CapCut) draft export |

## Optional Dependencies (commented out)

| Library | Purpose |
|---------|---------|
| faster-whisper | Local speech recognition |
| opencv-python | Image processing |
| torch / torchvision / torchaudio | CUDA support, ML inference |

## External Tools

| Tool | Purpose |
|------|---------|
| FFmpeg | Video/audio processing (required, system-installed) |
| ffprobe | Media file metadata (bundled with FFmpeg) |

## Configuration

| File | Format | Purpose |
|------|--------|---------|
| `config.toml` | TOML | Runtime configuration (gitignored) |
| `config.example.toml` | TOML | Configuration template |
| `project_version` | Plain text | Single-line version string |
| `Dockerfile` | Docker | Container build |
| `docker-compose.yml` | YAML | Container orchestration |
| `Makefile` | Make | Build/deploy shortcuts |

## LLM Provider Architecture

All LLM interactions use **OpenAI-compatible protocol** (`/v1/chat/completions`).

Supported providers via unified endpoint:
- OpenAI (GPT-4o, GPT-4o-mini)
- DeepSeek (deepseek-chat, deepseek-reasoner)
- Google Gemini (via compatible gateway)
- Alibaba Qwen (qwen-plus, qwen-turbo)
- SiliconFlow
- Moonshot
- Anthropic
- Cohere
- Together AI
- OpenRouter

Model naming: `provider/model_name` (e.g., `gemini/gemini-2.0-flash-lite`)

## Python Environment

- No `pyproject.toml` or `setup.py` — flat `requirements.txt`
- No virtual environment management in repo
- Docker deployment available as alternative
