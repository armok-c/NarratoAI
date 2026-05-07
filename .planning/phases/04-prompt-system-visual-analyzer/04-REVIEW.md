---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - src/lib.rs
  - src/prompt/error.rs
  - src/prompt/manager.rs
  - src/prompt/mod.rs
  - src/prompt/register.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/prompt/types.rs
  - src/prompt/validators.rs
  - src/visual/analyzer.rs
  - src/visual/error.rs
  - src/visual/frame_extractor.rs
  - src/visual/mod.rs
  - src/visual/types.rs
  - Cargo.toml
findings:
  critical: 2
  warning: 5
  info: 3
  total: 10
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed the prompt system (types, error enums, registry, template renderer, manager facade, validators, template registration) and visual analyzer (frame extraction with fast path + fallback, batch analysis orchestrator).

Found 2 critical issues and 5 warnings. The critical issues are: (1) `format!` in `analyze_video_frames` will panic if `prompt_template` contains literal `{` or `}` characters; (2) the `$variable` bare-dollar regex in the template renderer silently corrupts template content containing dollar signs followed by word characters. The warnings cover missing input validation in the analyzer, misleading field naming in `BatchAnalysisResult`, potential deadlock patterns with `RwLock`, template injection risk, and dead code with `#[allow(dead_code)]`.

## Critical Issues

### CR-01: `format!` macro will panic if prompt_template contains `{` or `}` characters

**File:** `src/visual/analyzer.rs:109-116`
**Issue:** The `analyze_video_frames` function uses `format!("{}...", prompt_template)` to embed the externally-provided `prompt_template` string into a larger prompt. The `format!` macro interprets `{` and `}` as format specifiers. If `prompt_template` contains literal braces (common in JSON examples, template instructions, or any structured text), the `format!` call will panic at runtime with `Error: formatting trait not implemented for ...` or `invalid format string`.

All built-in prompt templates use `${variable}` syntax (where `$` precedes the braces), so this is safe for current internal use. However, `analyze_video_frames` accepts an arbitrary `&str` from any caller. A template containing instructions like "output format: {key: value}" or any JSON example will cause a runtime panic that propagates up through the async runtime.

```rust
// Current code (analyzer.rs:109-116)
let rendered_prompt = format!(
    "{}\n\nIMPORTANT: Respond with valid JSON matching this schema: \
     {{frame_observations: [{{frame_number: u64, ...}}], ...}}",
    prompt_template  // <-- PANIC if prompt_template contains { or }
);
```

**Fix:** Use string concatenation instead of `format!` to avoid interpreting `prompt_template` as a format string:

```rust
let schema_suffix = "\n\nIMPORTANT: Respond with valid JSON matching this schema: \
     {frame_observations: [{frame_number: u64, timestamp: string, \
     scene_description: string, objects: [string], actions: [string], \
     on_screen_text: string|null, visual_salience: f64|null}], \
     overall_activity_summary: string}";
let rendered_prompt = prompt_template.to_string() + schema_suffix;
```

### CR-02: `$variable` regex false-matches dollar signs in template content, silently corrupting output

**File:** `src/prompt/template.rs:77`
**Issue:** The variable regex `r"\$\{(\w+)\}|\$(\w+)"` has a second branch `\$(\w+)` that matches bare `$variable` syntax. This regex is applied to the entire template string via `replace_all` in pass 2 (line 101). If a template contains literal dollar signs followed by word characters (e.g., FFmpeg filter expressions like `$fps`, shell variable references like `$HOME`, or JSON schema examples with `$schema`), pass 2 will silently replace them with either a matching variable value or an empty string.

The comment on line 107 `vars.get(name).copied().unwrap_or("")` means if the name does not exist in `vars`, it replaces with `""`, silently deleting content. If the name happens to match a provided variable, it substitutes the variable value -- corrupting the template.

For example, if a prompt template contained "Use $HOME environment variable" and a caller passed `vars = {"HOME": "/root"}`, the rendered output would be "Use /root environment variable" instead of the intended literal text.

While pass 1 checks for missing variables, the check on line 89 only reports names where `!vars.contains_key(name)`. If a `$` pattern matches a name that IS in vars, it gets silently substituted -- even if the `$` was meant as a literal character.

**Fix:** Remove the bare `$variable` syntax and require `${variable}` for all substitutions, or add an escape mechanism:

```rust
// Option A: Remove bare $variable, require ${} for all variables
let var_re = Regex::new(r"\$\{(\w+)\}")?;

// Option B: Add escape support (use $$ for literal $)
// Process escapes after variable substitution
```

## Warnings

### WR-01: `analyze_video_frames` does not validate `batch_size` or `max_concurrency` inputs

**File:** `src/visual/analyzer.rs:66-67`
**Issue:** The function accepts `batch_size: usize` and `max_concurrency: usize` but performs no validation. Passing `batch_size = 0` would create zero-size batches, likely causing the LLM provider to receive empty image lists per batch. Passing `max_concurrency = 0` could cause a panic in the LLM provider's concurrency semaphore. These are trivially triggerable by caller error.

**Fix:**
```rust
if batch_size == 0 {
    return Err(VisualError::Analysis("batch_size must be > 0".into()));
}
if max_concurrency == 0 {
    return Err(VisualError::Analysis("max_concurrency must be > 0".into()));
}
```

### WR-02: `BatchAnalysisResult.analyzed_batches` field is misleading -- counts total responses, not successful batches

**File:** `src/visual/analyzer.rs:187`
**Issue:** `analyzed_batches` is set to `raw_results.len()` (the total number of LLM responses), but the field name implies "analyzed" (i.e., successfully processed). A caller checking `analyzed_batches` would assume all those batches were successfully analyzed, but some may have failed and been recorded in the `errors` field. The field should either be renamed to `total_batches` or set to the count of successfully parsed batches.

**Fix:**
```rust
// Option A: Rename for clarity
analyzed_batches: raw_results.len(),  // rename field to total_batches

// Option B: Count only successful
analyzed_batches: raw_results.len() - errors.len(),
```

### WR-03: Template injection -- user-supplied values are not sanitized before insertion into LLM prompts

**File:** `src/prompt/template.rs:75-108` and `src/prompt/manager.rs:48-87`
**Issue:** The template renderer performs pure string substitution without any sanitization of variable values. When user-controlled data (e.g., `video_description`, `subtitle_content`) is passed as a variable value, it is interpolated verbatim into the prompt sent to the LLM. A malicious or unexpected subtitle content could contain prompt injection text designed to manipulate the LLM's output.

While the template system itself is not a security boundary (the LLM is the downstream consumer), the lack of any sanitization or escaping option means callers must be aware that all variable values are interpolated as-is. The `json` filter helps for JSON contexts but no filter exists for general text sanitization.

**Fix:** Document clearly that `template::render` performs no sanitization and callers are responsible for validating variable values. Consider adding a `sanitize` or `escape` filter for untrusted inputs.

### WR-04: Potential deadlock pattern with `RwLock` in `PromptManager` if reentrant usage occurs

**File:** `src/prompt/manager.rs:37-39` and `src/prompt/manager.rs:95-97`
**Issue:** `PromptManager` wraps `Arc<RwLock<PromptRegistry>>`. Methods like `get_prompt` acquire a read lock and return a `Prompt` clone (the lock is released after the method returns). `register_prompt` acquires a write lock. The current code does not hold locks across method boundaries, so no deadlock exists today. However, the API design makes it easy for a caller to accidentally call `register_prompt` while in a callback that was invoked during a `search_prompts` call, which could deadlock if the callback occurs on the same thread while a lock is held. Using `std::sync::RwLock` (not `tokio::sync::RwLock`) means holding a lock across an `.await` point would block the entire thread.

**Fix:** Document that `PromptManager` methods must not be called reentrantly from the same thread. Consider using `parking_lot::RwLock` which has better performance and explicitly documented non-reentrant semantics.

### WR-05: Unused `pub(crate)` function `parse_frame_number_from_name` with `#[allow(dead_code)]`

**File:** `src/visual/frame_extractor.rs:562-568`
**Issue:** The function `parse_frame_number_from_name` is marked `pub(crate)` and `#[allow(dead_code)]` but is never called from outside this module. It is only referenced in tests within the same module. The function duplicates the inline parsing logic in `rename_fast_path_frames` (lines 443-449).

**Fix:** Either remove the function and its tests if unused, or refactor `rename_fast_path_frames` to use this function instead of inline parsing, eliminating code duplication.

## Info

### IN-01: Test cleanup using `remove_dir_all` is not guaranteed on test failure

**File:** `src/visual/frame_extractor.rs:607`, `src/visual/analyzer.rs:407`, `src/visual/analyzer.rs:433`, `src/visual/analyzer.rs:458`
**Issue:** Multiple tests create temporary directories and clean them up with `let _ = std::fs::remove_dir_all(&temp_dir)` at the end. If the test panics (via assertion failure), the cleanup code never runs, leaving temp files behind. The project has `tempfile` in dev-dependencies but does not use it for these tests.

**Fix:** Use `tempfile::tempdir()` which automatically cleans up on drop (even on panic):

```rust
let temp_dir = tempfile::tempdir().expect("should create temp dir");
let temp_path = temp_dir.path();
// temp_dir auto-cleans when dropped
```

### IN-02: `template.rs` filter regex iterated three times over the same input

**File:** `src/prompt/template.rs:118`, `src/prompt/template.rs:132`, `src/prompt/template.rs:141`
**Issue:** The `filter_re` regex is iterated three separate times in `render()`: first to check missing variables (line 118), second to validate filter names (line 132), third to perform replacements (line 141). Each iteration re-executes the regex matching. For templates with many filter expressions, this is wasteful.

**Fix:** Merge into a single pass that validates and collects replacements simultaneously.

### IN-03: `seconds_to_hhmmssmmm` truncates milliseconds instead of rounding

**File:** `src/visual/frame_extractor.rs:475`
**Issue:** The millisecond calculation `((total_secs - total_secs.floor()) * 1000.0) as u64` uses truncation rather than rounding. For a fractional value like `0.9999999`, multiplied by 1000 gives `999.9999`, correctly truncated to `999`. But for `0.0005001`, multiplied by 1000 gives `0.5001`, truncated to `0` instead of the expected `1`. This is unlikely to cause visible issues at typical 3-second intervals but is a subtle correctness concern.

**Fix:** Use rounding: `((total_secs - total_secs.floor()) * 1000.0).round() as u64`

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
