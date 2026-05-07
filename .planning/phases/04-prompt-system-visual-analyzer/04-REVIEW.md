---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - narratoai-core/src/lib.rs
  - narratoai-core/src/prompt/mod.rs
  - narratoai-core/src/prompt/types.rs
  - narratoai-core/src/prompt/error.rs
  - narratoai-core/src/prompt/manager.rs
  - narratoai-core/src/prompt/validators.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/registry.rs
  - narratoai-core/src/prompt/template.rs
  - narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/types.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/analyzer.rs
  - narratoai-core/src/llm/provider.rs
  - narratoai-core/src/llm/types.rs
  - narratoai-core/src/ffmpeg/command.rs
  - narratoai-core/src/text_utils.rs
findings:
  critical: 2
  warning: 6
  info: 5
  total: 13
status: issues_found
---

# Phase 04: Prompt System + Visual Analyzer -- Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 22 (14 Rust source, 6 markdown templates + 2 cross-referenced modules)
**Status:** issues_found

## Summary

Reviewed the Prompt system (types, error, registry, template engine, manager, validators, register, and 6 template files) and Visual analyzer (frame extraction, data types, batch analysis orchestrator). The codebase is functionally correct with clear error types and thorough test coverage. However, two critical issues were identified: (1) the LLM analysis phase has no cancellation mechanism -- `LlmProvider::analyze_images()` trait does not accept a `CancellationToken`, making the longest pipeline stage uninterruptible; (2) `parse_and_retry` does not handle bare JSON array responses from LLMs (`[{...}, {...}]`), a common LLM failure mode. Additional warnings cover suppressed FFmpeg stderr (making extraction failures undebuggable), stale file inclusion risk in `collect_frame_paths`, fast-path error loss, documentation mismatch in quality range, and redundant `#[allow(dead_code)]` annotation.

## Critical Issues

### CR-01: LLM analysis phase has no cancellation mechanism

**File:** `narratoai-core/src/visual/analyzer.rs:155-169`
**Trait definition:** `narratoai-core/src/llm/provider.rs:43-53`

**Issue:** The `LlmProvider::analyze_images()` trait method does not accept a `CancellationToken` parameter. While `analyze_video_frames` checks cancellation before the LLM call (lines 121-125) via `cancel_after_extract` (a clone of the original token), the LLM call itself -- the longest phase of the pipeline, potentially sending multiple batches of images to a vision model -- is uncancellable. Once `analyze_images()` is dispatched, a cancellation request is ignored until the entire batch sequence completes, which could take minutes for long videos with many frames.

The `cancel` token is consumed by `extract_frames()` at line 112. Even `cancel_after_extract` (cloned at line 104) is only checked at the single point before `analyze_images` is called, not during execution. Furthermore, the `analyze_images` trait method lacks a `CancellationToken` parameter entirely, so no implementation can support cancellation.

**Fix (two parts):**

Part 1 -- Clone the cancel token and pass both to extract_frames and a pre-LLM check (already done partially, but the token is consumed):

```rust
// In analyzer.rs, before line 104:
let cancel_for_llm = cancel.clone();

// Line 106-112 stays the same (cancel consumed by extract_frames)

// Before line 155:
if let Some(ref cancel) = cancel_for_llm {
    if cancel.is_cancelled() {
        return Err(VisualError::Analysis("分析被取消".into()));
    }
}
```

Part 2 -- Add `Option<CancellationToken>` parameter to the trait and propagate:

```rust
// In llm/provider.rs:
async fn analyze_images(
    &self,
    images: &[PathBuf],
    prompt: &str,
    system_prompt: Option<&str>,
    batch_size: Option<usize>,
    max_concurrency: Option<usize>,
    response_format: Option<LlmResponseFormat>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    cancel: Option<CancellationToken>,  // ADD THIS
) -> Result<Vec<String>, LLMError>;
```

### CR-02: `parse_and_retry` cannot handle bare JSON array responses from LLM

**File:** `narratoai-core/src/visual/analyzer.rs:256-289`

**Issue:** The `parse_and_retry` function attempts two deserialization paths in sequence:

1. `serde_json::from_str::<BatchResponse>(cleaned)` -- expects a JSON object with `frame_observations` key
2. `serde_json::from_str::<FrameObservation>(cleaned)` -- expects a single JSON object

If an LLM returns a bare JSON array like `[{"frame_number": 0, ...}, {"frame_number": 1, ...}]` (a valid representation of multiple observations without the wrapping `{frame_observations: [...]}`), both parsers fail:

- `BatchResponse` fails because it expects `{ ... }` (a map), not `[ ... ]` (an array)
- `FrameObservation` fails because it expects a single object, not an array

This is a genuine LLM failure mode -- weaker models or models with system prompts that override the output schema may return an array directly despite being instructed to use the wrapping format. The current code silently drops these responses, resulting in `VisualError::Analysis("JSON 解析失败: ...")`.

**Fix:** Add a third fallback path that attempts deserialization as `Vec<FrameObservation>`:

```rust
fn parse_and_retry(json_text: &str) -> Result<ParsedBatch, VisualError> {
    let cleaned = text_utils::strip_code_fence(json_text);

    // Attempt 1: BatchResponse (wrapping object with frame_observations)
    match serde_json::from_str::<BatchResponse>(cleaned) {
        Ok(resp) => {
            return Ok(ParsedBatch {
                observations: resp.observations,
                overall_activity_summary: resp.overall_activity_summary,
            });
        }
        Err(e) => {
            tracing::debug!(error = %e, "BatchResponse parse failed, falling back");
        }
    }

    // Attempt 2: single FrameObservation
    match serde_json::from_str::<FrameObservation>(cleaned) {
        Ok(obs) => return Ok(ParsedBatch {
            observations: vec![obs],
            overall_activity_summary: None,
        }),
        Err(e) => {
            tracing::debug!(error = %e, "FrameObservation parse failed, falling back to array");
        }
    }

    // Attempt 3 (NEW): bare array of FrameObservation
    match serde_json::from_str::<Vec<FrameObservation>>(cleaned) {
        Ok(observations) => Ok(ParsedBatch {
            observations,
            overall_activity_summary: None,
        }),
        Err(e) => {
            tracing::warn!(
                error = %e,
                raw_preview = truncate_str(json_text, 200),
                "All JSON deserialization paths failed",
            );
            Err(VisualError::Analysis(format!(
                "JSON 解析失败: {}", e
            )))
        }
    }
}
```

## Warnings

### WR-01: `run_ffmpeg_with_cancel` suppresses FFmpeg stderr, losing all diagnostic information

**File:** `narratoai-core/src/visual/frame_extractor.rs:589-593`

**Issue:** Both stdout and stderr are redirected to `Stdio::null()`. When FFmpeg fails (codec unsupported, file corruption, invalid parameter, disk full), the stderr output containing the precise error message is discarded. The function returns only a boolean `true`/`false`. Callers in `extract_single_frame` (up to 4 fallback levels) report only the generic message `"所有 4 级回退均失败 (timestamp={}s)"` with zero diagnostic detail.

This makes extraction failures extremely difficult to debug in production -- the FFmpeg error message is the primary diagnostic tool, and it is systematically destroyed.

**Fix:** Capture stderr and log it when the process exits with failure:

```rust
fn run_ffmpeg_with_cancel(args: &[&str], cancel: &CancellationToken) -> bool {
    if cancel.is_cancelled() { return false; }
    let ffmpeg_bin = ffmpeg_path();
    let mut child = match std::process::Command::new(ffmpeg_bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())  // CHANGE: capture stderr
        .spawn()
    {
        Ok(c) => c,
        Err(e) => { tracing::warn!("FFmpeg spawn failed: {}", e); return false; }
    };
    match child.wait() {
        Ok(status) => {
            if !status.success() {
                let stderr = child.stderr.take()
                    .and_then(|s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        s.read_to_string(&mut buf).ok().map(|_| buf)
                    })
                    .unwrap_or_default();
                tracing::warn!("FFmpeg failed (exit={:?}): {}", status.code(), stderr.trim());
            }
            status.success()
        }
        Err(e) => { tracing::warn!("FFmpeg wait failed: {}", e); false }
    }
}
```

### WR-02: `collect_frame_paths` may include stale files from previous extraction runs

**File:** `narratoai-core/src/visual/analyzer.rs:299-343`

**Issue:** `collect_frame_paths` collects ALL files matching `keyframe_*.jpg` in `output_dir`. While `extract_frames` cleans stale `keyframe_*.jpg` files at lines 67-78 before extraction, there are two robustness gaps: (a) if `extract_frames` crashes after extraction but before returning, stale files from that run remain; (b) no filtering by mtime or session marker distinguishes files from different extraction runs. A concurrent process writing `keyframe_*` files to the same directory (unlikely but not prevented) would silently corrupt results.

The mismatch between `frame_count` (returned by `extract_frames`) and `frame_paths.len()` (used for `total_frames` in the result at line 242) is unverified -- if they diverge, the caller gets inconsistent metadata.

**Fix:** Have `extract_frames` return the set of extracted file paths alongside the count, and pass that list directly to the analyzer instead of re-scanning the directory:

```rust
// Return type for extract_frames:
pub async fn extract_frames(...) -> Result<(usize, Vec<PathBuf>), VisualError> {
    // ... on success, return (count, extracted_file_list)
}
```

Alternatively, write a manifest file (`frames_manifest.txt`) containing the sorted list of extracted filenames after extraction.

### WR-03: Fast path errors silently discarded by the catch-all

**File:** `narratoai-core/src/visual/frame_extractor.rs:90-110`

**Issue:** The `match` on `extract_frames_fast_path` uses `_ =>` as the catch-all, matching both `Ok(0)` and `Err(e)`. When the fast path returns a meaningful error (e.g., `"FFmpeg 启动失败: ..."`, `"视频路径无效"`), the error is discarded and the fallback path runs. The fallback likely fails with the same root cause but produces a different error message. The user sees the fallback's error, losing the root cause information provided by the fast path.

**Fix:** Separate the match arms and log the fast-path error before falling back:

```rust
match extract_frames_fast_path(...).await {
    Ok(count) if count > 0 => {
        // ... existing success path ...
        Ok(count)
    }
    Err(e) => {
        tracing::warn!("Fast path failed, falling back to per-frame extraction: {}", e);
        cleanup_fast_path_files(output_dir);
        extract_frames_fallback(...).await
    }
    Ok(0) => {
        tracing::info!("Fast path produced 0 frames, falling back to per-frame extraction");
        cleanup_fast_path_files(output_dir);
        extract_frames_fallback(...).await
    }
}
```

### WR-04: Quality range mismatch -- doc says 1-31, code validates 2-31

**File:** `narratoai-core/src/visual/analyzer.rs:70` (docstring)
**File:** `narratoai-core/src/visual/frame_extractor.rs:44` (validation)

**Issue:** The `analyze_video_frames` docstring documents `quality` as `"JPEG 压缩质量（1-31）"` but `extract_frames` validates `!(2..=31).contains(&quality_val)`, rejecting values of 1. The FFmpeg `-q:v` parameter range is technically 2-31 (value 1 is reserved/internal), making the code correct and the documentation wrong. A caller passing `Some(1)` expecting "maximum quality" receives an error.

**Fix:** Update the docstring in `analyzer.rs` line 70:
```rust
/// - `quality` — JPEG 压缩质量（2-31，值越小质量越高，默认 5），None 使用 `extract_frames` 内置默认值
```

### WR-05: `#[allow(dead_code)]` on actively used function `seconds_to_hhmmssmmm`

**File:** `narratoai-core/src/visual/frame_extractor.rs:511`

**Issue:** The function is annotated with `#[allow(dead_code)]` but is actively called in production code at line 493 (`rename_fast_path_frames`) and in tests (lines 666, 691, 729-735). This attribute will suppress a future compiler warning if `rename_fast_path_frames` is refactored to no longer use it, making real dead code harder to detect.

**Fix:** Remove `#[allow(dead_code)]`:
```rust
pub(crate) fn seconds_to_hhmmssmmm(total_secs: f64) -> String {
```

### WR-06: `BatchPartial` error renders `Vec<String>` via Debug format

**File:** `narratoai-core/src/visual/error.rs:15-20`

**Issue:** The `BatchPartial` variant's `Display` derives `{errors:?}`, which calls `Debug` on `Vec<String>`, producing output like `["错误一", "错误二"]`. This is inconsistent with the clean prefix-based format of the other variants (`"帧提取失败: ..."`, `"视觉分析失败: ..."`). The Debug format includes quotes and commas that are unnecessary for user-facing error messages.

**Fix:** Use `thiserror` with a manual format function or change to a joined string:

```rust
#[error("部分批次失败: 已分析 {analyzed_count}/{total_count} 批次, {errors}")]
BatchPartial {
    analyzed_count: usize,
    total_count: usize,
    #[error(display = "{}", .errors.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; "))]
    errors: Vec<String>,
},
```

## Info

### IN-01: Repeated directory-iteration patterns with inconsistent error handling

**File:** `narratoai-core/src/visual/frame_extractor.rs`

Three functions iterate the output directory with similar patterns but different error handling:

- `extract_frames` cleanup (lines 67-78): silently skips errors (`if let Ok(mut reader)`)
- `rename_fast_path_frames` (lines 450-470): propagates reader error via `?`
- `cleanup_fast_path_files` (lines 522-535): silently skips errors

Each function manually filters files by prefix/suffix. Extract a shared helper to reduce duplication:

```rust
fn iterate_keyframe_files(dir: &Path, prefix: &str, suffix: &str) -> Result<Vec<PathBuf>, VisualError> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| ...)? {
        let entry = entry.map_err(|e| ...)?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) && name.ends_with(suffix) {
            paths.push(entry.path());
        }
    }
    Ok(paths)
}
```

### IN-02: Hardcoded magic values in `analyzer.rs`

Several magic values appear without named constants:

| Value | Location | Purpose |
|-------|----------|---------|
| `3.0` | line 109 | Default frame extraction interval (seconds) |
| `0.1` | line 166 | LLM temperature for analysis |
| `4096` | line 167 | LLM max_tokens |
| `100` | validators.rs:100 | Minimum character count for plot analysis |
| `50` | validators.rs:68 | Minimum character count for narration script |
| `3` | validators.rs:79 | Minimum paragraph count for narration script |

Extract these as module-level `const` items with documentation:

```rust
const DEFAULT_INTERVAL_SECONDS: f64 = 3.0;
const ANALYSIS_TEMPERATURE: f32 = 0.1;
const ANALYSIS_MAX_TOKENS: u32 = 4096;
const MIN_PLOT_ANALYSIS_CHARS: usize = 100;
const MIN_NARRATION_CHARS: usize = 50;
const MIN_NARRATION_PARAGRAPHS: usize = 3;
```

### IN-03: Double validation of template variables

**Files:** `narratoai-core/src/prompt/manager.rs:63-73`, `narratoai-core/src/prompt/template.rs:88-105`

The `render_prompt` method in `PromptManager` checks required parameters against caller-supplied `vars` (accounting for defaults), and then `template::render` independently checks that ALL template variables (not just required ones) are present in the merged `vars`. The manager's check understands the semantics of `required` and `default`, while the template's check is purely syntactic. This creates an undocumented two-tier validation where the manager catches "missing required" (nice error message) and the template catches "referenced but not provided" (different error type, `TemplateRender` instead of `Validation`).

While functionally correct, a comment explaining this design intent would prevent future refactoring mistakes:

```rust
// Note: This check catches missing REQUIRED parameters with a Validation error.
// template::render provides a second safety net that catches any template variable
// (including optional ones without defaults) that is referenced but not provided.
// That check produces a TemplateRender error instead.
```

### IN-04: No guard against NaN in `seconds_to_hhmmssmmm`

**File:** `narratoai-core/src/visual/frame_extractor.rs:512`

```rust
let total_millis = (total_secs * 1000.0).round() as u64;
```

If `total_secs` is NaN (which shouldn't happen with current callers but is not prevented by the function signature), `NaN as u64` in Rust is defined as 0, so this won't panic. However, the result would be incorrect (`"000000000"`). Add a `debug_assert!` for caller-safety:

```rust
debug_assert!(!total_secs.is_nan(), "seconds_to_hhmmssmmm called with NaN");
debug_assert!(total_secs.is_finite(), "seconds_to_hhmmssmmm called with non-finite value");
```

### IN-05: Re-export in `visual/types.rs` is only used by tests in the same file

**File:** `narratoai-core/src/visual/types.rs:56`

```rust
pub(crate) use crate::text_utils::strip_code_fence;
```

This re-export is consumed only by the test module in the same file (test `test_strip_code_fence_json_prefix` at line 103). Production code in `analyzer.rs` imports `crate::text_utils` directly. Either remove the re-export or document it as a compatibility shim.

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
