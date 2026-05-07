---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T14:30:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 19
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report (Iteration 19)

**Fixed at:** 2026-05-07T14:30:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 19

**Summary:**
- Fix scope: warning
- Findings in scope: 3 (3 WARNING)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: BatchResponse uses deny_unknown_fields, contradicting the LLM-tolerance strategy

**Files modified:** `src/visual/analyzer.rs`
**Commit:** 9f2a65f
**Applied fix:** 移除 `#[serde(deny_unknown_fields)]` 属性，添加与 `FrameObservation` 一致的设计决策注释（说明未知字段被静默忽略以兼容 LLM 额外字段）。新增测试 `test_parse_and_retry_batch_with_extra_fields` 验证带额外字段的 BatchResponse JSON 能正确解析。

### WR-02: analyze_video_frames hardcodes interval_seconds=3.0 and quality

**Files modified:** `src/visual/analyzer.rs`
**Commit:** b221b4b
**Applied fix:** 在 `analyze_video_frames` 函数签名中添加 `interval_seconds: Option<f64>` 和 `quality: Option<u32>` 可选参数（位于 `max_concurrency` 之后、`progress` 之前）。`extract_frames` 调用改为 `interval_seconds.unwrap_or(3.0)` 替代硬编码 `3.0`，`quality` 直接传入。更新了函数 doc comment 的参数列表说明。

### WR-03: template.rs render() two-pass regex validation is fragile

**Files modified:** `src/prompt/template.rs`
**Commit:** d753b0b
**Applied fix:** 在 `render` 函数 doc comment 的 `# 注意` 部分添加约束说明：第 1/2 遍正则和第 3 遍正则是两套独立验证路径，添加新语法（如链式过滤器）时必须同时更新两处正则，否则新语法的变量会跳过必需参数检查。

## Verification

All fixes verified with:
1. Tier 1: Re-read modified file sections to confirm changes present and surrounding code intact
2. Tier 2: `cargo check --lib` passed after each fix (0 new warnings, 7 pre-existing warnings unrelated to changes)
3. `cargo test --lib`: 554 passed, 0 failed, 1 ignored (pre-existing)

---
_Fixed: 2026-05-07T14:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 19_
