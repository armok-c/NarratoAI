---
phase: 05-script-management
reviewed: 2026-04-29T14:30:00Z
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
  critical: 1
  warning: 2
  info: 2
  total: 5
status: issues_found
---

# Phase 05: Code Review Report (Re-review #3)

**Reviewed:** 2026-04-29T14:30:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

This is the third review cycle for the script management module. All 7 source files were reviewed at standard depth. All previously identified issues from rounds 1 and 2 remain resolved. One new critical issue was discovered through deeper analysis of the `deserialize_id` function's handling of extreme integer values.

The codebase continues to be well-structured with consistent error handling and comprehensive test coverage (42 unit tests + 5 integration tests, all passing). The new finding relates to data integrity rather than a runtime crash.

## Critical Issues

### CR-01: `deserialize_id` silently saturates out-of-range integer IDs to `i64::MAX`

**File:** `src/script/types.rs:25-26`
**Issue:** When `_id` is a JSON number too large for `i64` but representable as `f64` (e.g., `99999999999999999999`), the code path `f as i64` saturates to `i64::MAX` (9223372036854775807) without any error. This is silent data corruption: the caller receives a valid-looking `i64` value that does not match the input.

The chain works as follows:
1. `serde_json` parses the large integer as `f64` (too big for `i64` or `u64`)
2. `n.as_i64()` returns `None` (overflow)
3. `n.as_f64()` returns `Some(1e20)`
4. `1e20_f64 as i64` saturates to `i64::MAX` per Rust's float-to-int cast semantics
5. The value passes `clip._id <= 0` validation (it's positive)
6. The script is loaded with a corrupt `_id`

While extremely large integer IDs are unlikely from LLM output, the failure mode is silent data corruption with no error signal to the caller, which violates the project's error handling principle ("never silently swallow errors").

**Fix:** Check whether the float-to-int conversion lost precision before accepting the result:
```rust
if let Some(f) = n.as_f64() {
    let i = f as i64;
    // Reject if truncation or saturation occurred
    if i as f64 != f || i == i64::MAX || i == i64::MIN {
        return Err(de::Error::custom("_id 数值超出有效范围"));
    }
    Ok(i)
}
```

Alternatively, a simpler guard:
```rust
if let Some(f) = n.as_f64() {
    if f > i64::MAX as f64 || f < i64::MIN as f64 || f != f.floor() {
        return Err(de::Error::custom("_id 必须是有效整数，当前值超出范围"));
    }
    Ok(f as i64)
}
```

## Warnings

### WR-01: `deserialize_id` silently truncates fractional `_id` values like `1.9` to `1`

**File:** `src/script/types.rs:25-26`
**Issue:** When `_id` is a fractional float like `1.9`, `f as i64` truncates to `1` without warning. This is distinct from the documented `1.0` case. If two clips had `_id` values `1` and `1.9`, they would both deserialize to `_id = 1`, causing a silent duplicate ID collision. The prior review accepted this as IN-01, but adversarial analysis elevates it: truncation is data corruption, and `1.9` is a plausible LLM typo (more so than 20-digit integers).

**Fix:** Validate that the float has no fractional part before truncating:
```rust
if let Some(f) = n.as_f64() {
    if (f - f.floor()).abs() > f64::EPSILON {
        return Err(de::Error::custom("_id 浮点值必须是整数（如 1.0），不接受小数"));
    }
    Ok(f as i64)
}
```

### WR-02: `save_script` leaks temp file if `rename` fails

**File:** `src/script/mod.rs:146-148`
**Issue:** The atomic write pattern writes to a `.json.tmp` temp file then renames it. If the `write` succeeds but `rename` fails (e.g., target file locked by another process on Windows, permissions error), the temp file remains on disk permanently. No cleanup is attempted in the error path. Over time, repeated failures would accumulate orphaned `.json.tmp` files.

**Fix:** Add cleanup on rename failure:
```rust
let temp_path = path.with_extension("json.tmp");
std::fs::write(&temp_path, json.as_bytes()).map_err(ScriptError::Io)?;
if let Err(e) = std::fs::rename(&temp_path, path) {
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
    return Err(ScriptError::Io(e));
}
```

## Info

### IN-01: `Script` is a type alias (`Vec<ScriptClip>`) which limits API evolution

**File:** `src/script/types.rs:63`
**Issue:** `pub type Script = Vec<ScriptClip>` is a bare type alias. Any future metadata (schema version, generation timestamp, global settings) cannot be added without changing the alias to a struct and updating all call sites. This is acceptable for the current scope. Carryover from prior review.

**Fix:** No immediate action. If the script format evolves to include top-level metadata, refactor to a struct wrapper.

### IN-02: `save_script` does not validate the script before writing

**File:** `src/script/mod.rs:144-149`
**Issue:** `load_script` calls `validate()` after deserialization, but `save_script` does not validate before writing. This means a program could construct an invalid `Script` (e.g., with empty `picture` fields) via the edit API and save it to disk. The file would only fail validation when loaded again. This is an inconsistency in the "validate at boundaries" principle, but since `save_script` is a low-level function, the caller is responsible for ensuring data integrity.

**Fix:** Optional -- add `validate(script)?` at the start of `save_script` if defense-in-depth is desired:
```rust
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    validate(script)?;
    // ... rest of function
}
```

---
_Reviewed: 2026-04-29T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
