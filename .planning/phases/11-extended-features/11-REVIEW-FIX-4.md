---
phase: 11-extended-features
iteration: 4
fix_scope: critical_warning
source_review: 11-REVIEW.md
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 11 Code Review Fix Report (Iteration 4)

**Fixer:** gsd-code-fixer
**Date:** 2026-05-01
**Source:** 11-REVIEW.md (Iteration 9 — Fresh Review)
**Commit:** `9d8a744`

---

## Summary

修复了 Phase 11 第 9 轮审查中的 2 个 Warning 级别问题。无 Critical。Info 级别不在修复范围内。

---

## Fixes Applied

### WR-08: `get_video_formats` 添加 120s 超时

**文件:** `src/youtube/downloader.rs:130-134`
**严重度:** Warning → Fixed

将 `cmd.output().await` 替换为 `tokio::time::timeout(Duration::from_secs(120), cmd.output())`，与 `download_video` 中已有的 600s 超时保持一致的防御策略。超时返回 `YoutubeError::SubprocessError("元数据查询超时（120 秒）")`。

---

### WR-09: `download_videos` 添加 `task_id` 路径遍历校验

**文件:** `src/material/downloader.rs:145-149`
**严重度:** Warning → Fixed

在 `download_videos` 入口处添加校验，拒绝包含 `/`、`\` 或 `..` 的 `task_id` 值。与 YouTube 模块中 `rename` 参数的路径遍历校验策略保持一致。

---

## Skipped (Not in Scope)

| ID | 严重度 | 原因 |
|----|--------|------|
| INFO-01 | Info | 不在 critical_warning 修复范围 |
| INFO-02 | Info | 不在 critical_warning 修复范围 |
| INFO-03 | Info | 不在 critical_warning 修复范围 |
| INFO-04 | Info | 不在 critical_warning 修复范围 |
| INFO-05 | Info | 不在 critical_warning 修复范围 |
| INFO-06 | Info | 不在 critical_warning 修复范围 |
| INFO-07 | Info | 不在 critical_warning 修复范围 |

---

## Verification

- `cargo check` 通过（仅一个无关 dead_code 警告）
- 367 lib 测试通过，唯一失败项 `test_detect_encoders_format` 为既有的 hwaccel 测试，与本修复无关
- YouTube downloader 和 material downloader 模块测试全部通过
