# Feature Landscape

**Domain:** AI video narration + automated editing (desktop application)
**Researched:** 2026-04-27
**Context:** NarratoAI Rust rewrite -- feature audit from Python v0.7.8

## Table Stakes

Features users expect. Missing = product feels incomplete. These are the non-negotiable core of any AI video narration tool.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| LLM narration script generation | Core value proposition -- user provides video, gets narration text | Medium | Python has this via `generate_narration_script.py` + `ScriptGenerator`. Uses text LLM via OpenAI-compatible API. Rust rewrite needs async HTTP client + prompt templates. |
| TTS audio generation from script | No narration tool works without voice output | Medium | Python routes through `voice.py` to 7 engines. Edge-TTS (free) is default. Rust needs at minimum edge-tts protocol + one cloud TTS API. |
| Video clip cutting by timestamp | Basic editing operation -- split source into segments | Medium | `clip_video.py` uses FFmpeg subprocess. OST type (0/1/2) drives cutting strategy. Rust can use FFmpeg CLI binding or ffmpeg-next crate. |
| Audio-video merging | Final output must combine narration audio with video | Medium | `generate_video.py:merge_materials()` uses moviepy. Rust should use FFmpeg directly (more reliable than moviepy). |
| Subtitle generation + burn-in | Chinese content creators expect embedded subtitles | Medium | Python generates SRT from TTS word boundaries, burns via moviepy TextClip. Rust needs SRT generation + FFmpeg subtitles filter. |
| Background music mixing | Standard for narration videos | Low | Simple audio overlay with volume control. FFmpeg amix filter. |
| Project configuration (TOML) | Users need to set API keys, voice parameters, etc. | Low | Python uses `config.toml`. Rust uses `toml` crate, same pattern. |
| Task state + progress tracking | Users need to know if a 10-minute render is progressing | Low | Python has `MemoryState` dict. Rust can use file-based JSON state + Tauri events for real-time progress. |
| Video aspect ratio support | Chinese platforms use both 16:9 and 9:16 | Low | Enum with resolution mapping. Straightforward in Rust. |
| Script editing (JSON-based) | Users edit LLM output before generating video | Medium | Python loads/saves JSON scripts. Rust needs same JSON schema + CRUD operations. |

## Differentiators

Features that set this tool apart from generic video editors. Not universally expected, but highly valued by the target audience (Chinese short-video content creators).

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Vision-based frame analysis (Documentary mode) | Upload any video, AI watches it and writes narration automatically -- unique workflow | High | `DocumentaryFrameAnalysisService` extracts keyframes at intervals, sends to vision LLM in batches, generates structured analysis, then feeds to text LLM for narration. This is the flagship feature. Requires async batch processing with concurrency control. |
| Three distinct business modes (Documentary / SDE / SDP) | Covers different creator workflows -- narration, explanation, and highlight mixing | High | Each mode has its own pipeline: Documentary (frame analysis + narration), SDE (subtitle analysis + script editing), SDP (subtitle analysis + multi-clip mixing). Rust needs trait-based mode abstraction. |
| OST type system (original sound tracking) | Intelligent audio management per segment -- keep/remove/mix original sound | Medium | OST=0 (narration only), OST=1 (original audio only), OST=2 (mixed). Drives clipping, audio generation, and mixing strategy across the entire pipeline. Core differentiator from generic editors. |
| 7 TTS engines with unified routing | Choice of voice quality, cost, and language support | High | edge_tts (free), azure_speech, tencent_tts, soulvoice, tts_qwen, indextts2 (voice clone), doubaotts. Each has different API protocol, auth, and output format. Rust needs trait-based TTS provider system. |
| JianYing (CapCut) draft export | Export project to China's most popular video editor for manual refinement | Medium | `jianying_task.py` uses pyJianYingDraft library. Rust needs to generate JianYing draft JSON format directly (no Rust library exists -- need to reverse-engineer the format). |
| Prompt versioning system | Swap/upgrade LLM prompts without code changes | Medium | `PromptRegistry` with category/name/version structure, template rendering, validation. Valuable for A/B testing prompt quality. Straightforward in Rust with serde. |
| Visual analyzer (batch frame extraction + LLM analysis) | Automated video understanding -- extract keyframes, send to vision model | Medium | `VideoProcessor` extracts frames at intervals via FFmpeg, batches them for vision LLM API. Includes concurrency control (`vision_max_concurrency`). |
| Audio normalization (LUFS-based) | Professional audio loudness balancing | Medium | `AudioNormalizer` analyzes LUFS via FFmpeg loudnorm filter, adjusts levels. Makes output sound professional rather than amateur. |
| Smart volume control for mixed content | Auto-adjust volumes based on content type (narration vs original vs BGM) | Low | `AudioConfig.get_recommended_volumes_for_content()` provides presets. Rust equivalent is trivial. |
| YouTube video download as source | Pull source video directly from YouTube | Low | `YoutubeService` uses yt-dlp. Rust can shell out to yt-dlp or use rustube crate. |
| Pexels stock footage search | Find and download stock video clips by keyword | Low | `material.py` calls Pexels API. Simple HTTP client in Rust. |

## Anti-Features

Features to explicitly NOT build in the Rust rewrite. These are dead weight, legacy artifacts, or out of scope for the desktop app architecture.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Streamlit Web UI | Being replaced by Tauri + Vue 3 desktop app. Streamlit is Python-only, poor for desktop distribution. | Build Tauri command layer in this milestone, Vue 3 UI in subsequent milestone. |
| Redis state backend | Desktop single-user app has no need for distributed state. Over-engineering. | File-based JSON/TOML state only. If future need arises, add then. |
| Docker deployment | Desktop app bundles into a single installer. No container needed. | Tauri produces native installers (MSI/DMG/AppImage). |
| Legacy LLM interface (`llm.py`) | Python has TWO LLM code paths (`llm.py` old + `llm/` new) with `migration_adapter.py` bridging them. Carrying this forward makes no sense. | Clean single LLM service implementation in Rust. No migration adapter. |
| Gemini native SDK integration | Python has `gemini_analyzer.py` and `gemini_openai_analyzer.py` as legacy remnants. The new unified approach uses OpenAI-compatible protocol for everything. | OpenAI-compatible protocol only. All providers (including Gemini) go through the same endpoint. |
| moviepy dependency | Python uses moviepy for video composition and subtitle burn-in. It's slow, memory-hungry, and unreliable for large files. | Use FFmpeg directly via CLI bindings or ffmpeg-next crate. More reliable, faster, and doesn't load entire video into memory. |
| pydub dependency | Python uses pydub for audio manipulation. Unnecessary FFmpeg wrapper. | Use FFmpeg directly. Rust has better audio abstractions available. |
| faster-whisper local STT | Commented out in task.py, requires CUDA + large model download. Overkill for desktop app. | If needed later, use cloud-based STT API or whisper.cpp bindings. Do not include in initial rewrite. |
| HTTP API server | The Rust rewrite is a library + Tauri commands, not a web service. No REST endpoints needed. | Tauri invoke commands for all inter-process communication. |
| Multi-user / concurrent task management | Single-user desktop app. No auth, no user management, no concurrent task queue. | Sequential task processing. One task at a time. Simple state machine. |

## Feature Dependencies

```
Config System (TOML) --> LLM Service Layer --> Prompt System --> Script Generation
                                                                      |
                                                                      v
Config System --> TTS Engine Router --> Audio Generation ----------->+
                                |                                     |
                                v                                     v
                        FFmpeg Pipeline --> Video Clipping --> Audio/Subtitle Merge --> Clip Concatenation --> Final Composition
                                                                                                                        |
                                                                                                                        v
                                                                                                                JianYing Draft Export

Vision Analyzer --> LLM Vision API --> Frame Analysis JSON --> Text LLM --> Narration Script --> TTS --> ...
     |
     v
Video Processor (frame extraction via FFmpeg)

SDE Pipeline: Subtitle Text Parser --> LLM Analysis --> Script with OST types --> Pipeline above
SDP Pipeline: Subtitle Text Parser --> LLM Plot Analysis --> Multi-clip Script --> Pipeline above
```

Dependency chains for each mode:

```
Documentary: Video Upload -> Frame Extraction -> Vision LLM Analysis -> Text LLM Narration -> TTS -> FFmpeg Pipeline -> Output
SDE: Video + Subtitle Upload -> Subtitle Parse -> LLM Plot Analysis -> Script with OST -> TTS -> FFmpeg Pipeline -> Output
SDP: Video + Subtitle Upload -> Subtitle Parse -> LLM Plot Analysis -> Multi-clip Script -> TTS -> FFmpeg Pipeline -> Output
```

## MVP Recommendation

Prioritize in this order for the Rust rewrite:

1. **Configuration system** (TOML loading, validation) -- everything depends on this
2. **FFmpeg pipeline core** (clip, merge, mix, subtitle burn-in) -- replaces moviepy/pydub, fundamental to all modes
3. **LLM service layer** (OpenAI-compatible, Vision + Text) -- enables script generation
4. **TTS routing** (start with edge-tts only, add engines incrementally) -- enables voice output
5. **Documentary mode pipeline** (frame extraction + vision analysis + narration) -- flagship feature, validates entire architecture
6. **Script editing** (JSON load/save/edit) -- users need to review/modify LLM output
7. **SDE mode** (subtitle analysis + script with OST types) -- second business mode
8. **SDP mode** (multi-clip mixing) -- third business mode

Defer:
- **JianYing draft export**: Medium complexity, niche use case. Implement after core pipeline works.
- **5 additional TTS engines beyond edge-tts**: Add one at a time based on demand. Edge-tts covers 80% of users (free + good quality).
- **YouTube download**: Convenience feature, not core. yt-dlp subprocess works as interim.
- **Pexels stock footage**: Nice-to-have for a future milestone.
- **Audio normalization (LUFS)**: Professional feature. Basic volume control sufficient for MVP.

## Rust Rewrite Feature Priority Matrix

| Feature | User Impact | Technical Risk | Implementation Order |
|---------|-------------|----------------|---------------------|
| TOML config | Critical | Low | Phase 1 |
| FFmpeg core operations | Critical | Medium | Phase 1 |
| LLM service (OpenAI-compatible) | Critical | Medium | Phase 1 |
| Edge-TTS integration | Critical | Medium | Phase 2 |
| Documentary pipeline | High | High | Phase 2 |
| JSON script management | High | Low | Phase 2 |
| SDE pipeline | High | Medium | Phase 3 |
| SDP pipeline | High | Medium | Phase 3 |
| JianYing export | Medium | Medium | Phase 3 |
| Additional TTS engines | Medium | Low (repetitive) | Phase 4 |
| Audio normalization | Low | Low | Phase 4 |
| YouTube download | Low | Low | Phase 4 |
| Pexels integration | Low | Low | Phase 4 |

## Feature Completeness vs Python Version

| Python Feature | Rust MVP | Rust Full | Notes |
|----------------|----------|-----------|-------|
| TOML config loading | Yes | Yes | Direct port |
| FFmpeg video clipping | Yes | Yes | Better -- direct FFmpeg, no moviepy |
| FFmpeg video merging | Yes | Yes | Direct port |
| FFmpeg audio mixing | Yes | Yes | Direct port |
| Subtitle generation (SRT) | Yes | Yes | From TTS word boundaries |
| Subtitle burn-in | Yes | Yes | FFmpeg subtitles filter |
| BGM mixing | Yes | Yes | FFmpeg amix filter |
| LLM vision analysis | Yes | Yes | OpenAI-compatible HTTP |
| LLM text generation | Yes | Yes | OpenAI-compatible HTTP |
| Prompt versioning | Yes | Yes | Structured prompt registry |
| Edge-TTS | Yes | Yes | Protocol implementation |
| Azure Speech TTS | No | Yes | Phase 4 |
| Tencent TTS | No | Yes | Phase 4 |
| SoulVoice TTS | No | Yes | Phase 4 |
| Qwen TTS | No | Yes | Phase 4 |
| IndexTTS2 (voice clone) | No | Yes | Phase 4 |
| Doubao TTS | No | Yes | Phase 4 |
| Documentary mode | Yes | Yes | Flagship feature |
| SDE mode | Yes | Yes | Phase 3 |
| SDP mode | Yes | Yes | Phase 3 |
| JianYing export | No | Yes | Phase 3 |
| YouTube download | No | Yes | Phase 4 |
| Pexels stock footage | No | Yes | Phase 4 |
| Redis state | No | No | Anti-feature |
| Docker deployment | No | No | Anti-feature |
| Streamlit UI | No | No | Anti-feature |
| Vision batch concurrency control | Yes | Yes | tokio semaphore |
| OST type system | Yes | Yes | Core pipeline feature |
| Audio normalization | No | Yes | Phase 4 |
| FFmpeg hardware acceleration detection | Yes | Yes | Port FFmpegConfigManager |
| Smart volume control | Yes | Yes | Volume presets |
| Task state/progress | Yes | Yes | File-based JSON |

## Sources

- NarratoAI Python codebase v0.7.8 -- direct code analysis
- `.planning/codebase/ARCHITECTURE.md` -- architecture mapping
- `.planning/codebase/CONCERNS.md` -- existing technical debt and pain points
- `.planning/codebase/STRUCTURE.md` -- directory layout and file inventory
- `.planning/codebase/INTEGRATIONS.md` -- external service integrations
- `.planning/PROJECT.md` -- project scope and constraints
