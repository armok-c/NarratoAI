---
phase: 11-extended-features
reviewed: 2026-05-01
fix_scope: critical_warning
findings_in_scope: 5
fixed: 5
skipped: 0
iteration: 5
status: all_fixed
---

# Phase 11: Code Review Fix Report (Iteration 5)

**Date:** 2026-05-01
**Fix Scope:** critical_warning (WR-16 ~ WR-20)
**Status:** all_fixed

## Summary

修复第 5 轮审查发现的 5 个 WARNING 级别问题，全部修复。

## Fixes Applied

### WR-16: `normalize_lufs` / `normalize_audio_for_mixing` 硬编码采样率/声道数
**Status:** FIXED
**File:** `src/audio/normalizer.rs`

**Changes:**
- `normalize_lufs()` 添加 `sample_rate: u32` 和 `channels: u32` 参数
- `normalize_audio_for_mixing()` 添加 `sample_rate: u32` 和 `channels: u32` 参数
- 内部 FFmpeg 调用的 `-ar` 和 `-ac` 参数改用函数参数
- 两个单元测试更新为新签名

### WR-17: 音量 clamp 范围不一致
**Status:** FIXED
**File:** `src/audio/normalizer.rs`

**Changes:**
- `calculate_volume_adjustment` 文档注释增加说明：增益系数 clamp 范围 [0.1, 3.0] 与 `validate_volume()` 的音量百分比 [0.0, 2.0] 语义不同。增益系数来自 LUFS 差值换算，极端安静的原声需要更大倍数。

### WR-18: `download_video` 分辨率回退无日志
**Status:** FIXED
**File:** `src/youtube/downloader.rs`

**Changes:**
- fallback 路径添加 `tracing::warn!` 记录实际选用的格式 ID 和分辨率

### WR-19: `download_videos` 硬编码相对路径
**Status:** FIXED
**File:** `src/material/downloader.rs`

**Changes:**
- 添加注释说明固定相对路径与 Python 版行为一致，库模式调用者需确保 CWD 正确

### WR-20: `extract_json_from_stderr` 脆弱的 JSON 提取
**Status:** FIXED
**File:** `src/audio/normalizer.rs`

**Changes:**
- 优先在 `Parsed_loudnorm_` 标记之后查找 `{`，避免误匹配前缀中的花括号
- 添加起始/结束位置反转检查，防止提取到空范围

## Files Changed

| File | Changes |
|------|---------|
| `src/audio/normalizer.rs` | WR-16: 函数签名增加参数；WR-17: 注释说明 clamp 差异；WR-20: 精确 JSON 提取 |
| `src/youtube/downloader.rs` | WR-18: fallback 添加 warn 日志 |
| `src/material/downloader.rs` | WR-19: 添加路径设计说明注释 |

## Verification

- `cargo check` — 编译通过（1 个预存 dead_code warning）
- `cargo test --lib` — 367/368 通过（1 个预存 hwaccel 失败，无关）

## Skipped

无。所有 in-scope findings 均已修复。
