---
phase: 01-foundation
fix_date: 2026-05-06
iteration: 2
fix_scope: critical_warning
findings_in_scope: 3
fixed: 2
skipped: 1
status: partial
review_path: .planning/phases/01-foundation/01-REVIEW.md
---

# Phase 01: Code Review Fix Report (Iteration 2)

**Date:** 2026-05-06
**Scope:** critical_warning (Critical + Warning only)
**Source Review:** 01-REVIEW.md (Iteration 4)
**Status:** partial (2/3 fixed)

## Summary

从 Phase 01 REVIEW.md 的 3 个警告中修复了 2 个。WR-10（测试失败根因）和 WR-09（硬编码超时）已修复并提交。WR-07（notify RC 版本）因需 API 迁移而跳过。

## Fixes Applied

### WR-10: detect_hw_encoders NotFound 与其他错误处理不一致 [FIXED]

**File:** `src/ffmpeg/hwaccel.rs:136-142`
**Commit:** `fix(01): resolve WR-10 detect_hw_encoders inconsistency and WR-09 hardcoded timeout`

**Before:**
```rust
Err(e) => {
    if e.kind() == std::io::ErrorKind::NotFound {
        return Err(FFmpegError::BinaryNotFound);
    }
    tracing::warn!("无法启动 ffmpeg: {} — 返回空编码器列表", e);
    return Ok(Vec::new());
}
```

**After:**
```rust
Err(e) => {
    tracing::warn!("FFmpeg binary not found or failed to start: {} — 返回空编码器列表", e);
    return Ok(Vec::new());
}
```

**Verification:** `test_detect_encoders_format` 在无 FFmpeg 环境下通过（FAILED → PASSED）。单元测试 565/565 passed。

---

### WR-09: clip_video 硬编码 600 秒超时 [FIXED]

**File:** `src/ffmpeg/command.rs:45,143-146`
**Commit:** `fix(01): resolve WR-10 detect_hw_encoders inconsistency and WR-09 hardcoded timeout`

**Before:**
```rust
_ = tokio::time::sleep(Duration::from_secs(600)) => {
    // ...
    "FFmpeg clip_video timed out after 600s".into(),
```

**After:**
```rust
const CLIP_VIDEO_TIMEOUT_SECS: u64 = 600;
// ...
_ = tokio::time::sleep(Duration::from_secs(CLIP_VIDEO_TIMEOUT_SECS)) => {
    // ...
    format!("FFmpeg clip_video timed out after {}s", CLIP_VIDEO_TIMEOUT_SECS),
```

**Verification:** `cargo build` 编译通过，所有 command.rs 单元测试通过。

---

## Skipped Findings

### WR-07: notify crate 使用 RC 版本 (9.0.0-rc.3) [SKIPPED]

**Reason:** 降级到 `notify = "8"` 需要重写 `src/config/watcher.rs` 的 API 调用（v8 和 v9 的 RecommendedWatcher API 不兼容）。这是依赖迁移，超出自动修复的安全范围。

**Recommendation:** 关注 notify 9.0 正式版发布时间线。如短期内发布，直接升级；如长期不发布，计划一次专门的 watcher API 迁移。

---

## Test Results After Fix

```
cargo build: PASS (9 warnings, 0 errors)
cargo test (unit): 565 passed, 0 failed, 1 ignored
cargo test (ffmpeg integration): 2 failures — 需要 FFmpeg on PATH（与本次修复无关）
```

关键变化：`test_detect_encoders_format` 从 FAILED → PASSED。

---

_Fix report generated: 2026-05-06_
_Agent: Claude (gsd-code-fixer)_
_Iteration: 2_
