---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T20:00:00Z
depth: standard
files_reviewed: 20
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
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 4: Code Review Report (Iteration 3/3 -- Fix Verification)

**Reviewed:** 2026-05-07T20:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found (2 new info items; all 14 previous findings resolved)

## Summary

Iteration 3/3 re-review verifying that all 14 findings from the previous review (2 critical, 7 warning, 5 info) have been correctly resolved in the source code. Each finding was traced against the actual source code by reading every changed file. Commit messages were not trusted -- the code was verified directly.

**All 14 previous findings are confirmed resolved.** Each fix was verified against the actual source code:

- CR-01: `cancel: Option<CancellationToken>` parameter added to `analyze_images` trait method
- CR-02: `Vec<FrameObservation>` third fallback added to `parse_and_retry`
- WR-01: `run_ffmpeg_with_cancel` now pipes and reads stderr on failure
- WR-02: `extract_frames` returns `(usize, Vec<PathBuf>)` eliminating directory re-scan
- WR-03: Fast path match has three distinct arms (separate Err/Ok(0))
- WR-04: Docstring reads "2-31" matching code validation
- WR-05: No `#[allow(dead_code)]` on `seconds_to_hhmmssmmm` (changed to `pub(crate)`)
- WR-06: `BatchPartial` uses joined `String`, not Debug formatting
- WR-07: Version fallback parsing uses numeric `u64` comparison, not lexical
- IN-01: Shared `iterate_dir_files` helper consolidates directory iteration
- IN-02: Magic values extracted as named constants in analyzer.rs and validators.rs
- IN-03: Two-tier validation design documented in manager.rs
- IN-04: `debug_assert!` for NaN added to `seconds_to_hhmmssmmm`
- IN-05: Re-export documented in types.rs

Two pre-existing info-level code quality issues were identified that were not part of the previous finding set and were not introduced by the fixes.

---

## Previous Finding Verifications

### CR-01: `analyze_images` trait has cancel param
**Status: RESOLVED**

`src/llm/provider.rs:44-55` -- The `analyze_images` trait method signature includes `cancel: Option<CancellationToken>` as its final parameter:
```rust
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
    cancel: Option<CancellationToken>,
) -> Result<Vec<String>, LLMError>;
```

In `analyzer.rs:180`, the token is forwarded: `cancel_after_extract.clone()` is passed as the `cancel` parameter to `analyze_images`.

---

### CR-02: `parse_and_retry` has `Vec<FrameObservation>` fallback
**Status: RESOLVED**

`src/visual/analyzer.rs:269-311` -- `parse_and_retry` now attempts three deserialization strategies in order:

1. `BatchResponse` (line 273) -- standard wrapped schema
2. `FrameObservation` (line 286) -- single object fallback (LLM returns one at a time)
3. `Vec<FrameObservation>` (line 293) -- bare JSON array fallback

Only when all three fail does it return `VisualError::Analysis`. The third fallback handles the common LLM failure mode of returning a bare `[{...}, {...}]` array.

---

### WR-01: `run_ffmpeg_with_cancel` captures stderr
**Status: RESOLVED**

`src/visual/frame_extractor.rs:632-668` -- Key changes:

- Line 640: `.stderr(std::process::Stdio::piped())` -- stderr is now piped (not null)
- Lines 651-658: On non-zero exit code, `child.stderr.take()` reads the piped stderr via `read_to_string` and logs it via `tracing::warn!`
- Line 661: Returns `status.success()` (the boolean, not the exit code)

The function correctly handles the case where `child.stderr` is `None` (post-take) via `.unwrap_or_default()`.

---

### WR-02: `extract_frames` returns `Vec<PathBuf>`
**Status: RESOLVED**

`src/visual/frame_extractor.rs:44` -- Return type is `Result<(usize, Vec<PathBuf>), VisualError>` instead of just `Result<usize, VisualError>`.

All three fast-path match arms collect paths:
- Line 88: `let paths = collect_keyframe_paths_from_dir(output_dir); Ok((count, paths))`
- Line 103: same pattern in Err arm
- Line 118: same pattern in Ok(0) arm

In `analyzer.rs:121`, the call site destructures: `let (frame_count, frame_paths) = extract_frames(...)`.
In `analyzer.rs:256`, `total_frames: frame_paths.len()` uses the returned paths directly.

---

### WR-03: Fast path match has separate `Err`/`Ok(0)` arms
**Status: RESOLVED**

`src/visual/frame_extractor.rs:84-121` -- The match on `extract_frames_fast_path` result has three distinct arms:

1. `Ok(count) if count > 0` (line 84) -- success, collect and return
2. `Err(e)` (line 91) -- fallback; logs `"Fast path failed, falling back: {}"` with error detail
3. `Ok(0)` (line 106) -- fallback; logs `"Fast path produced 0 frames, falling back"` with no error

Both fallback arms consistently call `cleanup_fast_path_files(output_dir)` then `extract_frames_fallback(...)`.

The Rust compiler's NLL (non-lexical lifetimes) handles the `cancel: Option<CancellationToken>` variable correctly across the mutually exclusive arms. The `cancel.clone()` at line 80 preserves the original for the Err and Ok(0) arms.

---

### WR-04: Docstring says 2-31 not 1-31
**Status: RESOLVED**

`src/visual/frame_extractor.rs:33` -- Docstring reads:
```
/// `quality`: 输出 JPEG 质量，`q:v` 值范围 2-31，越小质量越高，默认 5
```

The validation at line 46-48 uses `!(2..=31).contains(&quality_val)`, which is consistent.

The `analyze_video_frames` docstring at `analyzer.rs:86` reads:
```
/// - `quality` — JPEG 压缩质量（2-31，值越小质量越高，默认 5），None 使用 `extract_frames` 内置默认值
```
This is also consistent.

---

### WR-05: No `#[allow(dead_code)]` on `seconds_to_hhmmssmmm`
**Status: RESOLVED**

`src/visual/frame_extractor.rs:511` -- The function is declared as `pub(crate) fn seconds_to_hhmmssmmm(...)`. There is no `#[allow(dead_code)]` annotation. The `pub(crate)` visibility naturally suppresses dead_code warnings.

It is called at:
- Line 259: `seconds_to_hhmmssmmm(timestamp_secs)` in `extract_frames_fallback`
- Line 493: `seconds_to_hhmmssmmm(total_secs)` in `rename_fast_path_frames`
- Line 725: in test code

---

### WR-06: `BatchPartial` uses joined string not Debug
**Status: RESOLVED**

`src/visual/error.rs:15-20`:
```rust
#[error("部分批次失败: 已分析 {analyzed_count}/{total_count} 批次，{errors}")]
BatchPartial {
    analyzed_count: usize,
    total_count: usize,
    errors: String,
},
```

The `errors` field is `String` (not `Vec<String>`). The `{errors}` in the format string displays the string directly (not Debug-formatted).

In `analyzer.rs:225-229`:
```rust
return Err(VisualError::BatchPartial {
    analyzed_count: success_count,
    total_count: raw_results.len(),
    errors: errors.join("; "),
});
```

The `Vec<String>` is joined with `"; "` before being stored, producing clean output like `"批次 2 解析失败: ...; 批次 5 解析失败: ..."`.

---

### WR-07: Version sorting is numeric not lexicographic
**Status: RESOLVED**

`src/prompt/registry.rs:205-211`:
```rust
versions.sort_by(|a, b| {
    let a_num: u64 = a.trim_start_matches('v').split('.').next()
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    let b_num: u64 = b.trim_start_matches('v').split('.').next()
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    a_num.cmp(&b_num)
});
```

The major version is extracted by stripping the leading `'v'`, splitting on `'.'`, taking the first segment, and parsing as `u64`. The comparison uses `a_num.cmp(&b_num)` for numeric ordering. This correctly handles multi-digit versions (`"v10.0"` sorts after `"v9.0"`).

---

### IN-01: Shared `iterate_dir_files` helper exists
**Status: RESOLVED**

`src/visual/frame_extractor.rs:575-612` -- The `iterate_dir_files` helper consolidates three previously-duplicated directory iteration patterns:

- `extract_frames` stale cleanup (lines 68-72): deletes `keyframe_*.jpg` before extraction
- `rename_fast_path_frames` (lines 461-470): collects `fastframe_*.jpg` files
- `cleanup_fast_path_files` (lines 523-526): deletes `fastframe_*.jpg` on cleanup
- `collect_keyframe_paths_from_dir` (lines 617-621): collects `keyframe_*.jpg` files after extraction

The helper supports prefix/suffix filtering, a callback, and silent error mode. All callers converge on this single implementation.

---

### IN-02: Magic values extracted as const
**Status: RESOLVED**

In `src/visual/analyzer.rs:31-39`:
```rust
const DEFAULT_INTERVAL_SECONDS: f64 = 3.0;
const ANALYSIS_TEMPERATURE: f32 = 0.1_f32;
const ANALYSIS_MAX_TOKENS: u32 = 4096;
```

In `src/prompt/validators.rs:9-16`:
```rust
const MIN_PLOT_ANALYSIS_CHARS: usize = 100;
const MIN_NARRATION_CHARS: usize = 50;
const MIN_NARRATION_PARAGRAPHS: usize = 3;
```

All constants are used in place of the original magic values in their respective functions.

---

### IN-03: Two-tier validation comment in manager.rs
**Status: RESOLVED**

`src/prompt/manager.rs:62-73` -- A comprehensive Chinese comment block ("两阶段校验设计（IN-03）") documents the design rationale:

- Tier 1 (manager.rs): Validates that all required ParameterDefs with no default are present in caller vars. Returns a clear `Validation` error naming the missing parameters.
- Tier 2 (template::render): Defensively checks that all `${variable}` references in the template have corresponding values in the merged variable map. Catches ParameterDef/template content drift.

Both testing paths (line 248-272 for missing variable in render, and line 383-409 for missing required param) confirm the tiered behavior.

---

### IN-04: `debug_assert` for NaN in `seconds_to_hhmmssmmm`
**Status: RESOLVED**

`src/visual/frame_extractor.rs:512`:
```rust
debug_assert!(!total_secs.is_nan(), "seconds_to_hhmmssmmm called with NaN");
```

This is the first executable line of the function, providing a debug-mode assertion before any arithmetic. It fires in debug builds (tests) while having zero runtime overhead in release builds.

---

### IN-05: Re-export documented in types.rs
**Status: RESOLVED**

`src/visual/types.rs:55-57`:
```rust
// strip_code_fence 已提取到 crate::text_utils
// 此 re-export 仅被本文件 test 模块使用，生产代码通过 crate::text_utils 直接导入（IN-05）
pub(crate) use crate::text_utils::strip_code_fence;
```

The comment explains that production code imports directly from `crate::text_utils`, and the re-export exists only for the test module in the same file.

---

## Info Issues (New)

### IN-06: Dead production code in analyzer.rs

**File:** `src/visual/analyzer.rs:321-397`

**Issue:** Two functions are defined but never called from production code:

1. `collect_frame_paths` (line 321) -- Collects `keyframe_*.jpg` files from a directory, validates filenames, and sorts numerically. Production code uses the paths returned directly from `extract_frames` (which internally calls `collect_keyframe_paths_from_dir` in `frame_extractor.rs:615-623`).

2. `extract_frame_number_from_keyframe` (line 372) -- Extracts frame number from `keyframe_XXXXXX_*.jpg` filenames. Only called by `collect_frame_paths` and test code.

Both functions are reachable from `#[cfg(test)]` modules, which is why the Rust compiler does not emit dead_code warnings. However, they are unreachable in production builds. The two implementations (`collect_frame_paths` with numeric sort vs. `collect_keyframe_paths_from_dir` with lexical sort) represent a maintenance hazard: any developer modifying one may expect it to affect behavior.

**Fix:** Remove `collect_frame_paths` and `extract_frame_number_from_keyframe` from production code. If numeric sorting (as opposed to lexical via `paths.sort()`) is desired, port the `extract_frame_number_from_keyframe` logic into `collect_keyframe_paths_from_dir` and delete the duplicates from `analyzer.rs`.

---

### IN-07: Duplicate unused format arguments in validators.rs

**File:** `src/prompt/validators.rs:86, 98, 119`

**Issue:** Three `format!` calls pass 3 arguments for 2 format placeholders. The third argument is silently ignored by the Rust `format!` macro -- this is valid Rust but indicates a copy-paste pattern.

Line 86:
```rust
format!(
    "解说文案过短: {} 字符（需要 >= {}）",
    normalized.chars().count(),
    MIN_NARRATION_CHARS,
    normalized.chars().count()  // duplicate of arg 1, ignored
)
```

Line 98:
```rust
format!(
    "解说文案段落数不足: {} 段（需要 >= {}）",
    paragraphs.len(),
    MIN_NARRATION_PARAGRAPHS,
    paragraphs.len()  // duplicate of arg 1, ignored
)
```

Line 119:
```rust
format!(
    "剧情分析内容不足: {} 字符（需要 >= {}）",
    trimmed.chars().count(),
    MIN_PLOT_ANALYSIS_CHARS,
    trimmed.chars().count()  // duplicate of arg 1, ignored
)
```

No runtime error results from this (extra args are silently discarded by `format!`). However, the pattern is a latent defect indicator: if someone later expands the format string to use a third placeholder, and the intent was for the third argument to be `MIN_PLOT_ANALYSIS_CHARS`, the duplicated value would produce incorrect output. The duplicated `.chars().count()` calls also represent wasted computation.

**Fix:** Remove the redundant third argument from each `format!` call.

---

_Reviewed: 2026-05-07T20:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
