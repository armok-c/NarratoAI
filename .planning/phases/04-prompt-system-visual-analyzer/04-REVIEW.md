---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T16:30:00Z
depth: standard
files_reviewed: 20
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
  - src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v2.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 1
  warning: 5
  info: 4
  total: 10
status: issues_found
---

# Phase 4: Code Review Report

**Reviewed:** 2026-05-07T16:30:00Z
**Depth:** standard
**Files Reviewed:** 20 (prompt system + visual analyzer + 7 templates)
**Status:** issues_found

## Summary

Re-reviewed 20 source files at standard depth covering the prompt system (types, error, registry, template engine, manager, validators, registration, 7 prompt templates) and visual analyzer (error, types, frame extractor, batch analyzer). Previous round's findings (CR-01 strip_code_fence logic bug, WR-02 empty-observations blind spot, WR-03 missing code-fence stripping in JSON validator) have been fixed. The fixes are verified correct in the current code.

Found 1 new critical issue and 5 warnings. The critical issue is a cross-module architectural coupling where `prompt/validators.rs` imports from `visual/types.rs`, creating an inappropriate dependency from the prompt module to the visual module. Warnings cover silent error swallowing in FFmpeg, inconsistent FFmpeg binary discovery between fast/fallback paths, fragile success-count computation, out-of-range visual_salience acceptance, and cancellation token lifetime issues in spawn_blocking.

## Critical Issues

### CR-01: Cross-module dependency -- prompt validators import from visual types

**File:** `src/prompt/validators.rs:35`
**Classification:** BLOCKER

**Issue:** The `validate_json` function calls `crate::visual::types::strip_code_fence()` to strip markdown code fences before JSON validation. This creates a hard dependency from the `prompt` module to the `visual` module. In `src/lib.rs`, these are sibling modules with no hierarchical relationship -- `prompt` is a self-contained template/rendering system that should not depend on video frame analysis code.

Consequences:
- The `prompt` module cannot compile without the `visual` module present
- If `strip_code_fence` behavior changes in `visual`, prompt validation silently changes
- The function is `pub(crate)`, meaning it was not designed as a stable cross-module API
- Future refactoring of either module risks breaking the other

**Fix:** Extract `strip_code_fence` to a shared utility module:

```rust
// New file: src/text_utils.rs
/// Strip markdown code fences (```json...``` or ```...```)
pub fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let after_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_start());
    let content = after_prefix.unwrap_or(trimmed);
    content
        .strip_suffix("```")
        .map(|s| s.trim_end())
        .unwrap_or(content)
}
```

Then update imports in both `src/prompt/validators.rs` and `src/visual/types.rs` to use `crate::text_utils::strip_code_fence`.

## Warnings

### WR-01: `run_ffmpeg_with_cancel` silently swallows all FFmpeg errors

**File:** `src/visual/frame_extractor.rs:560-577`
**Classification:** WARNING

**Issue:** The function returns `bool` instead of `Result`, discarding all error context. When FFmpeg fails to spawn (e.g., binary not installed, PATH misconfigured, permission denied) or `child.wait()` fails, the caller receives only `false`. The 4-level fallback in `extract_single_frame` tries all levels and reports only "all 4 levels failed (timestamp=Xs)" with no indication that the root cause is a missing FFmpeg binary. This makes production debugging extremely difficult.

**Fix:** At minimum, log the error at warn level:

```rust
fn run_ffmpeg_with_cancel(args: &[&str], cancel: &CancellationToken) -> bool {
    if cancel.is_cancelled() {
        return false;
    }
    let mut child = match std::process::Command::new("ffmpeg")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("FFmpeg spawn failed: {}", e);
            return false;
        }
    };
    match child.wait() {
        Ok(status) => status.success(),
        Err(e) => {
            tracing::warn!("FFmpeg wait failed: {}", e);
            false
        }
    }
}
```

### WR-02: Inconsistent FFmpeg binary discovery between fast path and fallback path

**File:** `src/visual/frame_extractor.rs:131,522,564`
**Classification:** WARNING

**Issue:** The fast path uses `ffmpeg_sidecar::command::FfmpegCommand` (which handles FFmpeg binary discovery, including the `download-ffmpeg` feature), but the fallback path calls `std::process::Command::new("ffprobe")` (line 522) and `std::process::Command::new("ffmpeg")` (line 564) directly. On systems where FFmpeg is not on PATH but is managed by `ffmpeg-sidecar` (e.g., the `download-ffmpeg` feature is enabled), the fast path succeeds but the fallback path silently fails with no clear error. This creates inconsistent behavior where the same code works on some systems but not others.

**Fix:** Use `ffmpeg_sidecar` consistently, or extract the resolved FFmpeg binary path and pass it to `std::process::Command`. At minimum, document this limitation in the function doc comments.

### WR-03: `analyzed_batches` computed by subtraction instead of explicit counter

**File:** `src/visual/analyzer.rs:179,211`
**Classification:** WARNING

**Issue:** The success count is computed as `raw_results.len() - errors.len()` in two places (lines 179 and 211). While currently correct (each batch produces at most one error), this semantic coupling is fragile. If future code changes allow multiple errors per batch or add partial-success tracking, the subtraction produces an incorrect count. The two computation sites also serve different purposes (one for the error path, one for the success path) but use the same implicit logic.

**Fix:** Use an explicit success counter:

```rust
let mut success_count: usize = 0;
for (idx, text) in raw_results.iter().enumerate() {
    match parse_and_retry(text) {
        Ok(batch) => {
            success_count += 1;
            observations.extend(batch.observations);
            // ...
        }
        Err(e) => {
            errors.push(err_msg);
        }
    }
}
// Then use success_count instead of raw_results.len() - errors.len()
```

### WR-04: `visual_salience` accepts out-of-range f64 values without validation

**File:** `src/visual/types.rs:22`
**Classification:** WARNING

**Issue:** The `visual_salience` field is documented as range "0.0-1.0" but deserializes any `f64` without range validation. The test at line 221-233 explicitly documents that `2.5` is accepted. If downstream code uses this value as a probability weight, UI progress fraction, or array index, out-of-range values from LLM responses could cause panics or incorrect behavior. Since LLM output is inherently unpredictable, validation is important.

**Fix:** Add a post-deserialization validation method:

```rust
impl FrameObservation {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(s) = self.visual_salience {
            if !(0.0..=1.0).contains(&s) {
                return Err(format!("visual_salience out of range [0,1]: {}", s));
            }
        }
        Ok(())
    }
}
```

Or clamp the value during deserialization using a custom deserializer.

### WR-05: `cancel.unwrap_or_default()` creates unowned token that prevents cancellation on task drop

**File:** `src/visual/frame_extractor.rs:117,194`
**Classification:** WARNING

**Issue:** When the caller passes `cancel: None`, `unwrap_or_default()` creates a new `CancellationToken` that is owned only by the `spawn_blocking` task. No external code can trigger cancellation. More importantly, if the calling async task is dropped (e.g., the HTTP connection closes), the `spawn_blocking` task continues running until completion because the cancellation token is never signaled. This can cause resource leaks (running FFmpeg processes, holding file handles) in server scenarios.

**Fix:** When `cancel` is `None`, pass a fresh `CancellationToken` and store the `Cancelled` handle to trigger on join handle drop. Alternatively, document this as intentional fire-and-forget behavior.

## Info

### IN-01: `truncate` filter uses hardcoded magic numbers

**File:** `src/prompt/template.rs:41-48`

**Issue:** The truncate filter uses hardcoded `100` (max chars) and `97` (chars before "...") with no configurability. Should be named constants for clarity.

### IN-02: `notify` crate pinned to RC version

**File:** `Cargo.toml:19`

**Issue:** `notify = "9.0.0-rc.3"` is a release candidate. The comment already acknowledges this risk. No action needed beyond awareness.

### IN-03: Regex patterns recompiled on every `render()` call

**File:** `src/prompt/template.rs:85-87,119-121`

**Issue:** Two `Regex::new()` calls compile identical patterns on every invocation. Use `std::sync::OnceLock<Regex>` for one-time initialization.

### IN-04: `builtin_filters()` allocates new HashMap on every `render()` call

**File:** `src/prompt/template.rs:10-57`

**Issue:** The 6-entry filter map is recreated per render call. Use `OnceLock` for lazy one-time initialization.

---

_Reviewed: 2026-05-07T16:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
