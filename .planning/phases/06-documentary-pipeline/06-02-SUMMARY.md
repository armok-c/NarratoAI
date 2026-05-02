---
phase: 06-documentary-pipeline
plan: 02
subsystem: documentary, pipeline
tags: [pipeline, ffmpeg, tts, clipping, audio-merge, concat, composite]

requires:
  - phase: 06-01
    provides: PipelineError, DocumentaryRequest, timestamp utils, SRT generation

provides:
  - PipelineState struct for 6-step pipeline state
  - run_documentary() public async entry point
  - 6 step functions: load_script, tts, clip, merge_audio_subtitle, concat, composite
  - OST-aware clipping (3 strategies)
  - Audio merge with TTS delay positioning
  - Subtitle merge with cumulative offset
  - Video concatenation via FFmpeg concat demuxer
  - Final composite with audio mixing + subtitle burning

affects: [06-04]

tech-stack:
  added: []
  patterns:
    - "PipelineState pattern: mutable state threaded through sequential steps"
    - "OST dispatch: match on OstType for clipping strategy"
    - "FFmpeg filter graph: adelay+amix for audio positioning"

key-files:
  created: []
  modified:
    - src/documentary/pipeline.rs
    - src/documentary/clip.rs
    - src/documentary/audio.rs
    - src/documentary/mod.rs

key-decisions:
  - "Sequential TTS generation (not parallel) for simplicity; parallel TTS can be added later"
  - "FFmpeg-sidecar used for all FFmpeg operations, wrapped in run_ffmpeg for async"
  - "Composite step builds filter_complex string programmatically"

patterns-established:
  - "Step functions accept &mut PipelineState and return Result<(), PipelineError>"
  - "Progress callback emitted at each step with ProgressStep enum"

---

## Plan 02: 6-Step Pipeline Core

Implemented the complete 6-step documentary pipeline orchestration.

### Tasks Completed

1. **06-02-01** — PipelineState, 6 step functions, clip.rs, audio.rs, run_documentary()
2. **06-02-02** — Unit tests for clip duration, timestamp range, cumulative time range

### Verification Results

- `cargo build` — PASS (0 errors)
- `cargo test --lib documentary` — PASS (47 tests, 0 failures)
- PipelineState defined with all required fields
- 6 step functions accept &mut PipelineState
- 3 OST clipping strategies implemented
- Script update populates video, duration, edited_time_range

## Self-Check: PASSED
