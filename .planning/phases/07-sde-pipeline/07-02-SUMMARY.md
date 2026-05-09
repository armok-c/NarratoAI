---
phase: 07-sde-pipeline
plan: 02
name: SDE Prompt Templates — plot_analysis v1.0 + script_generation v2.0
subsystem: prompt
tags:
  - prompt-template
  - sde
  - short-drama-narration
  - plot-analysis
  - script-generation
requires: []
provides:
  - plot_analysis_v1.0 template
  - script_generation_v2.0 template
  - plot_analysis registration in PromptRegistry
affects:
  - src/prompt/register.rs
  - src/prompt/types.rs (new OutputFormat::Text variant)
  - src/prompt/validators.rs (Text validation)
tech-stack:
  added: []
  patterns:
    - "Prompt template as pure Markdown with ${...} variable placeholders"
    - "Version coexistence (v1.0 backward-compatible, v2.0 as upgrade)"
key-files:
  created:
    - "src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md"
    - "src/prompt/templates/short_drama_narration/script_generation_v2.0.md"
  modified:
    - "src/prompt/register.rs"
    - "src/prompt/types.rs"
    - "src/prompt/validators.rs"
decisions:
  - "Deviation: Add OutputFormat::Text variant. The plan referenced OutputFormat::Text which did not exist in the Rust enum. Added as Rule 2 (missing critical functionality)."
  - "Deviation: Adjust test assertion. list_prompts() returns one prompt per unique name (default version), not per version. short_drama_narration has 2 names (plot_analysis + script_generation), not 3. Added per-version get() assertions to verify all 3 versions are registered."
metrics:
  duration: ~15 minutes
  completed: "2026-05-09"
---

# Phase 07 Plan 02: SDE Prompt Templates — plot_analysis v1.0 + script_generation v2.0

**One-liner:** Created two SDE prompt templates (plot_analysis v1.0 for subtitle-based plot analysis, script_generation v2.0 for JSON narration script generation) and registered them in the PromptRegistry alongside the existing script_generation v1.0.

## Files Created

### 1. `src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md` (66 lines)
Pure Markdown template for analyzing short drama subtitles. Contains:
- Character definition (professional script analyst)
- Task description (overall plot analysis + segment-by-segment analysis with timestamps)
- Input format specification (SRT format example)
- Output format requirements (structured sections with timestamp markers)
- Precautions and limitations
- `${subtitle_content}` variable placeholder

### 2. `src/prompt/templates/short_drama_narration/script_generation_v2.0.md` (267 lines)
Major upgrade from v1.0 with complete short drama narration conventions:
- 6 creative core elements: golden opening, plot refinement, highlight amplification, commentary, suspense setting, rhythm coordination
- Professional narration language techniques (atmosphere, emotional vocabulary, rhythm control)
- Strict technical requirements (timestamp non-overlap, 1/3 duration rule, continuity)
- OST=1 format specification: narration = "播放原片+N"
- OST ratio: 7:3 (original audio 70%, narration 30%)
- Single original audio segment: 3-8 seconds
- Strict JSON output format with complete items array example
- Variables: `${drama_name}`, `${plot_analysis}`, `${subtitle_content}`

## Files Modified

### 3. `src/prompt/register.rs`
- Added `plot_analysis` v1.0 registration with `subtitle_content` parameter
- Kept existing `script_generation` v1.0 for backward compatibility
- Added `script_generation` v2.0 with `drama_name`, `plot_analysis`, `subtitle_content` parameters
- Updated test assertions to verify all 3 versions exist (2 unique names + per-version get checks)

### 4. `src/prompt/types.rs`
- Added `OutputFormat::Text` variant (required by plot_analysis which outputs free-form text)

### 5. `src/prompt/validators.rs`
- Added `Text` arm to exhaustive match in `validate_output()`
- Added `validate_text()` function (non-empty check)

## Deviations from Plan

### 1. [Rule 2 - Missing Functionality] Added OutputFormat::Text variant

**Found during:** Task 3 (compilation)
**Issue:** The plan specified `OutputFormat::Text` for plot_analysis metadata, but this variant did not exist in the Rust enum. The enum only had `NarrationScript`, `PlotAnalysis`, and `Json`.
**Fix:** Added `Text` variant to `OutputFormat` enum in `src/prompt/types.rs`, plus corresponding validation in `src/prompt/validators.rs`.
**Files modified:** `src/prompt/types.rs`, `src/prompt/validators.rs`
**Commit:** `869cbce`

### 2. [Plan Correction] Adjusted test assertion for list_prompts behavior

**Found during:** Task 3 (test execution)
**Issue:** The plan asserted `list_prompts("short_drama_narration").len() == 3`, but `list_prompts()` returns one prompt per unique name (the default version). Since `script_generation` v1.0 and v2.0 share the same name, only 2 are returned.
**Fix:** Changed assertion to `list_prompts().len() == 2` and added per-version `get()` assertions to verify all 3 versions are individually accessible.
**Files modified:** `src/prompt/register.rs` (test section)
**Commit:** `869cbce`

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test prompt::register::tests` | All 4 passed |
| `cargo test --lib prompt::` | All passed |
| `plot_analysis_v1.0.md` exists, >= 50 lines | 66 lines - PASS |
| Contains `${subtitle_content}` | PASS |
| `script_generation_v2.0.md` exists, >= 150 lines | 267 lines - PASS |
| Contains `${drama_name}` | PASS |
| Contains `${plot_analysis}` | PASS |
| Contains `${subtitle_content}` | PASS |
| `register.rs` grep "plot_analysis" >= 3 | 8 references - PASS |
| Short_drama_narration has 2 names (3 versions) | plot_analysis v1.0 + script_generation v1.0 + v2.0 - PASS |

## Commit History

| Task | Description | Hash |
|------|-------------|------|
| 1 | Create plot_analysis v1.0 prompt template | `3bfbf9c` |
| 2 | Create script_generation v2.0 prompt template | `c42f762` |
| 3 | Register plot_analysis v1.0 + upgrade script_generation to v2.0 | `869cbce` |
| docs | Complete SDE prompt templates plan | `abd2c9a` |

## Deferred Issues

None. All deviations were handled inline during execution.

## Self-Check: PASSED

- plot_analysis_v1.0.md: FOUND (narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md)
- script_generation_v2.0.md: FOUND (narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md)
- register.rs: FOUND (narratoai-core/src/prompt/register.rs)
- Commits 3bfbf9c, c42f762, 869cbce: FOUND
