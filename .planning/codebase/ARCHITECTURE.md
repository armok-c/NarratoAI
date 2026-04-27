# Architecture

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Pattern

**Layered monolith** with Streamlit Web UI as the single entry point.

```
webui.py (Streamlit Entry)
  → webui/components/ (UI Panel Components)
    → webui/tools/ (Business Tool Handlers)
      → app/services/ (Business Logic Layer)
        → app/services/llm/ (LLM Provider Abstraction)
        → app/services/prompts/ (Versioned Prompt Registry)
        → app/services/documentary/ | SDE/ | SDP/ (3 Business Domains)
      → app/models/ (Pydantic Data Models + Constants)
      → app/config/ (TOML Config Loading)
      → app/utils/ (FFmpeg, Video Processing, Vision Analyzers)
```

## Layers

### 1. Presentation Layer
- `webui.py` — Streamlit app entry, page config, session state initialization
- `webui/components/` — UI panels (basic_settings, audio_settings, script_settings, subtitle_settings, video_settings, system_settings, ffmpeg_diagnostics)
- `webui/tools/` — Business tool handlers that bridge UI to services
- `webui/config/` — WebUI-specific configuration (settings)
- `webui/utils/` — File utils, caching, vision analyzer

### 2. Service Layer
- `app/services/task.py` — **Core pipeline orchestrator** (`start_subclip_unified()`)
- `app/services/voice.py` — TTS routing hub
- `app/services/llm/` — LLM service abstraction (Provider/Registry/Manager pattern)
- `app/services/prompts/` — Prompt versioning and management
- `app/services/documentary/` — Documentary mode (frame analysis + narration)
- `app/services/SDE/` — Short Drama Explanation mode
- `app/services/SDP/` — Short Drama Mix mode
- `app/services/state.py` — Abstract state management (Memory/Redis)
- `app/services/jianying_task.py` — JianYing draft export

### 3. Model Layer
- `app/models/schema.py` — Pydantic models (VideoClipParams, VideoAspect, etc.)
- `app/models/const.py` — Constants (task states, etc.)
- `app/models/exception.py` — Custom exceptions

### 4. Infrastructure Layer
- `app/config/config.py` — TOML config loading
- `app/config/defaults.py` — Default config values and normalization
- `app/config/ffmpeg_config.py` — FFmpeg hardware acceleration detection
- `app/config/audio_config.py` — Audio configuration
- `app/utils/ffmpeg_utils.py` — FFmpeg wrapper
- `app/utils/video_processor.py` — Video frame extraction
- `app/utils/utils.py` — General utilities

## Core Pipeline

Video processing pipeline in `app/services/task.py:start_subclip_unified()`:

```
1. Load Script → Read JSON script with narrations, timestamps, OST markers
2. TTS Generation → Generate voice narration per segment
3. Video Clipping → Cut video segments based on OST type:
   - OST=0: Narration only, clip by TTS duration, remove original audio
   - OST=1: Original audio only, clip by script timestamps precisely
   - OST=2: Mixed, clip by TTS duration, keep original audio
4. Audio/Subtitle Merge → Combine TTS, original audio, subtitles
5. Clip Concatenation → Merge all segments
6. Final Composition → Add background music, burn subtitles
```

## LLM Service Architecture

```
webui.py:main() → providers/__init__.py:register_all_providers()
                         ↓
           LLMServiceManager (factory + cache)
                         ↓
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
 VisionModelProvider  TextModelProvider  (extensible)
        ↓                ↓
 OpenAICompatibleProvider (single concrete impl)
   - analyze_images()   - generate_text()
   - batch processing   - structured output
        ↓                ↓
           UnifiedLLMService (facade)
                ↓
        migration_adapter.py (bridge to legacy API)
```

## Prompt System Architecture

```
PromptRegistry (category → name → version → prompt)
  ↑ register()
  │
  ├── documentary/ (frame_analysis, narration_generation)
  ├── short_drama_editing/ (plot_extraction, subtitle_analysis)
  └── short_drama_narration/ (plot_analysis, script_generation)
  │
  ↓ lookup
PromptManager → render templates → validate → return to caller
```

## Data Flow

```
User uploads video
  → webui (Streamlit)
    → video_processor extracts keyframes
      → LLM vision model analyzes frames
        → Prompt system renders analysis prompt
          → Analysis results → narration script
            → LLM text model generates script
              → Script saved as JSON
                → TTS generates audio
                  → FFmpeg clips video segments
                    → Audio/subtitle merge
                      → Final video composition
                        → Output video file
```

## State Management

- `MemoryState` (default): In-memory dict, single-process
- `RedisState` (optional): Distributed, multi-process capable
- Both implement `BaseState` ABC with `update_task()`, `get_task()`, `delete_task()`

## Entry Points

| Entry | File | Purpose |
|-------|------|---------|
| Web UI | `webui.py` | Streamlit application |
| CLI | `run.bat` | Windows launcher for Streamlit |
| Docker | `docker-compose.yml` | Container deployment |
