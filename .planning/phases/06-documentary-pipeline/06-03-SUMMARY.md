---
phase: 06-documentary-pipeline
plan: 03
subsystem: documentary, script-generation
tags: [vision-llm, frame-analysis, narration-generation, json-repair]

requires:
  - phase: 06-01
    provides: PipelineError, timestamp utils
  - phase: 04
    provides: frame_extractor, analyzer, LlmProvider
  - phase: 05
    provides: PromptManager

provides:
  - generate_documentary_script() public async entry point
  - ScriptGenRequest struct
  - FrameAnalysisResult struct
  - analyze_video: keyframe extraction + vision LLM analysis
  - generate_narration: text LLM → ScriptClip[]
  - convert_analysis_to_markdown: analysis → markdown
  - JSON repair: code fences, trailing commas, single quotes, object extraction

affects: [06-04]

tech-stack:
  added: []
  patterns:
    - "ScriptGenRequest: no derive(Debug, Clone) due to Fn closure"
    - "JSON repair chain: strip fence → direct parse → extract object → basic repairs"

key-files:
  created: []
  modified:
    - narratoai-core/src/documentary/script_gen.rs
    - narratoai-core/src/documentary/mod.rs

key-decisions:
  - "ScriptGenRequest.progress uses simple Fn(f32, &str) instead of ProgressStep enum"
  - "JSON repair handles 3 malformation patterns: code fences, trailing commas, single quotes"

---

## Plan 03: Frame Analysis → Script Generation

Implemented the pre-processing pipeline for generating documentary scripts from video analysis.

### Tasks Completed

1. **06-03-01** — ScriptGenRequest, analyze_video, generate_narration, convert_analysis_to_markdown, generate_documentary_script
2. **06-03-02** — Unit tests for markdown conversion, JSON repair, script parsing

### Verification Results

- `cargo build` — PASS (0 errors)
- `cargo test --lib documentary` — PASS (56 tests, 0 failures)
- generate_documentary_script is public async function
- analyze_video calls visual::frame_extractor and LlmProvider
- JSON repair handles code fences, trailing commas, single quotes
- Markdown conversion produces "## 片段" headers

## Self-Check: PASSED
