---
phase: 05-script-management
reviewed: 2026-04-29T10:15:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/script/types.rs
  - src/script/error.rs
  - src/script/mod.rs
  - src/script/edit.rs
  - tests/script_test.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-04-29T10:15:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Reviewed the Phase 05 script-management Rust crate covering data models (ScriptClip, OstType), error types, load/save/validate functions, and four immutable edit operations. The implementation is well-structured with good test coverage overall. However, two critical compatibility issues were found that will cause runtime failures when interoperating with the existing Python pipeline, along with several warnings around timestamp validation gaps and missing test coverage.

## Critical Issues

### CR-01: `editedTimeRange` timestamp format mismatch with Python pipeline

**File:** `src/script/types.rs:31-34`
**Issue:** The Rust `ScriptClip` model declares `source_time_range` and `edited_time_range` as `Option<String>` without any format documentation or validation constraints. Meanwhile, the Python pipeline's `update_script.py` (line 190) generates `editedTimeRange` values in the format `HH:MM:SS-HH:MM:SS` (no milliseconds), e.g., `"00:00:00-00:00:26"`. The Rust `validate_timestamp` function in `mod.rs` only accepts `HH:MM:SS,mmm-HH:MM:SS,mmm` with mandatory 3-digit millisecond fields. If any downstream Rust code (Phase 06+) calls `validate_timestamp` on a `sourceTimeRange` or `editedTimeRange` field loaded from Python-generated JSON, validation will reject valid data.

The Python `calculate_duration()` function (`app/services/update_script.py:48-77`) explicitly handles both `HH:MM:SS,mmm` and `HH:MM:SS` formats, proving the Python pipeline produces both. The test data in `subtitle_merger.py:198-200` shows `sourceTimeRange` values like `"00:00:00-00:00:26"` (no milliseconds).

While `validate_timestamp` currently only validates the `timestamp` field in `validate()`, this is a time bomb: any future validation of pipeline fields, or reuse of `validate_timestamp` for parsing `sourceTimeRange`/`editedTimeRange`, will break against real Python-produced data.

**Fix:** Either (a) add a separate `validate_pipeline_timestamp` that accepts both formats, or (b) make `validate_timestamp` accept both formats by making the millisecond part optional:

```rust
fn validate_timestamp(ts: &str) -> bool {
    let parts: Vec<&str> = ts.split('-').collect();
    if parts.len() != 2 { return false; }
    for part in &parts {
        let sub_parts: Vec<&str> = part.split(',').collect();
        // millisecond part is optional
        let (time_str, millis_str) = match sub_parts.len() {
            1 => (sub_parts[0], None),
            2 => (sub_parts[0], Some(sub_parts[1])),
            _ => return false,
        };
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 { return false; }
        for tp in &time_parts {
            if tp.len() != 2 || !tp.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
        if let Some(millis) = millis_str {
            if millis.len() != 3 || !millis.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
    }
    true
}
```

### CR-02: `_id` typed as `i64` will reject valid JSON from LLM-generated scripts with float `_id`

**File:** `src/script/types.rs:23`
**Issue:** The `ScriptClip._id` field is declared as `pub _id: i64`. The Python codebase (`app/utils/check_script.py:53`) validates `_id` with `isinstance(clip['_id'], int)`, which in Python 3 accepts `int` but rejects `float`. However, LLM-generated JSON commonly produces `_id` as a float (e.g., `1.0` instead of `1`). The Python JSON parser treats `1` as `int` but `1.0` as `float` -- and `isinstance(1.0, int)` returns `False` in Python 3.

More critically, in Rust, `serde_json` will fail to deserialize `"1.0"` or `1.0` (a JSON number without fractional part but typed as float) into `i64`. When an LLM generates `"_id": 1.0` (which is valid JSON), `serde_json::from_str` will error with a type mismatch. This will cause `load_script` to return `ScriptError::JsonParse`, rejecting otherwise valid scripts that the Python pipeline handles (Python's `json.loads` parses `1.0` as `float`, and downstream code treats it loosely).

Since this is an LLM-facing format, `_id` should tolerate both `1` and `1.0`. The Python side already has fragile validation (`isinstance(..., int)` rejects floats); the Rust side should be more robust, not less.

**Fix:** Use a custom deserializer or accept `f64` with validation, or use `serde_json::Value` with coercion:

```rust
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

fn deserialize_id<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    // Accept both integer and float (e.g., 1 and 1.0) for _id
    let val = serde_json::Value::deserialize(d)?;
    match val {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i)
            } else if let Some(f) = n.as_f64() {
                Ok(f as i64)
            } else {
                Err(de::Error::custom("_id must be a number"))
            }
        }
        _ => Err(de::Error::custom("_id must be a number")),
    }
}

// In ScriptClip:
#[serde(deserialize_with = "deserialize_id")]
pub _id: i64,
```

## Warnings

### WR-01: `validate_timestamp` accepts structurally valid but semantically invalid timestamps

**File:** `src/script/mod.rs:13-40`
**Issue:** The timestamp validator only checks structural format (2-digit HH:MM:SS, 3-digit mmm) but does not validate ranges. Values like `99:99:99,999-99:99:99,999` pass validation despite being impossible time values. The edit test at `edit.rs:187` explicitly uses `"99:99:99,999-99:99:99,999"` as a test value and it passes through `update_timestamp` without error.

While the Python `check_script.py` uses the same regex `^\d{2}:\d{2}:\d{2},\d{3}-\d{2}:\d{2}:\d{2},\d{3}$` and also does not validate ranges, adding range validation would be a genuine improvement over the Python version and prevent downstream parsing errors in Phase 06+.

**Fix:** Add range checks for hours (0-23), minutes (0-59), seconds (0-59):

```rust
// After parsing time_parts:
let h: u32 = time_parts[0].parse().unwrap_or(0);
let m: u32 = time_parts[1].parse().unwrap_or(0);
let s: u32 = time_parts[2].parse().unwrap_or(0);
if h > 23 || m > 59 || s > 59 {
    return false;
}
```

### WR-02: `validate_timestamp` does not check that start time precedes end time

**File:** `src/script/mod.rs:13-40`
**Issue:** The validator splits on `-` and checks both parts independently but never verifies that the start timestamp is chronologically before (or equal to) the end timestamp. A timestamp like `"00:00:10,000-00:00:05,000"` (end before start) passes validation. This would cause incorrect clip durations and video editing errors downstream.

**Fix:** Parse both timestamps into comparable values and verify ordering:

```rust
fn timestamp_to_millis(ts: &str) -> Option<u64> {
    let parts: Vec<&str> = ts.split(',').collect();
    let time_parts: Vec<&str> = parts[0].split(':').collect();
    if time_parts.len() != 3 { return None; }
    let h: u64 = time_parts[0].parse().ok()?;
    let m: u64 = time_parts[1].parse().ok()?;
    let s: u64 = time_parts[2].parse().ok()?;
    let ms: u64 = if parts.len() > 1 { parts[1].parse().ok()? } else { 0 };
    Some((h * 3600 + m * 60 + s) * 1000 + ms)
}

// In validate_timestamp, after structural validation:
let start_millis = timestamp_to_millis(parts[0])?;
let end_millis = timestamp_to_millis(parts[1])?;
if end_millis < start_millis { return false; }
```

### WR-03: `update_timestamp` accepts unvalidated timestamp strings

**File:** `src/script/edit.rs:25-32`
**Issue:** The `update_timestamp` function sets the timestamp field to any string without calling `validate_timestamp`. A caller can inject an invalid timestamp like `"bad-format"` via this function, and the resulting `Script` will pass through `save_script` successfully. The next `load_script` call on that file will fail with a validation error, but the invalid data is already persisted to disk.

This is inconsistent with the design where `load_script` validates on load but the edit API provides no guard against invalid mutations. The error type `ScriptError::InvalidTimestamp` is defined in `error.rs` but never used anywhere in the codebase.

**Fix:** Validate the timestamp before applying the edit:

```rust
pub fn update_timestamp(script: &Script, index: usize, ts: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    if !validate_timestamp(ts) {
        return Err(ScriptError::InvalidTimestamp(ts.to_string()));
    }
    let mut new_script = script.clone();
    new_script[index].timestamp = ts.to_string();
    Ok(new_script)
}
```

### WR-04: `update_picture` and `update_narration` accept empty/whitespace-only strings

**File:** `src/script/edit.rs:5-12` and `src/script/edit.rs:35-42`
**Issue:** The `validate` function in `mod.rs` rejects empty/whitespace `picture` and `narration` fields on load, but `update_picture` and `update_narration` accept any string including empty or whitespace-only values. After calling `update_narration(&script, 0, "")`, the resulting script will fail to load via `load_script` due to validation. The same inconsistency as WR-03 applies: invalid data can be persisted and will only surface on the next load.

**Fix:** Add validation in the edit functions, or at minimum add a note in documentation that callers are responsible for validation before saving:

```rust
pub fn update_narration(script: &Script, index: usize, text: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    if text.trim().is_empty() {
        return Err(ScriptError::Validation(vec![ValidationError {
            clip_index: index,
            field: "narration".to_string(),
            message: "必须是非空字符串".to_string(),
        }]));
    }
    let mut new_script = script.clone();
    new_script[index].narration = text.to_string();
    Ok(new_script)
}
```

## Info

### IN-01: `ScriptError::InvalidTimestamp` variant is defined but never constructed

**File:** `src/script/error.rs:18-19`
**Issue:** The `InvalidTimestamp(String)` variant is defined with a `#[error("...")]` attribute and tested in unit tests, but is never used in any production code path. It appears designed for WR-03's fix but was left unused.

**Fix:** Either use it (see WR-03 fix) or remove it to avoid dead code.

### IN-02: Test data `python_sample_json()` does not round-trip through save/reload

**File:** `tests/script_test.rs:39-84`
**Issue:** The `test_load_python_sample` test (line 154-173) only verifies loading, not the full save-reload round-trip with Python sample data. This is a missed opportunity to verify that the realistic multi-paragraph Chinese content survives serialization.

**Fix:** Add an assertion after saving and reloading:

```rust
let save_path = dir.path().join("python_roundtrip.json");
save_script(&script, &save_path).expect("保存应成功");
let reloaded = load_script(&save_path).expect("重新加载应成功");
assert_eq!(reloaded.len(), script.len());
assert_eq!(reloaded[0].narration, script[0].narration);
```

### IN-03: `OstType` enum uses `repr(u8)` which limits future extensibility

**File:** `src/script/types.rs:10`
**Issue:** The `#[repr(u8)]` attribute means `OstType` can only represent values 0-255. If the Python pipeline ever adds an OST value outside this range (unlikely but possible), deserialization would fail silently or with a confusing error. Additionally, `serde_repr` with `u8` will reject any non-integer JSON values (e.g., `"0"` as a string). The Python code only checks `isinstance(clip['OST'], int)`, which would reject string values too, so this is consistent but worth noting.

**Fix:** No immediate action needed. This is informational for future phases.

---

_Reviewed: 2026-04-29T10:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
