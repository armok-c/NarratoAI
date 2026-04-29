---
phase: 05-script-management
reviewed: 2026-04-29T15:35:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/script/edit.rs
  - src/script/error.rs
  - src/script/mod.rs
  - src/script/types.rs
  - tests/script_test.rs
findings:
  critical: 0
  warning: 0
  info: 1
  total: 1
status: issues_found
---

# Phase 05: Code Review Report (Re-review #4)

**Reviewed:** 2026-04-29T15:35:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

This is the fourth review cycle for the script management module. All 7 source files were reviewed at standard depth, with particular focus on verifying the fixes committed since review #3.

**Fix verification results:**

| Finding | Fix Commit | Status |
|---------|-----------|--------|
| CR-01: `deserialize_id` saturates out-of-range integers | `7bb9e6c` | **Verified correct** |
| WR-01: `deserialize_id` truncates fractional values | `7bb9e6c` | **Verified correct** |
| WR-02: `save_script` leaks temp file on rename failure | `c4add1b` | **Verified correct** |
| IN-02: `save_script` does not validate before save | `c4add1b` | **Verified correct** |

All 45 script-related tests pass. No new issues introduced by the fixes. The `deserialize_id` function now correctly rejects both out-of-range integers and fractional floats through a dual-check strategy (`i as f64 != f` for truncation detection plus `i == i64::MAX || i == i64::MIN` for saturation detection). The `save_script` function now validates before writing and cleans up the temp file on rename failure.

One carryover info item remains. No new critical or warning-level findings were discovered.

## Fix Verification Details

### CR-01 / WR-01 Fix: `deserialize_id` rejection of invalid values (`src/script/types.rs:19-38`)

The fix adds a combined roundtrip and boundary check:

```rust
let i = f as i64;
if i as f64 != f || i == i64::MAX || i == i64::MIN {
    return Err(de::Error::custom("_id 数值超出有效范围"));
}
```

Verified edge cases:
- `1.0` -> passes roundtrip (`1 as f64 == 1.0`), accepted. Correct.
- `1.9` -> fails roundtrip (`1 as f64 != 1.9`), rejected. Correct.
- `0.5` -> fails roundtrip (`0 as f64 != 0.5`), rejected. Correct.
- `-0.5` -> fails roundtrip (`0 as f64 != -0.5`), rejected. Correct.
- `-1.0` -> `n.as_i64()` returns `Some(-1)`, first branch handles it. `validate()` later rejects `_id <= 0`. Correct layered behavior.
- Oversized integer (e.g. `99999999999999999999`) -> `n.as_i64()` returns `None`, `f as i64` saturates to `i64::MAX`, boundary check rejects. Correct.
- Oversized float (e.g. `9223372036854775808.0`) -> `f as i64` saturates to `i64::MAX`, `i as f64 == f` may pass for this specific value (due to f64 rounding), but `i == i64::MAX` catches it. Correct.
- Negative overflow (e.g. `-9223372036854775809.0`) -> saturates to `i64::MIN`, caught by `i == i64::MIN`. Correct.

The saturation behavior of `f64 as i64` is consistent between debug and release builds in Rust (confirmed by test), so the guard is reliable.

### WR-02 Fix: Temp file cleanup on rename failure (`src/script/mod.rs:149-151`)

```rust
if let Err(e) = std::fs::rename(&temp_path, path) {
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
    return Err(ScriptError::Io(e));
}
```

The cleanup is best-effort (`let _ =`), which is appropriate -- if cleanup also fails, there is nothing further to do, and the original rename error is the more important one to propagate.

### IN-02 Fix: Validate before save (`src/script/mod.rs:145`)

```rust
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    validate(script)?;
    ...
```

This ensures invalid data is rejected before writing to disk. The validation happens before the temp file is created, so no cleanup is needed if validation fails.

## Info

### IN-01: `Script` is a type alias (`Vec<ScriptClip>`) which limits API evolution

**File:** `src/script/types.rs:68`
**Issue:** `pub type Script = Vec<ScriptClip>` is a bare type alias. Any future metadata (schema version, generation timestamp, global settings) cannot be added without changing the alias to a struct and updating all call sites. Carryover from prior reviews. Not a correctness issue.
**Fix:** No immediate action. Refactor to a struct wrapper if the script format evolves to include top-level metadata.

---
_Reviewed: 2026-04-29T15:35:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
