---
phase: 07-sde-pipeline
reviewed: 2026-05-09T21:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - narratoai-core/src/sde/error.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sde/timestamp.rs
  - narratoai-core/src/sde/mod.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/script_gen.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md
findings:
  critical: 1
  warning: 5
  info: 3
  total: 9
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-05-09
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed 9 source files across the SDE pipeline module (`narratoai-core/src/sde/`) and prompt registration. The code is well-structured with comprehensive tests and proper error propagation. However, one critical bug was found: `fix_trailing_commas` performs a global search-and-replace that corrupts legitimate comma content inside JSON string values. Several warnings address missing validation gaps and a subtle JSON repair logic flaw.

## Critical Issues

### CR-01: `fix_trailing_commas` corrupts commas inside JSON string values

**File:** `narratoai-core/src/sde/script_gen.rs:237-239`
**Issue:** The `fix_trailing_commas` function performs a blanket `text.replace(",}", "}").replace(",]", "]")` on the entire JSON text. This will incorrectly modify commas that appear inside string values. For example, a narration field containing `"Hello,}"` or `"Items A,B,C,}"` would have its content corrupted. Since this function is part of the JSON repair pipeline used to fix LLM output, it will encounter narration strings with arbitrary text, making this a real-world risk. The same issue applies to `",]"` inside strings.

Consider this LLM-generated JSON:
```json
{"items": [{"narration": "然后他说,}", "timestamp": "..."}]}
```
After `fix_trailing_commas`, the narration becomes `"然后他说}"` -- silently corrupting user data.

**Fix:**
```rust
/// Fix trailing commas using regex that only matches commas followed by
/// closing braces/brackets that are NOT inside a string context.
fn fix_trailing_commas(text: &str) -> String {
    // Simple approach: walk the string respecting string boundaries
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escape_next = false;

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if escape_next {
            escape_next = false;
            result.push(c);
            i += 1;
            continue;
        }

        if c == '\\' && in_string {
            escape_next = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }

        if !in_string && c == ',' {
            // Look ahead past whitespace for } or ]
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' || j < chars.len() && chars[j] == '\n' || j < chars.len() && chars[j] == '\r' || j < chars.len() && chars[j] == '\t' {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // Skip the comma -- it's a trailing comma
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}
```

## Warnings

### WR-01: `drama_name` is not validated but used in LLM prompt

**File:** `narratoai-core/src/sde/types.rs:60-106`
**Issue:** `SdeRequest::validate()` checks `subtitle_path`, `video_path`, numeric ranges, and color format, but never validates `drama_name`. The `drama_name` field defaults to an empty string and is passed directly into `step_generate_script` (pipeline.rs:112) where it becomes a template variable in the LLM prompt (script_generation_v2.0.md line 1 and 267: `${drama_name}`). An empty `drama_name` will produce a prompt containing `短剧《》` which could confuse the LLM or produce degraded output. Additionally, a `drama_name` containing template injection characters (like `${`, `#{`, or `</plot>`) could interfere with prompt template rendering.

**Fix:** Add validation in `SdeRequest::validate()`:
```rust
if self.drama_name.trim().is_empty() {
    return Err("drama_name 不能为空".to_string());
}
```

### WR-02: `subtitle_position` is not validated

**File:** `narratoai-core/src/sde/types.rs:60-106`
**Issue:** `subtitle_position` is a `String` field that defaults to `"bottom"` and is used in `pipeline.rs:437-441` to determine ASS subtitle alignment. The `match` only handles `"top"`, `"center"`, and falls through to `"2"` (bottom) for any other value. However, `validate()` never checks that the value is one of the three valid options. An arbitrary string like `"middle"` would silently default to bottom alignment without warning. This is a missing input validation at a system boundary.

**Fix:** Add validation:
```rust
if !matches!(self.subtitle_position.as_str(), "top" | "center" | "bottom") {
    return Err(format!(
        "subtitle_position 必须为 top/center/bottom 之一: {}",
        self.subtitle_position
    ));
}
```

### WR-03: `extract_first_json_object` mishandles nested mismatched bracket types

**File:** `narratoai-core/src/sde/script_gen.rs:242-290`
**Issue:** The function only tracks depth for the opening bracket type (`{` or `[`). If the text starts with `{` but contains `[` and `]` inside, the function correctly ignores them because it only increments depth on `opening` and decrements on `closing`. However, the function ignores the opposite bracket type entirely -- it does not track depth for `{` inside `[...]` or `[` inside `{...}`. This means a malformed string like `{a: [1, 2} ]` would not be handled correctly: the `}` at position of `2}` would decrement depth to 0 and return a truncated result, but this is actually correct behavior for malformed JSON (it would fail serde validation anyway). The real concern is that the function can produce a false positive match for `{[}` -- it would match `{` at depth 1, see `[` (ignored), then `}` at depth 0, returning `{[}`. While this would fail JSON parsing, it wastes repair attempts on garbage. Low severity but worth noting the limitation.

**Fix:** Document the limitation in a code comment. No code change needed as malformed extractions will fail serde validation.

### WR-04: `find_precise_range` tolerance can bridge non-contiguous segments

**File:** `narratoai-core/src/subtitle/timestamp.rs:99-111` (referenced via `narratoai-core/src/sde/timestamp.rs`)
**Issue:** The function uses a +-1 second tolerance when matching segments. With the test data showing 2-second gaps between segments, this is safe. But if subtitle segments have gaps of less than 2 seconds (common in fast-paced dialogue), the +-1 second tolerance on both sides can bridge non-contiguous segments. The function returns `matched.first().start_secs` to `matched.last().end_secs`, which would create an artificially wide time range spanning the gap. This affects OST=1 (OriginalSound) clips where `find_precise_range` is used to correct timestamps before video clipping, potentially including silence gaps in the output.

**Fix:** Consider tightening the tolerance or checking that matched segments are contiguous (each segment's `start_secs` is reasonably close to the previous segment's `end_secs`).

### WR-05: `run_sde` function is 590 lines -- well beyond maintainability threshold

**File:** `narratoai-core/src/sde/pipeline.rs:34-622`
**Issue:** The `run_sde` function spans approximately 590 lines (lines 34-622). While it is a sequential pipeline orchestrator with clear step comments, its length makes it difficult to review, test in isolation, and maintain. Each step (ParseSubtitle, PlotAnalysis, etc.) mixes state mutation, I/O, and error handling in the same function body. The project's own coding guidelines recommend functions under 50 lines and files under 800 lines.

**Fix:** Extract each pipeline step into its own private function (similar to how `step_analyze_plot` and `step_generate_script` are already extracted in `script_gen.rs`). The main `run_sde` function should only orchestrate the sequence:
```rust
pub async fn run_sde(...) -> Result<PathBuf, SdeError> {
    // validate + init state
    let mut state = init_state(request, config, registry, progress).await?;
    state = step_parse_subtitle(state).await?;
    state = step_plot_analysis(state, ...).await?;
    state = step_generate_script(state, ...).await?;
    state = step_load_script(state).await?;
    state = step_tts(state, ...).await?;
    state = step_clip(state, ...).await?;
    state = step_merge_audio(state, ...).await?;
    state = step_concat(state).await?;
    state = step_composite(state, ...).await?;
    state.output_video_path.ok_or_else(...)
}
```

## Info

### IN-01: `has_timecodes` has an overly broad matching pattern

**File:** `narratoai-core/src/sde/script_gen.rs:14-16`
**Issue:** The `has_timecodes` function is `#[cfg(test)]` only, so this has no production impact. However, `text.contains(':')` matches any text containing a colon (e.g., "The time is: now" or any URL). This means virtually any non-trivial text will return `true`, making the function a no-op in practice. If this is ever used in production, it should use a proper regex like `\d{2}:\d{2}:\d{2}[,.]\d{3}`.

**Fix:** Replace with a proper timestamp pattern if promoted to production code. Currently test-only, no action required.

### IN-02: `register.rs` test `test_register_all_prompts_full` counts prompts by name

**File:** `narratoai-core/src/prompt/register.rs:167-169`
**Issue:** The assertion `registry.list_prompts("short_drama_narration").len() == 2` counts unique prompt names, not total registrations. Since `script_generation` has two versions (v1.0 and v2.0) registered under the same name, `list_prompts` returns 2 entries (`plot_analysis` + `script_generation`), which happens to match the assertion. The subsequent `registry.get(...)` calls on lines 174-176 verify all 3 versions individually. The test is correct but the assertion on line 169 is coincidental -- if a third distinct name were added, the count would need updating without any indication.

**Fix:** Consider adding a comment explaining that `list_prompts` returns distinct names, not versions.

### IN-03: Prompt template uses `${variable}` syntax that could conflict with shell-like injection

**File:** `narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md:66`
**Issue:** The template uses `${subtitle_content}` as a placeholder. While the prompt rendering system presumably does simple string substitution, if the `subtitle_content` value itself contains `${...}` patterns, it could cause unexpected template expansion if the rendering engine performs multiple passes. The `script_generation_v2.0.md` template also uses `${drama_name}`, `${plot_analysis}`, and `${subtitle_content}`. This is an informational note -- the actual risk depends on the template engine implementation.

**Fix:** Verify that the prompt template rendering engine (`PromptManager::render_prompt`) performs single-pass substitution only.

---

_Reviewed: 2026-05-09T21:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
