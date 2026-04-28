---
phase: 01-foundation
reviewed: 2026-04-28T16:30:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - src/config/defaults.rs
  - src/config/mod.rs
  - src/config/types.rs
  - src/config/watcher.rs
  - src/error.rs
  - src/ffmpeg/command.rs
  - src/ffmpeg/hwaccel.rs
  - src/ffmpeg/mod.rs
  - src/ffmpeg/probe.rs
  - src/lib.rs
  - tests/config_test.rs
  - tests/ffmpeg_test.rs
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
---

# Phase 01: Re-Review Report (Post-Fix Verification)

**Reviewed:** 2026-04-28T16:30:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Re-reviewed all 12 source files from Phase 01 after fixes were applied for CR-01, WR-01 through WR-05. All 6 actionable fixes have been verified as correct and complete. One new WARNING-level issue was discovered during re-review: `probe_video` maps all `Command::new` spawn errors to `SpawnFailed`, including `NotFound`, creating an inconsistency with `detect_hw_encoders` which was fixed to return `BinaryNotFound` for the same case.

The 4 original Info items (IN-01 through IN-04) remain unchanged -- none were actioned, which is acceptable for Info-level findings.

## Fix Verification

### CR-01: HighPerformance profile encoder mismatch -- FIXED

**File:** `src/ffmpeg/hwaccel.rs:47`
**Verdict:** Correct. The `encoder` field now reads `"h264_nvenc"` (hardware encoder), which is consistent with `hwaccel_enabled: true` and `hwaccel_type: Some("auto")`. The profile data is no longer self-contradictory.

### WR-01: Dead-code FFmpegError variants -- FIXED

**File:** `src/ffmpeg/command.rs:59,123` and `src/ffmpeg/hwaccel.rs:138`
**Verdict:** Both variants are now wired into code paths:
- `FFmpegError::Timeout`: `clip_video` wraps the entire `spawn_blocking` call with `tokio::time::timeout(Duration::from_secs(600), ...)`, mapping the timeout to `FFmpegError::Timeout`.
- `FFmpegError::BinaryNotFound`: `detect_hw_encoders` now checks `e.kind() == std::io::ErrorKind::NotFound` and returns `Err(FFmpegError::BinaryNotFound)`.
- No regressions introduced.

### WR-02: test_load_example_config existence guard -- FIXED

**File:** `tests/config_test.rs:13-16`
**Verdict:** Correct. The test now checks `config_path.exists()` and returns early with an `eprintln` message if the file is absent. No panic on missing file.

### WR-03: test_hot_reload readiness signal -- FIXED

**File:** `tests/config_test.rs:172-176` and `src/config/watcher.rs:22,60-62`
**Verdict:** Correct. The test uses `std::sync::mpsc::channel()` and passes the sender to `start_watching_with_ready()`. The watcher sends `()` on the channel after `watcher.watch()` succeeds. The test awaits the signal with `recv_timeout(Duration::from_secs(5))` before writing the updated config. The `ConfigManager` now exposes `start_watching_with_ready()` as a public method. No fixed sleep remains. No regressions.

### WR-04: Hardcoded credential-like voice_uri default -- FIXED

**File:** `src/config/defaults.rs:56`
**Verdict:** Correct. `voice_uri` now defaults to `String::new()` (empty string). No credential-like value remains.

### WR-05: FfmpegEvent::Error silently swallowed -- FIXED

**File:** `src/ffmpeg/command.rs:84,94-96,102-106`
**Verdict:** Correct. A `had_errors: bool` flag accumulates across the event loop. After the loop completes, if `had_errors` is true, the function returns `Err(FFmpegError::ExecutionError(...))`. This check runs before the `child.wait()` exit-status check, which is the correct order -- errors from FFmpeg stderr are surfaced regardless of exit code.

## Warnings

### WR-06: probe_video does not return BinaryNotFound when ffprobe binary is missing

**File:** `src/ffmpeg/probe.rs:36`
**Issue:** The WR-01 fix correctly wired `FFmpegError::BinaryNotFound` into `detect_hw_encoders` (hwaccel.rs:138), checking `e.kind() == std::io::ErrorKind::NotFound`. However, `probe_video` in probe.rs performs the same `Command::new(&ffprobe_bin).output()` call but maps all IO errors uniformly to `SpawnFailed`:
```rust
.output()
.map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;
```
When ffprobe is not installed, the error is `io::ErrorKind::NotFound`, but the caller receives `SpawnFailed` instead of `BinaryNotFound`. This is an inconsistency within the FFmpeg subsystem -- two functions that invoke external FFmpeg binaries handle the same "binary not found" condition differently. A downstream caller handling `BinaryNotFound` to prompt the user to install FFmpeg would miss this case when it originates from `probe_video`.

**Fix:**
```rust
let output = match Command::new(&ffprobe_bin)
    .args([...])
    .arg(path.as_os_str())
    .output()
{
    Ok(o) => o,
    Err(e) => {
        if e.kind() == std::io::ErrorKind::NotFound {
            return Err(FFmpegError::BinaryNotFound);
        }
        return Err(FFmpegError::SpawnFailed(e.to_string()));
    }
};
```

## Info

### IN-01: `_watcher` field naming convention misleading -- STILL PRESENT

**File:** `src/config/mod.rs:16`
**Status:** Unchanged. The `_watcher` field still uses the leading-underscore convention. This is an Info item and its presence is acceptable.

### IN-02: AppConfig::validate() is a no-op stub -- STILL PRESENT

**File:** `src/config/mod.rs:73-76`
**Status:** Unchanged. The method returns `Ok(())` unconditionally with no TODO markers. This is an Info item and its presence is acceptable.

### IN-03: test_load_empty_toml discards parsed values without assertions -- STILL PRESENT

**File:** `src/config/types.rs:312-317`
**Status:** Unchanged. The test still uses `let _ =` for all parsed fields. This is an Info item and its presence is acceptable.

### IN-04: .gitignore includes Python-project patterns -- NOT IN REVIEW SCOPE

**Status:** `.gitignore` was excluded from this re-review's file list (only source files in scope). No assessment made.

---

_Reviewed: 2026-04-28T16:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
