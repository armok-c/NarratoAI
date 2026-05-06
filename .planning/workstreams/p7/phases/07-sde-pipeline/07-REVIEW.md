---
phase: 07-sde-pipeline
reviewed: 2026-05-06T21:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - Cargo.toml
  - src/documentary/pipeline.rs
  - src/lib.rs
  - src/prompt/register.rs
  - src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v2.0.md
  - src/prompt/types.rs
  - src/prompt/validators.rs
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/pipeline.rs
  - src/sde/script_gen.rs
  - src/sde/subtitle.rs
  - src/sde/timestamp.rs
  - src/sde/types.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 07: Code Review Report (Fifth Pass)

**Reviewed:** 2026-05-06T21:00:00Z
**Depth:** standard
**Files Reviewed:** 15 (expanded scope: added `src/documentary/pipeline.rs`)
**Status:** issues_found

## Summary

Fifth-pass review of the SDE pipeline module. This pass expanded scope to 15 files including `src/documentary/pipeline.rs` (which had 5 step functions changed to `pub(crate)` in plan 04). All files were read in full and checked for bugs, security issues, and quality problems.

All 19 previously-fixed findings from passes 1-4 were verified as correctly applied:

- **WR-01 (pass 4):** `subtitle_color` hex validation confirmed at `types.rs:89-93` -- `hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit())`.
- **WR-02 (pass 4):** `parse_script` no longer performs file I/O (`_task_dir` parameter unused); caller `run_sde` uses `tokio::fs::write` at `pipeline.rs:134`.
- **WR-03 (pass 4):** `validate_output` calls confirmed at `script_gen.rs:58-62` and `script_gen.rs:117-121`.
- **Pass 3 fixes:** "播放原片" OST=1 restriction (`pipeline.rs:156`), `subtitle_font` sanitization (`pipeline.rs:438-444`).
- **Pass 2 fixes:** `strip_code_fence` json5-before-json, `tts_volume` validation, `apply_repair_steps` helper.
- **Pass 1 fixes:** Millisecond padding, FFmpeg concat injection guard, GBK-before-GB18030, path existence checks.

Two new warnings and three info items found. No critical issues.

## Warnings

### WR-01: Test `test_parse_script_saves_file` is provably broken -- asserts file that `parse_script` never writes

**File:** `src/sde/script_gen.rs:539-547`
**Issue:** The test `test_parse_script_saves_file` asserts that `script_final.json` exists on disk after calling `parse_script()`. However, `parse_script` (line 302) accepts `_task_dir` (underscore-prefixed = unused) and performs **no file I/O**. The function's doc comment (line 301) explicitly states "calling code is responsible for asynchronously saving." The actual file saving is done by `run_sde` at `pipeline.rs:134` using `tokio::fs::write`. Running `cargo test test_parse_script_saves_file` confirms the test panics every time with "script_final.json should be saved." This is a test reliability bug -- the test suite has one consistently failing test.

**Fix:** Remove the broken test or replace it with one that tests the actual contract:

```rust
// Option A: Remove test_parse_script_saves_file entirely
// (file saving is tested at the pipeline integration level)

// Option B: Replace with a correct test:
#[test]
fn test_parse_script_returns_valid_clips() {
    let json = make_items_json(r#"[{"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559", "OST": 0}]"#);
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let clips = parse_script(&json, dir.path()).expect("parse should succeed");
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0]._id, 1);
    assert_eq!(clips[0].ost, OstType::NarrationOnly);
}
```

### WR-02: Sync `std::fs::write` in SDE concat step blocks async runtime

**File:** `src/sde/pipeline.rs:317`
**Issue:** `run_sde` is an `async fn` running on the tokio runtime, yet line 317 uses blocking `std::fs::write` to write `concat_list.txt`. The same pattern exists in `documentary/pipeline.rs:195`. Pass 4 fixed this for `parse_script` (which no longer writes files), but the concat-list writes in both pipeline files were not addressed. The file is small so the blocking duration is negligible, but it is inconsistent with the async pattern used elsewhere in the same function (e.g., `tokio::fs::write` at `pipeline.rs:134`).

**Fix:**
```rust
// Replace line 317:
tokio::fs::write(&concat_list_path, &concat_content)
    .await
    .map_err(|e| SdeError::Io { source: e })?;
```

## Info

### IN-01: Unused import `OstType` in `script_gen.rs`

**File:** `src/sde/script_gen.rs:10`
**Issue:** `use crate::script::types::OstType;` is imported but never used in the file. The Rust compiler emits `warning: unused import`.
**Fix:** Remove the import line.

### IN-02: Dead code -- `has_timecodes` with overly broad heuristic

**File:** `src/sde/script_gen.rs:14-17`
**Issue:** The `#[cfg(test)]` function `has_timecodes` checks `text.contains(':')` as a timecode indicator, which matches virtually any text with a colon (e.g., `"error: something"`). The associated test (`test_has_timecodes`) gives false confidence in the detection logic. The subtitle module already has a proper `has_srt_timecodes` using a regex (`\d{2}:\d{2}:\d{2}[,.]\d{3}`) at `subtitle.rs:25-27`.
**Fix:** Either reuse `subtitle::has_srt_timecodes` or remove the dead function and its test.

### IN-03: Dead code -- `extract_text_from_srt` and `extract_text_from_ass`

**File:** `src/sde/subtitle.rs:357` and `src/sde/subtitle.rs:385`
**Issue:** Both functions are private and never called from anywhere in the crate. The Rust compiler warns about dead code for both.
**Fix:** Remove both functions, or annotate with `#[allow(dead_code)]` if intended for future use.

---

_Reviewed: 2026-05-06T21:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
