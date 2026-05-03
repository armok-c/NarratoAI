---
status: all_fixed
phase: 06-documentary-pipeline
review_date: 2026-05-03
fix_date: 2026-05-04
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 5
fixer: gsd-code-fixer
previous_fix: 2026-05-03 (Re-Fix #4 — WR-27/WR-28 fixed)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #5)

## Summary

第五次修复：修复 Re-Review #4 发现的 2 个 WARNING 级别问题（WR-29、WR-30），全部通过 `cargo check` 和测试验证。

## Fixes Applied

### WR-29: `validate()` 未校验 `voice_pitch` 范围 — **已修复**
- **提交**: `fe38314` — `fix(06): add voice_pitch range validation in validate() (WR-29)`
- **变更**: `src/documentary/types.rs:58-60` — 在 `voice_rate` 校验之后添加 `voice_pitch` 范围检查 `[-10.0, 10.0]`

### WR-30: 集成测试进度值与实现不同步 — **已修复**
- **提交**: `736c7d0` — `fix(06): sync MergeAudio progress value to 70.0 in test (WR-30)`
- **变更**: `tests/documentary_integration_test.rs:135` — `MergeAudio` 进度值从 `60.0` 改为 `70.0`，与 `pipeline.rs:174` 实现一致

## Verification

| 验收条件 | 结果 |
|----------|------|
| `voice_pitch` 校验逻辑存在 | ✅ types.rs:58-60 |
| 校验范围 `[-10.0, 10.0]` 与其他参数风格一致 | ✅ |
| 测试进度值 `70.0` 与 `pipeline.rs:174` 一致 | ✅ |
| `cargo check` 编译通过 | ✅ 零 error |
| `test_documentary_request_default` 通过 | ✅ |
| `test_progress_callback_receives_all_steps` 通过 | ✅ |
| `test_pipeline_error_variants` 通过 | ✅ |

## Out of Scope (INFO, 未修复)

| ID | 问题 | 状态 |
|----|------|------|
| IN-02 | `parse_timestamp_range` 使用 `split('-')` | 遗留，影响极低 |
| IN-05 | `merge_srt_files(&[])` 返回 `Ok("")` | 遗留，调用方已确保非空 |
| IN-27 | 关键帧缓存回退到当前工作目录 | 遗留 |
| IN-28 | 忽略的 FFmpeg 测试体为空 | 遗留 |
| IN-29 | 测试 WordBoundary 数值使用非标准分组 | 遗留 |
| IN-31 | 四个字幕/宽高字段未使用 | 设计决策，需后续版本规划 |

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-27/WR-28 | Re-Fix #4 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-18~WR-26 | Re-Fix #3 全部 9 项 | ✅ 已修复 | 保持不变 |
