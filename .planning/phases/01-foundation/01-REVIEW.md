---
phase: 01-foundation
reviewed: 2026-04-28T13:15:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - Cargo.toml
  - Cargo.lock
  - .gitignore
  - src/lib.rs
  - src/error.rs
  - src/config/types.rs
  - src/config/defaults.rs
  - src/config/mod.rs
  - src/config/watcher.rs
  - src/ffmpeg/mod.rs
  - src/ffmpeg/command.rs
  - src/ffmpeg/probe.rs
  - src/ffmpeg/hwaccel.rs
  - tests/config_test.rs
  - tests/ffmpeg_test.rs
findings:
  critical: 1
  warning: 5
  info: 10
  total: 16
status: issues_found
---

# Phase 01: Foundation Code Review Report

**Reviewed:** 2026-04-28T13:15:00Z
**Depth:** standard
**Files Reviewed:** 15 (10 Rust source files, 2 config files, 1 lock file, 1 gitignore, 1 manifest)
**Status:** issues_found

## Summary

Rust library skeleton, config system, and FFmpeg operation layer review. The codebase is generally well-structured with idiomatic Rust patterns (thiserror, serde with `#[serde(default)]`, tokio spawn_blocking, OnceLock for lazy statics). However, **no previous findings have been addressed** -- all 10 issues from the prior review still apply. Additionally, 6 new issues were identified.

**Previous review findings re-verified:** CR-01 (ProgressCallback API contract violation), WR-01 (RwLock poisoning inconsistency), WR-02 (hardcoded Unix test paths), and WR-03 (time-sensitive test) all remain unfixed.

**New findings in this review:** WR-04 (ffprobe duration silently falls back to 0.0), WR-05 (orphaned FFmpeg process on iter() failure), IN-07 through IN-10 (precision risk in f64-to-string seek, fast-seek accuracy tradeoff, tokio "full" features, and missing output validation in integration tests).

Build verification: `cargo build` passes with zero errors. The 26 unit tests pass. Integration tests requiring FFmpeg binary behave correctly (fail with clear message when FFmpeg is absent).

---

## Critical Issues

### CR-01: ProgressCallback 传入时间位置而非进度分数

**File:** `src/ffmpeg/command.rs:78`
**Severity:** BLOCKER
**Status:** Unfixed since prior review

**Issue:** The `ProgressCallback` type documents its `Option<f64>` parameter as "进度百分比" (progress percentage), but `clip_video` passes the raw output of `parse_time_to_secs(&p.time)` -- which returns the current playback time position in seconds, not a normalized 0.0–1.0 fraction.

For a 10-second clip, the callback receives values like `Some(5.0)` (5-second mark) when the consumer expects `Some(0.5)` (50% complete). Phase 6 pipeline consumers (UI progress bars, ETA calculations) would display completely wrong values.

```rust
// Line 78 — bug: passes seconds instead of normalized progress
let secs = parse_time_to_secs(&p.time);
cb(secs, "视频裁剪中");
```

**Fix:** Normalize by the clip duration:

```rust
let secs = parse_time_to_secs(&p.time);
let fraction = secs.map(|s| if duration > 0.0 { s / duration } else { 0.0 });
cb(fraction, "视频裁剪中");
```

Also update the `ProgressCallback` doc comment to say "0.0 to 1.0 progress fraction" instead of "百分比".

**Risk:** No consumer exists in Phase 1, but the API contract is already broken. Fixing this in Phase 6 would require updating all consumers -- fix now while the API surface is small.

---

## Warnings

### WR-01: RwLock 中毒处理不一致

**File:** `src/config/mod.rs:50`
**Severity:** WARNING
**Status:** Unfixed since prior review

**Issue:** `ConfigManager::get()` uses `.read().unwrap()`, which panics if the RwLock is poisoned (another thread panicked while holding the write lock). However, `ConfigWatcher`'s reload callback (watcher.rs:26) uses `if let Ok(mut guard) = config.write()` which silently ignores poisoning. This inconsistency means:

- If the write lock is poisoned, the watcher silently stops updating config
- `get()` still panics on the poisoned read lock

```rust
// mod.rs:50 — panics on poison
pub fn get(&self) -> AppConfig {
    self.config.read().unwrap().clone()
}

// watcher.rs:26 — silently ignores poison
if let Ok(mut guard) = config.write() {
    *guard = new_config;
}
```

**Fix:** Either (a) use consistent poisoning strategy throughout -- either `.unwrap()` everywhere (panic on poison) or `if let Ok` everywhere (resilient to poison). Option (b) is to use `std::sync::PoisonError::into_inner()` to recover from poisoning by extracting the inner value.

### WR-02: 测试使用硬编码 Unix 路径

**Files:**
- `src/config/mod.rs:74`
- `src/ffmpeg/command.rs:127-128`
- `tests/config_test.rs:109`

**Severity:** WARNING
**Status:** Unfixed since prior review

**Issue:** Tests use hardcoded Unix-style absolute paths that have different semantics on Windows:

- `Path::new("/nonexistent/config.toml")` -- On Windows, resolves to `C:\nonexistent\config.toml` (current drive), not an obviously invalid path.
- `Path::new("/tmp/nonexistent_video_12345.mp4")` -- Same issue.

These tests functionally pass because these paths also don't exist on Windows, but the semantics are incorrect and the paths confuse developers reading the code on Windows.

**Fix:** Use `tempfile::TempDir` to create guaranteed-non-existent paths:

```rust
// Instead of Path::new("/nonexistent/config.toml"):
let dir = tempfile::TempDir::new().unwrap();
let nonexistent = dir.path().join("config.toml"); // guaranteed not to exist
```

Or use `std::env::temp_dir().join("uuid-nonExistentFile")`.

### WR-03: spawn_blocking 测试时序断言可能不稳定

**File:** `src/ffmpeg/command.rs:163`
**Severity:** WARNING
**Status:** Unfixed since prior review

**Issue:** `test_spawn_blocking_non_blocking` creates two tasks each sleeping 100ms and asserts total elapsed time < 180ms. On a loaded CI system or with a cold tokio blocking thread pool, OS scheduling delays can push this over 180ms, causing flaky failures.

```rust
assert!(
    elapsed.as_millis() < 180,   // tight — only 20ms margin for 200ms work
    ...
);
```

**Fix:** Widen the threshold to 250ms, or better, replace the timing-based assertion with a concurrency verification using `std::sync::Barrier`:

```rust
use std::sync::{Arc, Barrier};
let barrier = Arc::new(Barrier::new(3)); // 2 tasks + main
let b = barrier.clone();
let task1 = tokio::task::spawn_blocking(move || { b.wait(); 1 });
// etc.
// Assert that both tasks completed (proves they ran concurrently)
```

### WR-04: probe_video 在 duration 字段缺失或非字符串时静默返回 0.0

**File:** `src/ffmpeg/probe.rs:50-53`
**Severity:** WARNING
**Status:** New

**Issue:** The duration parsing chain uses `as_str()` then `parse::<f64>()` then `unwrap_or(0.0)`. Three failure modes all silently yield 0.0:

1. **Missing `duration` field** (e.g., live stream, corrupted file): `json["format"]["duration"]` = `Value::Null` -> `as_str()` returns `None` -> `0.0`
2. **Non-string duration** (some ffprobe versions output JSON numbers): `as_str()` returns `None` -> `0.0`
3. **Unparseable duration string** (e.g., `"N/A"`): `parse::<f64>()` returns `Err` -> `0.0`

```rust
let duration_secs = json["format"]["duration"]
    .as_str()
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(0.0);  // ALL failure modes silently become 0.0
```

A downstream consumer dividing duration into a time budget or calculating frame count would silently get incorrect results (no error, no warning, just `0.0`).

Note: Standard ffprobe `-print_format json` outputs `duration` as a **string** (e.g., `"120.500000"`), but this is an implementation detail of ffprobe's JSON serializer, not a guaranteed contract.

**Fix:** Validate the duration after parsing and return an error if it is missing or zero:

```rust
let duration_secs = json["format"]["duration"]
    .as_str()
    .and_then(|s| s.parse::<f64>().ok())
    .ok_or_else(|| FFmpegError::OutputParseError(
        "Missing or invalid duration in ffprobe output".into()
    ))?;
```

Or at minimum, emit a `tracing::warn!` when falling back to 0.0.

### WR-05: child.iter() 失败后 FFmpeg 子进程成为孤儿

**File:** `src/ffmpeg/command.rs:70-72`
**Severity:** WARNING
**Status:** New

**Issue:** In `clip_video`, `cmd.spawn()` starts the FFmpeg process, then `child.iter()` is called to get the event iterator. If `iter()` fails (e.g., pipe creation failure, unexpected FFmpeg output format), the function returns `Err` early -- but the already-spawned FFmpeg child process is **never killed** and becomes an orphan.

```rust
let mut child = cmd
    .spawn()
    .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

let iter = child                                            // child is alive here
    .iter()
    .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?; // if this fails, child leaks
```

While `iter()` failure is rare in practice (it occurs when the subprocess's stderr pipe cannot be set up), when it does happen, the orphaned FFmpeg process will consume system resources until killed externally.

**Fix:** Use `.spawn()` directly without `.iter()` and manage stderr reading manually, or wrap in a struct with a Drop impl that kills the child:

```rust
struct FfmpegGuard {
    child: Option<ffmpeg_sidecar::command::FfmpegCommand>,
}
impl Drop for FfmpegGuard {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
        }
    }
}
```

---

## Info

### IN-01: notify 依赖使用 RC 版本

**File:** `Cargo.toml:14`
**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** `notify = "9.0.0-rc.3"` is a release candidate. The API may change before stable release.

**Recommendation:** Track notify stable release and upgrade; document the RC usage rationale.

### IN-02: 配置结构体未使用 deny_unknown_fields

**Files:** `src/config/types.rs:4-164` (all struct definitions)
**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** No `#[serde(deny_unknown_fields)]` on any config struct. TOML typos (e.g., `vision_llm_proivder`) are silently ignored instead of producing an error.

**Recommendation:** Add `#[serde(deny_unknown_fields)]` once the config schema stabilizes to catch user typos.

### IN-03: IndexTTS2Section 的 repetition_penalty 默认值为 10.0

**File:** `src/config/defaults.rs:93`
**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** `repetition_penalty` defaults to `10.0`, which is far outside the typical range (1.0-2.0). This value would severely penalize any repetition, likely causing abnormal output. Propagated from `config.example.toml`.

**Recommendation:** Verify if 10.0 is intentional (perhaps a special IndexTTS2 semantics) or a config mistake. If incorrect, update both `defaults.rs` and `config.example.toml`.

### IN-04: test_hot_reload 包含 1700ms 的 sleep 等待

**File:** `tests/config_test.rs:167,179`
**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** The hot reload test sleeps 200ms + 1500ms = 1700ms total. This is slow for a unit test and still potentially flaky on platforms with delayed file notifications.

**Recommendation:** Consider polling with retries (e.g., check config every 100ms for up to 3 seconds) instead of fixed sleep, or add platform-specific timing notes.

### IN-05: NamedTempFile 先创建再覆写的模式

**Files:** `src/config/mod.rs:91-93`, `tests/config_test.rs:60-67,84-86`
**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** Tests use `NamedTempFile::new()` to create an empty file, then immediately overwrite it with `std::fs::write(file.path(), ...)`. On Windows, if the OS holds an exclusive lock on the temp file, the overwrite could fail.

**Recommendation:** Prefer `tempfile::TempDir` and create/overwrite files within it to avoid potential lock conflicts.

### IN-06: Clippy 建议（可 derive 的 Default、空行）

**Files:**
- `src/config/defaults.rs:3` -- 文档注释后的多余空行
- `src/config/defaults.rs:43` -- `AzureSection::Default` 可 derive（所有字段均为默认值）
- `src/config/defaults.rs:113` -- `ProxySection::Default` 可 derive（所有字段均为默认值）

**Severity:** INFO
**Status:** Unfixed since prior review

**Issue:** Three clippy warnings persist. `AzureSection` and `ProxySection` have manual `Default` impls where `#[derive(Default)]` would suffice.

**Recommendation:** Add `#[derive(Default)]` to `AzureSection` and `ProxySection` and remove manual impl blocks. Remove the blank line at defaults.rs:3.

### IN-07: f64-to-string 转换在 FFmpeg seek/duration 参数中的精度风险

**File:** `src/ffmpeg/command.rs:60,63`
**Severity:** INFO
**Status:** New

**Issue:** `start.to_string()` on `f64` values can produce very long decimal representations for computed values. For example, `(0.1f64 + 0.2f64).to_string()` yields `"0.30000000000000004"` in Rust. Passing such a string to FFmpeg could cause misinterpretation of the timestamp.

```rust
cmd.seek(start.to_string())       // potential: "0.30000000000000004"
    .input(&input_path)
    .duration(duration.to_string()) // same risk
```

FFmpeg is generally tolerant of extra decimal places, but the behavior is implementation-defined. Round-tripping computed floating-point values through `to_string()` is a known source of subtle bugs.

**Recommendation:** Use `format!("{:.3}", value)` to limit to millisecond precision (3 decimal places), which is the standard precision for video timestamps:

```rust
cmd.seek(format!("{:.3}", start))
    .input(&input_path)
    .duration(format!("{:.3}", duration))
```

### IN-08: clip_video 使用 fast seek（`-ss` 在 `-i` 之前）牺牲帧精度

**File:** `src/ffmpeg/command.rs:60-64`
**Severity:** INFO
**Status:** New

**Issue:** The FFmpeg command is built as `-ss <start> -i <input> ...`, which is "fast seek" (keyframe-based). This is faster but not frame-accurate -- the output may start at the nearest keyframe before the requested time, not at the exact frame.

For video clipping, frame-accurate (slow) seek is typically preferred, achieved by placing `-ss` after `-i`:
```
ffmpeg -i <input> -ss <start> ...
```

**Recommendation:** Document the tradeoff explicitly in the function doc comment. If frame accuracy is required for the use case, change the argument order or add an option:

```rust
// Frame-accurate seek (slow but precise)
cmd.input(&input_path)
    .seek(start.to_string())
    .duration(duration.to_string())
```

Note: The `ffmpeg-sidecar` crate's `.seek()` method adds `-ss` at the current position in the argument chain, so the relative ordering depends on when `.seek()` is called.

### IN-09: tokio 依赖启用 features = ["full"]，拉入所有特性

**File:** `Cargo.toml:7`
**Severity:** INFO
**Status:** New

**Issue:** `features = ["full"]` enables all tokio features, including experimental and rarely-used ones (`io-util`, `sync`, `rt-multi-thread`, `macros`, `fs`, `net`, `signal`, `process`, `tracing`, etc.). This increases compile time and binary size.

The current code only uses: `tokio::task::spawn_blocking`, `tokio::join!`, and `#[tokio::test]`. The minimal feature set is:

```toml
tokio = { version = "1.52.1", features = ["rt-multi-thread", "macros"] }
```

**Recommendation:** Replace `features = ["full"]` with the minimal feature set once all tokio usage patterns are known.

### IN-10: 集成测试未验证 clip_video 输出文件的时长/有效性

**File:** `tests/ffmpeg_test.rs:103-130`
**Severity:** INFO
**Status:** New

**Issue:** `test_clip_video_async` only checks that the output file exists and has size > 0 bytes. It does not verify:
- The output file is a valid video (probe fails?)
- The duration is ~1 second (as requested)
- The start time is correct
- The output has the same resolution as the input

```rust
assert!(output_path.exists());
let file_size = output_path.metadata().unwrap().len();
assert!(file_size > 0);  // a 1-byte corrupt file also passes
```

**Recommendation:** Add a probe check on the output file:

```rust
let output_info = probe_video(&output_path).expect("output should be probeable");
assert!(
    (output_info.duration_secs - 1.0).abs() < 0.1,
    "output duration should be ~1s, got {}",
    output_info.duration_secs
);
assert_eq!(output_info.width, 320);
assert_eq!(output_info.height, 240);
```

---

## Test Status

| Test Suite | Count | Result | Notes |
|-----------|-------|--------|-------|
| `cargo test --lib` (unit tests) | 26 | 26/26 pass | All unit tests pass |
| `cargo test --test config_test` | 6 | 6/6 pass | Config integration tests pass |
| `cargo test --test ffmpeg_test` | 5 | 3-5/5 pass* | FFmpeg-dependent tests require system FFmpeg |
| `cargo clippy` | 3 warnings | See IN-06 | All from derivable Default impls and blank line |

*FFmpeg-dependent tests (`test_probe_system_video`, `test_clip_video_async`) call `find_ffmpeg()` which panics with a clear install instruction if FFmpeg is not available. This is not a code defect -- it is correct behavior for tests requiring external dependencies.

---

## Cross-Cutting Concerns

### API Contract Issues
The `ProgressCallback` type doc says "progress percentage" but passes seconds (CR-01). The `parse_time_to_secs` function name is correct but its result is misused.

### Portability
Three test locations use Unix-style absolute paths (WR-02). The `create_test_video()` helper spawns `ffmpeg` via PATH lookup, which works cross-platform.

### Error Handling
- `probe_video` silently swallows missing/invalid duration (WR-04) -- should propagate as error
- `clip_video` orphans child process on iter() failure (WR-05) -- should kill child
- `ConfigManager::get()` uses `.unwrap()` on RwLock (WR-01) -- inconsistent with watcher

### Test Quality
- Time-sensitive assertion in `test_spawn_blocking_non_blocking` (WR-03) -- may flake in CI
- `test_clip_video_async` doesn't validate output content (IN-10)
- `test_hot_reload` is slow (1700ms) and potentially flaky across platforms (IN-04)

---

_Reviewed: 2026-04-28T13:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Build: cargo build passes (0 errors, 0 warnings)_
