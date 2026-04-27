# Integrations

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## LLM Services

| Service | Protocol | Config Section | Files |
|---------|----------|----------------|-------|
| Vision Models | OpenAI-compatible REST | `[app]` vision_* | `app/services/llm/openai_compatible_provider.py` |
| Text Models | OpenAI-compatible REST | `[app]` text_* | `app/services/llm/openai_compatible_provider.py` |
| Gemini (native) | Google GenerativeAI SDK | (legacy) | `app/utils/gemini_analyzer.py`, `app/utils/gemini_openai_analyzer.py` |

**Key files:**
- `app/services/llm/unified_service.py` — Unified facade (analyze_images, generate_text)
- `app/services/llm/manager.py` — Factory + caching for providers
- `app/services/llm/providers/__init__.py` — register_all_providers()
- `app/services/llm/migration_adapter.py` — New/old interface bridge

## TTS (Text-to-Speech) Engines

| Engine | Provider | Config Section | Status |
|--------|----------|----------------|--------|
| edge_tts | Microsoft Edge (free) | `[ui]` edge_* | Default |
| azure_speech | Azure Cognitive Services | `[azure]` | Optional |
| tencent_tts | Tencent Cloud | `[tencent]` | Optional |
| soulvoice | SoulVoice API | `[soulvoice]` | Optional |
| tts_qwen | Alibaba Qwen3 TTS | `[tts_qwen]` | Optional |
| indextts2 | Self-hosted voice clone | `[indextts2]` | Optional |
| doubaotts | ByteDance Doubao | `[doubaotts]` | Optional |

**Routing:** `app/services/voice.py:tts()` dispatches by engine name string.

## Video Processing

| Integration | Purpose | Files |
|-------------|---------|-------|
| FFmpeg | Video clipping, merging, audio mixing | `app/utils/ffmpeg_utils.py`, system binary |
| moviepy | Video composition, subtitle burn-in | `app/services/generate_video.py`, `app/services/merger_video.py` |
| pydub | Audio manipulation | `app/services/audio_merger.py`, `app/services/audio_normalizer.py` |

## JianYing (CapCut) Export

| Integration | Purpose | Files |
|-------------|---------|-------|
| pyJianYingDraft | Export to JianYing draft format | `app/services/jianying_task.py` |

## State Management

| Backend | Purpose | Files |
|---------|---------|-------|
| Memory (dict) | Default, in-memory task state | `app/services/state.py:MemoryState` |
| Redis | Optional distributed task state | `app/services/state.py:RedisState` |

Config: `[app]` enable_redis, redis_host, redis_port, redis_db, redis_password

## YouTube

| Integration | Purpose | Files |
|-------------|---------|-------|
| YouTube Service | Video upload/processing (referenced) | `app/services/youtube_service.py` |

## Proxy Support

| Feature | Config | Files |
|---------|--------|-------|
| HTTP/HTTPS Proxy | `[proxy]` http, https, enabled | Used by requests/openai clients |

## CI/CD

| Service | Purpose | Files |
|---------|---------|-------|
| GitHub Actions | Auto-release, code review, Discord notifications | `.github/workflows/` |
| Docker Hub | Container deployment | `Dockerfile`, `docker-compose.yml` |
