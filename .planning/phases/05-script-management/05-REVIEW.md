---
phase: 05-script-management
reviewed: 2026-04-29T10:30:00Z
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
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 05: Code Review Report (Re-review)

**Reviewed:** 2026-04-29T10:30:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

This is a re-review after commit 2ce41d2 which fixed 9 prior issues (2 CR, 4 WR, 3 IN). All previously identified critical and warning issues have been confirmed resolved. The codebase is a Rust library crate for script management (JSON script load/save/validate/edit) with good test coverage (68 unit tests all passing).

Previous issue resolution verified:

- **CR-01 (timestamp format compatibility):** Fixed. `timestamp_to_millis` now accepts both `HH:MM:SS,mmm` and `HH:MM:SS` (optional milliseconds), matching Python pipeline output. Test 12 explicitly validates `00:00:00-00:00:26` passes.
- **CR-02 (_id float deserialization):** Fixed. Custom `deserialize_id` function in `types.rs:19-33` accepts both integer and float JSON numbers, converting floats via `f as i64`. Test 8 in `types.rs` verifies `1.0` deserializes to `1`.
- **WR-01 (timestamp range validation):** Fixed. `timestamp_to_millis` now checks `h > 23 || m > 59 || s > 59` at line 28. Tests verify `99:99:99,999` is rejected.
- **WR-02 (start/end ordering):** Fixed. `validate_timestamp` at line 60 checks `end_millis >= start_millis`. Tests verify `00:00:10-00:00:05` is rejected.
- **WR-03 (update_timestamp validation):** Fixed. `update_timestamp` in `edit.rs:37` now calls `validate_timestamp(ts)` before applying the edit, returning `ScriptError::InvalidTimestamp` on failure.
- **WR-04 (empty string validation in edit API):** Fixed. Both `update_narration` (edit.rs:10) and `update_picture` (edit.rs:49) now reject empty and whitespace-only strings.
- **IN-01 (InvalidTimestamp unused):** Fixed. Now used in `edit.rs:38`.
- **IN-02 (Python sample round-trip):** Not addressed, superseded by broader round-trip coverage in `tests/script_test.rs`.
- **IN-03 (repr(u8) extensibility):** Informational, no action needed.

Two new warnings and two info items were found. No critical issues remain.

## Critical Issues

None.

## Warnings

### WR-01: `save_script` does not use atomic file writes -- crash during write produces corrupted file

**File:** `src/script/mod.rs:141-144`
**Issue:** `save_script` writes JSON directly via `std::fs::write(path, json.as_bytes())`. If the process crashes (OOM, signal, power loss) mid-write, the destination file will contain partial or zero-length content. A subsequent `load_script` on the same path would fail with `JsonParse` error, losing previously valid data. For a script management module where user-edited content is at stake, atomic write (write to temp file + rename) is the standard defensive pattern.

**Fix:**
```rust
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    let json = serde_json::to_string_pretty(script).map_err(ScriptError::JsonParse)?;
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, json.as_bytes()).map_err(ScriptError::Io)?;
    std::fs::rename(&temp_path, path).map_err(ScriptError::Io)?;
    Ok(())
}
```

### WR-02: `validate_timestamp` uses naive `split('-')` which is fragile for format evolution

**File:** `src/script/mod.rs:48`
**Issue:** The timestamp range format `HH:MM:SS,mmm-HH:MM:SS,mmm` uses `-` as the range separator. The code splits on `-` via `ts.split('-')` and requires exactly 2 parts. This works correctly for the current format. However, the SRT/VTT subtitle format uses ` --> ` as separator. If the system ever needs to parse SRT-format timestamps or the format evolves to include spaces around the hyphen, the naive split would produce wrong results or silently reject valid data. The split-based approach also means corrupted data with extra hyphens (e.g., from a partially written file) would split into >2 parts and return a generic `false` rather than a clear error identifying the problem.

This is not a current correctness bug since the `HH:MM:SS,mmm-HH:MM:SS,mmm` format never contains internal hyphens, but the approach is fragile.

**Fix:** Consider splitting on a more specific delimiter or extracting start/end by position. For now, adding a comment documenting the assumption would suffice:
```rust
// NOTE: Split on '-' assumes the timestamp format never contains internal hyphens.
// The format "HH:MM:SS,mmm-HH:MM:SS,mmm" guarantees this because all components
// are digits, colons, and commas.
let parts: Vec<&str> = ts.split('-').collect();
```

## Info

### IN-01: `deserialize_id` silently truncates fractional _id values

**File:** `src/script/types.rs:25-26`
**Issue:** When `_id` is a float like `1.9`, the code does `f as i64` which truncates to `1` without warning. This is intentional for the `1.0` case (documented as "LLM 生成的 `1.0`"), but `1.9` would silently become `1` rather than being rejected. LLMs are unlikely to produce `1.9` as an _id, so this is low risk. If stricter validation is desired, check that the float value equals its truncated form.

**Fix:** Optional stricter check:
```rust
if (f - f.floor()).abs() > f64::EPSILON {
    return Err(de::Error::custom("_id 浮点值必须是整数，如 1.0"));
}
```

### IN-02: `Script` is a type alias (`Vec<ScriptClip>`) which limits API evolution

**File:** `src/script/types.rs:63`
**Issue:** `pub type Script = Vec<ScriptClip>` is a bare type alias. Any future metadata (schema version, generation timestamp, global settings) cannot be added to the Script type without changing the alias to a struct and updating all call sites. This is acceptable for the current scope but limits extensibility.

**Fix:** No immediate action needed. If the script format evolves to include top-level metadata, refactor to:
```rust
pub struct Script {
    pub clips: Vec<ScriptClip>,
    pub version: Option<String>,
}
```

---

_Reviewed: 2026-04-29T10:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
