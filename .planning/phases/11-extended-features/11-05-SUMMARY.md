---
phase: 11-extended-features
plan: 05
subsystem: tests
tags: [integration-tests, stubs, ffmpeg, yt-dlp, pexels, pixabay]
requires: [02, 03, 04]
affects: [tests/audio_integration.rs, tests/youtube_integration.rs, tests/material_integration.rs]
tech-stack:
  added: []
  patterns:
    - "Integration test stub pattern with #[ignore] per D-28, D-29"
    - "External binary detection helpers (find_ffmpeg, find_ytdlp) before test functions"
    - "API key env var checking in material tests"
key-files:
  created:
    - tests/audio_integration.rs
    - tests/youtube_integration.rs
    - tests/material_integration.rs
  modified: []
metrics:
  duration: null
  completed_date: 2026-04-30
  tasks: 1
  commits: 1
---

# Phase 11 Plan 05: Integration Test Stubs Summary

## One-liner

Create 3 integration test stub files (audio_integration.rs, youtube_integration.rs, material_integration.rs) with `#[ignore]` markers matching PATTERNS.md mapping, enabling VALIDATION.md cargo test commands to find test modules.

## Task Completion

### Task 1: Create 3 integration test stub files

**Status:** Complete

**Created files:**

| File | Tests | External Dependencies | Verification |
|------|-------|----------------------|--------------|
| `tests/audio_integration.rs` | `test_normalize_lufs_two_pass`, `test_get_audio_rms_with_real_file` | FFmpeg (find_ffmpeg helper) | Compiles, 2 #[ignore] |
| `tests/youtube_integration.rs` | `test_ytdlp_version`, `test_get_video_formats` | yt-dlp (find_ytdlp helper) | Compiles, 2 #[ignore] |
| `tests/material_integration.rs` | `test_pexels_search_integration`, `test_pixabay_search_integration` | PEXELS_API_KEY / PIXABAY_API_KEY env vars | Compiles, 2 #[ignore] |

**Compilation verification:**
```text
cargo test --test audio_integration --test youtube_integration --test material_integration --no-run
Finished test profile [unoptimized + debuginfo] in 0.45s
```

**Test discovery (all #[ignore], none executed):**
```text
Running tests/audio_integration.rs — 0 passed, 0 failed, 2 ignored
Running tests/material_integration.rs — 0 passed, 0 failed, 2 ignored
Running tests/youtube_integration.rs — 0 passed, 0 failed, 2 ignored
```

**Acceptance criteria:**
- [x] tests/audio_integration.rs exists with find_ffmpeg + 2 #[ignore] tests
- [x] tests/youtube_integration.rs exists with find_ytdlp + 2 #[ignore] tests
- [x] tests/material_integration.rs exists with PEXELS_API_KEY/PIXABAY_API_KEY env var check + 2 #[ignore] tests
- [x] All 3 test files compile (cargo test --test ... --no-run passes)
- [x] Per PATTERNS.md mapping: audio=ffmpeg_test analog, youtube=script_test analog, material=llm_test analog

**Commit:** `d83d8f9`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Warning] Unused import in audio_integration.rs**

- **Found during:** Task 1 (cargo test compilation)
- **Issue:** `use std::path::Path` was unused in the stub (no Path-typed calls yet)
- **Fix:** Removed the unused import line
- **Files modified:** `tests/audio_integration.rs`
- **Commit:** Included in d83d8f9 (fixed before commit)

### Out-of-Scope Discoveries

**Pre-existing compilation error in `tests/tts_test.rs:113`**

- `tts::synthesize()` signature changed (8 args now required) but test passes only 7
- Blocks `cargo test` full suite compilation
- Not caused by this plan
- Documented in `.planning/phases/11-extended-features/deferred-items.md`

## Decisions Made

None — this plan followed the exact specification from PATTERNS.md and the plan file.

## Known Stubs

All 3 files are explicitly stubs per plan design. Each test has `assert!(true, "存根 — 执行阶段填充具体测试逻辑")` placeholder. The plan's purpose is to create discoverable test modules for VALIDATION.md commands; implementation will be filled in later execution phases.

## Threat Flags

No new threat surface introduced. Test stubs do not call external services,
do not read sensitive data, and are `#[ignore]` by default.

## Verification

- [x] All 3 test files compile individually
- [x] All 6 tests are `#[ignore]` (none run with plain `cargo test`)
- [x] External dependency detection functions present (find_ffmpeg, find_ytdlp)
- [x] API key env var checking in material tests
- [x] Chinese error messages per project convention
- [x] Per PATTERNS.md integration test pattern

## Self-Check: PASSED

File existence checks:
- [x] tests/audio_integration.rs — 32 lines, 2 #[ignore] tests
- [x] tests/youtube_integration.rs — 54 lines, 2 #[ignore] tests, find_ytdlp helper
- [x] tests/material_integration.rs — 26 lines, 2 #[ignore] tests, env var checks

Commit hash d83d8f9 verified via `git show --stat HEAD`.
