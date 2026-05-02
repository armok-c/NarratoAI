---
phase: 11-extended-features
reviewed: 2026-05-01T19:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - src/audio/mod.rs
  - src/audio/normalizer.rs
  - src/audio/volume.rs
  - tests/audio_integration.rs
findings:
  critical: 2
  warning: 5
  info: 2
  total: 9
status: issues_found
---

# Phase 11: 音频处理模块代码审查报告

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

审查了音频处理模块的 4 个文件，包括 normalizer.rs（LUFS 两遍标准化、RMS 回退、symphonia 解码）、volume.rs（音量配置与验证）和集成测试。发现 2 个 Critical 级别问题：symphonia `SampleBuffer` 容量不匹配导致缓冲区溢出风险，以及 `validate_volume()` / `VolumeConfig::validate()` 对 NaN/Infinity 输入无防护。另有 5 个 Warning 和 2 个 Info 问题。

## Critical Issues

### CR-01: symphonia SampleBuffer 容量不匹配导致潜在 panic

**File:** `src/audio/normalizer.rs:287`
**Issue:** `SampleBuffer` 通过 `get_or_insert_with` 仅在第一次解码时创建，使用首次 packet 的 `capacity` 和 `spec`。后续 packet 若具有不同的采样率、声道数或更大的 capacity，`copy_interleaved_ref` 会 panic（symphonia 内部做 capacity 检查）。

在多段音频文件中（如 VBR 编码或格式切换的流），不同 packet 完全可能有不同的 capacity 或 spec。`get_or_insert_with` 只在 `None` 时初始化，后续 packet 不会重建 buffer。

**Fix:**
```rust
// 每次解码都重新创建 SampleBuffer，或在 spec/capacity 变化时重建
if let Ok(audio_buf) = decoder.decode(&packet) {
    let spec = *audio_buf.spec();
    let cap = audio_buf.capacity() as u64;
    // 每次重新分配，确保规格匹配
    let mut buf = SampleBuffer::<f32>::new(cap, spec);
    buf.copy_interleaved_ref(audio_buf);
    all_samples.extend_from_slice(buf.samples());
}
```

### CR-02: validate_volume() 和 VolumeConfig::validate() 不处理 NaN/Infinity

**File:** `src/audio/volume.rs:33,206`
**Issue:** IEEE 754 浮点比较中，`NaN < 0.0` 和 `NaN > 2.0` 均为 `false`，因此 NaN 会绕过所有检查直接通过验证。`Infinity` 同理——`Infinity > 2.0` 为 `true` 会被 clamp 到 2.0（这实际上是安全的），但 `NaN` 会原样传递到下游 FFmpeg filter 参数中，生成无效的 `volume=NaN dB` 或 `loudnorm=measured_I=NaN` 字符串。

`VolumeConfig::validate()` (line 33) 和 `validate_volume()` (line 206) 都有此问题。

如果配置文件中某个字段缺失或损坏，TOML 反序列化可能产生 `0.0`（通过 `#[serde(default)]`），但如果上游传入计算结果（如 `calculate_volume_adjustment` 的返回值），NaN 是可能的。

**Fix:**
```rust
// VolumeConfig::validate()
impl VolumeConfig {
    pub fn validate(&self) -> Result<(), super::normalizer::AudioError> {
        for (name, val) in [
            ("tts_volume", self.tts_volume),
            ("original_volume", self.original_volume),
            ("bgm_volume", self.bgm_volume),
        ] {
            if !val.is_finite() || val < 0.0 || val > 2.0 {
                return Err(super::normalizer::AudioError::InvalidVolume(
                    format!("{}={} 不在 [0.0, 2.0] 范围内", name, val),
                ));
            }
        }
        Ok(())
    }
}

// validate_volume()
pub fn validate_volume(volume: f64, name: &str) -> f64 {
    let min = 0.0;
    let max = 2.0;
    if !volume.is_finite() {
        tracing::warn!("{} 音量 {} 非有限浮点数，已重置为 1.0", name, volume);
        1.0
    } else if volume < min {
        tracing::warn!("{} 音量 {} 低于最小值 {}，已调整", name, volume, min);
        min
    } else if volume > max {
        tracing::warn!("{} 音量 {} 超过最大值 {}，已调整", name, volume, max);
        max
    } else {
        volume
    }
}
```

## Warnings

### WR-01: extract_json_from_stderr 使用 rfind('}') 可能匹配到 JSON 块之外的右花括号

**File:** `src/audio/normalizer.rs:121`
**Issue:** FFmpeg stderr 输出除了 loudnorm JSON 之外，还可能包含其他带 `}` 的文本（如日志消息、编码器信息等）。使用 `text.rfind('}')` 搜索整个 stderr 中的最后一个 `}`，如果 JSON 块之后有其他包含 `}` 的内容，提取的字符串尾部会包含多余的字符，导致 JSON 解析失败。

当前代码使用 `rfind("Parsed_loudnorm_")` 定位 JSON 起始区域，这是好的。但结束位置 `rfind('}')` 搜索的是整个文本，而非 loudnorm 标记之后的部分。

**Fix:**
```rust
// 仅在 loudnorm 标记之后搜索结束花括号
let search_start = text
    .rfind("Parsed_loudnorm_")
    .map(|pos| pos + "Parsed_loudnorm_".len())
    .unwrap_or(0);
let start = text[search_start..]
    .find('{')
    .map(|i| search_start + i)
    .ok_or_else(|| {
        AudioError::LoudnormAnalysisFailed("未在 FFmpeg stderr 中找到 JSON 输出".into())
    })?;
let end = text[search_start..]
    .rfind('}')
    .map(|i| search_start + i)
    .ok_or_else(|| {
        AudioError::LoudnormAnalysisFailed("未在 FFmpeg stderr 中找到 JSON 结束符".into())
    })?;
```

### WR-02: LoudnormData 解析失败静默回退到 0.0，导致两遍 loudnorm 产生错误结果

**File:** `src/audio/normalizer.rs:73-95`
**Issue:** `measured_I()`, `measured_LRA()`, `measured_TP()`, `measured_thresh()`, `offset()` 全部使用 `unwrap_or(0.0)`。如果 FFmpeg 输出的 JSON 中某个字段为空字符串或非法值（某些 FFmpeg 版本在静音音频时会输出 `"input_i": ""` 或 `"input_i": "-inf"`），解析失败会静默得到 0.0。

当 `measured_I = 0.0` 传入第二遍 loudnorm 的 `measured_I=0.0` 时，FFmpeg 的线性标准化会计算错误的增益，可能导致输出音频严重失真或削波。这是一个静默错误——不会 panic 也不会报错，但输出结果不正确。

**Fix:**
```rust
pub fn measured_I(&self) -> Result<f64, AudioError> {
    self.input_i.parse().map_err(|_| {
        AudioError::LoudnormAnalysisFailed(
            format!("无法解析 measured_I: '{}'", self.input_i)
        )
    })
}

// 或者至少对 "-inf" / 空字符串做特殊处理
pub fn measured_I(&self) -> f64 {
    match self.input_i.as_str() {
        "-inf" | "-Inf" | "" => -70.0, // 使用极小值而非 0.0
        s => s.parse().unwrap_or(-70.0),
    }
}
```

### WR-03: get_audio_rms 将所有 IO 错误误报为 FileNotFound

**File:** `src/audio/normalizer.rs:244-245`
**Issue:** `std::fs::File::open` 可能因权限不足（PermissionDenied）、路径过长（InvalidFilename）、符号链接问题等多种原因失败，但代码将所有这些错误都映射为 `AudioError::FileNotFound`。这会误导用户和调试——实际问题是权限但报错说文件未找到。

`AudioError` 当前没有合适的 IO 权限错误变体，但 `IoError` 变体已经存在且更适合这类情况。

**Fix:**
```rust
let file = std::fs::File::open(input)
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AudioError::FileNotFound(input.display().to_string())
        } else {
            AudioError::IoError(format!("打开音频文件失败 {}: {}", input.display(), e))
        }
    })?;
```

### WR-04: normalize_lufs 和 normalize_audio_for_mixing 不验证 sample_rate/channels 为 0

**File:** `src/audio/normalizer.rs:177-184,337-343`
**Issue:** `normalize_lufs()` 和 `normalize_audio_for_mixing()` 接受 `sample_rate: u32` 和 `channels: u32` 参数，但未验证这些值不为 0。如果上游传入 0（如配置文件中 `sample_rate = 0`），FFmpeg 会收到 `-ar 0 -ac 0` 参数，行为未定义——可能崩溃或产生空文件。

虽然配置默认值是 44100 和 2，但 `#[serde(default)]` 对 `u32` 类型的默认值是 0，如果配置文件中有空值或类型错误，反序列化会静默得到 0。

**Fix:**
```rust
pub fn normalize_lufs(
    input: &Path,
    output: &Path,
    target_lufs: f64,
    max_peak: f64,
    sample_rate: u32,
    channels: u32,
) -> Result<(), AudioError> {
    if sample_rate == 0 {
        return Err(AudioError::InvalidVolume("sample_rate 不能为 0".into()));
    }
    if channels == 0 {
        return Err(AudioError::InvalidVolume("channels 不能为 0".into()));
    }
    // ...
}
```

### WR-05: 集成测试全部为空存根，缺乏实际验证

**File:** `tests/audio_integration.rs:19-32`
**Issue:** 两个集成测试 `test_normalize_lufs_two_pass` 和 `test_get_audio_rms_with_real_file` 均为存根，只包含 `assert!(true)`。这意味着两遍 loudnorm 标准化和 symphonia RMS 解码这两个核心功能路径没有任何端到端验证。

特别考虑到 CR-01（SampleBuffer 容量问题）和 WR-02（JSON 解析静默失败），集成测试的缺失使得这些 bug 在实际环境中更难被发现。

**Fix:** 使用 FFmpeg 生成已知响度的测试音频文件，然后验证 `normalize_lufs` 输出文件的响度接近目标值，`get_audio_rms` 返回值在预期范围内。示例：
```rust
#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test_tone.wav");
    let output = dir.path().join("test_normalized.mp3");

    // 使用 ffmpeg 生成 -20 LUFS 的 1kHz 正弦波
    Command::new("ffmpeg")
        .args(["-f", "lavfi", "-i", "sine=frequency=1000:duration=5",
               "-af", "loudnorm=I=-20:TP=-1:LRA=7",
               "-ar", "44100", "-ac", "2", "-y"])
        .arg(&input)
        .output()
        .unwrap();

    let result = normalize_lufs(&input, &output, -23.0, -1.0, 44100, 2);
    assert!(result.is_ok());
    assert!(output.exists());
}
```

## Info

### IN-01: calculate_volume_adjustment 对 NaN 输入无防护

**File:** `src/audio/normalizer.rs:321-331`
**Issue:** `calculate_volume_adjustment()` 是纯函数，接受 `f64` 参数。如果 `tts_lufs` 或 `original_lufs` 为 NaN，`10.0_f64.powf(...)` 的结果也是 NaN，clamp 对 NaN 无效（NaN 与任何数的比较都返回 false），最终返回 `(NaN, NaN)`。虽然上游目前通过 FFmpeg 解析获得 LUFS 值（不太可能为 NaN），但作为公共 API 应具有防御性。

**Fix:**
```rust
pub fn calculate_volume_adjustment(tts_lufs: f64, original_lufs: f64) -> (f64, f64) {
    let target_lufs = -20.0;
    let tts_lufs = if tts_lufs.is_finite() { tts_lufs } else { -20.0 };
    let original_lufs = if original_lufs.is_finite() { original_lufs } else { -20.0 };
    // ...
}
```

### IN-02: MixingConfig::from_section 忽略 dynamic_range_compression 配置

**File:** `src/audio/volume.rs:97-103`
**Issue:** `MixingConfig::from_section` 硬编码 `dynamic_range_compression: false`，未从 `AudioSection` 读取该字段。如果 `AudioSection` 中未来添加了 `dynamic_range_compression` 配置项，此处会被遗漏。当前 `AudioSection` 中确实没有此字段，所以不是 bug，但硬编码 `false` 而不添加注释说明原因会误导维护者。

**Fix:** 添加注释说明原因，或在 `AudioSection` 中添加对应字段：
```rust
impl MixingConfig {
    /// 从配置段构建混合配置
    /// 注意：dynamic_range_compression 当前硬编码为 false，AudioSection 中暂无对应配置项
    pub fn from_section(section: &AudioSection) -> Self {
        Self {
            crossfade_duration: section.crossfade_duration,
            bgm_fade_out: section.bgm_fade_out,
            dynamic_range_compression: false, // TODO: 从配置读取
        }
    }
}
```

---

_Reviewed: 2026-05-01T19:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
