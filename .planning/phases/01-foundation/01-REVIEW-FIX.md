---
phase: 01-foundation
fixed_at: 2026-04-30T09:50:00+08:00
review_path: .planning/phases/01-foundation/01-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 01: Foundation - Code Review Fix Report

**Fixed at:** 2026-04-30T09:50:00+08:00
**Source review:** .planning/phases/01-foundation/01-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (2 Critical + 1 Warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: FFmpeg codec copy parameter order error leading to silent re-encoding

**Files modified:** `src/ffmpeg/command.rs`
**Commit:** f69539c

**Applied fix:** Moved `.codec_video("copy")`, `.codec_audio("copy")`, and `.overwrite()` before `.output(&output_path)`. Previously these options appeared after the output URL, causing FFmpeg to silently ignore the codec copy directives and fall back to full re-encoding (libx264). The generated FFmpeg command line now places `-c:v copy -c:a copy -y` before the output file argument, ensuring they are applied to the correct output stream.

### CR-02: `spawn_blocking` timeout creates orphan FFmpeg subprocess leaking resources

**Files modified:** `src/ffmpeg/command.rs`, `Cargo.toml`
**Commit:** 2fe871b

**Applied fix:** Replaced `tokio::time::timeout` wrapping `spawn_blocking` with a `CancellationToken` + `tokio::select!` pattern. The blocking task now checks `cancel.is_cancelled()` before processing each FFmpeg event. When the 600s timeout fires via `tokio::time::sleep`, the cancellation token is signalled, and the blocking task cooperatively kills the FFmpeg child process (`child.kill()` + `child.wait()`) before returning. Added `tokio-util` v0.7 with `rt` feature to `Cargo.toml` for `CancellationToken`.

### WR-01: `ffmpeg-sidecar` download_ffmpeg feature exposes build-time supply chain risk

**Files modified:** `Cargo.toml`
**Commit:** d4b54ff

**Applied fix:** Added a `[features]` section with `default = []` and `download-ffmpeg = ["ffmpeg-sidecar/download_ffmpeg"]`. Changed the `ffmpeg-sidecar` dependency from `{ version = "2.5.1", features = ["download_ffmpeg"] }` to just `"2.5.1"`. The automatic FFmpeg binary download is now opt-in via `cargo build --features download-ffmpeg`. Production builds that require pre-installed FFmpeg should use `cargo build --no-default-features`.

---

_Fixed: 2026-04-30T09:50:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
