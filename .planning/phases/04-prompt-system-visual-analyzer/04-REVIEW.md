---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T12:00:00Z
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
  critical: 1
  warning: 2
  info: 2
  total: 5
status: issues_found
---

# Phase 4: Code Review Report (Re-review #4, Iteration 5)

**Reviewed:** 2026-04-30T12:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed the complete prompt system and visual analyzer subsystems (19 files total). All four previously identified and fixed issues (CR-01 byte-index slice, WR-01 pub visibility, WR-02 unused parameter, WR-01 re-review #3 frame_data metadata) remain correctly applied. The `truncate_str` helper properly handles UTF-8 boundaries, `ParsedBatch`/`parse_and_retry` are private, the `_context` parameter is gone, and `frame_data` is removed from the frame_analysis registration.

One new BLOCKER was found: `PromptManager::render_prompt()` never applies default values from `ParameterDef.default` for optional parameters. When a caller omits an optional parameter like `language` (declared `required: false, default: Some("zh-CN")`), the template engine treats `${language}` as a missing required variable and returns a runtime error, contradicting the metadata declaration. Two warnings: misleading error messages from the `TemplateRender` variant being reused for non-template errors (RwLock poisoning, regex compilation), and unknown filter names being silently preserved instead of raising errors.

## Previous Fixes Verification

| Previous Issue | Status | Location |
|---|---|---|
| CR-01: Byte-index slice on UTF-8 | Verified fixed | `src/visual/analyzer.rs:271` -- `truncate_str` uses `char_indices().nth()` |
| WR-01 (original): `ParsedBatch`/`parse_and_retry` were pub | Verified fixed | `src/visual/analyzer.rs:36,198` -- both are private |
| WR-02: Unused `_context` parameter | Verified fixed | No `_context` parameter exists anywhere |
| WR-01 (re-review #3): `frame_data` in metadata | Verified fixed | `src/prompt/register.rs:40-43` -- only `video_description` and `language` parameters |

## Critical Issues

### CR-01: Optional parameter default values are never applied during template rendering

**File:** `src/prompt/manager.rs:48-57` and `src/prompt/template.rs:76-99`
**Issue:** `PromptManager::render_prompt()` calls `template::render()` directly with the caller-provided `vars` HashMap. The `render()` function treats any `${variable}` in the template content as a hard requirement -- if it is missing from `vars`, the function returns `PromptError::TemplateRender("缺少必需参数: language")`.

However, `ParameterDef` declares parameters like `language` and `style` as `required: false` with `default: Some("zh-CN")` or `default: Some("正式")`. These default values exist only as metadata and are never consulted during rendering.

This means any caller that omits an optional parameter (relying on its declared default) will get a runtime error claiming the parameter is "required", which contradicts the `required: false` declaration. For example, rendering the `frame_analysis` template without providing `language` will fail:

```
TemplateRender("缺少必需参数: language")
```

even though `language` is declared as `required: false, default: Some("zh-CN")`.

All four templates in `register.rs` declare optional parameters with defaults (`language` in all four, `style` in two), so this affects every prompt in the system.

**Fix:** In `PromptManager::render_prompt()`, before calling `template::render()`, merge the caller-provided `vars` with default values from the prompt's metadata. One approach:

```rust
pub fn render_prompt(
    &self,
    category: &str,
    name: &str,
    version: Option<&str>,
    vars: &HashMap<&str, &str>,
) -> Result<String, PromptError> {
    let prompt = self.get_prompt(category, name, version)?;

    // Merge defaults: caller vars take precedence over parameter defaults
    let mut merged: std::collections::HashMap<String, String> = HashMap::new();
    for param in &prompt.metadata.parameters {
        if let Some(ref default) = param.default {
            merged.insert(param.name.clone(), default.clone());
        }
    }
    for (k, v) in vars {
        merged.insert(k.to_string(), v.to_string());
    }

    let merged_refs: HashMap<&str, &str> = merged
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    template::render(&prompt.content, &merged_refs)
}
```

Alternatively, change `template::render()` to accept a type that carries defaults, or add a separate `render_with_defaults()` method.

## Warnings

### WR-01: Misleading error messages from TemplateRender variant reuse

**File:** `src/prompt/error.rs:6-7`, `src/prompt/manager.rs:38,66,74,93,101`, `src/prompt/template.rs:79,113`
**Issue:** The `PromptError::TemplateRender` variant has the display attribute `#[error("模板渲染失败: 缺少必需参数 {0}")]`, which produces misleading messages when used for non-template-rendering errors:

- RwLock poisoning becomes: `"模板渲染失败: 缺少必需参数 注册中心读取锁失败: ..."`
- Regex compilation failure becomes: `"模板渲染失败: 缺少必需参数 正则编译失败: ..."`

The `TemplateRender(String)` variant is being used as a catch-all for at least three distinct error categories: missing variables, regex compilation failures, and lock poisoning. Each produces a confusing compound message.

**Fix:** Change the display attribute to a generic message:
```rust
#[error("模板渲染失败: {0}")]
TemplateRender(String),
```

Or introduce separate error variants for lock failures and regex errors:
```rust
#[error("注册中心并发访问失败: {0}")]
RegistryLock(String),

#[error("内部错误: {0}")]
Internal(String),
```

### WR-02: Unknown filter names silently preserved instead of raising errors

**File:** `src/prompt/template.rs:132-141`
**Issue:** When a template uses `${variable|unknown_filter}`, the `render()` function silently preserves the original `${variable|unknown_filter}` text in the output instead of raising an error. The fallback branch at line 137-139 returns the raw match:

```rust
} else {
    // 过滤器未找到或变量不存在，保留原始文本
    caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
}
```

A typo in a filter name (e.g., `${name|uppper}` instead of `${name|upper}`) will go undetected and produce a prompt with literal `${name|uppper}` text sent to the LLM, wasting an API call and producing unpredictable results.

**Fix:** After the filter regex pass, check whether any `${...|...}` patterns remain in the output and report them as errors. Alternatively, validate all filter names in a pre-pass before applying them:

```rust
// Validate filter names before applying
for caps in filter_re.captures_iter(&result) {
    let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    if !filter_name.is_empty() && !filters.contains_key(filter_name) {
        return Err(PromptError::TemplateRender(format!(
            "未知过滤器: {}", filter_name
        )));
    }
}
```

## Info

### IN-01: PromptError::Version variant is dead code

**File:** `src/prompt/error.rs:22-23`
**Issue:** The `Version(String)` variant of `PromptError` is never constructed in production code. It only appears in a unit test (line 64). This is dead code that increases the API surface unnecessarily.
**Fix:** Either remove the variant or add a code path that uses it (e.g., version string validation during registration).

### IN-02: total_frames reported from extract_frames count, not actual file count

**File:** `src/visual/analyzer.rs:184`
**Issue:** `BatchAnalysisResult.total_frames` is set to `frame_count` (returned by `extract_frames()`) rather than `frame_paths.len()` (the actual number of files collected and sent to the LLM). These should normally be the same, but if a naming mismatch or filesystem race occurred, they could diverge. The value sent to the LLM is based on `frame_paths`, while the value reported to the caller is `frame_count`.
**Fix:** Consider using `frame_paths.len()` for `total_frames` since it represents the actual data processed, or add a consistency check if the counts differ.

---

_Reviewed: 2026-04-30T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 5 (re-review #4)_
