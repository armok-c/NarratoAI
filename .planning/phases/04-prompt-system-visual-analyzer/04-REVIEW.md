---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - narratoai-core/Cargo.toml
  - narratoai-core/src/lib.rs
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
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/types.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/analyzer.rs
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 18 (prompt system + visual analyzer + templates)
**Status:** issues_found

## Summary

Adversarial review of the Phase 4 narratoai-core Rust code: Prompt system (types, error, registry, template rendering, manager facade, validators, register functions, 4 template markdown files) and Visual analyzer infrastructure (frame extractor with 4-level fallback, analyzer orchestrator, types). Overall architecture is well-structured with clean module boundaries. No BLOCKER-level issues found -- no panics, race conditions, data loss risks, or auth bypasses. However, 4 WARNING-level issues involve incorrect behavior: template filter pass re-interprets variable values as filter expressions (a template injection vector), silent frame ordering corruption from unparseable filenames, prompts silently dropped from list output, and API layer gap between parameter validation and template content. 5 INFO-level quality issues noted.

## Warnings

### WR-01: Template filter pass 3 re-interprets variable values as filter expressions

**File:** `narratoai-core/src/prompt/template.rs:120-163`
**Issue:** The `render()` function applies filter syntax (`${var|filter_name}`) in pass 3 on the *output* of pass 2 (variable substitution). If a variable value happens to contain text matching `${var|filter}`, pass 3 will reinterpret it as a filter expression and apply transformations. This is effectively a template injection vector.

Example with `vars = {"name": "${subject|upper}"}` and template `"Hello, ${name}!"`:
1. Pass 2 replaces `${name}` with `"${subject|upper}"`.
2. Pass 3 sees `${subject|upper}` and applies the `upper` filter to `subject`. If `subject` is in vars, the user gets an unexpected uppercase transformation. If `subject` is missing, pass 3's validation error fires with a confusing "缺少必需参数: subject" message.

The code comment at lines 82-83 documents "调用方负责验证变量值的合法性" (callers validate variable values), but no defense-in-depth exists at the API boundary. This is also a potential security concern if user-provided data ever reaches template variables -- filter expressions could be smuggled.

The builtin `json` filter (line 52-54) is a special concern: calling `serde_json::to_string()` on a value containing `${...|...}` would produce a JSON string that still contains the filter expression as literal text, and pass 3 would interpret it on the JSON representation, not the original value.

**Fix:** Apply pass 3 filter matching on the original template string (extracting filter expressions before variable substitution), not on the substituted result. A safer approach is a single-pass rendering pipeline that processes both plain variables and filter expressions from the source template, escaping any `${...}` patterns in substitution values.

```rust
// Single pass: extract all placeholders (plain + filter) from source template,
// then resolve each against the vars HashMap, applying filters inline.
// This prevents substituted values from being re-parsed.
```

Alternatively, after pass 2, escape the substituted regions before pass 3, then unescape the final result.

### WR-02: Unparseable keyframe filenames silently sort to front, corrupting LLC input order

**File:** `narratoai-core/src/visual/analyzer.rs:310-327`
**Issue:** `extract_frame_number_from_keyframe()` returns `0` for any filename that cannot be parsed. When `collect_frame_paths()` sorts frames (lines 310-313), all unparseable filenames silently sort to frame number 0, displacing the actual frame 0. The LLM receives frames in wrong order with no error or warning.

Example: if a file `keyframe_INVALID_xxx.jpg` somehow exists in the output directory:
- `extract_frame_number_from_keyframe` strips `"keyframe_"`, gets `"INVALID_xxx.jpg"`, splits on `_` gets `["INVALID", "xxx.jpg"]`, tries to parse `"INVALID"` as `u64` which fails, returns `0`.
- Frame `000000` also returns `0`.
- Both sort equally -- order between them is undefined (stable sort not guaranteed, and `sort_by` with equal values preserves insertion order for `sort_by` in practice but not guaranteed by `sort()`).

In normal operation all files follow the `keyframe_{:06}_{:09}.jpg` convention, so this only manifests with external modification or extraction error. The silent failure makes it hard to diagnose.

**Fix:** Validate all filenames before sorting. If any filename cannot be parsed, return an error from `collect_frame_paths()`. Also add a warning log when parse fails.

```rust
fn extract_frame_number_from_keyframe(path: &Path) -> Result<u64, VisualError> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let num_str = name.strip_prefix("keyframe_")
        .and_then(|s| s.split('_').next())
        .ok_or_else(|| VisualError::FrameExtraction(
            format!("无法从文件名解析帧序号: {}", name)
        ))?;
    num_str.parse::<u64>().map_err(|_| VisualError::FrameExtraction(
        format!("帧序号不是有效数字: {} (from {})", num_str, name)
    ))
}
```

### WR-03: `list_prompts` silently drops prompts with no default version

**File:** `narratoai-core/src/prompt/registry.rs:139-163`
**Issue:** `list_prompts(category)` only includes prompt names that have a default version set in `default_versions`. If a prompt name exists in the registry (versions registered) but none have `is_default=true` (register() fallback at line 62 only sets default when `!has_default`), the prompt is silently dropped from the result. There is no logging, error, or indication to the caller.

This happens if a name has versions but the default_versions entry was accidentally removed or never set. The empty result misleads callers into thinking the category has no prompts.

**Fix:** Add a fallback when no default version is found for a name: use the first available version (sorted by version string). Log a warning when falling back:

```rust
let version = self.default_versions
    .get(category)
    .and_then(|m| m.get(name.as_str()))
    .or_else(|| {
        let fallback = names_map.get(name)
            .and_then(|m| m.keys().next().map(|s| s.as_str()));
        if let Some(v) = fallback {
            tracing::warn!("no default version for {}.{}, using {}", category, name, v);
        }
        fallback
    });
```

### WR-04: Parameter validation in `render_prompt` has unreachable gap with template content

**File:** `narratoai-core/src/prompt/manager.rs:63-73`
**Issue:** `render_prompt` validates that all `ParameterDef` entries with `required: true` and no default are present in `vars`. However, there is no cross-validation between template content and `ParameterDef` declarations at registration time. If a template uses `${some_var}` that isn't declared in `ParameterDef`, the error surfaces inside `template::render()`, producing a `TemplateRender` error rather than a `Validation` error. This creates an inconsistent error path:

- Missing required-param (in `ParameterDef`): `Validation` error with clear message (manager.rs)
- Missing template-var (not in `ParameterDef`, not in `vars`): `TemplateRender` error (template.rs) -- user checks ParameterDef, doesn't find the variable, gets confused.

All 4 reviewed templates match their ParameterDefs correctly, so no current bug exists. But future prompt additions risk this gap because no compile-time or registration-time check enforces consistency.

**Fix:** Add validation at registration time (in `register.rs`) that cross-checks all `${variable}` references in the template content against declared `ParameterDef` entries:

```rust
fn validate_prompt_parameters(prompt: &Prompt) -> Result<(), PromptError> {
    let re = regex::Regex::new(r"\$\{(\w+)\}").unwrap();
    let mut referenced: HashSet<&str> = HashSet::new();
    for caps in re.captures_iter(&prompt.content) {
        if let Some(name) = caps.get(1).map(|m| m.as_str()) {
            // Skip names followed by | (filter syntax) -- handled separately
            referenced.insert(name);
        }
    }
    let declared: HashSet<&str> = prompt.metadata.parameters.iter()
        .map(|p| p.name.as_str())
        .collect();
    for var in &referenced {
        if !declared.contains(var) {
            return Err(PromptError::Validation(format!(
                "模板引用了未声明的参数 '{}'", var
            )));
        }
    }
    Ok(())
}
```

## Info

### IN-01: Unnecessary `#[allow(dead_code)]` on `seconds_to_hhmmssmmm`

**File:** `narratoai-core/src/visual/frame_extractor.rs:497`
**Issue:** `seconds_to_hhmmssmmm` is `pub(crate)` and used by both `rename_fast_path_frames` (line 480) and `extract_frames_fallback` (line 234). The `#[allow(dead_code)]` annotation is redundant. It suppresses future dead-code warnings that would legitimately detect unused functions during refactoring.

**Fix:** Remove `#[allow(dead_code)]`.

### IN-02: Unnecessary `pub(crate)` re-export of `strip_code_fence` in `visual/types.rs`

**File:** `narratoai-core/src/visual/types.rs:56`
**Issue:** `strip_code_fence` is re-exported as `pub(crate) use crate::text_utils::strip_code_fence` but is only used in the test module (lines 104, 114 via `use super::*;`). No production code in `visual/types.rs` calls it. The re-export unnecessarily broadens the module's API surface.

**Fix:** Remove the re-export and import directly in the test module:
```rust
#[cfg(test)]
mod tests {
    use crate::text_utils::strip_code_fence;
    // ...
}
```

### IN-03: Misleading test comment -- "三个 prompt 版本" vs count by name

**File:** `narratoai-core/src/prompt/register.rs:172`
**Issue:** The comment reads `// 验证 short_drama_narration 分类下三个 prompt 版本均已注册` but the preceding assertion tests `list_prompts(...)` which returns 2 (one per unique name using default version, since `script_generation` has both v1.0 and v2.0). The comment implies 3 versions in the list_prompts result, but the test actually passes because it asserts `len() == 2`.

**Fix:** Update the comment:
```rust
// 验证 short_drama_narration: 2 个名称 (plot_analysis, script_generation), 3 个版本
```

### IN-04: `analyze_video_frames` does not expose cancel token

**File:** `narratoai-core/src/visual/analyzer.rs:99-100`
**Issue:** `analyze_video_frames` passes `None` for cancel to `extract_frames`. Frame extraction (especially the fallback path) can be long-running (many frames, 4-level retry per frame). Without a cancel token in the public API, the operation cannot be externally cancelled, potentially blocking the tokio runtime thread pool.

**Fix:** Add a `cancel: Option<CancellationToken>` parameter to `analyze_video_frames` and propagate it:
```rust
pub async fn analyze_video_frames(
    ...
    cancel: Option<CancellationToken>,
) -> Result<BatchAnalysisResult, VisualError> {
    let frame_count = extract_frames(
        video_path, output_dir, interval_seconds.unwrap_or(3.0),
        quality, None, cancel,
    ).await...;
```

### IN-05: `register.rs` references template files excluded from review scope

**File:** `narratoai-core/src/prompt/register.rs:78`, `:110`, `:144`
**Issue:** The `register_all_prompts()` function uses `include_str!()` at compile time to embed three template files that are not in the review scope:
- `templates/short_drama_editing/subtitle_analysis_v2.0.md` (line 78)
- `templates/short_drama_narration/plot_analysis_v1.0.md` (line 110)
- `templates/short_drama_narration/script_generation_v2.0.md` (line 144)

These files were confirmed present on disk but were not reviewed. Any bug or injection vulnerability in these templates is invisible to this review despite being compiled into the binary.

**Fix:** Include all `include_str!`-ed template files in the review scope during code reviews.

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
