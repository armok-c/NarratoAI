---
phase: 01-foundation
group: B
reviewed: 2026-04-28T15:50:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - src/ffmpeg/mod.rs
  - src/ffmpeg/command.rs
  - src/ffmpeg/probe.rs
  - src/ffmpeg/hwaccel.rs
findings:
  critical: 1
  warning: 3
  info: 4
  total: 8
status: issues_found
---

# Phase 01: 代码审查报告 (组 B — FFmpeg 操作层)

**审查时间:** 2026-04-28T15:50:00Z
**深度:** standard
**审查文件:** 4
**状态:** issues_found (1 CRITICAL, 3 WARNING, 4 INFO)

## 概要

本报告是 Phase 01 FFmpeg 操作层的第二次审查，涉及 4 个源文件的完整分析。

### 修复验证结果

| 原编号 | 状态 | 说明 |
|--------|------|------|
| CR-01 | 已验证通过 | ProgressCallback 归一化正确，duration > 0.0 防除零 |
| WR-04 | 已验证通过 | Duration 解析失败正确传播 OutputParseError |
| WR-05 | 已验证通过 | 孤儿子进程 kill+wait 清理正确 |
| IN-07 | 已验证通过 | `format!("{:.3}", ...)` 替换 `to_string()` |
| IN-08 | 已验证通过 | Fast seek 注释已添加 |
| IN-13 | 已验证通过 | `vec![]` 改为数组字面量 `[]` |

所有 6 项修复验证通过，未发现回归。

---

## Critical

### CR-B01: clip_video 缺少 `-c copy`，实际执行转码而非无损裁剪

**File:** `src/ffmpeg/command.rs:58-65`

**Issue:**
生成的 FFmpeg 命令为：
```
ffmpeg -loglevel level+info -ss <start> -i <input> -t <duration> -y <output>
```
缺少 `-c:v copy -c:a copy`，导致每一帧被解码再编码。函数名语义上承诺"裁剪 (clip)"，意味着快速 seek + 流拷贝（毫秒级），但实际行为是完整转码（分钟级）。

影响：
- 性能：1 分钟视频 clip 可能从 ~0.1s 变为 ~5-10s
- 质量：重新编码引入额外的质量损失
- 内存/CPU：不必要的解码/编码开销

**Fix:**
在 `.overwrite()` 后增加流拷贝参数：

```rust
cmd.overwrite()
    .codec_video("copy")
    .codec_audio("copy");
```

若需要支持"裁剪+转码"场景（如改变编码器），可添加一个 `reencode: bool` 参数。

---

## Warnings

### WR-B01: HighPerformance Profile 使用无效 encoder "auto"

**File:** `src/ffmpeg/hwaccel.rs:47`

**Issue:**
FFmpeg 没有名为 `"auto"` 的编码器。`-c:v auto` 会导致 FFmpeg 报错退出。

`recommend_profile()` 在非 Windows 且有 NVIDIA 的系统上返回 `HwAccelProfile::HighPerformance`。如果下游代码将此 profile 的 `encoder` 字段直接传给 FFmpeg 命令构建器，将导致失败。

**Fix:**
将 `encoder` 改为实际有效的编码器名称（如 `"h264_nvenc"`），或改为 `None`/`Option<String>` 表示"自动选择"，由命令构建器决定是否使用硬件编码器。

---

### WR-B02: WindowsNvidia Profile 使用无效 hwaccel 类型 "nvenc_pure"

**File:** `src/ffmpeg/hwaccel.rs:74`

**Issue:**
`hwaccel_type: "nvenc_pure"` 不是 FFmpeg 有效的 `-hwaccel` 值。

FFmpeg 支持的 hwaccel 值为：`none`, `auto`, `vdpau`, `dxva2`, `d3d11va`, `vaapi`, `qsv`, `videotoolbox`, `cuda`, `nvdec`。

NVENC 是**编码器**（使用 `-c:v h264_nvenc`），不是硬件加速解码器。NVENC 编码不需要 `-hwaccel` 参数。

**Fix:**
将 `hwaccel_type` 改为 `None` 或 `"cuda"`：
```rust
hwaccel_type: None,  // NVENC 编码不需要 hwaccel 参数
```
或者如果需要 NVDEC 硬件解码 + NVENC 硬件编码：
```rust
hwaccel_type: Some("cuda"),  // 或 "nvdec"
```

---

### WR-B03: probe_video 对无视频流输入静默返回零值

**File:** `src/ffmpeg/probe.rs:69-78`

**Issue:**
当输入文件为纯音频（无视频流）时，`probe_video` 返回 `Ok(VideoInfo { width: 0, height: 0, codec_name: "unknown", ... })` 而非 `Err`。

调用方若未先验证 `width > 0 && height > 0` 就直接构造 FFmpeg 裁剪命令，将失败且错误信息不明确。

**Fix:**
在探测到视频流但 width/height 为 0 时返回错误：

```rust
if width == 0 || height == 0 {
    return Err(FFmpegError::OutputParseError(
        format!("视频流无有效分辨率: {}x{}", width, height)
    ));
}
```

---

## Info

### IN-B01: Compatibility 和 UniversalSoftware 的 FFmpeg 参数完全一致

**File:** `src/ffmpeg/hwaccel.rs:55-67,96-109`

两个 profile 的 encoder (libx264)、preset (medium)、crf (23)、pixel_format (yuv420p)、compatibility_level (5) 完全相同，仅 name/description 不同。建议合并为一个 profile，或明确两者之间的差异。

### IN-B02: FFmpegError::Timeout 变体全局未使用

**File:** `src/error.rs:29`

`Timeout(String)` 变体已定义但无任何代码构造它。`clip_video` 也未实现超时机制。考虑删除该变体或为 `run_ffmpeg` 添加超时包装。

### IN-B03: detect_hw_encoders 静默兜底

**File:** `src/ffmpeg/hwaccel.rs:128-170`

FFmpeg 不可用或 `-encoders` 失败时静默返回空 `Vec`。如果用户安装了 FFmpeg 但配置错误（如损坏的二进制），`detect_hw_encoders` 不会报告任何问题，上层认为"系统无硬件编码器"而静默回退到软件编码。

建议添加 `tracing::warn!` 日志。

### IN-B04: quality_preset 与 additional_args 存在重叠

**File:** `src/ffmpeg/hwaccel.rs:42-109`

4/5 个 profile 同时在 `quality_preset` 和 `additional_args` 中定义了 `-preset` 参数（如 `quality_preset: "fast"` + `additional_args: "-preset fast"`）。如果命令构建器同时使用两者，将重复传递 `-preset`。

macOS profile 的 `quality_preset: "medium"` 与 `additional_args` 中的 `-q:v 65` 语义不一致（medium 是 x264 preset，q:v 是 Apple VideoToolbox 质量参数）。

---

## 异步安全性分析

`clip_video` 和 `probe_video_async` 的 spawn_blocking 使用正确：
- Path 转为 PathBuf（满足 Send + 'static）
- Arc<ProgressCallback> 闭包满足 Send + Sync
- spawn_blocking 不阻塞 tokio 运行时

未发现异步安全问题。

## 威胁模型验证

| Threat | Status |
|--------|--------|
| T-02-01: ffmpeg-sidecar builder API handles arg escaping | ✅ 使用 FfmpegCommand builder，非字符串拼接 |
| T-02-02: serde_json Value extraction with defaults | ✅ probe_video 使用安全的 Value 提取 |
| T-02-03: spawn_blocking threadpool | ✅ tokio 默认线程池 |

---

_审查时间: 2026-04-28T15:50:00Z_
_审查人: Claude (gsd-code-reviewer)_
_深度: standard_
