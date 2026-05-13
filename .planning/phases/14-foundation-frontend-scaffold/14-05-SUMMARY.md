---
phase: 14-foundation-frontend-scaffold
plan: 05
subsystem: testing
tags: [integration-test, assert-true-cleanup, tdet-03, rust]

# Dependency graph
requires:
  - phase: 14-foundation-frontend-scaffold
    plan: 14-04
    provides: gap closure plan framework
provides:
  - Real integration tests for audio LUFS normalization and RMS analysis
  - Real integration test for YouTube yt-dlp format retrieval
  - TDET-03 complete: zero assert!(true) stubs in entire codebase
affects: [narratoai-core]

# Tech tracking
tech-stack:
  added: []
  patterns: [integration tests moved from root tests/ to crate-level narratoai-core/tests/]

key-files:
  created:
    - narratoai-core/tests/audio_lufs_integration.rs
    - narratoai-core/tests/youtube_integration.rs
  modified: []
  deleted:
    - tests/audio_integration.rs
    - tests/youtube_integration.rs
---

## Summary

Closed SC5 gap: replaced 3 `assert!(true)` stubs with real integration tests and moved them from dead root `tests/` to `narratoai-core/tests/`.

## Tasks Completed

### Task 1: Replace audio integration test stubs
- Created `narratoai-core/tests/audio_lufs_integration.rs` with 2 tests:
  - `test_normalize_lufs_two_pass` (#[ignore]): generates sine wave WAV, runs LUFS two-pass normalization, verifies output LUFS within [-26, -20]
  - `test_get_audio_rms_with_real_file`: generates sine wave, calls `get_audio_rms()`, verifies RMS dB in [-10, 0]
- Deleted `tests/audio_integration.rs`

### Task 2: Replace YouTube integration test stub
- Created `narratoai-core/tests/youtube_integration.rs` with 2 tests:
  - `test_ytdlp_version` (#[ignore]): verifies yt-dlp is installed and returns version
  - `test_get_video_formats` (#[ignore]): calls yt-dlp --dump-json on known public video, validates format list
- Deleted `tests/youtube_integration.rs`

## Verification

- `cargo test -p narratoai-core --test audio_lufs_integration -- --include-ignored` → 2/2 passed
- `cargo test -p narratoai-core --test youtube_integration` → compiled, 2 ignored (expected)
- `grep -rn 'assert!(true)'` → zero matches across entire codebase
- TDET-03: COMPLETE

## Deviations

None.
