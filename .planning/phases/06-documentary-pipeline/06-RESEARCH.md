# Phase 6: Documentary Pipeline — Research

**Phase:** 6 - Documentary Pipeline
**Researched:** 2026-05-02
**Status:** Complete

---

## 1. Domain Understanding

### What Phase 6 Delivers

Phase 6 implements the **flagship feature** of NarratoAI — the complete documentary video production pipeline. It has two distinct subsystems:

1. **6-Step Video Processing Pipeline** (`run_documentary()`): Sequential orchestration that takes a pre-edited script and produces a final composite video. Steps: Load Script → TTS Generation → Video Clipping → Audio/Subtitle Merge → Clip Concatenation → Final Composite.

2. **Frame Analysis → Script Generation Pre-processor** (`generate_documentary_script()`): Independent function that extracts keyframes from a video, analyzes them via vision LLM, then generates a narration script via text LLM. Users call this first, edit the result, then feed it into the pipeline.

### Why This Phase Is Complex

- **Cross-cutting integration**: Phase 6 is where ALL prior modules (FFmpeg, TTS, LLM, Script, Prompt, Visual) converge for the first time.
- **OST-driven branching**: Three distinct audio/video processing paths (OST=0/1/2) affect clipping, audio merging, and final composite differently.
- **FFmpeg filter chain complexity**: Python version's step 6 (final composite) uses moviepy for audio mixing + subtitle burning. Rust version must replicate this with pure FFmpeg filter chains.
- **Timestamp arithmetic**: Multiple time formats (HH:MM:SS, HH:MM:SS,sss, seconds float) used across script timestamps, video clips, TTS durations, and SRT timings.

## 2. Python Reference Analysis

### Core Pipeline (`app/services/task.py`)

The Python `start_subclip_unified()` function is the authoritative reference:

| Step | Python Function | Rust Mapping | Key Complexity |
|------|----------------|--------------|----------------|
| 1. Load Script | `json.load(f)` | `script::load_script()` | Direct reuse — Phase 5 |
| 2. TTS Generation | `voice.tts_multiple()` | `tts::synthesize()` in loop | Filter OST=0/2 only; batch call |
| 3. Video Clipping | `clip_video.clip_video_unified()` | New `step_clip()` | OST=0: TTS-duration crop, remove audio; OST=1: timestamp crop, keep audio; OST=2: TTS-duration crop, keep audio |
| 4. Audio/Subtitle Merge | `audio_merger.merge_audio_files()` + `subtitle_merger.merge_subtitle_files()` | New `step_merge_audio_subtitle()` | Time-positioned overlay for audio; time-offset for subtitles |
| 5. Clip Concatenation | `merger_video.combine_clip_videos()` | New `step_concat()` | Resolution normalization + concat demuxer + OST-aware audio handling |
| 6. Final Composite | `generate_video.merge_materials()` | New `step_composite()` | Audio mixing (TTS+BGM+original) + subtitle burning |

### OST Strategy Details (from `clip_video.py`)

- **OST=0 (NarrationOnly)**: Start time from `timestamp`, end time calculated from TTS `duration`. Audio removed (`-an`). File prefix `ost0_vid_`.
- **OST=1 (OriginalSound)**: Strictly use `timestamp` start-end. Audio kept (`-c:a aac`). File prefix `ost1_vid_`.
- **OST=2 (Mixed)**: Start time from `timestamp`, end time from TTS `duration`. Audio kept. File prefix `ost2_vid_`.

### Script Timestamp Update (`update_script.py`)

After clipping, each ScriptClip gets enriched with:
- `video`: path to clipped video
- `audio`: path to TTS audio
- `subtitle`: path to SRT fragment
- `sourceTimeRange`: actual clipped time range (from filename)
- `duration`: calculated from sourceTimeRange
- `editedTimeRange`: cumulative position in final video (00:00:00-based)

This update logic is critical — it connects step 3's output to steps 4/5/6's input.

### Audio Merging (`audio_merger.py`)

Uses pydub's `AudioSegment`:
1. Creates silent base of total_duration
2. Overlays each TTS audio at position = accumulated_duration
3. OST=1 segments get silence gap (no TTS audio to overlay)
4. Exports as MP3

Rust approach: Use FFmpeg `adelay` + `amix` filters or build a silence base + overlay approach using ffmpeg-sidecar.

### Subtitle Merging (`subtitle_merger.py`)

1. Sorts by `editedTimeRange` start
2. For each SRT fragment: parse blocks, apply time offset from `editedTimeRange`, reindex
3. Output single merged SRT file

This is pure string/date manipulation — straightforward Rust port.

### Video Concatenation (`merger_video.py`)

Complex multi-step process:
1. Process each clip to target resolution (scale+pad) with/without audio
2. Concat all video streams (no audio) via concat demuxer
3. For clips with OST>0: extract audio segments with `adelay` timing
4. Create silence base + mix delayed audio tracks via `amix`
5. Merge video+audio streams

**Rust simplification**: Since step 3 already handles OST-based audio removal/keeping per clip, the concat step can be simpler — either pass-through (copy codec) if clips are already normalized, or re-encode with resolution normalization.

### Final Composite (`generate_video.py`)

Uses moviepy (Python video library):
1. Load concatenated video
2. Extract original audio track (if keep_original_audio)
3. Mix: TTS audio + original audio + BGM (with volume control)
4. Burn subtitles (TextClip with font/size/color/position)
5. Export final video

**Rust approach**: Single FFmpeg command with complex `-filter_complex`:
- `[0:a]` = TTS audio → volume filter
- `[1:a]` = original audio → volume filter  
- `[2:a]` = BGM → loop + volume + fadeout
- `amix` all audio tracks
- `subtitles` filter for SRT burning
- Hardware-accelerated encoding

### Frame Analysis → Script Generation

Two-step LLM process in `DocumentaryFrameAnalysisService`:

**Step A — `analyze_video()`**:
1. Extract keyframes via FFmpeg (cached by video hash + interval)
2. Chunk into batches of N frames
3. Send each batch to vision LLM with analysis prompt
4. Parse structured JSON response (frame_observations + overall_activity_summary)
5. Save analysis artifact to JSON

**Step B — `generate_narration()`**:
1. Convert analysis JSON to markdown
2. Send to text LLM with narration_generation prompt template
3. Parse JSON response with items array → list of ScriptClip-like dicts
4. Default OST=2 for all generated segments

## 3. Rust Codebase Integration Points

### Existing Modules to Reuse

| Module | What Phase 6 Uses | Location |
|--------|-------------------|----------|
| `script::types` | `ScriptClip`, `OstType`, `Script` | `src/script/types.rs` |
| `script` | `load_script()`, `save_script()`, `validate()` | `src/script/mod.rs` |
| `tts` | `TtsProvider::synthesize()`, `synthesize()` router, `TtsOutput` (with `word_boundaries`) | `src/tts/mod.rs` |
| `ffmpeg::command` | `run_ffmpeg()`, `clip_video()`, `ProgressCallback` | `src/ffmpeg/command.rs` |
| `ffmpeg::probe` | Video info, duration, audio detection | `src/ffmpeg/probe.rs` |
| `ffmpeg::hwaccel` | Hardware acceleration detection | `src/ffmpeg/hwaccel.rs` |
| `llm::provider` | `LlmProvider` trait (text + vision) | `src/llm/provider.rs` |
| `prompt::manager` | `PromptManager::get_prompt()`, `render()` | `src/prompt/manager.rs` |
| `visual::frame_extractor` | `extract_frames()` | `src/visual/frame_extractor.rs` |
| `visual::analyzer` | Batch frame analysis with LLM | `src/visual/analyzer.rs` |
| `config::types` | `AppConfig` (TTS engine, voice, FFmpeg params) | `src/config/types.rs` |

### New Code to Create

| File | Purpose |
|------|---------|
| `src/documentary/mod.rs` | Module entry, public API |
| `src/documentary/pipeline.rs` | `PipelineState`, `run_documentary()`, 6 step functions |
| `src/documentary/error.rs` | `PipelineError` enum (thiserror) |
| `src/documentary/types.rs` | `DocumentaryRequest`, `ProgressCallback` type alias |
| `src/documentary/subtitle.rs` | SRT generation from word boundaries, merge with time offsets |
| `src/documentary/audio.rs` | Audio merging (silence base + TTS overlay) |
| `src/documentary/script_gen.rs` | `generate_documentary_script()` — frame analysis → LLM narration |
| `src/documentary/clip.rs` | OST-aware video clipping (3 strategies) |

### Dependencies Already Available

From `Cargo.toml`:
- `ffmpeg-sidecar` (2.5.1) — FFmpeg CLI wrapper
- `tokio` (full) — async runtime
- `serde`/`serde_json` — JSON handling
- `thiserror` (2.0) — error derivation
- `tempfile` (dev-dep) — test directories
- `uuid` — unique task IDs
- `regex` — timestamp parsing
- `chrono` — time arithmetic

**New dependency needed**: None identified. All functionality can be built with existing crates + FFmpeg CLI.

## 4. Technical Challenges & Risks

### High Risk

1. **FFmpeg filter chain construction for step 6**: Python uses moviepy for audio mixing + subtitle burning. Rust must replicate with FFmpeg's `-filter_complex`. The filter chain varies based on: has TTS audio? has original audio? has BGM? has subtitles? That's 2^4 = 16 combinations. Must test all paths.

2. **Timestamp format handling**: Python code juggles 3 formats:
   - `HH:MM:SS` (script timestamp)
   - `HH:MM:SS,sss` (SRT format, comma for milliseconds)
   - Float seconds (TTS duration, FFmpeg seek)
   Rust must have robust parsing/conversion utilities.

3. **TTS duration → video clip length mapping**: OST=0/2 clips use TTS audio duration to determine video clip end time. If TTS returns inaccurate duration, clips may be too short/long. Python adds `extra_seconds=0` (no padding) for OST=0/2. Edge cases when TTS duration > timestamp range.

### Medium Risk

4. **Concatenation with mixed audio states**: Some clips have audio (OST=1,2), some don't (OST=0). FFmpeg concat demuxer requires all streams to match. Python handles this by extracting and re-mixing audio separately. Rust must handle this carefully.

5. **Word boundary → SRT generation**: Edge-TTS returns word boundaries in 100-nanosecond units. Must convert to SRT's `HH:MM:SS,mmm` format. Python's `submaker` handles this; Rust must implement equivalent logic.

6. **Progress tracking granularity**: Python updates state at fixed points (0%, 20%, 60%, 80%, 100%). Rust must provide same granularity with `ProgressCallback`.

### Low Risk

7. **Frame analysis → script generation**: This subsystem calls existing Rust modules (visual, llm, prompt) directly. Main complexity is in LLM output parsing (JSON repair logic for malformed responses).

8. **Task directory management**: Python uses `utils.task_dir(task_id)` for intermediate files. Rust can use `std::env::temp_dir()` + task ID or a configurable output directory.

## 5. Validation Architecture

### Test Strategy

Given the pipeline's FFmpeg dependency, testing requires:

1. **Unit tests** (no FFmpeg needed):
   - Timestamp parsing/conversion
   - SRT generation from word boundaries
   - Script update logic (duration calculation, editedTimeRange)
   - LLM narration output parsing
   - PipelineError display messages (Chinese)

2. **Integration tests** (FFmpeg required):
   - End-to-end pipeline with a short test video (3-5 seconds)
   - Each OST type (0, 1, 2) tested independently
   - Mixed OST script (all three types)
   - Audio merge correctness (TTS positioned at correct offsets)
   - Subtitle merge correctness (time offsets applied)
   - Final composite output is valid video file

3. **Test fixtures needed**:
   - Short test video file (or generate via FFmpeg in test setup)
   - Pre-recorded TTS audio (or mock TTS provider)
   - Sample ScriptClip JSON with all 3 OST types

### Verification Checkpoints

Per success criteria from ROADMAP.md:

| # | Criterion | How to Verify |
|---|-----------|---------------|
| 1 | 6-step pipeline runs end-to-end | Integration test: provide script + video → get output video |
| 2 | OST branching correct | Test each OST type independently, check audio tracks via ffprobe |
| 3 | Frame analysis → narration | Test generate_documentary_script with mock LLM |
| 4 | SRT from word boundaries | Unit test: word boundaries → SRT string, verify timestamps |
| 5 | Progress tracking | Integration test: verify callback receives all progress updates |

## 6. Implementation Recommendations

### Suggested Wave Structure

**Wave 1** (Foundation — no blockers):
- Plan 01: Error types + DocumentaryRequest + PipelineState + timestamp utilities + module setup
- Plan 02: SRT generation (word boundaries → SRT) + subtitle merge

**Wave 2** (Depends on Wave 1):
- Plan 03: 6-step pipeline core (step_load_script, step_tts, step_clip, step_merge_audio_subtitle, step_concat, step_composite + run_documentary)
- Plan 04: Frame analysis → script generation (generate_documentary_script)

**Wave 3** (Depends on Wave 2):
- Plan 05: Integration tests (end-to-end pipeline + OST branching + progress tracking)

### Key Design Decisions for Planner

1. **FFmpeg command construction**: Use `ffmpeg-sidecar`'s `FfmpegCommand` builder for type safety where possible, fall back to raw arg construction for complex filter chains.

2. **Intermediate file management**: Use a task directory pattern (`{output_dir}/{task_id}/`) for all intermediate files (TTS audio, clips, merged audio/subtitle). Clean up optional.

3. **Async TTS batch**: Use `futures::stream::iter(...).buffer_unordered(concurrency)` for parallel TTS generation, matching Python's batch pattern.

4. **Error propagation**: Each step returns `Result<(), PipelineError>`. `PipelineError` wraps domain-specific errors via `From<DomainError>` impls.

5. **Subtitle generation**: Generate SRT fragments immediately after each TTS call (step 2), storing in `TtsResult`. This matches D-09 from CONTEXT.md.

---

## RESEARCH COMPLETE

**Key findings:**
- Python reference code is comprehensive and well-documented — 1:1 alignment is feasible
- No new dependencies needed — all functionality buildable with existing crates + FFmpeg
- Main complexity is FFmpeg filter chain construction for the final composite step
- Suggested 5 plans in 3 waves: foundation → pipeline core + script gen → integration tests
- Word boundary → SRT conversion must handle 100-nanosecond units from TTS
