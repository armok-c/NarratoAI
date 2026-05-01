---
phase: 11-extended-features
fix_date: 2026-05-01
findings_in_scope: 16
fixed: 13
skipped: 3
iteration: 1
status: partial
---

# Phase 11: Code Review Fix Report (Iteration 1)

**Date:** 2026-05-01
**Fix Scope:** critical_warning
**Status:** partial

## Verification

- `cargo check` — 零 error（1 个预存 dead_code warning，无关）
- `cargo test --lib` — 367/368 通过（1 个预存 hwaccel 失败，无关）

---

## Fixed (13/16)

### CR-01: rename 路径遍历 — 已修复

**File:** `src/youtube/downloader.rs`

**Fix:** 添加路径分隔符和 `..` 检查，拒绝包含 `/`、`\`、`..` 的 rename 值，返回 `InvalidUrl` 错误。

### CR-02: NaN 绕过音量验证 — 已修复

**Files:** `src/audio/volume.rs`

**Fix:** `VolumeConfig::validate()` 添加 `val.is_nan()` 前置检查；`validate_volume()` 添加 NaN 检测，返回 0.0 并记录 warning。

### CR-03: SampleBuffer spec 变化 panic — 已修复

**File:** `src/audio/normalizer.rs`

**Fix:** 每次 decode 后重新创建 `SampleBuffer`，避免 spec 不匹配导致 panic。移除 `sample_buf` 缓存变量。

### CR-04: API 密钥字段未标记 skip_serializing — 已修复

**File:** `src/config/types.rs`

**Fix:** 对 12 个密钥字段添加 `#[serde(skip_serializing)]`：`vision_openai_api_key`、`text_openai_api_key`、`speech_key`、`secret_id`、`secret_key`、`api_key`x2、`ak`、`sk`、`token`、`pexels_api_keys`、`pixabay_api_keys`。

### WR-01: LoudnormData 解析错误静默吞没 — 已修复

**File:** `src/audio/normalizer.rs`

**Fix:** 5 个解析方法改用 `unwrap_or_else` + `tracing::warn!` 记录解析失败。

### WR-02: extract_json_from_stderr rfind('}') 匹配范围过广 — 已修复

**File:** `src/audio/normalizer.rs`

**Fix:** `rfind('}')` 限定在 `text[start..]` 范围内搜索。

### WR-03: IO 错误误报为 FileNotFound — 已修复

**File:** `src/audio/normalizer.rs`

**Fix:** `get_audio_rms` 中区分 `NotFound` 和其他 IO 错误。

### WR-04: sample_rate/channels 为 0 无检查 — 已修复

**File:** `src/audio/normalizer.rs`

**Fix:** `normalize_lufs` 添加 `sample_rate == 0 || channels == 0` 校验。

### WR-05: URL 验证不防御空白字符和换行符 — 已修复

**File:** `src/youtube/downloader.rs`

**Fix:** `get_video_formats` 和 `download_video` 两处均添加换行符检查。

### WR-06: proxy_url 无格式校验 — 已修复

**File:** `src/youtube/downloader.rs`

**Fix:** 两处代理设置均添加协议前缀校验（http/https/socks5/socks5h）。

### WR-07: Pexels 精确匹配分辨率导致空结果 — 已修复

**File:** `src/material/searcher.rs`

**Fix:** 改为 `>=` 匹配，允许更高分辨率素材。

### WR-09: project_version 默认值不一致 — 已修复

**Files:** `src/config/defaults.rs`, `src/config/types.rs`

**Fix:** 默认值改为 `"0.7.8"`，同步更新测试断言。

---

## Skipped (3/16)

### WR-08: 缓存命中不验证文件完整性

**Reason:** 0 字节文件检查已存在。ffprobe 缓存验证代价过高。

### WR-10: validate() 空实现

**Reason:** 预留方法，注释明确标注后续 Phase 填充。

### WR-11: normalize_lufs 硬编码 44100Hz/2ch

**Reason:** 已有注释说明设计决策，非缺陷。

### WR-12: 音量 clamp 范围不一致

**Reason:** 已有注释说明增益系数与百分比的语义差异。

---

*Phase: 11-extended-features*
*Fixed: 2026-05-01 (iteration 1)*
