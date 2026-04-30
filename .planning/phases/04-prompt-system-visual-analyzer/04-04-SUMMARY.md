---
phase: 04-prompt-system-visual-analyzer
plan: 04
type: execute
subsystem: visual
tags:
  - visual-analyzer
  - batch-orchestration
  - module-integration
  - serde
requires:
  - "Phase 2 LlmProvider::analyze_images()"
  - "Phase 1 ProgressCallback / extract_frames"
  - "Plan 02 (04-02): visual/error.rs, visual/types.rs, visual/frame_extractor.rs"
  - "Plan 03 (04-03): prompt/*"
provides:
  - "src/visual/analyzer.rs — analyze_video_frames() full pipeline"
  - "src/lib.rs — pub mod prompt; pub mod visual; integration"
affects: []
tech-stack:
  added: []
  patterns:
    - "serde_json::from_str<FrameObservation>() with markdown fence stripping"
    - "error collection: per-batch failures aggregated, not aborting"
    - "online guardrails: zero-frame check + empty result barrier"
key-files:
  created:
    - "src/visual/analyzer.rs (422 lines)"
  modified:
    - "src/visual/mod.rs (added pub mod analyzer)"
    - "src/lib.rs (added pub mod prompt; pub mod visual;)"
  restored:
    - "src/visual/error.rs (Rule 3)"
    - "src/visual/types.rs (Rule 3)"
    - "src/visual/frame_extractor.rs (Rule 3)"
    - "src/prompt/mod.rs, types.rs, error.rs, registry.rs, template.rs (Rule 3)"
    - "Cargo.toml (removed duplicate uuid, added regex)"
decisions:
  - "D-14: Error collection — single batch JSON parse failure does not abort other batches"
  - "D-19: Hybrid prompt mode — JSON schema hardcoded in analyzer, analysis prompt from parameter"
  - "VisualError::BatchPartial returned when all batches fail to parse"
fueatures: []
metrics:
  duration: ""
  completed_date: "2026-04-30"
---

# Phase 4 Plan 4: Visual Analysis Orchestrator + Module Integration Summary

**One-liner:** Batch visual analysis orchestrator connecting frame extraction to LLM vision analysis with JSON parsing, error collection, and serde deserialization guardrails, integrated into the crate through lib.rs module declarations.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| - | Restore Plan 02 visual module (Rule 3) | d131134 | src/visual/error.rs, types.rs, frame_extractor.rs, mod.rs |
| 1 | Create visual analysis orchestrator (analyzer.rs) | f847dab | src/visual/analyzer.rs |
| - | Restore Plan 03 prompt module + fix Cargo.toml (Rule 3) | 5f27939 | src/prompt/*, Cargo.toml |
| 2 | Module integration (lib.rs) | d806497 | src/lib.rs |
| - | Suppress unused variable warning | eff665b | src/visual/analyzer.rs |

### Task 1: Visual Analysis Orchestrator (analyzer.rs)

Created `src/visual/analyzer.rs` (422 lines) with three public APIs:

- **`analyze_video_frames()`** — Full 9-step pipeline orchestration:
  1. Progress callback (0%: "开始帧提取")
  2. Frame extraction via `extract_frames()` (3s interval, default quality)
  3. **Online guardrail**: Zero-frame check — returns `VisualError::FrameExtraction`
  4. Collect frame paths matching `keyframe_*.jpg`
  5. Progress callback (30%: "提取到 N 帧，开始分析")
  6. LLM vision analysis via `LlmProvider::analyze_images()` with hardcoded JSON schema prompt (D-19 hybrid mode), low temperature (0.1), max_tokens 4096
  7. Batch JSON deserialization with error collection (D-14)
  8. **Online guardrail**: Empty result barrier — all batches failed returns `VisualError::BatchPartial`
  9. Return `BatchAnalysisResult` with tracing info event

- **`parse_and_retry()`** — JSON deserialization with markdown code block stripping:
  - Defensive cleaning: strips ` ```json ` / ` ``` ` fences
  - Retries up to `max_attempts` (default 2)
  - Returns `VisualError::Analysis` on exhaustion

- **`collect_frame_paths()`** — Collects `keyframe_*.jpg` from `output_dir`, sorted, errors on empty

**Tests:** 8 unit tests covering:
- `test_parse_and_retry_valid_json` — valid JSON
- `test_parse_and_retry_code_block_wrapped` — ` ```json ` fence stripping
- `test_parse_and_retry_backtick_wrapped` — ` ``` ` fence stripping
- `test_parse_and_retry_invalid_json` — plain text → `VisualError::Analysis`
- `test_parse_and_retry_max_attempts` — retry limit validation
- `test_collect_frame_paths` — correct collection from temp dir
- `test_collect_frame_paths_empty` — empty dir → error
- `test_collect_frame_paths_filters_non_matching` — non-keyframe files filtered out

### Task 2: Module Integration (mod.rs + lib.rs)

- `src/visual/mod.rs`: Contains `pub mod analyzer; pub mod error; pub mod frame_extractor; pub mod types;`
- `src/lib.rs`: Added `pub mod prompt;` and `pub mod visual;` after `pub mod script;`

## Deviations from Plan

### Rule 3 — Auto-fix Blocking Issues

**1. Missing Plan 02/03 dependency files restored**

- **Found during:** Pre-execution worktree check
- **Issue:** The worktree HEAD commit (`866d193`) does not contain files from Plans 02 and 03, which are required dependencies. `src/visual/` and `src/prompt/` directories were absent.
- **Fix:** Restored all 4 visual module files (error.rs, types.rs, frame_extractor.rs, mod.rs) and all 5 prompt module files (mod.rs, types.rs, error.rs, registry.rs, template.rs) from the base commit `ea447066`.
- **Files created:**
  - `src/visual/error.rs` — VisualError enum (FrameExtraction/Analysis/BatchPartial)
  - `src/visual/types.rs` — FrameObservation, BatchAnalysisResult, strip_code_fence
  - `src/visual/frame_extractor.rs` — extract_frames, fast path, 4-level fallback
  - `src/visual/mod.rs` — module declarations
  - `src/prompt/mod.rs` — module declarations
  - `src/prompt/types.rs` — ModelType, OutputFormat, PromptMetadata, Prompt
  - `src/prompt/error.rs` — PromptError enum
  - `src/prompt/registry.rs` — PromptRegistry with 3-level index
  - `src/prompt/template.rs` — regex-based render with 6 filters
- **Commits:** d131134, 5f27939

**2. Duplicate `uuid` in Cargo.toml blocks compilation**

- **Found during:** First cargo check attempt
- **Issue:** Cargo.toml had `uuid = { version = "1.23.1", features = ["v4"] }` on line 24 and `uuid = { version = "1", features = ["v4"] }` on line 32 (duplicate key). TOML parser rejected it.
- **Fix:** Removed the duplicate `uuid` entry on line 32.
- **Files modified:** `Cargo.toml`
- **Commit:** 5f27939

**3. Missing `regex` dependency in Cargo.toml**

- **Found during:** First cargo check attempt
- **Issue:** Prompt module template.rs uses `regex::Regex`, but `regex` was not listed in Cargo.toml dependencies.
- **Fix:** Added `regex = "1.11"` to Cargo.toml.
- **Files modified:** `Cargo.toml`
- **Commit:** 5f27939

**4. Missing `use tracing::error` import in analyzer.rs**

- **Found during:** First cargo check attempt
- **Issue:** analyzer.rs uses `error!()` macro but only imports `info` and `warn`.
- **Fix:** Changed `use tracing::{info, warn}` to `use tracing::{error, info, warn}`.
- **Files modified:** `src/visual/analyzer.rs`
- **Commit:** f847dab (inline)

**5. Missing `overall_activity_summary` field in BatchAnalysisResult initializer**

- **Found during:** First cargo check attempt
- **Issue:** `BatchAnalysisResult` has `overall_activity_summary: Option<String>`, but the constructor in `analyze_video_frames()` omitted this field.
- **Fix:** Added `overall_activity_summary: None` to the initializer.
- **Files modified:** `src/visual/analyzer.rs`
- **Commit:** f847dab (inline)

## Test Results

| Test Suite | Tests | Result |
|-----------|-------|--------|
| `visual::analyzer::tests` | 8 | All pass |
| `visual::error::tests` | 3 | All pass |
| `visual::types::tests` | 10 | All pass |
| `visual::frame_extractor::tests` | 9 | All pass |
| `prompt::error::tests` | 5 | All pass |
| `prompt::registry::tests` | 7 | All pass |
| `prompt::template::tests` | 14 | All pass |
| `error::tests` (pre-existing) | — | Pass |
| `ffmpeg::hwaccel::tests` (pre-existing) | 1 FAIL | Pre-existing — requires FFmpeg binary on system |
| **Full suite** (excl. integration) | 241 pass, 1 fail, 1 ignored | Pre-existing failure unrelated to this plan |

## Verification Criteria

- [x] `src/visual/analyzer.rs` contains `analyze_video_frames()` async function
- [x] `analyze_video_frames()` calls `extract_frames()` for frame extraction
- [x] `analyze_video_frames()` calls `LlmProvider::analyze_images()` for vision LLM
- [x] `parse_and_retry()` can strip ```json code blocks
- [x] `collect_frame_paths()` matches `keyframe_*.jpg`
- [x] Zero-frame check: 0 frames returns `VisualError::FrameExtraction`
- [x] Empty result barrier: all observations empty returns `VisualError::BatchPartial`
- [x] Error collection: single batch failure does not abort other batches
- [x] `cargo check` passes with zero warnings
- [x] All plan-related `cargo test` suites pass

## Deferred Issues

- `ffmpeg::hwaccel::tests::test_detect_encoders_format` — Pre-existing failure requires FFmpeg system installation. Not caused by this plan's changes.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| none | — | No new threat surface introduced beyond what Plan 02's frame_extractor and the planned threat model covers. |

## Self-Check: PASSED

- `src/visual/analyzer.rs` exists (422 lines, meets min_lines: 200)
- `src/visual/mod.rs` contains `pub mod analyzer; pub mod error; pub mod frame_extractor; pub mod types;`
- `src/lib.rs` contains `pub mod prompt; pub mod visual;` (after `pub mod script;`)
- Commits d131134, f847dab, 5f27939, d806497, eff665b all exist in git log
