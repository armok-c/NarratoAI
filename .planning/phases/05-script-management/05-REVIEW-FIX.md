---
phase: 05-script-management
fix_date: 2026-04-29
fix_scope: Critical + Warning
findings_fixed: 2
findings_deferred: 2 (Info-level, excluded from default scope)
commit: pending
tests_after_fix: 68 passed, 0 failed
---

# Phase 05: Code Review Fix Report (Re-review Fix)

**Fixed:** 2026-04-29
**Scope:** Critical + Warning (default)
**Source Review:** 05-REVIEW.md (re-review after commit 2ce41d2)

## Summary

This is a fix pass for the **re-review** findings (WR-01/WR-02 in the second review cycle). The original 6 findings (2 CR, 4 WR) were fixed in commit 2ce41d2. This pass addresses the 2 new Warning findings discovered during re-review.

- Findings in scope: 2 (Warning)
- Fixed: 2
- Deferred: 2 (Info-level)

## Fixes Applied

### WR-01: `save_script` does not use atomic file writes (status: fixed)

**File:** `src/script/mod.rs:144-150`
**Change:** Replaced direct `std::fs::write` with write-to-temp + rename pattern.

```rust
// Before:
std::fs::write(path, json.as_bytes()).map_err(ScriptError::Io)?;

// After:
let temp_path = path.with_extension("json.tmp");
std::fs::write(&temp_path, json.as_bytes()).map_err(ScriptError::Io)?;
std::fs::rename(&temp_path, path).map_err(ScriptError::Io)?;
```

**Rationale:** If the process crashes mid-write, the temp file is corrupted but the original file remains intact. `rename` is atomic on POSIX and best-effort on Windows.

### WR-02: `validate_timestamp` uses naive `split('-')` (status: fixed)

**File:** `src/script/mod.rs:48-50`
**Change:** Added comment documenting the assumption that the timestamp format never contains internal hyphens.

```rust
// NOTE: Split on '-' assumes the timestamp format never contains internal hyphens.
// The format "HH:MM:SS,mmm-HH:MM:SS,mmm" guarantees this because all components
// are digits, colons, and commas.
let parts: Vec<&str> = ts.split('-').collect();
```

**Rationale:** The current format is safe. If SRT/VTT format support is needed later, this comment alerts developers to update parsing.

## Deferred (Info-level)

| ID | Finding | Reason |
|----|---------|--------|
| IN-01 | `deserialize_id` silently truncates fractional values | Low risk: LLMs don't produce `_id: 1.9`. Optional stricter check can be added if needed. |
| IN-02 | `Script` is a bare type alias limiting API evolution | No immediate need. Refactor to struct when top-level metadata is required. |

## Verification

- **Tests:** 68 library unit tests passed, 0 failed
- **No new tests added:** Fixes are defensive improvements that don't change external behavior
- **Files modified:** `src/script/mod.rs` only

---

_Fixed: 2026-04-29_
_Agent: gsd-code-fixer_
