---
phase: 01-foundation
fixed: 2026-04-28T20:10:00Z
fix_scope: critical_warning
findings_in_scope: 6
fixed: 6
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-04-28T20:10:00Z
**Source review:** .planning/phases/01-foundation/01-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: ProgressCallback 传入时间位置而非进度分数

**Files modified:** `src/ffmpeg/command.rs`
**Commit:** e0d7d8e

**Applied fix:** Updated `ProgressCallback` doc comment from "进度百分比" (percentage) to "0.0 到 1.0 的进度分数" (progress fraction 0.0-1.0). In `clip_video`, changed the progress callback invocation to normalize the parsed seconds by the clip duration: `let fraction = secs.map(|s| if duration > 0.0 { s / duration } else { 0.0 })` and passes `fraction` instead of raw seconds to the callback.

### WR-01: RwLock 中毒处理不一致

**Files modified:** `src/config/mod.rs`, `src/config/watcher.rs`
**Commit:** 2c83165

**Applied fix:** Changed both `ConfigManager::get()` (mod.rs:50) and `ConfigWatcher` reload callback (watcher.rs:26) to use `.unwrap_or_else(|e| e.into_inner())` for consistent poisoning recovery. Previously, `get()` used `.unwrap()` (panics on poison) while the watcher used `if let Ok` (silently ignores poison). Now both recover the inner value and continue.

### WR-02: 测试使用硬编码 Unix 路径

**Files modified:** `src/config/mod.rs`, `src/ffmpeg/command.rs`, `tests/config_test.rs`
**Commit:** ed0ae1e

**Applied fix:** Replaced hardcoded Unix paths (`/tmp/nonexistent_*`, `/nonexistent/config.toml`) with `std::env::temp_dir().join("narratoai_test_...")` in three test locations to be cross-platform compatible. Also fixed a `Path` vs `PathBuf` type mismatch that was exposed by this change.

### WR-03: spawn_blocking 测试时序断言可能不稳定

**Files modified:** `src/ffmpeg/command.rs`
**Commit:** b7fb8f4

**Applied fix:** Replaced timing-based concurrency assertion (`elapsed.as_millis() < 180`) with a `std::sync::Barrier`-based verification. Three-party barrier (2 tasks + main) proves concurrent execution: if `spawn_blocking` ran tasks serially, the first task would hold the blocking thread while blocked on the barrier, preventing the second task from ever starting, causing a deadlock. Concurrent execution allows both tasks to reach the barrier and proceed.

### WR-04: probe_video 在 duration 字段缺失或非字符串时静默返回 0.0

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** e4335d9

**Applied fix:** Changed the duration parsing chain from `.unwrap_or(0.0)` (silently returns 0.0 for missing/null/unparseable duration) to `.ok_or_else(|| FFmpegError::OutputParseError(...))?` which propagates the error to the caller. Three failure modes (missing field, non-string value, unparseable string) now all produce `FFmpegError::OutputParseError` instead of silent 0.0.

### WR-05: child.iter() 失败后 FFmpeg 子进程成为孤儿

**Files modified:** `src/ffmpeg/command.rs`
**Commit:** 2f1fb5c

**Applied fix:** Replaced `.map_err(...)?` on `child.iter()` with a `match` expression that kills and reaps the already-spawned FFmpeg child process (`child.kill()` + `child.wait()`) before returning `FFmpegError::SpawnFailed`. Prevents resource leaks from orphaned subprocesses when the event iterator fails to initialize.

---

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build` | Passes (0 errors, 0 warnings) |
| `cargo test --lib` | 26/26 unit tests pass |
| `cargo test --test config_test` | 6/6 integration tests pass |
| `cargo test --test ffmpeg_test` | 3/5 pass, 2 skipped (FFmpeg not installed on test machine -- pre-existing) |

---

_Fixed: 2026-04-28T20:10:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
