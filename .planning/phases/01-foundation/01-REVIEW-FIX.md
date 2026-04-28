---
phase: 01-foundation
fixed_at: 2026-04-28T08:02:18Z
review_path: .planning/phases/01-foundation/01-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-04-28T08:02:18Z
**Source review:** .planning/phases/01-foundation/01-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: HighPerformance profile has hwaccel_enabled=true but uses software-only encoder

**Files modified:** `src/ffmpeg/hwaccel.rs`
**Commit:** cb4a13e
**Applied fix:** Changed `encoder: "libx264"` (software) to `encoder: "h264_nvenc"` (hardware) for the `HighPerformance` profile, matching its `hwaccel_enabled: true` flag and "NVIDIA/AMD 硬件编码" description. Now consistent with `WindowsNvidia` profile which also enables hwaccel and uses `h264_nvenc`.

### WR-01: Two FFmpegError variants are dead code

**Files modified:** `src/ffmpeg/hwaccel.rs`, `src/ffmpeg/command.rs`
**Commit:** 76fe86c
**Applied fix:** Wired both previously-dead error variants into production code paths:

- `FFmpegError::BinaryNotFound`: In `detect_hw_encoders`, when `std::process::Command` fails with `std::io::ErrorKind::NotFound`, the function now returns `Err(FFmpegError::BinaryNotFound)` instead of silently returning an empty encoder list. Warnings for other IO error kinds (e.g., `PermissionDenied`) still return `Ok(Vec::new())`.

- `FFmpegError::Timeout`: In `clip_video`, the `spawn_blocking` future is now wrapped with `tokio::time::timeout(Duration::from_secs(600), ...)`. If FFmpeg hangs for more than 10 minutes, the function returns `Err(FFmpegError::Timeout(...))` instead of blocking indefinitely.

**Requires human verification:** The `BinaryNotFound` wiring changes `detect_hw_encoders` semantics -- the function now returns an `Err` when the binary is missing rather than `Ok(empty)`. The test `test_detect_encoders_format` asserts `result.is_ok()` and may need updating if ffmpeg is not installed in the test environment. The `Timeout` wiring introduces a 600-second hard timeout on clip operations; verify this default is appropriate for expected video lengths.

### WR-02: test_load_example_config depends on external config.example.toml with no guard

**Files modified:** `tests/config_test.rs`
**Commit:** daeaf29
**Applied fix:** Added a runtime existence check (`config_path.exists()`) before attempting to load `config.example.toml`. If the file is missing (deleted, renamed, partial checkout), the test prints a skip message and returns early instead of panicking.

### WR-03: test_hot_reload uses sleep-based synchronization that can be flaky

**Files modified:** `src/config/watcher.rs`, `src/config/mod.rs`, `tests/config_test.rs`
**Commit:** 866178f
**Applied fix:** Replaced the fixed `std::thread::sleep(Duration::from_millis(200))` with a proper readiness signal mechanism:

- Added an optional `ready_signal: Option<Sender<()>>` parameter to `ConfigWatcher::new()`. The signal is sent immediately after `watcher.watch()` succeeds, indicating the file-system listener is established.
- Added `ConfigManager::start_watching_with_ready()` method for callers that need the readiness signal.
- The test now creates a `std::sync::mpsc::channel()`, passes the sender to `start_watching_with_ready()`, and waits via `recv_timeout(Duration::from_secs(5))` instead of sleeping.

### WR-04: Hardcoded credential-like value in SoulVoiceSection default

**Files modified:** `src/config/defaults.rs`
**Commit:** d66ee9b
**Applied fix:** Replaced the credential-like `voice_uri` default value (`speech:mcg3fdnx:clzkyf4vy00e5qr6hywum4u84:bzznlkuhcjzpbosexitr`) with `String::new()`, preventing potential unauthorized use of shared TTS credentials in unconfigured deployments.

### WR-05: FfmpegEvent::Error silently swallowed during clip_video progress loop

**Files modified:** `src/ffmpeg/command.rs`
**Commit:** 789f02f
**Applied fix:** Added a `had_errors` flag that is set to `true` when `FfmpegEvent::Error` events are received during the progress iteration loop. After the loop completes, if `had_errors` is `true`, the function returns `Err(FFmpegError::ExecutionError(...))` instead of silently continuing. Errors are still logged via `tracing::error!`.

**Requires human verification:** This changes error handling semantics -- previously, non-fatal FFmpeg errors during stream processing were logged but `clip_video` would still return `Ok(())` if the process exited with code 0. Now any `FfmpegEvent::Error` causes the function to fail. Verify this is the desired behavior for all callers.

---

_Fixed: 2026-04-28T08:02:18Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
