---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T15:20:00Z
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
  warning: 4
  info: 4
  total: 9
status: issues_found
---

# Phase 4: Code Review Report

**Reviewed:** 2026-05-07T15:20:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed the Prompt System (types, registry, template engine, manager, validators, registration, 4 prompt templates) and Visual Analyzer (frame extractor, analyzer, types, error) across 19 source files at standard depth.

The architecture is well-structured: clean error enums with thiserror, proper `RwLock` usage with poison handling, correct 3-level registry index, sensible 4-level FFmpeg fallback with per-level cancellation checks, and numeric frame sorting. Template variables in all .md files are consistent with their `ParameterDef` declarations in `register.rs`. Previous review findings (CR-01 busy-wait, WR-02 shared temp dirs, WR-03 lexical sort, WR-04 interval upper bound, WR-05 missing cancel checks) have all been fixed.

Found 1 critical issue and 4 warnings. The critical issue is a logic error in `strip_code_fence` that fails to strip the closing ` ``` ` when JSON content abuts the fence without trailing whitespace. This causes downstream JSON deserialization to fail on affected LLM responses. Warnings cover O(n^2) duplicate detection in template rendering, an empty-result blind spot in the analysis pipeline, missing code-fence stripping in the JSON validator, and an uncanceled FFmpeg event loop in the fast path.

## Critical Issues

### CR-01: `strip_code_fence` fails to strip closing fence when JSON abuts ` ``` ` without trailing whitespace

**File:** `src/visual/types.rs:51-60`
**Classification:** BLOCKER

**Issue:** The function strips the opening ` ```json ` or ` ``` ` correctly, but when stripping the closing ` ``` ` suffix, it has a flawed conditional: if the remaining string after stripping ` ``` ` does NOT end with whitespace, it returns `content` (the unstripped version with ` ``` ` still attached). This means LLM responses like `` ```json\n{"key":"value"}``` `` (where JSON closes directly against the fence with no newline before ` ``` `) will not have the closing fence removed.

Trace for input `` "```json\n{}```" ``:
1. `strip_prefix("```json")` = `Some("\n{}```")`
2. `trim_start()` = `Some("{}```")`
3. `content = "{}```"`
4. `content.strip_suffix("```")` = `Some("{}")` (correctly stripped)
5. `s = "{}"`, `s.ends_with(|c| c.is_whitespace())` = **false**
6. Returns `content = "{}```"` -- **closing fence preserved, deserialization will fail**

The `else` branch on line 57 is the bug: it should return `s` (the stripped version), not `content`.

**Fix:**

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
        .map(|s| s.trim_end())
        .unwrap_or(content)
}
```

The fix simplifies the logic: if we can strip the closing fence, trim trailing whitespace from the remainder and return it. If not, return content as-is.

## Warnings

### WR-01: O(n^2) duplicate detection in template variable validation

**File:** `src/prompt/template.rs:96`
**Classification:** WARNING

**Issue:** The `render` function uses `missing.contains(&name.to_string())` to deduplicate missing variable names. This is an O(n) scan inside an O(n) loop (where n = number of template variables), giving O(n^2) worst case. For templates with many unique variables this degrades, though typical templates have fewer than 20 variables so practical impact is low.

The same pattern is repeated for filter variables on line 128.

**Fix:** Use a `HashSet<String>` for the missing set:

```rust
let mut missing: HashSet<String> = HashSet::new();
for caps in var_re.captures_iter(template) {
    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    if !name.is_empty() && !vars.contains_key(name) {
        missing.insert(name.to_string());
    }
}
```

### WR-02: Empty-result blind spot -- zero observations with zero errors succeeds silently

**File:** `src/visual/analyzer.rs:177-205`
**Classification:** WARNING

**Issue:** The empty-result barrier on line 178 checks `observations.is_empty() && !errors.is_empty()`. If all LLM batches succeed but every batch returns an empty `frame_observations` array, then `observations` is empty and `errors` is also empty. The function returns a `BatchAnalysisResult` with 0 observations, 0 errors, and `analyzed_batches == raw_results.len()`. The caller receives a "success" that contains no useful data.

This scenario can occur if the LLM returns valid JSON like `{"frame_observations": [], "overall_activity_summary": "no frames detected"}`.

**Fix:** Add a check after the barrier:

```rust
if observations.is_empty() {
    return Err(VisualError::Analysis(
        "所有批次返回空观察结果".into(),
    ));
}
```

### WR-03: `validate_json` does not strip code fences before parsing

**File:** `src/prompt/validators.rs:34-53`
**Classification:** WARNING

**Issue:** The JSON validator calls `serde_json::from_str(trimmed)` on the trimmed output, but does not strip ` ```json...``` ` code fences. The codebase already has `strip_code_fence` in `visual/types.rs`, and the `parse_and_retry` function in `analyzer.rs` correctly strips fences before deserializing. However, the `validate_output` function used in `sde/script_gen.rs:117` does not strip fences first.

If an LLM returns JSON wrapped in a code fence, the validator will reject it with "JSON format invalid", even though the actual JSON is valid. The `visual/types::strip_code_fence` function is `pub(crate)`, so it is accessible from the validators module.

**Fix:** Strip code fence before JSON validation:

```rust
fn validate_json(output: &str) -> Result<(), PromptError> {
    let cleaned = crate::visual::types::strip_code_fence(output);
    let trimmed = cleaned.trim();
    // ... rest unchanged
}
```

### WR-04: Fast-path FFmpeg event loop blocks indefinitely on stalled FFmpeg

**File:** `src/visual/frame_extractor.rs:157-166`
**Classification:** WARNING

**Issue:** The fast path iterates `child.iter()` inside a `for event in iter` loop. Cancellation is only checked when a new event arrives (`if cancel.is_cancelled()` on line 158). If FFmpeg stalls (e.g., seeking to a very late timestamp, or I/O hang), the thread blocks on `iter.next()` inside `spawn_blocking` without checking the cancellation token. This can delay cancellation by minutes for large videos or slow I/O.

The code comment on lines 153-156 already documents this limitation. The fallback path avoids this by using blocking `child.wait()` with cancellation checks between calls. No fix is required beyond the existing documentation, but callers should be aware that `CancellationToken` is best-effort for the fast path.

**Fix:** No code change required (limitation is documented). If stronger cancellation is needed, the fast path should be converted to use `std::process::Command` directly with `child.wait()`, similar to the fallback path's `run_ffmpeg_with_cancel`.

## Info

### IN-01: Regex patterns recompiled on every `render()` call

**File:** `src/prompt/template.rs:85-87,119-121`

**Issue:** Two `Regex::new()` calls compile the same patterns (`r"\$\{(\w+)\}"` and `r"\$\{(\w+)\|(\w+)\}"`) on every `render()` invocation. Use `std::sync::OnceLock<Regex>` or `lazy_static!` to compile once.

### IN-02: `builtin_filters()` allocates new `HashMap` on every `render()` call

**File:** `src/prompt/template.rs:10-57`

**Issue:** The 6-entry filter HashMap is recreated on every template render. Use `OnceLock` for lazy one-time initialization.

### IN-03: `validate_narration_script` paragraph minimum may reject valid single-paragraph outputs

**File:** `src/prompt/validators.rs:74-83`

**Issue:** The validator requires at least 3 paragraphs (split by `\n\n`). Some LLMs may produce valid narration as a single continuous paragraph. The 3-paragraph minimum is a heuristic that may need tuning based on actual LLM output patterns.

### IN-04: `script_generation` v2.0 silently overrides v1.0 default status

**File:** `src/prompt/register.rs:115-144`

**Issue:** Both `script_generation` v1.0 and v2.0 are registered with `is_default: true`. The second registration silently overrides the first's default. This is intentional (v2.0 should be the default), but the behavior is implicit. A comment clarifying intent would help future maintainers.

---

_Reviewed: 2026-05-07T15:20:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
