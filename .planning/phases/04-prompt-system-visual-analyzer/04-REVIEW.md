---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T14:30:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 4
  warning: 7
  info: 5
  total: 16
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-07T14:30:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Reviewed all 18 source files from Phase 04 (Prompt Subsystem + Visual Analyzer) at standard depth. The architecture is sound: clean error types with thiserror, proper RwLock usage, and a well-structured 3-level registry. Template variables in all .md files are consistent with their register.rs ParameterDef declarations.

Found 4 critical issues, 7 warnings, and 5 info items. The critical issues center on silent failure modes in the template renderer and visual subsystem: empty-string substitution hiding missing variables, orphan FFmpeg processes on cancellation, and incorrect code-fence stripping that can corrupt JSON.

## Critical Issues

### CR-01: Template renderer silently substitutes empty string for unmatched captures

- **File:** `src/prompt/template.rs:108-114`
- **Severity:** CRITICAL

In the second pass (`replace_all`), `vars.get(name).copied().unwrap_or("")` silently substitutes an empty string when variable lookup fails. If the first-pass validation logic is ever modified or a variable name edge case slips through, this produces corrupted template output with no error signal — the LLM receives a prompt with missing data.

**Fix:** Replace `unwrap_or("")` with `unreachable!()` (since pass 1 validates all variables) or return an error:
```rust
vars.get(name).copied().unwrap_or_else(|| {
    unreachable!("variable '{}' passed validation in pass 1 but not found in pass 2", name)
})
```

### CR-02: Filter replacement silently preserves raw `${var|filter}` tokens

- **File:** `src/prompt/template.rs:147-156`
- **Severity:** CRITICAL

The third-pass filter replacement has a fallback that preserves raw `${var|filter}` tokens when lookup fails (lines 150-155). This bypasses the validation guarantee from lines 122-129. Un-replaced template tokens could be sent to the LLM, causing unpredictable behavior.

**Fix:** Remove the silent fallback:
```rust
match (filters.get(filter_name), vars.get(var_name)) {
    (Some(filter_fn), Some(value)) => filter_fn(value),
    _ => unreachable!(
        "filter '{}' and variable '{}' passed validation but not found during replacement",
        filter_name, var_name
    ),
}
```

### CR-03: `extract_single_frame` spawns FFmpeg processes without cancellation — orphan processes

- **File:** `src/visual/frame_extractor.rs:264-392`
- **Severity:** CRITICAL

`run_ffmpeg` uses `std::process::Command::output()` with no timeout or kill mechanism. When `extract_frames_fallback` is cancelled between frames, any in-progress `run_ffmpeg` call continues to completion (or hangs indefinitely). The fast path uses `ffmpeg_sidecar` with `child.kill()` on cancel, but the fallback path's raw `Command` has no process lifecycle control.

If the user cancels or the application shuts down, orphan FFmpeg processes continue consuming CPU and holding file handles.

**Fix:** Replace `run_ffmpeg` with a version that spawns the process, polls cancellation, and kills on cancel:
```rust
fn run_ffmpeg_with_cancel(args: &[&str], cancel: &CancellationToken) -> bool {
    let mut child = match std::process::Command::new("ffmpeg")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(_) => return false,
        }
    }
}
```

### CR-04: `strip_code_fence` incorrectly strips JSON content containing triple backticks

- **File:** `src/visual/types.rs:44-50`
- **Severity:** CRITICAL

`trim_end_matches("```")` strips individual backtick characters (char set), not the exact triple-backtick sequence. A JSON string ending with `` ` `` or containing ```` ``` ```` (e.g., a code example in `scene_description`) will be incorrectly truncated, causing deserialization to fail.

Example: `{"scene_description": "code: x=```"}` — the `trim_end_matches` strips the last 3 backtick characters from within the JSON content.

**Fix:** Use precise string matching instead of character-set trimming:
```rust
pub(crate) fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let after_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_start());
    let content = after_prefix.unwrap_or(trimmed);
    content
        .strip_suffix("```")
        .map(|s| {
            if s.ends_with(|c: char| c.is_whitespace()) {
                s.trim_end()
            } else {
                content
            }
        })
        .unwrap_or(content)
}
```

## Warnings

### WR-01: `expect()` in json filter can panic in library code

- **File:** `src/prompt/template.rs:53`
- **Severity:** WARNING

The `json` filter uses `.expect()` on `serde_json::to_string(s)`. While serializing a `&str` to JSON is unlikely to fail, using `expect` in library code risks a panic during template rendering that would crash the application.

**Fix:** Use a safe fallback: `serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s.replace('"', "\\\"")))`

### WR-02: `search()` performs O(n) linear scan with repeated allocation

- **File:** `src/prompt/registry.rs:107-129`
- **Severity:** WARNING

`search()` calls `.to_lowercase()` for every prompt on every search, allocating new `String` objects per comparison. This is wasteful for frequent searches.

**Fix:** Pre-compute lowercase versions in `PromptMetadata` or use a case-insensitive comparison helper.

### WR-03: `validate_narration_script` fails on Windows-style line endings

- **File:** `src/prompt/validators.rs:74-77`
- **Severity:** WARNING

`split("\n\n")` requires exact LF pairs. On Windows, `\r\n\r\n` won't match. LLM output may use mixed line endings, producing incorrect paragraph counts.

**Fix:** Normalize line endings first: `let normalized = trimmed.replace("\r\n", "\n");` then split on `"\n\n"`.

### WR-04: `seconds_to_hhmmssmmm` uses floating-point modulo — field overflow risk

- **File:** `src/visual/frame_extractor.rs:468-474`
- **Severity:** WARNING

Floating-point modulo arithmetic can produce `minutes >= 60` or `secs >= 60` due to rounding, generating invalid timestamps. The `{:02}` format does not truncate, so values > 99 break the 9-digit format.

**Fix:** Use integer millisecond arithmetic:
```rust
let total_millis = (total_secs * 1000.0).round() as u64;
let hours = total_millis / 3_600_000;
let minutes = (total_millis % 3_600_000) / 60_000;
let secs = (total_millis % 60_000) / 1000;
let millis = total_millis % 1000;
format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis)
```

### WR-05: `extract_frames_fast_path` calls `child.wait()` twice

- **File:** `src/visual/frame_extractor.rs:148-158`
- **Severity:** WARNING

After the event iterator exhausts, `child.wait()` is called again at line 158. The `ffmpeg_sidecar` iterator may already reap the child process internally, making this second `wait()` redundant or error-prone.

**Fix:** Verify `ffmpeg_sidecar` documentation. If `iter()` to exhaustion already calls `wait()`, remove line 158.

### WR-06: `get_video_duration` uses raw `std::process::Command` instead of ffmpeg-sidecar

- **File:** `src/visual/frame_extractor.rs:493-521`
- **Severity:** WARNING

Inconsistent with the fast path which uses `ffmpeg_sidecar`. `ffprobe` may not be on PATH even when `ffmpeg` is available via sidecar. No timeout or cancellation support.

**Fix:** Use `ffmpeg_sidecar` consistently or document the `ffprobe` binary requirement.

### WR-07: `parse_and_retry` silently discards BatchResponse parse errors

- **File:** `src/visual/analyzer.rs:221-226`
- **Severity:** WARNING

When `serde_json::from_str::<BatchResponse>` fails, the error is silently swallowed and falls through to single-observation parsing. The root cause (e.g., a typo in a field name) is lost, making debugging difficult.

**Fix:** Log the parse error at debug level before falling through:
```rust
match serde_json::from_str::<BatchResponse>(cleaned) {
    Ok(resp) => return Ok(ParsedBatch { ... }),
    Err(e) => {
        tracing::debug!(error = %e, "BatchResponse parse failed, falling back to single");
    }
}
```

## Info

### IN-01: Regex compilation on every `render()` call

- **File:** `src/prompt/template.rs:85-87` and `117-119`
- **Severity:** INFO

Two regex patterns are compiled on every `render()` call. Use `OnceLock<Regex>` to compile once.

### IN-02: `builtin_filters()` allocates new HashMap on every `render()` call

- **File:** `src/prompt/template.ts:10-57`
- **Severity:** INFO

6-entry `HashMap` is recreated on every render. Use `OnceLock` for lazy initialization.

### IN-03: `#[allow(dead_code)]` on `seconds_to_hhmmssmmm` may be unnecessary

- **File:** `src/visual/frame_extractor.rs:467`
- **Severity:** INFO

The function is `pub(crate)` but has `#[allow(dead_code)]`. If called from other modules, remove the annotation. If not, remove `pub(crate)`.

### IN-04: `BatchAnalysisResult.errors` is stringly-typed

- **File:** `src/visual/types.rs:37`
- **Severity:** INFO

The `errors` field is `Vec<String>` — stringly-typed errors that cannot be programmatically inspected. Consider a structured error type for future refinement.

### IN-05: `extract_single_frame` error message refers to `.jpg` even for PNG/BMP fallbacks

- **File:** `src/visual/frame_extractor.rs:218`
- **Severity:** INFO

The final error message always refers to `.jpg` even when levels 3/4 produce PNG/BMP. The error message could be confusing.

---

_Reviewed: 2026-05-07T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
