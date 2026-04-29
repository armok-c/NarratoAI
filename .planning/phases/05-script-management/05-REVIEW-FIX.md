---
phase: 05-script-management
fix_date: 2026-04-29
fix_scope: all
findings_in_scope: 5
fixed: 4
skipped: 1
iteration: 1
status: all_fixed
---

# Phase 05: Code Review Fix Report (Re-review #3)

**Fixed at:** 2026-04-29
**Source review:** `.planning/phases/05-script-management/05-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 4 (CR-01, WR-01, WR-02, IN-02)
- Skipped: 1 (IN-01 -- deferred, no code change needed)

## Fixed Issues

### CR-01 + WR-01: `deserialize_id` silently saturates out-of-range integer IDs / silently truncates fractional `_id` values

**Files modified:** `src/script/types.rs`
**Commit:** `7bb9e6c`
**Applied fix:** Added precision-loss and saturation checks to the `as_f64` branch of `deserialize_id`. The float-to-int conversion now validates that:
1. The converted integer round-trips back to the same float value (`i as f64 != f` catches fractional truncation like `1.9 -> 1`)
2. The result is not `i64::MAX` or `i64::MIN` (catches saturation from oversized values like `99999999999999999999`)

Before:
```rust
} else if let Some(f) = n.as_f64() {
    Ok(f as i64)
}
```

After:
```rust
} else if let Some(f) = n.as_f64() {
    let i = f as i64;
    if i as f64 != f || i == i64::MAX || i == i64::MIN {
        return Err(de::Error::custom("_id 数值超出有效范围"));
    }
    Ok(i)
}
```

New tests added:
- `test_deserialize_id_oversized_integer` -- verifies that `_id: 99999999999999999999` is rejected
- `test_deserialize_id_fractional_fails` -- verifies that `_id: 1.9` is rejected

### WR-02: `save_script` leaks temp file if `rename` fails

**Files modified:** `src/script/mod.rs`
**Commit:** `c4add1b`
**Applied fix:** Wrapped the `std::fs::rename` call in an `if let Err(e)` block that performs best-effort cleanup of the temp file before returning the error.

Before:
```rust
std::fs::rename(&temp_path, path).map_err(ScriptError::Io)?;
```

After:
```rust
if let Err(e) = std::fs::rename(&temp_path, path) {
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
    return Err(ScriptError::Io(e));
}
```

### IN-02: `save_script` does not validate before writing

**Files modified:** `src/script/mod.rs`
**Commit:** `c4add1b` (same commit as WR-02)
**Applied fix:** Added `validate(script)?;` at the start of `save_script`, ensuring invalid scripts are rejected before any file I/O occurs.

New test added:
- `test_save_script_validates` -- verifies that saving a script with blank `picture` returns a `Validation` error and no file is written

## Skipped Issues

### IN-01: `Script` is a type alias (`Vec<ScriptClip>`) limiting API evolution

**File:** `src/script/types.rs:63`
**Reason:** Review explicitly states "No immediate action." This is a structural concern for future evolution, not a bug. If the script format evolves to include top-level metadata (schema version, generation timestamp), the type alias can be refactored to a struct wrapper at that time.
**Original issue:** `pub type Script = Vec<ScriptClip>` is a bare type alias. Any future metadata cannot be added without changing the alias to a struct and updating all call sites.

---

## Verification

- `cargo check`: passes (no warnings)
- `cargo test --lib`: 71 unit tests pass (11 in types, 13 in mod, 47 in other modules)
- `cargo test --test script_test`: 5 integration tests pass
- `test_clip_video_async` failure is pre-existing (fails on HEAD before any changes)

_Fixed: 2026-04-29_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
