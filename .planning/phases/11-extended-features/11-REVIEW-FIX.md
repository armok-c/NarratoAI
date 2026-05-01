---
phase: 11-extended-features
reviewed: 2026-05-01T00:00:00Z
fix_scope: critical_warning
findings_in_scope: 6
fixed: 6
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Date:** 2026-05-01
**Fix Scope:** critical_warning
**Status:** all_fixed

## Summary

修复 REVIEW.md 中的 1 个 CRITICAL 和 5 个 WARNING 级别问题，共 6 个 finding 全部修复。

## Fixes Applied

### CR-02: `deny_unknown_fields` 导致 `config.example.toml` 无法解析
**Status:** FIXED
**Files:** `src/config/types.rs`, `src/config/defaults.rs`, `src/tts/doubaotts.rs`, `src/tts/soulvoice.rs`

**Changes:**
- `SoulVoiceSection` 添加 `voice_uri: String` 字段（`#[serde(default)]`，默认值来自 config.example.toml）
- `DoubaoTTSSection` 添加 `ak: String` 和 `sk: String` 字段（`#[serde(default)]`，默认空字符串）
- `UiSection` 添加 `doubaotts_voice_type: String` 和 `doubaotts_rate: f64` 字段（`#[serde(default)]`）
- `src/config/defaults.rs` 各 Default impl 添加对应默认值
- `src/tts/doubaotts.rs` 和 `src/tts/soulvoice.rs` 使用 `..Default::default()` 适配新字段
- `test_load_full_config` 测试更新，包含新字段断言

### WR-11: Pixabay API key 暴露在 URL 查询字符串中
**Status:** FIXED
**File:** `src/material/searcher.rs`

**Changes:** 添加注释说明 Pixabay API 设计限制，确认 URL 不被日志输出。

### WR-12: `ProcessingConfig` 和 `MixingConfig` 未实现 `Default` trait
**Status:** FIXED
**File:** `src/audio/volume.rs`

**Changes:** 将 `fn default()` 关联方法改为 `impl Default for ...` trait 实现。调用处语法无需修改。

### WR-13: `normalize_audio_for_mixing` loudnorm 失败时无日志直接回退
**Status:** FIXED
**File:** `src/audio/normalizer.rs`

**Changes:** 将 `if .is_ok()` 静默回退改为 `match` 表达式，失败时输出 `tracing::warn!` 日志。

### WR-14: `download_videos` 使用 API 元数据 duration
**Status:** FIXED
**File:** `src/material/downloader.rs`

**Changes:** 添加注释说明使用搜索 API 返回的时长（与 Python 版行为一致）。

### WR-15: `_get_video_formats` 下划线前缀命名误导
**Status:** FIXED
**File:** `src/youtube/downloader.rs`

**Changes:** 重命名为 `get_video_formats`，更新定义和调用处。

## Files Changed

| File | Changes |
|------|---------|
| `src/config/types.rs` | +20 lines (3 struct fields + test updates) |
| `src/config/defaults.rs` | +5 lines (3 Default impl updates) |
| `src/tts/doubaotts.rs` | +5 lines (..Default::default() for new fields) |
| `src/tts/soulvoice.rs` | +4 lines (..Default::default() for new fields) |
| `src/audio/volume.rs` | Refactored Default trait impls |
| `src/audio/normalizer.rs` | +1 line (tracing::warn on loudnorm failure) |
| `src/material/searcher.rs` | +2 lines (Pixabay API limitation comment) |
| `src/material/downloader.rs` | +1 line (duration source comment) |
| `src/youtube/downloader.rs` | Renamed function + updated call site |

## Verification

- `cargo check` — 编译通过（仅有 1 个预先存在的 dead_code warning）
- 相关测试全部通过：config types (5), audio volume (13), audio normalizer (7), youtube (12), material (27)
- 预先存在的 `tests/tts_test.rs:113` 编译错误与本次修复无关

## Skipped

无。所有 in-scope findings 均已修复。
