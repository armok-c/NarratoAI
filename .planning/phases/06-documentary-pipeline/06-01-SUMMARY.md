---
phase: 06-documentary-pipeline
plan: 01
subsystem: documentary
tags: [error-types, timestamp, srt, subtitle, pipeline-foundation]

requires:
  - phase: 05
    provides: ScriptClip, OstType, Script types
  - phase: 02
    provides: TtsOutput, WordBoundary types
  - phase: 01
    provides: thiserror error patterns, module structure

provides:
  - PipelineError enum with 10 variants covering all pipeline steps
  - DocumentaryRequest struct with 18 fields and default values
  - ProgressStep enum and ProgressCallback type
  - TtsResult struct for TTS output tracking
  - Timestamp parsing/conversion utilities (5 public functions)
  - SRT generation from word boundaries with offset support
  - SRT merge with time offset and sequential re-indexing
  - Documentary module skeleton with all sub-module stubs

affects: [06-02, 06-03, 06-04]

tech-stack:
  added: []
  patterns:
    - "PipelineError: thiserror derive with Chinese Display messages"
    - "Timestamp conversion: secs_to_srt_time / secs_to_ffmpeg_time dual format"
    - "SRT generation: word boundary 100ns units → seconds → HH:MM:SS,mmm"

key-files:
  created:
    - src/documentary/mod.rs
    - src/documentary/error.rs
    - src/documentary/types.rs
    - src/documentary/timestamp.rs
    - src/documentary/subtitle.rs
    - src/documentary/pipeline.rs
    - src/documentary/clip.rs
    - src/documentary/audio.rs
    - src/documentary/script_gen.rs
  modified:
    - src/lib.rs

key-decisions:
  - "PipelineError includes FFmpeg and LLM error variants beyond the plan spec (forward-looking for Plans 02/03)"
  - "Stub modules (pipeline, clip, audio, script_gen) created as empty files to allow compilation"

patterns-established:
  - "Chinese error messages: all PipelineError Display impls use Chinese text"
  - "100ns → seconds conversion: consistent with Python edge-tts SubMaker format"
  - "Dual time format: SRT uses comma (HH:MM:SS,mmm), FFmpeg uses dot (HH:MM:SS.mmm)"

---

## Plan 01: Foundation — Error Types, Request/State Structs, Timestamp Utilities

Created the documentary module skeleton with all shared types that Plans 02-04 depend on.

### Tasks Completed

1. **06-01-01** — Module skeleton, PipelineError, DocumentaryRequest, timestamp utilities
2. **06-01-02** — SRT generation from word boundaries, SRT merge with offsets

### Verification Results

- `cargo build` — PASS (0 errors)
- `cargo test --lib documentary` — PASS (38 tests, 0 failures)
- PipelineError: 10 variants, all with Chinese Display messages
- DocumentaryRequest: 18 fields with sensible defaults
- Timestamp functions: 5 public functions covering all formats
- SRT generation: word boundaries → valid SRT format
- SRT merge: offset application + sequential re-indexing

## Self-Check: PASSED
