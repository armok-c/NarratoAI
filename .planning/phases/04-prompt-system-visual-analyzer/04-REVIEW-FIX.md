---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-04-30T15:50:25Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 5
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 4: Code Review Fix Report (Re-Review #4 Fixes)

**Fixed at:** 2026-04-30T15:50:25Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 5

**Summary:**
- Findings in scope: 3 (1 critical, 2 warnings)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: Optional parameter default values are never applied during template rendering

**Files modified:** `src/prompt/manager.rs`
**Commit:** 16eb304
**Applied fix:** Added default-value merging logic in `PromptManager::render_prompt()` before calling `template::render()`. The method now iterates over `prompt.metadata.parameters`, collects any `ParameterDef.default` values into a `HashMap<String, String>`, then overlays caller-provided `vars` on top (caller takes precedence). The merged map is converted to `HashMap<&str, &str>` and passed to `template::render()`. This ensures optional parameters like `language` (default: `"zh-CN"`) and `style` (default: `"正式"` or `"幽默风趣"`) are automatically filled in when callers omit them. Also removed unused `ParameterDef` import from the test module.

### WR-01: Misleading error messages from TemplateRender variant reuse

**Files modified:** `src/prompt/error.rs`
**Commit:** 16eb304
**Applied fix:** Changed the `TemplateRender` variant's format attribute from `#[error("模板渲染失败: 缺少必需参数 {0}")]` to `#[error("模板渲染失败: {0}")]`. This variant is used as a catch-all for three distinct error categories (missing variables, regex compilation failures, and RwLock poisoning), so the hardcoded "缺少必需参数" prefix was misleading for non-template errors. Updated the `test_template_render_error_message_chinese` test assertion to check for `"模板渲染失败"` instead of `"缺少必需参数"`.

### WR-02: Unknown filter names silently preserved instead of raising errors

**Files modified:** `src/prompt/template.rs`
**Commit:** 16eb304
**Applied fix:** Added a pre-pass validation loop using `filter_re.captures_iter(&result)` that checks every filter name against the `filters` map. If a non-empty filter name is not found in the map, the function returns `PromptError::TemplateRender(format!("未知过滤器: {}", filter_name))` immediately. The existing `replace_all` pass remains unchanged as a safety fallback, but the validation ensures typos like `${name|uppper}` are caught before the template output is sent to the LLM.

## Verification

- `cargo check --lib`: compiles with zero errors and zero warnings
- `cargo test --lib`: all 273 tests pass (1 ignored -- edge_tts integration test)

```
running 274 tests
test result: ok. 273 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.96s
```

## Skipped Issues

None -- the 2 info-level findings (IN-01: dead Version variant; IN-02: total_frames count source) are out of scope per default Critical + Warning only policy.

---
_Fixed: 2026-04-30T15:50:25Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 5_
