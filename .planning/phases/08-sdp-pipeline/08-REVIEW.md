---
phase: 08-sdp-pipeline
reviewed: 2026-05-08T16:30:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - narratoai-core/src/subtitle/mod.rs
  - narratoai-core/src/subtitle/error.rs
  - narratoai-core/src/subtitle/types.rs
  - narratoai-core/src/subtitle/parser.rs
  - narratoai-core/src/subtitle/timestamp.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sde/timestamp.rs
  - narratoai-core/src/sde/error.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/mod.rs
  - narratoai-core/src/lib.rs
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/sdp/mod.rs
  - narratoai-core/src/sdp/error.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/sdp/script_gen.rs
  - narratoai-core/src/sdp/clip.rs
  - narratoai-core/src/sdp/pipeline.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 08: SDP Pipeline Code Review Report (Re-review 3)

**Reviewed:** 2026-05-08T16:30:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

This is the FOURTH review (re-review 3) of Phase 08. Previous reviews found and fixed CR-01, CR-02, WR-01 through WR-09, and identified 3 info items (IN-01 through IN-03).

**CR-02 and WR-09 fix verification:**
- **CR-02 (FIXED and VERIFIED):** `sdp/script_gen.rs` line 234 now uses `normalized_text.as_str()` (full SRT/ASS with timecodes) for the second LLM call (`plot_extraction`), not `subtitle_plain`. The fix is correctly applied.
- **WR-09 (FIXED and VERIFIED):** `sdp/clip.rs` lines 97-104 now returns an explicit `SdpError::Validation` error in the `else` branch when a clip timestamp lacks the `-` separator. The fix is correctly applied.

All previously fixed issues (CR-01, CR-02, WR-01 through WR-09) remain correctly fixed in the current code.

This review found 2 new warnings. No new critical issues were found.

## Warnings

### WR-10: SdpRequest::validate does not check `script_path` existence when provided, leading to confusing "No such file" IO error later

**File:** `narratoai-core/src/sdp/types.rs:52-80`
**Issue:** `SdpRequest::validate()` checks `subtitle_path.exists()` and `video_path.exists()` at lines 59-63, but does not check `script_path.exists()` when `script_path` is `Some(...)`. The `run_sdp` pipeline function at `sdp/pipeline.rs:43` calls `load_script(&script_path)` which performs blocking `std::fs::read_to_string` -- this will fail with a generic IO error like "No such file or directory" rather than a clear validation error.

Compare with `SdeRequest::validate()` which checks all required file paths. The `script_path` is optional (None means auto-generate), but when it IS provided by the caller, its existence should be verified at validation time.

**Fix:** Add a check for `script_path` existence in `validate()`:
```rust
if let Some(ref sp) = self.script_path {
    if !sp.exists() {
        return Err(format!("脚本文件不存在: {}", sp.display()));
    }
}
```

### WR-11: `run_sdp` calls synchronous `load_script` (blocking filesystem read) directly on the tokio async runtime

**File:** `narratoai-core/src/sdp/pipeline.rs:43`
**Issue:** `state.script = crate::script::load_script(&script_path)?;` performs a blocking `std::fs::read_to_string` call directly on the async runtime thread. The codebase already uses `tokio::task::spawn_blocking` for `parse_subtitle_file` (line 54-61), but `load_script` -- which does the same kind of blocking I/O -- is called synchronously.

This is inconsistent with the pattern used elsewhere in the same function and could block the tokio runtime for large script files.

**Fix:** Wrap `load_script` in `spawn_blocking`:
```rust
let script_path_clone = script_path.clone();
state.script = tokio::task::spawn_blocking(move || {
    crate::script::load_script(&script_path_clone)
})
.await
.map_err(|e| SdpError::Io {
    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
})??;
```

## Previously Confirmed Fixed Issues

All previously reported issues remain correctly fixed:

- **CR-01** (subtitle_segments never populated): Fixed in `sdp/pipeline.rs:52-62`
- **CR-02** (step 2 sends plain text to LLM): Fixed in `sdp/script_gen.rs:234`
- **WR-01** (timestamp parse failure silently swallowed): Fixed in `sdp/clip.rs:61-68`
- **WR-02** (zero-duration clips silently skipped): Fixed in `sdp/clip.rs:79-83`
- **WR-03** (missing file existence check): Fixed in `sdp/types.rs:59-63`
- **WR-04** (concat missing newline guard): Fixed in `sdp/pipeline.rs:104-108`
- **WR-05** (unused _config parameter): Fixed, removed from signature
- **WR-06** (sync blocking I/O in async context): Fixed with tokio::fs
- **WR-07** (redundant TTS skip condition): Fixed in `sde/pipeline.rs`
- **WR-08** (missing 90% progress checkpoint): Fixed in `sdp/pipeline.rs:83`
- **WR-09** (invalid timestamp silently skipped): Fixed in `sdp/clip.rs:97-104`

Previously reported info items (IN-01: duplicate secs_to_timestamp_str, IN-02: unused _encoding, IN-03: SDE vs SDP different timestamp parsers) remain as-is and are not re-reported.

---

_Reviewed: 2026-05-08T16:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
