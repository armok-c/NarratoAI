---
status: all_fixed
phase: 06-documentary-pipeline
review_date: 2026-05-04
fix_date: 2026-05-04
findings_in_scope: 1
fixed: 1
skipped: 0
iteration: 7
fixer: gsd-code-fixer
previous_fix: 2026-05-04 (Re-Fix #6 — WR-31 fixed)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #7)

## Summary

第七次修复：修复 Re-Review #6 发现的 1 个 WARNING 级别问题（WR-32），通过 `cargo check` 和测试验证。

## Fixes Applied

### WR-32: `extract_json_object` 字节/字符索引混用导致 panic — **已修复**
- **提交**: `10ceff4` — `fix(06): use char-based indexing in extract_json_object to avoid UTF-8 panic (WR-32)`
- **变更**: `src/documentary/script_gen.rs:342-376` — 将 `text.find('{')`（字节索引）改为 `chars.iter().position(|&c| c == '{')`（字符索引），将 `text[start..=i].to_string()`（字节切片）改为 `chars[start..=i].iter().collect()`（字符切片收集），消除多字节 UTF-8 内容导致的 panic 风险

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| WR-32 修复: 统一字符索引 | ✅ script_gen.rs:342-376 |

## Out of Scope (INFO, 未修复)

| ID | 问题 | 状态 |
|----|------|------|
| IN-02 | `parse_timestamp_range` 使用 `split('-')` | 遗留，影响极低 |
| IN-05 | `merge_srt_files(&[])` 返回 `Ok("")` | 遗留，调用方已确保非空 |
| IN-27 | 关键帧缓存回退到当前工作目录 | 遗留 |
| IN-28 | 忽略的 FFmpeg 测试体为空 | 遗留 |
| IN-29 | 测试 WordBoundary 数值使用非标准分组 | 遗留 |
| IN-31 | 四个字幕/宽高字段未使用 | 设计决策，需后续版本规划 |
| IN-32 | `parse_script_clips` 不可达死代码 | 遗留，影响极低 |

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-31 | Re-Fix #6 — `parse_batch_response` 字符串切片 panic | ✅ 已修复 | 保持不变 |
| WR-29/WR-30 | Re-Fix #5 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-27/WR-28 | Re-Fix #4 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-18~WR-26 | Re-Fix #3 全部 9 项 | ✅ 已修复 | 保持不变 |
