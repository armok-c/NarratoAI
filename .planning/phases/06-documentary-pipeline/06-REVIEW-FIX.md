---
phase: 06-documentary-pipeline
fixed_date: 2026-05-07
iteration: 4
findings_in_scope: 6
fixed: 5
skipped: 1
status: partial
---

# Phase 06: Code Review Fix Report (Iteration 4)

**Fixed at:** 2026-05-07
**Source review:** .planning/phases/06-documentary-pipeline/06-REVIEW.md
**Iteration:** 4

**Summary:**
- Findings in scope: 6
- Fixed: 5
- Skipped: 1

## Fixed Issues

### CR-01 + WR-03: Integer truncation in audio delay + silent skip of missing TTS

**Files modified:** `src/documentary/audio.rs`
**Commit:** 66a7608
**Applied fix:** Two changes applied atomically in the same code block:
1. Changed `(cumulative_offset * 1000.0) as u64` to `(cumulative_offset * 1000.0).round() as u64` to minimize cumulative truncation error.
2. Replaced `if let Some(tts) = tts_results.get(...)` with `ok_or_else` to return an explicit `PipelineError::AudioMerge` when a clip requiring TTS is missing its result, instead of silently skipping.

### WR-01: Misleading underscore separators in subtitle test data

**Files modified:** `src/documentary/subtitle.rs`
**Commit:** 3719a2b
**Applied fix:** Changed numeric literals in `make_word_boundaries()` to match their comments: `500_000_0` to `5_000_000` (0.5s), `1_200_000_0` to `12_000_000` (1.2s), `2_000_000_0` to `20_000_000` (2.0s).

### WR-02: Missing subtitle_font_size bounds validation

**Files modified:** `src/documentary/types.rs`
**Commit:** 66ec5ce
**Applied fix:** Added validation in `validate()` to ensure `subtitle_font_size` is within range [1, 200], returning a descriptive error message if outside bounds.

### WR-04: Undocumented unused _request parameter

**Files modified:** `src/documentary/pipeline.rs`
**Commit:** 3cc747f
**Applied fix:** Added doc comment explaining that `_request` is retained for future per-request audio volume control during merge.

## Skipped Issues

### WR-05: Sequential clip processing

**File:** `src/documentary/clip.rs:89-124`
**Reason:** Quality improvement (parallelism), not a correctness bug. No immediate action required per reviewer recommendation.
**Original issue:** `clip_all_videos` processes clips sequentially in a for loop, missing parallelism opportunity for FFmpeg invocations.

## Verification

| Check | Result |
|-------|--------|
| `cargo check` compiles clean | Zero errors (7 pre-existing warnings unrelated to fixes) |
| All fixes re-read verified | Fix text present, surrounding code intact |

---

_Fixed: 2026-05-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 4_
