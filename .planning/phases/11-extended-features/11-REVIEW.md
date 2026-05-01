---
phase: 11-extended-features
reviewed: 2026-05-01T00:00:00Z
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
  - src/material/mod.rs
  - src/material/searcher.rs
  - src/material/downloader.rs
  - src/youtube/mod.rs
  - src/youtube/downloader.rs
  - tests/audio_integration.rs
  - tests/material_integration.rs
  - tests/youtube_integration.rs
findings:
  critical: 1
  warning: 5
  info: 6
  total: 12
status: issues_found
---

# Phase 11: Code Review Report (Re-review after Gap Closure)

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

本次复审验证了前一轮审查的 4 个 gap closure 修复（CR-01 RMS 公式、WR-01 URL 编码、WR-05/06/07 YouTube 修复、WR-10 死代码）均正确完成。发现 1 个新的 BLOCKER：`deny_unknown_fields` 配置结构体与 `config.example.toml` 字段不匹配，导致用户按示例配置启动时会解析失败。另有 5 个 WARNING 和 6 个 INFO 级别问题。

## Previously Fixed Issues — Verified

| Issue | Status | Notes |
|-------|--------|-------|
| CR-01: RMS 参考电平公式 | FIXED | `20.0 * rms.log10()` 正确，旧公式已移除，test_get_audio_rms_formula 验证 |
| WR-01: URL 编码 | FIXED | 自定义 urlencoding() 覆盖 CJK/特殊字符，两处搜索均应用 |
| WR-05: 分辨率标准化去重 | FIXED | normalize_resolution() 辅助函数存在，无重复逻辑 |
| WR-06: 复用 dump-json title | FIXED | _get_video_formats() 返回 title，无冗余 --get-title 子进程 |
| WR-07: 下载后文件存在性验证 | FIXED | download_video() 返回前检查 output_path.exists() |
| WR-10: 死代码清理 | FIXED | find_ytdlp() 中不可达分支已移除 |

---

## Critical Issues

### CR-02: `deny_unknown_fields` 导致 `config.example.toml` 无法解析

**Files:** `src/config/types.rs:70-89,115-123,163-177`, `config.example.toml:87,123-124,151-152`

**Description:** 三个配置结构体使用 `#[serde(deny_unknown_fields)]`，但 `config.example.toml` 包含 Rust 结构体中不存在的字段：

| config.example.toml 字段 | Rust 结构体 | 存在？ |
|---|---|---|
| `[soulvoice] voice_uri` | `SoulVoiceSection` | **缺失** |
| `[doubaotts] ak` | `DoubaoTTSSection` | **缺失** |
| `[doubaotts] sk` | `DoubaoTTSSection` | **缺失** |
| `[ui] doubaotts_voice_type` | `UiSection` | **缺失** |
| `[ui] doubaotts_rate` | `UiSection` | **缺失** |

用户按文档将 `config.example.toml` 复制为 `config.toml` 时，`ConfigManager::load()` 将因 `deny_unknown_fields` 拒绝未知字段而报 TOML 解析错误。

**Impact:** 生产阻塞 — 应用无法加载配置。

**Recommended fix:**
```rust
// SoulVoiceSection 增加:
#[serde(default)]
pub voice_uri: String,

// DoubaoTTSSection 增加:
#[serde(default)]
pub ak: String,
#[serde(default)]
pub sk: String,

// UiSection 增加:
#[serde(default)]
pub doubaotts_voice_type: String,
#[serde(default)]
pub doubaotts_rate: f64,
```

并在 `src/config/defaults.rs` 中添加对应默认值。

> **Note:** 此问题在 gap closure 阶段被标记为 "Phase 12 引入，非 Phase 11 范围"。本次复审重新标记为 CR，因为它直接影响 Phase 11 模块的可运行性。

---

## Warnings

### WR-11: Pixabay API key 暴露在 URL 查询字符串中

**File:** `src/material/searcher.rs:278`

```rust
let api_url = format!(
    "https://pixabay.com/api/videos/?q={}&video_type=all&per_page=50&key={}",
    urlencoding(search_term),
    api_key,
);
```

**Description:** Pixabay API 要求 key 放在 URL 中（这是其 API 设计），但 key 可能出现在代理日志、进程列表等位置。Pexels API 正确使用了 `Authorization` header。

**Impact:** 低-中。这是 Pixabay API 的固有设计，但应确保 URL 不被日志记录。

**Recommended fix:** 确认无 URL 日志输出，添加注释说明此已知限制。

### WR-12: `ProcessingConfig` 和 `MixingConfig` 未实现 `Default` trait

**File:** `src/audio/volume.rs:55,86`

**Description:** 两个结构体实现了 `default()` 关联方法而非 `Default` trait，导致无法用于 `T: Default` 约束或 `#[serde(default)]` 场景。

**Recommended fix:**
```rust
impl Default for ProcessingConfig {
    fn default() -> Self {
        Self { enable_smart_volume: true, enable_audio_normalization: true, target_lufs: -20.0, max_peak: -1.0 }
    }
}
impl Default for MixingConfig {
    fn default() -> Self {
        Self { crossfade_duration: 0.5, bgm_fade_out: 2.0, dynamic_range_compression: false }
    }
}
```

### WR-13: `normalize_audio_for_mixing` loudnorm 失败时无日志直接回退

**File:** `src/audio/normalizer.rs:332-334`

**Description:** 两遍 loudnorm 标准化失败后，代码静默回退到 RMS 标准化。loudnorm 错误被完全丢弃，操作者无法知道使用了低质量回退。

**Recommended fix:**
```rust
match normalize_lufs(audio_path, &output_path, target_lufs, -1.0) {
    Ok(()) => return Ok(output_path),
    Err(e) => tracing::warn!("loudnorm 两遍标准化失败，回退到 RMS 标准化: {}", e),
}
```

### WR-14: `download_videos` 使用 API 元数据 duration 而非实际下载文件 duration 累加

**File:** `src/material/downloader.rs:201`

```rust
total_duration += item.duration as f64;
```

**Description:** 累计时长使用搜索 API 返回的 `item.duration` 而非 ffprobe 探测的实际文件时长。API 元数据通常准确，但可能与实际文件时长不一致。

**Impact:** 低。与 Python 版行为一致，可能是有意设计。

### WR-15: `_get_video_formats` 下划线前缀命名误导

**File:** `src/youtube/downloader.rs:102`

**Description:** Rust 中 `_` 前缀通常表示未使用的变量或死代码，但 `_get_video_formats` 被 `download_video()` 调用，包含重要逻辑。命名降低了可发现性。

**Recommended fix:** 重命名为 `get_video_formats`（无下划线），按需设为 `pub(crate)`。

---

## Info

### IR-08: `DoubaoTTSSection` 字段名与 `config.example.toml` 不一致

**File:** `src/config/types.rs:163-177`, `config.example.toml:123-124`

Rust 结构体使用 `appid`/`token`，config 示例使用 `ak`/`sk`。与 CR-02 相关。

### IR-09: `SoulVoiceSection` 缺少 `voice_uri` 字段

**File:** `src/config/types.rs:115-123`, `config.example.toml:87`

与 CR-02 相关。

### IR-10: `UiSection` 缺少 `doubaotts_voice_type` 和 `doubaotts_rate` 字段

**File:** `src/config/types.rs:70-89`, `config.example.toml:151-152`

与 CR-02 相关。

### IR-11: `notify` crate 使用 RC 版本

**File:** `Cargo.toml:19`

```toml
notify = "9.0.0-rc.3"
```

RC 版本 API 不稳定，注释中已标注此风险。稳定版发布后应升级或锁定。

### IR-12: `VolumeConfig` 未实现 `Default` trait

**File:** `src/audio/volume.rs:18-23`

简单数据结构有明显的默认值（balanced profile 值），实现 `Default` 可提升易用性。

### IR-13: `MixingConfig::from_section` 硬编码 `dynamic_range_compression: false`

**File:** `src/audio/volume.rs:99`

`dynamic_range_compression` 不可从 `[audio]` 配置段配置。可能是有意推迟到 Phase 6。

---

## Cross-File Analysis

**接口一致性:** 错误类型 (`AudioError`, `MaterialError`, `YoutubeError`) 定义良好，中文 Display 消息一致，`thiserror` 使用规范。

**配置对齐:** 如 CR-02 所述，`config.example.toml` 与 Rust 配置结构体存在显著不匹配。

**死代码:** 审查文件中无死代码。所有 `pub` 项均被使用、测试或作为库 API。

**安全性:** 非测试生产代码中无 `unwrap()` 或 `expect()` 调用。所有 FFmpeg/yt-dlp 子进程使用 `.args()` 数组模式。YouTube 下载有 URL 验证。API key 未被日志输出。

---

*Phase: 11-extended-features*
*Re-reviewed: 2026-05-01 (post gap closure)*
