---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 19
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
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 04: Code Review Report (Re-review after fixes)

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Re-reviewed all 19 files from Phase 04 after the previous review's fixes were applied. The previous review found 2 critical and 5 warning issues. Checking the fix report (commit 3003559), the following were resolved:

- **CR-01** (format! panic with braces): Marked as false positive -- `prompt_template` is always a pre-registered template string from the registry, never arbitrary user input. The `format!` with `{{` escaped braces is safe for this use case.
- **CR-02** (bare `$variable` regex): Confirmed fixed -- the template engine now only uses `${variable}` syntax (regex `r"\$\{(\w+)\}"`), no bare `$variable` branch.
- **WR-01** (batch_size/max_concurrency validation): Confirmed fixed -- validation added at lines 76-81.
- **WR-02** (analyzed_batches naming): Confirmed fixed -- now counts `raw_results.len() - errors.len()` (successful batches only).
- **WR-04** (RwLock reentrancy documentation): Confirmed fixed -- documentation added.
- **WR-05** (parse_frame_number_from_name): Confirmed fixed -- `rename_fast_path_frames` now calls `parse_frame_number_from_name`.

Remaining issues from previous review (skipped, now re-verified):

- **WR-03** (template injection / no sanitization): Skipped -- by design, documented in code comment line 79.
- **IN-01** (tempfile not used in tests): Skipped -- cosmetic.
- **IN-02** (triple regex iteration): Skipped -- performance, not correctness.
- **IN-03** (millisecond truncation): Skipped -- negligible precision issue.

After thorough re-examination of all 19 files at standard depth, I found the following new issues:

## Warnings

### WR-01: BatchResponse uses deny_unknown_fields, contradicting the LLM-tolerance strategy used elsewhere

**File:** `src/visual/analyzer.rs:29`
**Issue:** `BatchResponse` is annotated with `#[serde(deny_unknown_fields)]`, but it deserializes LLM JSON responses. LLMs (especially non-OpenAI models) frequently return extra fields beyond the requested schema. When extra fields are present, serde rejects the entire JSON, causing `parse_and_retry` to fall back to the single-`FrameObservation` path -- which only works if the LLM happened to return a single observation.

This is directly contradictory to the deliberate design choice in `FrameObservation` (`src/visual/types.rs:7`), which intentionally omits `deny_unknown_fields` with the comment: "未知字段被静默忽略，兼容 LLM 返回额外字段的场景（D-15）". `BatchResponse` should follow the same strategy.

The impact is reduced analysis quality: a batch response with extra fields silently degrades from batch parsing to single-observation fallback, losing the `overall_activity_summary` and potentially misinterpreting the response structure.

**Fix:**
```rust
// Remove deny_unknown_fields -- LLM responses may contain extra fields
#[derive(serde::Deserialize)]
struct BatchResponse {
    #[serde(rename = "frame_observations")]
    observations: Vec<FrameObservation>,
    overall_activity_summary: Option<String>,
}
```

### WR-02: analyzer::render_prompt hardcodes interval_seconds=3.0 and other extract_frames parameters

**File:** `src/visual/analyzer.rs:84-91`
**Issue:** `analyze_video_frames` calls `extract_frames` with hardcoded `interval_seconds: 3.0` and `quality: None`. The function signature does not expose these parameters to callers. This means all video analysis uses exactly 3-second intervals regardless of video type, length, or content density.

For short clips (under 30 seconds), 3-second intervals may be too coarse. For long documentaries (2+ hours), it may produce excessive frames. The Python version likely allows configuration of these parameters.

This is a functional limitation rather than a bug, but it reduces the utility of the analyzer for different use cases.

**Fix:** Add optional parameters to `analyze_video_frames`:
```rust
pub async fn analyze_video_frames(
    video_path: &Path,
    output_dir: &Path,
    llm_provider: &dyn LlmProvider,
    prompt_template: &str,
    batch_size: usize,
    max_concurrency: usize,
    interval_seconds: Option<f64>,    // Add
    quality: Option<u32>,              // Add
    progress: Option<ProgressCallback>,
) -> Result<BatchAnalysisResult, VisualError> {
    // ...
    let frame_count = extract_frames(
        video_path,
        output_dir,
        interval_seconds.unwrap_or(3.0),
        quality,
        None,
        None,
    ).await
    // ...
}
```

### WR-03: template.rs render() pass ordering allows filter-only variables to bypass required parameter checks

**File:** `src/prompt/template.rs:80-156`
**Issue:** The template renderer runs three passes: (1) check missing plain `${variable}`, (2) replace plain variables, (3) check and replace `${variable|filter}`. This means a template like `${required_param|upper}` would NOT be caught by pass 1's required parameter check, because the `\w+` regex in pass 1 does not match `required_param|upper` (the `|` breaks the `\w+` match).

Pass 3 does have its own missing-variable check (lines 120-132), so the variable IS validated -- but via a different code path with a different error message format. This means:
- `${required_param}` missing --> error from pass 1: "缺少必需参数: required_param"
- `${required_param|upper}` missing --> error from pass 3: "缺少必需参数: required_param"

The behavior is consistent, so this is not a bug. However, if `PromptManager::render_prompt` (which checks `prompt.metadata.parameters` required fields before calling `template::render`) is used, it checks whether `vars.contains_key(param.name.as_str())`. This check works correctly for both syntaxes since the key is just the variable name.

The real concern is that the two-pass validation is fragile -- if someone adds a third syntax (e.g., `${var|filter1|filter2}`), the `(\w+)\|(\w+)` regex in pass 3 would not match, silently allowing unvalidated variables through.

**Fix:** Add a comment documenting the design constraint, or consolidate into a single validation pass that handles both syntaxes:
```rust
// Single regex that captures both ${variable} and ${variable|filter}
let all_vars_re = Regex::new(r"\$\{(\w+)(?:\|(\w+))?\}")?;
```

## Info

### IN-01: get_video_duration uses std::process::Command instead of ffmpeg-sidecar

**File:** `src/visual/frame_extractor.rs:494`
**Issue:** `get_video_duration()` uses `std::process::Command::new("ffprobe")` directly, while the rest of the codebase uses the `ffmpeg-sidecar` crate. If the project enables the `download-ffmpeg` feature (which downloads FFmpeg to a custom path), the system `ffprobe` may not be the same binary. This is inconsistent but low-risk since `download-ffmpeg` is not in the default features.

**Fix:** Consider using `ffmpeg-sidecar`'s ffprobe wrapper, or at minimum resolving the ffprobe path from the same source as ffmpeg.

### IN-02: tests in frame_extractor.rs use manual temp dir cleanup instead of tempfile crate

**File:** `src/visual/frame_extractor.rs:577-604`, `src/visual/frame_extractor.rs:645-654`
**Issue:** Multiple tests manually create temp directories with `std::env::temp_dir()` and clean up with `std::fs::remove_dir_all()`. If a test assertion panics, cleanup code is skipped and temp files remain. The project has `tempfile` in dev-dependencies but does not use it.

**Fix:** Use `tempfile::tempdir()` for automatic cleanup on drop.

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
