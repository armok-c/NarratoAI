---
phase: 07-sde-pipeline
reviewed: 2026-05-08T14:30:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - narratoai-core/src/sde/error.rs
  - narratoai-core/src/sde/mod.rs
  - narratoai-core/src/sde/script_gen.rs
  - narratoai-core/src/sde/timestamp.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/documentary/mod.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/documentary/subtitle.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/documentary/clip.rs
  - narratoai-core/src/documentary/error.rs
  - narratoai-core/src/documentary/timestamp.rs
  - narratoai-core/src/documentary/audio.rs
  - tests/documentary_integration_test.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 07: Code Review Report (15th Pass)

**Reviewed:** 2026-05-08
**Depth:** standard
**Files Reviewed:** 16
**Status:** clean

## Summary

15th pass review covering all 16 files in the SDE and documentary pipeline modules. Previous pass (14th) found 1 warning (WR-01: documentary pipeline SRT path escaping incomplete). That warning is verified as fixed in commit `ca0f449` -- `documentary/pipeline.rs:390-398` now escapes `:`, `[`, `]`, `;` identically to `sde/pipeline.rs:521-529`.

The new uncommitted changes are clean:
- Dead code removal (`ProgressStep` enum, `collect_keyframe_paths`) is correct -- no remaining references found.
- SRT sequence numbering fix in `generate_srt_from_word_boundaries` is a legitimate correctness fix: the old `i + 1` from `enumerate()` produced non-contiguous SRT sequence numbers when word boundaries with negative timestamps were skipped via `continue`. The new `seq` counter ensures contiguous 1-based numbering.
- Test update correctly aligns with the string-based `ProgressCallback` signature.

All 16 reviewed files meet quality standards. No issues found.

## Warning Verification

### WR-01 (14th pass): Documentary pipeline SRT path escaping -- VERIFIED FIXED

**File:** `narratoai-core/src/documentary/pipeline.rs:390-398`

The documentary pipeline's SRT path escaping now matches the SDE pipeline exactly:

```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace(':', "\\:")
    .replace("'", "\\'")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace(';', "\\;")
    .replace('\n', "")
    .replace('\r', "");
```

Fix confirmed in commit `ca0f449`.

## Info

### IN-01: Dead code removal is correct

**Files:** `narratoai-core/src/documentary/types.rs`, `narratoai-core/src/documentary/mod.rs`, `narratoai-core/src/documentary/script_gen.rs`

The `ProgressStep` enum was removed from `documentary/types.rs` and its re-export from `documentary/mod.rs`. No source code references remain. The `collect_keyframe_paths` function (19 lines) was removed from `script_gen.rs` -- it was a dead duplicate of `collect_keyframe_paths_from_dir` in `visual/frame_extractor.rs`. The `analyze_video` function receives `keyframe_files` directly from `extract_frames` and never calls this function.

### IN-02: SRT sequence numbering is a correctness fix, not just refactor

**File:** `narratoai-core/src/documentary/subtitle.rs:22-42`

The old code used `i + 1` from `enumerate()` as the SRT sequence number. When a word boundary produced a negative timestamp (lines 26-31 `continue`), the sequence would skip a number (e.g., 1, 2, 4, 5 instead of 1, 2, 3, 4). The SRT format requires contiguous sequence numbers. The new `seq` counter (initialized to 1, incremented only when a block is pushed) produces correct contiguous numbering.

### IN-03: Test update is clean

**File:** `tests/documentary_integration_test.rs:115-166`

The progress callback test was updated to use the string-based `ProgressCallback` type alias (`Fn(&str, f32, &str)`) instead of the removed `ProgressStep` enum. The test correctly imports `ProgressCallback` from `narratoai_core::documentary` and uses `&str` step names. All assertions (step count, step names, non-decreasing percentages, final 100%) remain valid.

---

_Reviewed: 2026-05-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Pass: 15th_
