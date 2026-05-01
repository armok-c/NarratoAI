---
phase: 11-extended-features
reviewed: 2026-05-01
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
  critical: 4
  warning: 12
  info: 8
  total: 24
status: issues_found
---

# Phase 11: Code Review Report (Fresh Review, Iteration 6)

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

5 个审查子代理并行审查了全部 16 个文件。发现 4 个 Critical、12 个 Warning、8 个 Info 级别问题。主要关注点：YouTube 下载的 rename 路径遍历漏洞、音量验证的 NaN 绕过、symphonia 解码器 spec 变化时的 panic 风险、以及 API 密钥序列化泄露。

## Verification

- `cargo check` — 零 error（1 个预存 dead_code warning，无关）
- `cargo test --lib` — 367/368 通过（1 个预存 hwaccel 失败，无关）

---

## Critical

### CR-01: `rename` 参数路径遍历

**File:** `src/youtube/downloader.rs:239-255`

```rust
let output_template = match rename {
    Some(name) => name.to_string(),
    None => format!("{}_video", base_resolution),
};
// ...
"-o",
&format!("{}.{}", output_template, output_ext),
```

**Description:** `rename` 参数未经任何路径分隔符过滤，直接拼入 yt-dlp `-o` 输出模板。传入 `"../../../tmp/evil"` 可将视频写入 CWD 之外的任意位置。第 278 行的文件存在性检查会确认写入成功后正常返回。

**Recommended fix:** 拒绝包含 `/`、`\` 或 `..` 的 rename 值：

```rust
let output_template = match rename {
    Some(name) => {
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(YoutubeError::InvalidUrl("rename 参数包含非法路径字符".into()));
        }
        name.to_string()
    }
    None => format!("{}_video", base_resolution),
};
```

### CR-02: NaN 绕过音量验证

**Files:** `src/audio/volume.rs:33`, `src/audio/volume.rs:206-213`

```rust
// volume.rs:33 — VolumeConfig::validate()
if val < 0.0 || val > 2.0 {

// volume.rs:206-213 — validate_volume()
if volume < min {       // NaN < 0.0 → false
} else if volume > max { // NaN > 2.0 → false
} else {
    volume              // NaN 直接通过
}
```

**Description:** IEEE 754 中 `NaN < 0.0` 和 `NaN > 2.0` 均为 false，NaN 会绕过所有验证直接传入 FFmpeg filter 参数，导致未定义行为。

**Recommended fix:** 两处均添加 `is_nan()` 检查：

```rust
if val.is_nan() || val < 0.0 || val > 2.0 {
```

### CR-03: symphonia SampleBuffer spec 变化时 panic

**File:** `src/audio/normalizer.rs:287-288`

```rust
let buf = sample_buf.get_or_insert_with(|| SampleBuffer::<f32>::new(cap, spec));
buf.copy_interleaved_ref(audio_buf);
```

**Description:** `get_or_insert_with` 仅首次创建 `SampleBuffer`。后续 packet 若 capacity 或 spec 变化（多段音频、VBR 编码），`copy_interleaved_ref` 因 spec 不匹配会 panic。

**Recommended fix:** 每次 decode 后检查 spec 是否一致，不一致时重建 buffer：

```rust
let spec = *audio_buf.spec();
let cap = audio_buf.capacity() as u64;
let need_rebuild = sample_buf.as_ref().map_or(true, |b| b.spec() != spec);
if need_rebuild {
    sample_buf = Some(SampleBuffer::<f32>::new(cap, spec));
}
let buf = sample_buf.as_mut().unwrap();
```

### CR-04: API 密钥字段未标记 `skip_serializing`

**File:** `src/config/types.rs` (12 个字段)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub vision_openai_api_key: String,  // 无 #[serde(skip_serializing)]
    pub text_openai_api_key: String,
    // ... 共 12 个密钥字段
}
```

**Description:** `AppConfig` 派生了 `Serialize`，但所有密钥字段（`vision_openai_api_key`、`text_openai_api_key`、`secret_id`、`secret_key`、`api_key`×4、`ak`、`sk`、`token`）没有 `#[serde(skip_serializing)]`。任何序列化操作都会将明文 API 密钥写入输出目标。

**Recommended fix:** 对所有密钥字段添加 `#[serde(skip_serializing)]`：

```rust
#[serde(skip_serializing)]
pub vision_openai_api_key: String,
```

---

## Warnings

### WR-01: LoudnormData 解析错误静默吞没为 0.0

**File:** `src/audio/normalizer.rs:73-95`

```rust
fn parse_measured_i(&self) -> f64 {
    self.measured_i.parse().unwrap_or(0.0)
}
```

**Description:** 5 个解析方法均用 `unwrap_or(0.0)` 静默吞没解析错误。0.0 作为 `measured_I` 传入两遍 loudnorm 会产生错误增益（将音频推到极响）。

**Recommended fix:** 返回 `Result<f64, AudioError>` 或至少用 `tracing::warn!` 记录解析失败。

### WR-02: `extract_json_from_stderr` rfind('}') 匹配范围过广

**File:** `src/audio/normalizer.rs:121`

**Description:** `rfind('}')` 搜索整个 stderr 文本而非 loudnorm 标记之后的部分，可能匹配到 JSON 块之外的 `}`。与上一轮 WR-20 相同，仍未修复。

### WR-03: IO 错误误报为 FileNotFound

**File:** `src/audio/normalizer.rs:244-245`

**Description:** `get_audio_rms` 将所有 IO 错误（含权限不足等）误报为 `AudioError::FileNotFound`，误导调试方向。

**Recommended fix:** 添加 `AudioError::IoError` 变体或在错误消息中包含原始 IO 错误信息。

### WR-04: sample_rate/channels 为 0 无检查

**File:** `src/audio/normalizer.rs:177-184`

**Description:** `normalize_lufs` 和 `normalize_audio_for_mixing` 不验证 `sample_rate`/`channels` 为 0，传入 FFmpeg `-ar 0 -ac 0` 会导致未定义行为。

### WR-05: URL 验证不防御空白字符和换行符

**File:** `src/youtube/downloader.rs:107-109`

**Description:** `starts_with` 校验可通过含换行符的 URL（如 `"http://evil.com\n--some-flag"`），可能被 yt-dlp 误解析。

**Recommended fix:** 增加 `url.trim().contains(['\n', '\r'])` 检查。

### WR-06: proxy_url 无格式校验

**File:** `src/youtube/downloader.rs:114-117`

**Description:** 任意字符串均可作为代理 URL 传入 yt-dlp `--proxy` 参数，不校验协议前缀（http/https/socks5）。

### WR-07: Pexels 精确匹配分辨率导致空结果

**File:** `src/material/searcher.rs:249`

**Description:** Pexels 搜索用 `==` 精确匹配分辨率，API 返回的固定分辨率列表（如 1920x1080, 3840x2160）与目标值（如 "1080p"）格式不同，容易空结果。

### WR-08: 缓存命中不验证文件完整性

**File:** `src/material/downloader.rs:59-67`

**Description:** 缓存命中仅检查 `exists()` + 文件大小，不验证内容完整性。损坏文件（如下载中断）会永远命中缓存。

**Recommended fix:** 对 0 字节文件跳过缓存，或在加载时用 ffprobe 验证。

### WR-09: project_version 默认值不一致

**Files:** `src/config/defaults.rs:7` vs `config.example.toml:2`

```rust
// defaults.rs
project_version: "0.1.0".to_string(),
// config.example.toml
project_version = "0.7.8"
```

**Description:** Rust 默认值 `"0.1.0"` 与示例配置 `"0.7.8"` 不匹配。

### WR-10: validate() 空实现

**File:** `src/config/mod.rs`

**Description:** `AppConfig::validate()` 为空实现，timeout、batch_size、volume 等字段无范围校验。配置中的非法值（如负数超时）不会在启动时报错。

### WR-11: normalize_lufs 硬编码 44100Hz/2ch，忽略配置

**File:** `src/audio/normalizer.rs:194,345-346`

**Description:** `AudioSection` 定义了 `sample_rate`（默认 44100）和 `channels`（默认 2）配置字段，但函数硬编码输出格式。用户修改 `[audio]` 段无效。与上一轮 WR-16 相同，仍未修复。

### WR-12: 音量 clamp 范围不一致

**Files:** `src/audio/normalizer.rs:307` vs `src/audio/volume.rs:205`

**Description:** `calculate_volume_adjustment()` 原声调整系数 clamp 到 `[0.1, 3.0]`，`validate_volume()` 限制为 `[0.0, 2.0]`。同一概念的不同上限可能导致混淆。与上一轮 WR-17 相同，仍未修复。

---

## Info

### IR-01: notify crate 使用 RC 版本（预存）

**File:** `Cargo.toml:19` — `notify = "9.0.0-rc.3"`

RC 版本 API 不稳定，稳定版发布后应升级。与上一轮 IR-14 相同。

### IR-02: lib.rs 所有模块 pub，部分可降为 pub(crate)

**File:** `src/lib.rs`

12 个模块均为 `pub mod`，内部实现模块（如 `audio::normalizer`）可能更适合 `pub(crate)` 以限制 API 表面积。

### IR-03: deny_unknown_fields 限制前向兼容性

**File:** `src/config/types.rs` (12 处)

全层级启用 `deny_unknown_fields`，新增配置字段会导致旧配置文件加载失败。

### IR-04: reqwest::blocking::Client 未复用连接池

**File:** `src/material/searcher.rs`

每次搜索请求重新构建 `reqwest::blocking::Client`，无法复用 TCP 连接池和 TLS 会话。

### IR-05: Pixabay 缺少 401 区分

**File:** `src/material/searcher.rs:292-294`

Pixabay 未像 Pexels 那样区分 401 认证错误，用户难以判断 key 配置是否有误。

### IR-06: 集成测试存根为设计意图

**Files:** `tests/audio_integration.rs:23,31`, `tests/youtube_integration.rs:36`, `tests/material_integration.rs`

所有集成测试为 `#[ignore]` 存根（`assert!(true)`），按设计推迟到后续阶段填充。

### IR-07: download_videos 硬编码相对路径

**File:** `src/material/downloader.rs:148` — `PathBuf::from("storage").join("temp").join(task_id)`

下载目录固定为相对路径，库模式下调用者无法控制存储位置。与上一轮 WR-19 相同。

### IR-08: save_video 临时文件无并发保护

**File:** `src/material/downloader.rs:86`

临时文件路径格式为 `{cache_key}.tmp`，并发调用相同 URL 时会冲突。与上一轮 IR-18 相同。

---

## Cross-File Analysis

**安全性:** 所有 FFmpeg/yt-dlp 子进程使用 `.args()` 数组模式（无 shell 注入）。YouTube 下载有 URL 前缀验证。但 rename 路径遍历（CR-01）和 NaN 绕过（CR-02）是新的安全风险。

**错误类型:** `AudioError`、`MaterialError`、`YoutubeError` 定义规范，中文 Display 消息一致，thiserror 使用正确。但部分错误路径有吞没/误报问题（WR-01、WR-03）。

**配置对齐:** `config.example.toml` 与 `AppConfig` 字段结构匹配，但默认值存在不一致（WR-09）且验证为空（WR-10）。

**模块结构:** `audio/`、`youtube/`、`material/` 三个模块结构清晰。`lib.rs` 导出完整。

---

*Phase: 11-extended-features*
*Reviewed: 2026-05-01 (fresh review, iteration 6)*
