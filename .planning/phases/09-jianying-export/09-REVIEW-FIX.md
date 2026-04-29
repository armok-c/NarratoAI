---
phase: 09-jianying-export
fixed_at: 2026-04-29T13:00:00Z
review_path: .planning/phases/09-jianying-export/09-REVIEW.md
iteration: 4
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 09: Code Review Fix Report (Iteration 4)

**Fixed at:** 2026-04-29T13:00:00Z
**Source review:** .planning/phases/09-jianying-export/09-REVIEW.md
**Iteration:** 4

**Summary:**
- Findings in scope: 4 (2 Critical + 2 Warning)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: JSON injection via unescaped draft_name in DRAFT_META_INFO_TEMPLATE

**Files modified:** `src/jianying/builder.rs`
**Commit:** (uncommitted -- applied as working tree change)
**Applied fix:** Replaced the naive `String::replace()` approach with `serde_json::Value` parsing. The `DRAFT_META_INFO_TEMPLATE` is now parsed as `serde_json::Value`, then `draft_id` and `draft_name` fields are set programmatically via `serde_json::json!()`. This eliminates JSON injection by ensuring all values are properly escaped through serde's serialization. The pattern matches the existing safe approach already used in `ScriptFile::save` for `draft_content.json`.

### CR-02: Path traversal via unsanitized draft_name

**Files modified:** `src/jianying/builder.rs`
**Commit:** (uncommitted -- applied as working tree change)
**Applied fix:** Added `validate_draft_name()` function that rejects draft names containing `/`, `\`, `..`, or `\0` characters. Called at the top of `DraftFolder::create_draft` before the name is used in `draft_path.join(draft_name)`. Added test `test_validate_draft_name_rejects_traversal` covering path traversal, forward/backslash, null byte, double-dot, and valid name acceptance.

### WR-01: OST=NarrationOnly silently skips audio when clip.audio is None

**Files modified:** `src/jianying/builder.rs`
**Commit:** (uncommitted -- applied as working tree change)
**Applied fix:** Replaced `if let Some(ref audio_path) = clip.audio` with `match clip.audio` that returns `JianYingError::MissingField { field: "audio", clip_index: i }` in the `None` arm. This ensures that when `OstType::NarrationOnly` or `OstType::Mixed` is declared but no audio is provided, the error is surfaced immediately instead of silently producing a semantically incorrect draft.

### WR-02: trange() accepts negative start values

**Files modified:** `src/jianying/time.rs`
**Commit:** (uncommitted -- applied as working tree change)
**Applied fix:** Added `start_us < 0` to the existing validation condition in `trange()`, changing `if duration_us < 0` to `if start_us < 0 || duration_us < 0`. This brings `trange()` into parity with `trange_from_secs()` which already validates both parameters. Added test `test_trange_negative_start_returns_none` confirming that `trange("-2s", "5s")` returns `None`.

## Verification Results

- **cargo check**: All fixes compile cleanly.
- **cargo test --lib**: 138 unit tests passed, 0 failed (2 new tests added).
- New tests confirmed passing:
  - `test_validate_draft_name_rejects_traversal` (CR-02)
  - `test_trange_negative_start_returns_none` (WR-02)

---

_Fixed: 2026-04-29T13:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 4_
