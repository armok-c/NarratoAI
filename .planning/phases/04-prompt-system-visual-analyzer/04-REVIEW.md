---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - narratoai-core/Cargo.toml
  - narratoai-core/src/lib.rs
  - narratoai-core/src/text_utils.rs
  - narratoai-core/src/prompt/mod.rs
  - narratoai-core/src/prompt/types.rs
  - narratoai-core/src/prompt/error.rs
  - narratoai-core/src/prompt/registry.rs
  - narratoai-core/src/prompt/template.rs
  - narratoai-core/src/prompt/manager.rs
  - narratoai-core/src/prompt/validators.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/types.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/analyzer.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 04: Prompt System + Visual Analyzer -- Code Review Report

**Reviewed:** 2026-05-07
**Depth:** standard
**Files Reviewed:** 19 (12 Rust source, 7 Markdown templates)
**Status:** issues_found

## Summary

Reviewed the Prompt system (types, error, registry, template engine, manager, validators, register, and 7 template files) and Visual analyzer (frame extraction, data types, batch analysis orchestrator). The codebase is well-structured with clear error types, thorough test coverage, and sensible design patterns. However, three warnings were identified: a validation regex gap that lets filter-using variables bypass registration-time checks, a double-wrapped error that produces a tripled Chinese prefix in error messages, and a cancellation token ownership issue that makes the LLM analysis phase uncancellable. Prior WR-01/WR-02/WR-03 from the previous review pass have been correctly addressed (progress forwarding, cancel token parameter, and stale keyframe cleanup are now in place).

## Warnings

### WR-01: `validate_prompt_parameters` regex misses filter syntax

**File:** `narratoai-core/src/prompt/registry.rs:84`
**Issue:** The `validate_prompt_parameters` method uses regex `r"\$\{(\w+)\}"` to find template variables and validate them against `ParameterDef`. However, the template rendering engine in `template.rs:81` uses `r"\$\{(\w+)(?:\|(\w+))?\}"` which also matches `${variable|filter}` syntax. The `validate_prompt_parameters` regex does not match filter-style placeholders because `|` is not a `\w` character and the regex lacks the optional `(?:\|(\w+))?` group. As a result, a template using `${undeclared_var|upper}` would pass registration-time validation silently (the variable goes undetected) and only fail at render time with a `TemplateRender` error instead of the intended registration-time `Validation` error. This creates a validation gap where undeclared variables with filters are caught later and with a different error type.

The current templates do not use filters, so this is not triggered today, but it is a latent defect that will affect future template authors who use the filter syntax.

**Fix:** Update the regex in `validate_prompt_parameters` (line 84) to match the render-time regex, and adjust the variable extraction logic to handle both captured groups:

```rust
// In registry.rs, line 84:
let re = Regex::new(r"\$\{(\w+)(?:\|(\w+))?\}").map_err(|e| {
    PromptError::TemplateRender(format!("正则编译失败: {}", e))
})?;
```

The capture group extraction loop (lines 96-101) should then be updated to collect from group 1 (variable name) only, since group 2 (filter name) is optional:

```rust
for caps in re.captures_iter(&prompt.content) {
    if let Some(name) = caps.get(1).map(|m| m.as_str()) {
        if !name.is_empty() && !declared.contains(name) {
            undeclared.push(name);
        }
    }
}
```

---

### WR-02: `map_err` in `analyze_video_frames` double-wraps `FrameExtraction` error

**File:** `narratoai-core/src/visual/analyzer.rs:113-116`
**Issue:** The `extract_frames` function already returns `VisualError::FrameExtraction(...)`, whose `Display` implementation (via `thiserror`) produces `"帧提取失败: {0}"`. The `map_err` at line 113 wraps this in ANOTHER `VisualError::FrameExtraction` with `format!("帧提取失败: {}", e)`, where `e.to_string()` already contains the prefix. The resulting `Display` output produces a tripled prefix:

```
帧提取失败: 帧提取失败: 帧提取失败: <original detail>
```

instead of the correct single prefix:

```
帧提取失败: <original detail>
```

This makes error messages confusing and unnecessarily verbose during debugging.

**Fix:** Replace with a passthrough that preserves the original error while still logging:

```rust
.map_err(|e| {
    error!(error = %e, "帧提取失败");
    e  // return original error unchanged
})?;
```

---

### WR-03: Cancel token consumed by `extract_frames`, unavailable for LLM analysis

**File:** `narratoai-core/src/visual/analyzer.rs:106-112`
**Issue:** The `cancel: Option<CancellationToken>` parameter is moved into `extract_frames()` at line 110. After `extract_frames` returns, the token is no longer available in `analyze_video_frames`. The subsequent `analyze_images()` call at line 160 -- which can be a long-running LLM operation involving multiple batched API requests -- cannot be cancelled. Furthermore, the `analyze_images` method signature does not accept a `CancellationToken`, so even if the token were preserved, it could not be passed through.

This means that if a cancel signal arrives during the LLM analysis phase (which can take many seconds or even minutes for large batches of images sent to a vision LLM), the operation continues to completion with no way to interrupt it. The cancel token is effectively single-use, protecting only frame extraction.

**Fix:** Clone the cancel token before passing it to `extract_frames`, and check it before launching `analyze_images`:

```rust
// Before line 106:
let cancel_for_after_extract = cancel.clone();

// Line 110: pass the original
let frame_count = extract_frames(
    video_path, output_dir,
    interval_seconds.unwrap_or(3.0), quality,
    progress_for_extract,
    cancel,
)
.await
.map_err(|e| { error!(error = %e, "帧提取失败"); e })?;

// Before line 137 (rendered_prompt):
if let Some(ref cancel) = cancel_for_after_extract {
    if cancel.is_cancelled() {
        return Err(VisualError::Analysis("分析被取消".into()));
    }
}
```

Additionally, consider adding a `CancellationToken` parameter to `LlmProvider::analyze_images` for true responsive cancellation during the LLM phase.

---

## Info

### IN-01: Unnecessary `#[allow(dead_code)]` on `seconds_to_hhmmssmmm`

**File:** `narratoai-core/src/visual/frame_extractor.rs:511`
The function `seconds_to_hhmmssmmm` is used in production code at line 493 (`rename_fast_path_frames`) and in tests. The `#[allow(dead_code)]` attribute is redundant and misleading -- it suggests the function is unused when it is actively called. Remove the annotation.

### IN-02: Progress callback double-boxing

**File:** `narratoai-core/src/visual/analyzer.rs:97-101`
`shared_progress` wraps `Option<ProgressCallback>` in `Arc`, and `progress_for_extract` creates a NEW `Box<dyn Fn(...)>` whose closure dereferences the `Arc` to call the original. This creates a call chain with three layers of indirection: `Box(outer)` -> `Arc` -> `Box(inner)` -> `dyn Fn`. Consider restructuring to avoid the outer `Box`, for example by storing an `Arc<dyn Fn(...) + Send + Sync>` directly rather than `Arc<Box<dyn Fn(...)>>`, or by removing the second boxing entirely and passing `shared_progress` directly (as `Option<Arc<...>>`) when the callee can accept the `Arc`-wrapped form.

### IN-03: Template style annotation contradicts the style variable

**File:** `narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md:13`
Line 13 reads:
```
3. 使用${style}风格的语调——正式但不僵硬
```

When the `style` variable resolves to a non-formal value (e.g., `"幽默风趣"`, `"轻松活泼"`), the instruction becomes self-contradictory: "使用幽默风趣风格的语调--正式但不僵硬." The hardcoded `"--正式但不僵硬"` suffix conflicts with the variable content. Remove the contradictory suffix:

```
3. 使用${style}风格的语调
```

---

## File-by-File Summary

| File | Lines | Findings |
|------|-------|----------|
| `Cargo.toml` | 48 | None |
| `lib.rs` | 34 | None |
| `text_utils.rs` | 19 | None (correct) |
| `prompt/mod.rs` | 8 | None |
| `prompt/types.rs` | 56 | None |
| `prompt/error.rs` | 68 | None |
| `prompt/registry.rs` | 378 | WR-01 |
| `prompt/template.rs` | 329 | None |
| `prompt/manager.rs` | 400 | None |
| `prompt/validators.rs` | 271 | None |
| `prompt/register.rs` | 194 | None |
| `prompt/templates/documentary/frame_analysis_v1.0.md` | 22 | None |
| `prompt/templates/documentary/narration_generation_v2.0.md` | 19 | IN-03 |
| `prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md` | 87 | None |
| `prompt/templates/short_drama_editing/plot_extraction_v2.0.md` | 110 | None |
| `prompt/templates/short_drama_narration/plot_analysis_v1.0.md` | 67 | None |
| `prompt/templates/short_drama_narration/script_generation_v1.0.md` | 19 | None |
| `prompt/templates/short_drama_narration/script_generation_v2.0.md` | 268 | None |
| `visual/mod.rs` | 4 | None |
| `visual/error.rs` | 72 | None |
| `visual/types.rs` | 237 | None |
| `visual/frame_extractor.rs` | 831 | IN-01 |
| `visual/analyzer.rs` | 566 | WR-02, WR-03, IN-02 |

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
