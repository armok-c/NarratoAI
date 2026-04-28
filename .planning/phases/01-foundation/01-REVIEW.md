---
phase: 01-foundation
reviewed: 2026-04-28T14:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - .gitignore
  - Cargo.lock
  - Cargo.toml
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
  critical: 1
  warning: 5
  info: 4
  total: 10
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-28T14:00:00Z
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

Reviewed the Rust core (`narratoai-core`) foundation layer. Key areas examined: config system (types, defaults, serde correctness, hot-reload), FFmpeg layer (command construction, hardware acceleration profiles, video probing), error type hierarchy, project configuration, and test reliability.

Found one critical data-model error in the hardware acceleration profiles (contradictory hwaccel/encoder metadata), several dead-code variants in the error enum, a fragile test dependency on an external file, a hardcoded credential-like value in defaults, and minor quality issues.

## Critical Issues

### CR-01: HighPerformance profile has hwaccel_enabled=true but uses software-only encoder

**File:** `src/ffmpeg/hwaccel.rs:41-53`
**Issue:** The `HwAccelProfile::HighPerformance` entry sets `hwaccel_enabled: true` (claiming hardware acceleration is active) but uses `encoder: "libx264"`, which is a pure software encoder. The `WindowsNvidia` profile at line 75 correctly pairs `hwaccel_enabled: true` with `encoder: "h264_nvenc"` (a real hardware encoder). Any downstream code that inspects `hwaccel_enabled` to select encoding paths will incorrectly believe hardware acceleration is active for HighPerformance when software encoding is actually used. The profile data is self-contradictory.

**Fix:** Change the encoder to match the hardware acceleration claim, or set `hwaccel_enabled: false` to match the actual software encoder:

```rust
// Option A: Use a real hardware encoder (NVIDIA/AMD)
hwaccel_enabled: true,
encoder: "h264_nvenc",   // or "h264_amf" for AMD

// Option B: Be truthful about software encoding
hwaccel_enabled: false,
// keep encoder: "libx264"
```

## Warnings

### WR-01: Two FFmpegError variants are dead code

**File:** `src/error.rs:29,38`
**Issue:** Two variants of `FFmpegError` are defined but never constructed anywhere in the codebase:

- `FFmpegError::Timeout(String)` (line 29) -- no FFmpeg operation enforces a timeout.
- `FFmpegError::BinaryNotFound` (line 38) -- `detect_hw_encoders` (hwaccel.rs line 128) returns `Ok(Vec::new())` on launch failure rather than raising this variant.

Dead error variants mislead callers into writing unreachable error-handling branches and add unnecessary maintenance burden. The missing `Timeout` variant is especially concerning because `clip_video` in command.rs can hang indefinitely if FFmpeg stalls.

**Fix:** Remove both unused variants, or wire them into the appropriate code paths:

```rust
// In detect_hw_encoders (hwaccel.rs), raise BinaryNotFound:
Err(e) => {
    if e.kind() == std::io::ErrorKind::NotFound {
        return Err(FFmpegError::BinaryNotFound);
    }
    tracing::warn!("...");
    return Ok(Vec::new());
}

// In clip_video (command.rs), wrap with timeout:
// (requires tokio::time::timeout)
```

### WR-02: test_load_example_config depends on external config.example.toml with no guard

**File:** `tests/config_test.rs:11-12`
**Issue:** The integration test `test_load_example_config` loads `config.example.toml` from the project root via `env!("CARGO_MANIFEST_DIR")`. If the file is renamed, deleted, or absent (partial checkout), the test panics with an assertion failure that does not distinguish "file missing" from "parse error." This is a fragile external dependency for a test.

**Fix:** Add an existence check with an early return, or embed a test-specific TOML string:

```rust
let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
if !config_path.exists() {
    eprintln!("Skipping: config.example.toml not found");
    return;
}
```

### WR-03: test_hot_reload uses sleep-based synchronization that can be flaky

**File:** `tests/config_test.rs:149-194`
**Issue:** The hot-reload test waits 200ms (`std::thread::sleep(200)`) for the filesystem watcher to initialize before writing the update. On a loaded system or under CI, the watcher may not be ready within 200ms, causing the write to be missed. The subsequent 5-second polling loop compensates, but the initial sleep is a race condition. Additionally, the `TempDir` is dropped at test end while the watcher may still hold a file handle, causing platform-dependent cleanup failures.

**Fix:** Use a readiness signal (e.g., a `tokio::sync::oneshot` channel from the watcher) instead of a fixed sleep:

```rust
// Replace sleep(200) with:
let (tx, rx) = oneshot::channel();
// Pass tx into ConfigWatcher, which signals when watch is established
// Then await the signal instead of sleeping
```

### WR-04: Hardcoded credential-like value in SoulVoiceSection default

**File:** `src/config/defaults.rs:56-57`
**Issue:** The default `voice_uri` value (`speech:mcg3fdnx:clzkyf4vy00e5qr6hywum4u84:bzznlkuhcjzpbosexitr`) follows the pattern `speech:<appid>:<userid>:<token>` and appears to contain a real identifier rather than a trivial placeholder. If this default is ever activated in an unconfigured deployment, it could lead to unauthorized use of a shared TTS credential or data leakage.

**Fix:** Replace with an empty string or an obviously fake placeholder:

```rust
voice_uri: String::new(),
```

### WR-05: FfmpegEvent::Error silently swallowed during clip_video progress loop

**File:** `src/ffmpeg/command.rs:92-95`
**Issue:** When `FfmpegEvent::Error` events occur during the progress iteration loop, they are only logged via `tracing::error!` and processing continues unconditionally. If FFmpeg emits non-fatal errors mid-stream but eventually exits with code 0, the function returns `Ok(())` even though the output may be truncated or corrupt. Callers have no way to detect that FFmpeg reported errors.

**Fix:** Accumulate error flags and return an error if any were received:

```rust
let mut had_errors = false;
for event in iter {
    match event {
        FfmpegEvent::Progress(p) => { /* ... */ }
        FfmpegEvent::Error(e) => {
            tracing::error!("FFmpeg error: {}", e);
            had_errors = true;
        }
        _ => {}
    }
}
if had_errors {
    return Err(FFmpegError::ExecutionError(
        "FFmpeg reported errors during processing".into(),
    ));
}
```

## Info

### IN-01: `_watcher` field naming convention is misleading

**File:** `src/config/mod.rs:15`
**Issue:** The `_watcher` field uses a leading underscore, which Rust convention reserves for "intentionally unused" bindings. However, this field IS semantically important -- it owns the `ConfigWatcher`, and dropping it would stop the file-system listener, breaking hot-reload. The `_` prefix suppresses the dead-code warning and misleads readers.

**Fix:** Rename to `watcher`:

```rust
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    watcher: Option<ConfigWatcher>,
}
```

### IN-02: AppConfig::validate() is a no-op stub

**File:** `src/config/mod.rs:62-64`
**Issue:** The `validate` method unconditionally returns `Ok(())` with no actual validation. While documented as Phase 1 behavior, the method can be called by downstream code that believes it has verified the configuration, giving false assurance. Invalid states (negative timeouts, empty required fields) pass silently.

**Fix:** Add TODO markers and at minimum validate basic range constraints:

```rust
pub fn validate(&self) -> Result<(), ConfigError> {
    // TODO(Phase 2): validate API key presence for configured providers
    // TODO(Phase 2): validate timeout values > 0
    Ok(())
}
```

### IN-03: test_load_empty_toml discards all parsed values without assertions

**File:** `src/config/types.rs:311-317`
**Issue:** The test parses an empty TOML string with `toml::from_str("")` and discards every field via `let _ =`. This verifies only that parsing does not panic, but cannot catch regressions where default values silently change. The test in `config_test.rs` (line 84-106) does a more thorough comparison against `AppConfig::default()`.

**Fix:** Assert specific default values:

```rust
let config: AppConfig = toml::from_str("").expect("空 TOML 应成功解析");
assert_eq!(config.app.project_version, AppConfig::default().app.project_version);
assert_eq!(config.frames.frame_interval_input, 3);
```

### IN-04: .gitignore includes Python-project patterns irrelevant to the Rust crate

**File:** `.gitignore:5-14`
**Issue:** Multiple entries reference Python project paths (e.g., `/app/services/__pycache__`, `app/models/faster-whisper-large-v2/*`, `resource/scripts/*.json`) that do not exist in the `narratoai-core` Rust crate. While harmless, these clutter the file and could mask similarly-named Rust build artifacts. The entries were inherited from the shared project root with the Python codebase.

**Fix:** Group Python-specific entries under a comment section for clarity, or remove them if the Python project is fully migrated:

```gitignore
# Python project (legacy)
/app/services/__pycache__
/app/__pycache__/
# ...
```

---

_Reviewed: 2026-04-28T14:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
