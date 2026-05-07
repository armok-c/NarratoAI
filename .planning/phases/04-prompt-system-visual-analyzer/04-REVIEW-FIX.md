---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T16:00:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 2
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-05-07T16:00:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 7 (2 CRITICAL + 5 WARNING)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: 消除 run_ffmpeg_with_cancel 的忙等轮询

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 881673f
**Applied fix:** 将 `try_wait()` + `sleep(50ms)` 轮询循环替换为入口取消检查 + 阻塞 `child.wait()`，消除每秒 20 次的无效唤醒。取消检查移至调用点（帧间/回退级别间）。

### CR-02: 文档说明事件循环的取消延迟限制

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** f907352
**Applied fix:** 在 `extract_frames_fast_path` 的事件循环上方添加注释，说明 ffmpeg_sidecar `iter()` 的固有阻塞限制及替代方案。

### WR-01: 移除 json 过滤器的无效 fallback

**Files modified:** `src/prompt/template.rs`
**Commit:** 7e8e4b7
**Applied fix:** 移除 `serde_json::to_string` 的 fallback（仅转义双引号，不处理反斜杠/换行/控制字符），替换为 `expect()`，因为 `serde_json::to_string(&str)` 对 `&str` 是不可能失败的。

### WR-02: 测试使用 tempfile::tempdir() 替代硬编码路径

**Files modified:** `src/visual/frame_extractor.rs`, `src/visual/analyzer.rs`
**Commits:** 07d7de3 (frame_extractor), 600d1c3 (analyzer)
**Applied fix:** 将 8 个测试函数中的硬编码临时目录（`std::env::temp_dir().join("narratoai_test_*")`）替换为 `tempfile::tempdir()`，消除并行测试时的竞态条件。共修改 frame_extractor.rs 中 5 个测试 + analyzer.rs 中 3 个测试。

### WR-03: collect_frame_paths 使用数字排序

**Files modified:** `src/visual/analyzer.rs`
**Commit:** e8778da
**Applied fix:** 新增 `extract_frame_number_from_keyframe` 辅助函数，将 `paths.sort()`（字典排序）替换为 `paths.sort_by()` 基于帧序号的数字排序，避免超过 6 位数字时排序错误。

### WR-04: 添加 interval_seconds 上限校验

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** e0daae9
**Applied fix:** 在现有的 `<= 0` 和 `is_nan()` 检查后添加 `interval_seconds > 86400.0` 上限校验，拒绝过大的帧提取间隔（最大 86400 秒/1 天）。

### WR-05: 在 extract_single_frame 的回退级别间添加取消检查

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 1f96dc8
**Applied fix:** 在 `extract_single_frame` 的 4 级回退（Level 1-4）之间各添加 `cancel.is_cancelled()` 检查，确保取消时不会继续尝试后续回退级别。

## Verification

All fixes verified with:
1. Tier 1: Re-read modified file sections to confirm changes present and surrounding code intact
2. Tier 2: `cargo check --lib` passed after all fixes (0 new errors, 7 pre-existing warnings unrelated to changes)
3. `cargo test --lib`: 554 passed, 0 failed, 1 ignored (pre-existing)

---

_Fixed: 2026-05-07T16:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
