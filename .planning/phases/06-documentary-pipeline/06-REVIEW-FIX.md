---
status: all_fixed
phase: 06-documentary-pipeline
review_date: 2026-05-04
fix_date: 2026-05-05
findings_in_scope: 8
fixed: 8
skipped: 0
iteration: 8
fixer: gsd-code-fixer
previous_fix: 2026-05-04 (Re-Fix #7 — WR-32 fixed)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #8, --all)

## Summary

第八次修复：基于 `--all` 标志，将 INFO 发现纳入修复范围。修复 Re-Review #6 发现的 4 项可安全修复的 INFO 问题（IN-02、IN-05、IN-29、IN-32），跳过 3 项设计决策/功能补全类 INFO（IN-27、IN-28、IN-31）。通过 `cargo check` 和全部测试验证。

## Fixes Applied

### IN-02: `parse_timestamp_range` 使用 `splitn` 限制分割次数 — **已修复**
- **提交**: `0d9c8a1` — `fix(06): address 4 INFO findings`
- **变更**: `src/documentary/timestamp.rs:7` — `split('-')` → `splitn(2, '-')`，避免时间戳值含内部短横线时意外分割

### IN-05: `merge_srt_files` 空输入保护 — **已修复**
- **提交**: `0d9c8a1` — `fix(06): address 4 INFO findings`
- **变更**: `src/documentary/subtitle.rs:44-46` — 添加空切片早期错误返回 `Err(SrtGeneration)`，不再静默返回 `Ok("")`

### IN-29: 测试 WordBoundary 数值使用标准分组 — **已修复**
- **提交**: `0d9c8a1` — `fix(06): address 4 INFO findings`
- **变更**: `tests/documentary_integration_test.rs:50-72` — `5_000_000_0` → `50_000_000` 等共 9 处数值重分组

### IN-32: `parse_script_clips` 移除不可达死代码 — **已修复**
- **提交**: `0d9c8a1` — `fix(06): address 4 INFO findings`
- **变更**: `src/documentary/script_gen.rs:267` — 移除 `.or_else(|| parsed.as_array())` 不可达分支，简化为直接 `.ok_or_else()`

## Skipped (设计决策/功能补全)

| ID | 问题 | 跳过原因 |
|----|------|----------|
| IN-27 | 关键帧缓存回退到当前工作目录 | 设计决策，变更缓存策略可能影响其他行为 |
| IN-28 | 忽略的 FFmpeg 测试体为空 | 功能补全，非代码修复范畴 |
| IN-31 | 四个字幕/宽高字段未使用 | 设计决策，需后续版本规划使用方式 |

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error（1 warning，`get_azure_voices` 不在审查范围） |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| IN-02 修复: `splitn(2, '-')` | ✅ timestamp.rs:7 |
| IN-05 修复: 空输入错误返回 | ✅ subtitle.rs:44-46 |
| IN-29 修复: 标准数值分组 | ✅ documentary_integration_test.rs:50-72 |
| IN-32 修复: 移除不可达代码 | ✅ script_gen.rs:267 |

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-32 | Re-Fix #7 — `extract_json_object` 字节/字符索引混用 | ✅ 已修复 | 保持不变 |
| WR-31 | Re-Fix #6 — `parse_batch_response` 字符串切片 panic | ✅ 已修复 | 保持不变 |
| WR-29/WR-30 | Re-Fix #5 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-27/WR-28 | Re-Fix #4 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-18~WR-26 | Re-Fix #3 全部 9 项 | ✅ 已修复 | 保持不变 |
