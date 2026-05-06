---
phase: 07-sde-pipeline
reviewed: 2026-05-06T16:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/pipeline.rs
  - src/sde/script_gen.rs
  - src/sde/subtitle.rs
  - src/sde/timestamp.rs
  - src/sde/types.rs
  - src/documentary/pipeline.rs
  - src/documentary/types.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 7: Code Review Report (Eighth Pass)

**Reviewed:** 2026-05-06T16:00:00Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Eighth-pass review of the SDE pipeline module. All prior fixes verified:

- **WR-01** (sync write in async TTS): Verified fixed in `src/documentary/pipeline.rs:78` using `tokio::fs::write`. No regression.
- **WR-02** (subtitle_color validation): Verified fixed in `src/documentary/types.rs:77-80`. Consistent with SDE side.
- **Passes 1-6** (20 fixes): No regressions detected.

Two new warnings found, both related to async purity (same category as previously-fixed WR-01):

1. WR-08: `src/documentary/audio.rs:170` calls sync `write_srt_file` (which uses `std::fs::write`) from an `async fn` — identical pattern to WR-01 which was fixed in `documentary/pipeline.rs` but missed in `audio.rs`.
2. WR-09: `src/sde/pipeline.rs` lines 52, 701, 734 use `std::fs::create_dir_all` in async functions — same category of blocking I/O in async context.

Two info-level items: dead code in `subtitle.rs` (carried over from IN-01/IN-02) and an unused `_task_dir` parameter.

## Warnings

### WR-08: Sync `write_srt_file` in async `merge_subtitle_files`

**File:** `src/documentary/audio.rs:170`
**Issue:** `merge_subtitle_files` is a `pub async fn` that calls `crate::documentary::subtitle::write_srt_file(&merged, &output_path)?`. Internally, `write_srt_file` uses `std::fs::write` (line 68 of `subtitle.rs`), which blocks the Tokio runtime thread during disk I/O. This is the identical pattern to seventh-pass WR-01, which was fixed in `documentary/pipeline.rs` step_tts but missed in `audio.rs`.

This function is called from both SDE pipeline (step 6, line 286-292) and documentary pipeline (step 4), so the impact covers both pipelines.

Severity: WARNING (not CRITICAL) because subtitle files are typically small (a few KB), blocking time is negligible, and runtime starvation is unlikely.

**Fix:**
```rust
// Option A: Replace write_srt_file call with direct tokio::fs::write
tokio::fs::write(&output_path, &merged)
    .await
    .map_err(|e| PipelineError::Io(e))?;

// Option B: Make write_srt_file async internally
```

### WR-09: Sync `std::fs::create_dir_all` in async functions

**File:** `src/sde/pipeline.rs:52`, `src/sde/pipeline.rs:701`, `src/sde/pipeline.rs:734`
**Issue:** Three calls to `std::fs::create_dir_all` in `async fn` contexts. `create_dir_all` is a synchronous blocking operation (may involve filesystem traversal and permission checks). Documentary pipeline has the same pattern at line 484.

Severity: WARNING because directory creation happens once per task and is fast, but violates the same async purity principle as WR-08.

**Fix:**
```rust
// Replace:
std::fs::create_dir_all(&task_dir)?;
// With:
tokio::fs::create_dir_all(&task_dir).await?;
```

## Info

### IN-03: Dead code — `extract_text_from_srt` and `extract_text_from_ass`

**File:** `src/sde/subtitle.rs:357-417`
**Issue:** These two functions are only called from their own `#[cfg(test)]` tests. No production code path references them. `cargo check` emits `dead_code` warnings. Carried over from seventh-pass IN-01/IN-02.
**Fix:** Add `#[cfg(test)]` attribute or remove the functions.

### IN-04: Unused `_task_dir` parameter in `parse_script`

**File:** `src/sde/script_gen.rs:302`
**Issue:** `parse_script(raw_json: &str, _task_dir: &Path)` has an unused `_task_dir` parameter. The function comment states "caller is responsible for async-saving to task_dir/script_final.json", confirming the parameter is unused by design. Every caller constructs a `Path` reference that is never consumed.
**Fix:** Remove the parameter or use it for file saving as originally intended.

---

## Cross-Pipeline Consistency (SDE vs Documentary)

| Check | SDE | Documentary | Status |
|-------|-----|-------------|--------|
| subtitle_color validation | types.rs:89-92 | types.rs:77-80 | Consistent |
| TTS loop write | tokio::fs::write:184 | tokio::fs::write:78 | Consistent |
| concat logic | pipeline.rs:301-386 | pipeline.rs:182-269 | Consistent |
| composite logic | pipeline.rs:453-602 | pipeline.rs:325-463 | Consistent (mirrored) |
| subtitle merge write | delegates to merge_subtitle_files | same | Shared issue (WR-08) |
| create_dir_all | std::fs (3 places) | std::fs (1 place) | Shared issue (WR-09) |

## Security / Unsafe / Panic Scan

| Check | Result |
|-------|--------|
| `unsafe` blocks | None |
| Bare `.unwrap()` in production | None (only `LazyLock` regex init, compile-time safe) |
| Hardcoded secrets | None |
| Panic paths | `script_gen.rs:253` `unreachable!()` after exhaustive match — logically safe |
| Large functions (>50 lines) | `run_sde`: ~580 lines (orchestrator, well-structured but oversized) |

---

_Reviewed: 2026-05-06T16:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
