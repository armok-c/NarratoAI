---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T09:24:48Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-05-07T09:24:48Z
**Source review:** `.planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Template filter pass 3 re-interprets variable values as filter expressions

**Files modified:** `narratoai-core/src/prompt/template.rs`
**Commit:** `5a62fca`
**Applied fix:** Replaced three-pass rendering (variables substitution -> filter reinterpretation on substituted output) with a single-pass pipeline. Uses one regex `r"\$\{(\w+)(?:\|(\w+))?\}"` that matches both `${variable}` and `${variable|filter}` directly from the source template. This prevents substituted variable values from being re-parsed as filter expressions, eliminating the template injection vector.

### WR-02: Unparseable keyframe filenames silently sort to front, corrupting LLC input order

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** `b03342d`
**Applied fix:** Changed `extract_frame_number_from_keyframe()` to return `Result<u64, VisualError>` instead of defaulting to `0` on parse failure. Added pre-sort validation in `collect_frame_paths()` that returns an error if any filename cannot be parsed, with a warning log. The sort uses `.unwrap()` after validation (safe, since all paths have been pre-validated).

### WR-03: `list_prompts` silently drops prompts with no default version

**Files modified:** `narratoai-core/src/prompt/registry.rs`
**Commit:** `066779d`
**Applied fix:** Added fallback in `list_prompts()`: when a prompt name has no default version in `default_versions`, the method now falls back to the first available version (sorted by version string) and emits a `tracing::warn!` log. This prevents silent omission of prompts from list results.

### WR-04: Parameter validation in `render_prompt` has unreachable gap with template content

**Files modified:** `narratoai-core/src/prompt/registry.rs`, `narratoai-core/src/prompt/manager.rs`
**Commit:** `f89fb2f`
**Applied fix:** Added `validate_prompt_parameters()` method to `PromptRegistry` that cross-checks all `${variable}` references in the template content against declared `ParameterDef` entries. Called at the start of `register()`, ensuring undeclared template variables produce a `Validation` error at registration time rather than a confusing `TemplateRender` error at rendering time. Updated 4 test cases in `registry.rs` and `manager.rs` that created prompts with template variables but no corresponding `ParameterDef` declarations.

---

_Fixed: 2026-05-07T09:24:48Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
