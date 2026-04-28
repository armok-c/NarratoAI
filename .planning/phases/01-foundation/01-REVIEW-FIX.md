---
phase: 01-foundation
fixed: 2026-04-28
fix_scope: all
findings_in_scope: 13
fixed: 9
skipped: 4
iteration: 2
status: partial
---

# Phase 01: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-04-28T22:00:00Z
**Source review:** `.planning/phases/01-foundation/01-REVIEW.md`
**Iteration:** 2

**Summary:**

| Metric | Count |
|--------|-------|
| Findings in scope | 13 |
| Fixed | 9 |
| Skipped | 4 |
| Status | partial |

## Fixed Issues

### IN-04: test_hot_reload uses 1700ms total sleep

**Files modified:** `tests/config_test.rs`
**Commit:** `4566a80`
**Applied fix:** Replaced the fixed 1500ms sleep after config write with a polling loop (50ms interval, 5s timeout). The test now succeeds immediately after the watcher fires instead of always waiting 1.5s. Config integration tests complete in 0.25s (was ~1.9s).

### IN-05: NamedTempFile write-through pattern

**Files modified:** `src/config/mod.rs`, `tests/config_test.rs`
**Commit:** `d6cced0`
**Applied fix:** Replaced `NamedTempFile::new() + std::fs::write()` pattern with `TempDir + file path` in all three test locations. Avoids the potential Windows OS lock issue where creating an empty NamedTempFile then immediately overwriting it could cause file contention. The `NamedTempFile` import was also removed from `tests/config_test.rs` as it became unused.

### IN-06: Clippy suggestions -- derivable Default impls + doc comment formatting

**Files modified:** `src/config/types.rs`, `src/config/defaults.rs`
**Commits:** `bfcd268`, `e0edd4a`
**Applied fix:**
1. Added `#[derive(Default)]` to `AzureSection` and `ProxySection` struct definitions in `types.rs`. Removed the manual `impl Default` blocks from `defaults.rs` (21 lines of boilerplate eliminated). Resolves `clippy::derivable_impls`.
2. Removed empty line between doc comment and the `impl Default for AppSection` block, resolving `clippy::empty_line_after_doc_comments`.

### IN-07: f64-to-string precision in FFmpeg seek/duration

**File modified:** `src/ffmpeg/command.rs`
**Commit:** `59da3c7`
**Applied fix:** Changed `start.to_string()` to `format!("{:.3}", start)` and `duration.to_string()` to `format!("{:.3}", duration)`. This limits decimal precision to milliseconds, avoiding floating-point rounding artifacts like "0.30000000000000004".

### IN-08: Fast seek (keyframe-based) sacrifices frame accuracy

**File modified:** `src/ffmpeg/command.rs`
**Commit:** `59da3c7`
**Applied fix:** Added inline comment `// -ss before -i = fast seek (keyframe-based, not frame-accurate)` before the seek call, documenting the tradeoff. Frame-accurate slow seek (`-i <input> -ss <start>`) can be added in a future phase if needed.

### IN-10: Integration tests do not validate output content

**File modified:** `tests/ffmpeg_test.rs`
**Commit:** `cafc36c`
**Applied fix:** Added `probe_video()` call in `test_clip_video_async` after the file size check. The new assertions verify output has positive duration, positive width/height, and non-empty codec name, ensuring the clipped output is a valid video.

### IN-11: AppConfig manual Default impl is derivable

**Files modified:** `src/config/types.rs`, `src/config/defaults.rs`
**Commit:** `4fe3d5f`
**Applied fix:** Added `#[derive(Default)]` to the `AppConfig` struct definition in `types.rs`. Removed the manual `impl Default for AppConfig` block from `defaults.rs` (17 lines eliminated). All sub-fields already implement `Default`. Resolves `clippy::derivable_impls`.

### IN-12: bool_assert_comparison in test

**File modified:** `src/config/types.rs`
**Commit:** `3544b39`
**Applied fix:** Changed `assert_eq!(config.proxy.enabled, false, ...)` to `assert!(!config.proxy.enabled, ...)`. Resolves `clippy::bool_assert_comparison`.

### IN-13: useless_vec in test

**File modified:** `src/ffmpeg/hwaccel.rs`
**Commit:** `e0ef670`
**Applied fix:** Changed `let variants = vec![...]` to `let variants = [...]` (array literal). `.len()` is available on fixed-size arrays. Resolves `clippy::useless_vec`.

## Skipped Issues

### IN-01: notify dependency uses RC version

**File:** `Cargo.toml:14`
**Reason:** Defer -- RC version tracking is a maintenance task. `notify = "9.0.0-rc.3"` should be upgraded to the stable release when available. Not actionable without a known stable version.
**Original issue:** notify = "9.0.0-rc.3" is a release candidate. API may change before stable.

### IN-02: Config structs do not use deny_unknown_fields

**File:** `src/config/types.rs:4-164`
**Reason:** Defer -- requires a design decision on schema stabilization. Adding `#[serde(deny_unknown_fields)]` would cause failures if users have typos in their config files. Should be decided in a later phase with input from the project maintainer.
**Original issue:** No `#[serde(deny_unknown_fields)]`. TOML typos are silently ignored.

### IN-03: IndexTTS2Section repetition_penalty default 10.0

**File:** `src/config/defaults.rs:93`
**Reason:** Defer -- requires verification of whether 10.0 is intentional. The value is propagated from `config.example.toml` and may be correct for the specific TTS model used. Needs human decision to confirm or change.
**Original issue:** repetition_penalty defaults to 10.0, far outside typical range (1.0-2.0).

### IN-09: tokio features = ["full"]

**File:** `Cargo.toml:7`
**Reason:** Defer -- all tokio usage patterns are not yet known (current phase only uses `rt-multi-thread` and `macros`). Feature minimization should be done in a later phase when the complete tokio surface is visible.
**Original issue:** features = ["full"] enables all tokio features, increasing compile time and binary size.

---

## Build and Test Results

| Suite | Before | After |
|-------|--------|-------|
| `cargo build` | 0 errors, 0 warnings | 0 errors, 0 warnings |
| `cargo test --lib` (unit tests) | 26/26 pass | 26/26 pass |
| `cargo test --test config_test` | 6/6 pass | 6/6 pass (0.25s, was ~1.9s) |
| `cargo test --test ffmpeg_test` | 3/5 pass* | 3/5 pass* |

*FFmpeg-dependent tests fail with clear install instruction when FFmpeg is absent (expected, not a regression).

---

_Fixed: 2026-04-28_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
