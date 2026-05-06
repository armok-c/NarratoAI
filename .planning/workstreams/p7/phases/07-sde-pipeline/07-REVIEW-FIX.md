---
phase: 07-sde-pipeline
fixed_at: 2026-05-06T13:00:00Z
review_path: .planning/workstreams/p7/phases/07-sde-pipeline/07-REVIEW.md
iteration: 1
findings_in_scope: 10
fixed: 8
skipped: 2
status: all_fixed
---

# Phase 07: Code Review Fix Report

**Fixed at:** 2026-05-06T13:00:00Z
**Source review:** .planning/workstreams/p7/phases/07-sde-pipeline/07-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 10
- Fixed: 8
- Skipped: 2 (WR-05 subsumed by CR-01, WR-08 not an issue)

## Fixed Issues

### CR-01: Millisecond parsing bug -- single/double digit ms values produce wrong seconds

**Files modified:** `src/sde/timestamp.rs`
**Commit:** 7d99d76 (fix), c357400 (tests)
**Applied fix:** Replaced the global `replace('.', ",")` normalization and divisor-based millisecond logic with separator detection (comma vs dot after last colon) and pad-to-3-digit approach. `"05.2"` now correctly parses as 5.200s instead of 5.020s. Added test cases for single-digit and two-digit millisecond values.

### CR-02: Command injection via newlines in FFmpeg concat file paths

**Files modified:** `src/sde/pipeline.rs`
**Commit:** 81f373e
**Applied fix:** Added newline/r carriage return validation for video paths before building the FFmpeg concat list. Paths containing `\n` or `\r` now return a validation error instead of allowing injection of arbitrary file directives.

### CR-03: Encoding detection logic inversion for GBK/GB18030

**Files modified:** `src/sde/subtitle.rs`
**Commit:** 5b42037
**Applied fix:** Changed `||` to `&&` in both GBK and GB18030 detection blocks, so encodings are only accepted when `!had_errors` AND content is meaningful. Prevents returning garbled data from failed decodes.

### WR-01: Remove dead code build_doc_request

**Files modified:** `src/sde/pipeline.rs`
**Commit:** 5b4189e
**Applied fix:** Removed the unused `build_doc_request()` function (22 lines), its test `test_build_doc_request_maps_sde_fields` (41 lines), and the now-unused `DocumentaryRequest` import.

### WR-02: has_timecodes is overly broad

**Files modified:** `src/sde/script_gen.rs`
**Commit:** cec80a3
**Applied fix:** Marked `has_timecodes()` with `#[cfg(test)]` since it is only used in test code. The function matches any text with colons (`:`), which would produce false positives in production paths.

### WR-03: Remove dead code clip.rs

**Files modified:** `src/sde/clip.rs` (deleted), `src/sde/mod.rs`
**Commit:** f2488d8
**Applied fix:** Deleted the entire `src/sde/clip.rs` file (201 lines) and removed `pub mod clip;` from `mod.rs`. The `sde_step_clip()` function was never called; pipeline.rs inlines the OST=1 correction logic directly.

### WR-04: Remove stub module audio.rs

**Files modified:** `src/sde/audio.rs` (deleted), `src/sde/mod.rs`
**Commit:** 1b933f8
**Applied fix:** Deleted `src/sde/audio.rs` (23 lines) and removed `pub mod audio;` from `mod.rs`. The stub `sde_step_merge_audio_subtitle()` always returned `Ok(())` with actual logic inlined in pipeline.rs.

### WR-06: SdeRequest::validate() should check path existence

**Files modified:** `src/sde/types.rs`
**Commit:** c26cbf5
**Applied fix:** Added `exists()` checks for `subtitle_path` and `video_path` in `validate()`, returning clear "file not found" errors. Updated `test_sde_request_validate_valid` to create temp files for the existence check.

### WR-07: Potential panic in normalize_ass_timestamp with empty frac

**Files modified:** `src/sde/subtitle.rs`
**Commit:** f55efc3
**Applied fix:** Added explicit empty `frac` guard before padding logic: if `frac.is_empty()`, return `"000"` instead of relying on format string behavior. Makes the edge case handling explicit and robust.

## Skipped Issues

### WR-05: parse_srt_timestamp rejects valid timestamps with 0 milliseconds

**File:** `src/sde/timestamp.rs:57-68`
**Reason:** Subsumed by CR-01 -- the pad-to-3-digit approach in CR-01 standardizes all millisecond handling including the zero case.
**Original issue:** Inconsistent handling of millisecond digit counts.

### WR-08: register_all_prompts test asserts only 2 prompts but 3 registered

**File:** `src/prompt/register.rs:151`
**Reason:** Not an issue -- the registry correctly keys on (category, name, version). `list_prompts` returns only the default version per name, so `.len() == 2` is correct for 2 unique names. The test at lines 153-155 already verifies all 3 versions are retrievable individually.
**Original issue:** Suspected v1.0 script_generation was silently discarded.

## Verification

- `cargo check`: Compiles with 0 errors (4 pre-existing warnings unrelated to fixes)
- `cargo test --lib`: 512 passed, 0 failed, 1 ignored
- Pre-existing doctest failure in `src/tts/common.rs` (Unicode arrow in doc comment) -- not related to any fix

---

_Fixed: 2026-05-06T13:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
