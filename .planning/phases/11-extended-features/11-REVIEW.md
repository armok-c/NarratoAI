---
phase: 11-extended-features
reviewed: 2026-05-01T17:30:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - Cargo.toml
  - config.example.toml
  - src/audio/mod.rs
  - src/audio/normalizer.rs
  - src/audio/volume.rs
  - src/config/defaults.rs
  - src/config/types.rs
  - src/lib.rs
  - src/material/downloader.rs
  - src/material/mod.rs
  - src/material/searcher.rs
  - src/youtube/downloader.rs
  - src/youtube/mod.rs
  - tests/audio_integration.rs
  - tests/material_integration.rs
  - tests/youtube_integration.rs
findings:
  critical: 0
  warning: 5
  info: 5
  total: 10
status: issues_found
---

# Phase 11: Code Review Report (Post-fix Verification)

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

前一轮审查的 6 个问题（CR-02, WR-11–15）全部验证修复完成。本轮未发现新的 Critical 问题，发现 5 个 WARNING 和 5 个 INFO 级别问题。主要关注点：音频标准化函数硬编码采样率/声道数（忽略配置字段）、音量 clamp 范围不一致、以及下载路径硬编码。

## Verification

- `cargo check` — 零 error（1 个预存 dead_code warning，无关）
- `cargo test --lib` — 367/368 通过（1 个预存 hwaccel 失败，无关）
- `config.example.toml` ↔ Rust 结构体完全对齐（test_load_full_config 通过）

## Previously Fixed Issues — Verified

| Issue | Status | Verification |
|-------|--------|-------------|
| CR-02: `deny_unknown_fields` config 不匹配 | FIXED | `SoulVoiceSection` 增加 `voice_uri`，`DoubaoTTSSection` 增加 `ak`/`sk`，`UiSection` 增加 `doubaotts_voice_type`/`doubaotts_rate`；`test_load_full_config` 全字段断言通过 |
| WR-11: Pixabay API key 在 URL 中 | FIXED | 行 277-278 添加注释说明 Pixabay API 设计限制，确认无 URL 日志输出 |
| WR-12: ProcessingConfig/MixingConfig 未实现 Default | FIXED | `impl Default for ProcessingConfig`（volume.rs:53-62）和 `impl Default for MixingConfig`（volume.rs:85-93）已存在 |
| WR-13: loudnorm 失败时无日志 | FIXED | normalizer.rs:334 添加 `tracing::warn!("loudnorm 两遍标准化失败，回退到 RMS 标准化: {}", e)` |
| WR-14: download_videos 使用 API 元数据 duration | FIXED | downloader.rs:201 添加注释说明与 Python 版行为一致 |
| WR-15: `_get_video_formats` 下划线前缀误导 | FIXED | 重命名为 `get_video_formats`（downloader.rs:102） |

---

## Warnings

### WR-16: `normalize_lufs` / `normalize_audio_for_mixing` 硬编码 44100Hz / 双声道，忽略配置

**Files:** `src/audio/normalizer.rs:194,345-346`

```rust
// normalize_lufs (line 194)
.args(["-ar", "44100", "-ac", "2"])

// normalize_audio_for_mixing (line 345-346, RMS 回退路径)
.args(["-ar", "44100", "-ac", "2"])
```

**Description:** `AudioSection` 配置定义了 `sample_rate`（默认 44100）和 `channels`（默认 2）两个字段，用户可在 `config.toml` 中调整。但 `normalize_lufs()` 和 `normalize_audio_for_mixing()` 两个函数将输出格式硬编码为 44100Hz / 双声道，配置字段无实际效果。

**Impact:** 用户修改 `[audio]` 段的 `sample_rate` 或 `channels` 无效，产生错误预期。

**Recommended fix:** 将采样率和声道数作为函数参数，或提供从 `AudioSection` 构建参数的封装函数。

### WR-17: 音量 clamp 范围不一致

**Files:** `src/audio/normalizer.rs:307`, `src/audio/volume.rs:205`

```rust
// normalizer.rs:307 — 原声调整系数允许到 3.0
let orig_adj = original_adjustment.clamp(0.1, 3.0);

// volume.rs:205 — 音量验证上限为 2.0
let max = 2.0;
```

**Description:** `calculate_volume_adjustment()` 将原声音量调整系数 clamp 到 `[0.1, 3.0]`，但 `validate_volume()` 限制为 `[0.0, 2.0]`。两个函数处理同一概念（音量增益）但上限不同。当增益系数超过 2.0 时，normalizer 允许但 volume 验证器会拒绝。

**Impact:** 中。不同模块的音量限制不一致可能导致混淆。

**Recommended fix:** 统一 clamp 范围，或在 `calculate_volume_adjustment` 的文档注释中说明使用 `[0.1, 3.0]` 的原因（它计算的是增益系数而非音量百分比）。

### WR-18: `download_video` 分辨率回退忽略目标分辨率

**File:** `src/youtube/downloader.rs:216-227`

```rust
let fallback = formats
    .iter()
    .find(|f| f.vcodec != "none")  // 任何有视频的格式
    .map(|f| f.format_id.clone())
```

**Description:** 当目标分辨率无匹配格式时，fallback 选取任何有视频轨道的格式，可能是 144p 或 360p。此时用户请求 1080p 但实际下载最低分辨率视频，无任何提示。

**Impact:** 低-中。与 Python 版行为可能一致，但静默降级可能导致输出质量不符合预期。

**Recommended fix:** 在 fallback 时添加 `tracing::warn!` 记录实际选用的分辨率。

### WR-19: `download_videos` 使用硬编码相对路径

**File:** `src/material/downloader.rs:148`

```rust
let save_dir = PathBuf::from("storage").join("temp").join(task_id);
```

**Description:** 下载目录固定为 `storage/temp/{task_id}`（相对于进程 CWD）。无法通过配置指定基础路径，且不同 CWD 的调用会产生不同的下载位置。

**Impact:** 低。与 Python 版行为一致（Python 使用 `os.path.join`），但在库模式下调用者无法控制存储路径。

### WR-20: `extract_json_from_stderr` 使用脆弱的首尾花括号匹配

**File:** `src/audio/normalizer.rs:109-115`

```rust
let start = text.find('{').ok_or_else(|| ...)?;
let end = text.rfind('}').ok_or_else(|| ...)?;
```

**Description:** 通过查找第一个 `{` 和最后一个 `}` 来提取 JSON 块。如果 FFmpeg stderr 在 JSON 之前或之后包含其他含花括号的文本（如错误消息），将匹配到错误的边界，导致 JSON 解析失败。

**Impact:** 低。实际 FFmpeg loudnorm 输出格式稳定（JSON 在末尾，前缀无花括号），但在非标准 FFmpeg 版本或异常输出时可能触发。

**Recommended fix:** 使用更精确的匹配（如查找 `[Parsed_loudnorm_` 后的 `{`），或在现有逻辑外添加 JSON 解析错误时的 fallback 提示。

---

## Info

### IR-14: `notify` crate 使用 RC 版本（预存）

**File:** `Cargo.toml:19`

```toml
notify = "9.0.0-rc.3"
```

RC 版本 API 不稳定。稳定版发布后应升级。

### IR-15: `VolumeConfig` 未实现 `Default` trait

**File:** `src/audio/volume.rs:18-23`

简单数据结构有明显的默认值（balanced profile 值），实现 `Default` 可提升易用性。

### IR-16: `MixingConfig::from_section` 硬编码 `dynamic_range_compression: false`

**File:** `src/audio/volume.rs:101`

`dynamic_range_compression` 不可从 `[audio]` 配置段配置。可能是有意推迟到 Phase 6 FFmpeg 实现阶段。

### IR-17: `download_videos` 返回路径列表但无下载元数据

**File:** `src/material/downloader.rs:191-212`

函数返回 `Vec<PathBuf>` 但不返回每个文件对应的搜索词、时长或来源信息。调用方无法区分哪些文件来自哪个搜索词。

### IR-18: `save_video` 临时文件路径无并发保护

**File:** `src/material/downloader.rs:86`

临时文件路径格式为 `{cache_key}.tmp`，并发调用相同 URL 时会冲突。虽然上游 `download_videos` 有 URL 去重，但 `save_video` 作为 `pub` 函数可能被直接调用。

---

## Cross-File Analysis

**配置对齐:** `config.example.toml` 与 `AppConfig` 所有 section 字段完全匹配。`test_load_full_config` 覆盖全部 section 和关键字段。

**错误类型:** `AudioError`、`MaterialError`、`YoutubeError` 定义规范，中文 Display 消息一致，thiserror 使用正确。

**安全性:** 非 test 生产代码无 `unwrap()`/`expect()` 调用。所有 FFmpeg/yt-dlp 子进程使用 `.args()` 数组模式。YouTube 下载有 URL 前缀验证。API key 未被日志输出。

**死代码:** 审查文件中无死代码。所有 `pub` 项均被使用、测试或作为库 API。

**模块结构:** `audio/`、`youtube/`、`material/` 三个模块结构清晰，`mod.rs` 仅做 re-export。`lib.rs` 导出正确。

---

*Phase: 11-extended-features*
*Reviewed: 2026-05-01 (post-fix verification)*
