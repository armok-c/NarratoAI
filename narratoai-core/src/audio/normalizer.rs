//! 音频响度标准化模块
//!
//! 提供 LUFS 标准化（两遍 loudnorm）和 RMS 回退标准化。
//! 1:1 对齐 Python 版 `audio_normalizer.py`。
//!
//! ## 函数概览
//!
//! - `analyze_lufs()` — 第一遍 loudnorm，分析音频响度
//! - `normalize_lufs()` — 两遍 loudnorm（分析 + 应用）
//! - `get_audio_rms()` — symphonia 解码 → RMS dB（无法 loudnorm 时的回退）
//! - `calculate_volume_adjustment()` — 纯函数，计算 TTS/原声增益系数
//! - `normalize_audio_for_mixing()` — 编排入口，先 loudnorm 再回退 RMS

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use thiserror::Error;

/// 音频处理领域错误
///
/// 6 个变体全部为中文 Display，与 Phase 1 错误约定一致。
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("LUFS 分析失败: {0}")]
    LoudnormAnalysisFailed(String),

    #[error("LUFS 标准化失败: {0}")]
    LoudnormNormalizeFailed(String),

    #[error("RMS 回退失败: {0}")]
    RmsFallbackFailed(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无效音量值: {0}")]
    InvalidVolume(String),

    #[error("IO 错误: {0}")]
    IoError(String),
}

impl From<std::io::Error> for AudioError {
    fn from(err: std::io::Error) -> Self {
        AudioError::IoError(err.to_string())
    }
}

/// FFmpeg loudnorm JSON 输出
///
/// loudnorm 第一遍分析（print_format=json）输出的所有字段。
/// 第二遍标准化需要这些 measured 值来实现精确的响度匹配。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoudnormData {
    pub input_i: String,
    pub input_lra: String,
    pub input_tp: String,
    pub input_thresh: String,
    pub target_offset: String,
    pub output_i: String,
    pub output_lra: String,
    pub output_tp: String,
    pub output_thresh: String,
    pub normalization_type: String,
}

#[allow(non_snake_case)]
impl LoudnormData {
    fn safe_parse(field_name: &str, value: &str, fallback: f64) -> f64 {
        match value.parse::<f64>() {
            Ok(v) if v.is_finite() => v,
            _ => {
                tracing::warn!(
                    "{} 解析失败 ('{}')，使用回退值 {}",
                    field_name,
                    value,
                    fallback
                );
                fallback
            }
        }
    }

    pub fn measured_I(&self) -> f64 {
        Self::safe_parse("measured_I", &self.input_i, -23.0)
    }

    pub fn measured_LRA(&self) -> f64 {
        Self::safe_parse("measured_LRA", &self.input_lra, 7.0)
    }

    pub fn measured_TP(&self) -> f64 {
        Self::safe_parse("measured_TP", &self.input_tp, -1.0)
    }

    pub fn measured_thresh(&self) -> f64 {
        Self::safe_parse("measured_thresh", &self.input_thresh, -34.0)
    }

    pub fn offset(&self) -> f64 {
        Self::safe_parse("target_offset", &self.target_offset, 0.0)
    }
}

/// 从 FFmpeg stderr 输出中提取 JSON 块
///
/// loudnorm filter 将 JSON 输出到 stderr，格式为：
/// ```text
/// [Parsed_loudnorm_0 @ 0x...] {
///     "input_i": "-27.61",
///     ...
/// }
/// ```
fn extract_json_from_stderr(stderr: &[u8]) -> Result<String, AudioError> {
    let text = String::from_utf8_lossy(stderr);

    // loudnorm 输出格式：[Parsed_loudnorm_<N> @ 0x...] { ... }
    // 查找最后一段 loudnorm 输出（两遍标准化时第一遍结果也在 stderr 中）
    let marker = "Parsed_loudnorm_";
    let search_start = text
        .rfind(marker)
        .map(|pos| {
            // 跳过 "Parsed_loudnorm_" 后的实例编号直到 " @ " 或 "{"
            let rest = &text[pos + marker.len()..];
            rest.find(|c: char| !c.is_ascii_digit())
                .map(|i| pos + marker.len() + i)
                .unwrap_or(pos + marker.len())
        })
        .unwrap_or(0);
    let start = text[search_start..]
        .find('{')
        .map(|i| search_start + i)
        .ok_or_else(|| {
            AudioError::LoudnormAnalysisFailed("未在 FFmpeg stderr 中找到 JSON 输出".into())
        })?;
    let end = text[start..].rfind('}').map(|i| start + i).ok_or_else(|| {
        AudioError::LoudnormAnalysisFailed("未在 FFmpeg stderr 中找到 JSON 结束符".into())
    })?;
    if end <= start {
        return Err(AudioError::LoudnormAnalysisFailed(
            "JSON 边界异常：结束位置早于起始位置".into(),
        ));
    }
    Ok(text[start..=end].to_string())
}

/// 第一遍 loudnorm 分析
///
/// 通过 FFmpeg loudnorm filter 获取音频的 measured_I / LRA / TP / thresh / target_offset。
/// loudnorm JSON 输出在 stderr 中，需从 stderr 文本中提取 JSON 块。
pub fn analyze_lufs(input: &Path) -> Result<LoudnormData, AudioError> {
    if !input.exists() {
        return Err(AudioError::FileNotFound(input.display().to_string()));
    }

    let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();
    let filter = "loudnorm=I=-23:TP=-1:LRA=7:print_format=json";

    let output = Command::new(&ffmpeg_bin)
        .args(["-hide_banner", "-nostats", "-i"])
        .arg(input.as_os_str())
        .args(["-af", filter, "-f", "null", "-"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AudioError::LoudnormAnalysisFailed("未找到 FFmpeg 二进制文件".into())
            } else {
                AudioError::LoudnormAnalysisFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AudioError::LoudnormAnalysisFailed(format!(
            "FFmpeg 退出码 {:?}: {}",
            output.status.code(),
            stderr,
        )));
    }

    let json_str = extract_json_from_stderr(&output.stderr)?;
    let data: LoudnormData = serde_json::from_str(&json_str)
        .map_err(|e| AudioError::LoudnormAnalysisFailed(format!("JSON 解析失败: {}", e)))?;

    Ok(data)
}

/// 两遍 loudnorm 标准化
///
/// 第一遍 `analyze_lufs` 获取 measured 值，第二遍 loudnorm 应用标准化。
/// 输出采样率和声道数由参数控制（通常来自 `AudioSection` 配置）。
pub fn normalize_lufs(
    input: &Path,
    output: &Path,
    target_lufs: f64,
    max_peak: f64,
    sample_rate: u32,
    channels: u32,
) -> Result<(), AudioError> {
    if !input.exists() {
        return Err(AudioError::FileNotFound(input.display().to_string()));
    }

    if sample_rate == 0 || channels == 0 {
        return Err(AudioError::InvalidVolume(format!(
            "sample_rate={} channels={} 不能为 0",
            sample_rate, channels
        )));
    }

    // 第一遍：分析
    let data = analyze_lufs(input)?;

    // 第二遍：应用标准化
    let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();

    let filter = format!(
        "loudnorm=I={}:TP={}:LRA=7:measured_I={}:measured_LRA={}:measured_TP={}:measured_thresh={}:offset={}",
        target_lufs,
        max_peak,
        data.measured_I(),
        data.measured_LRA(),
        data.measured_TP(),
        data.measured_thresh(),
        data.offset(),
    );

    let result = Command::new(&ffmpeg_bin)
        .args(["-y", "-hide_banner", "-i"])
        .arg(input.as_os_str())
        .args(["-af", &filter])
        .args([
            "-ar",
            &sample_rate.to_string(),
            "-ac",
            &channels.to_string(),
        ])
        .arg(output.as_os_str())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AudioError::LoudnormNormalizeFailed("未找到 FFmpeg 二进制文件".into())
            } else {
                AudioError::LoudnormNormalizeFailed(e.to_string())
            }
        })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(AudioError::LoudnormNormalizeFailed(format!(
            "FFmpeg 退出码 {:?}: {}",
            result.status.code(),
            stderr,
        )));
    }

    Ok(())
}

/// 通过 symphonia 解码计算音频 RMS（dB）
///
/// 作为 loudnorm 分析失败时的回退方案。
/// 解码全部 PCM f32 采样 → 计算 RMS → 转换为 dB。
pub fn get_audio_rms(input: &Path) -> Result<f64, AudioError> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AudioError::FileNotFound(input.display().to_string())
        } else {
            AudioError::IoError(format!("{}: {}", input.display(), e))
        }
    })?;
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());

    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| AudioError::RmsFallbackFailed(format!("探测失败: {}", e)))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::RmsFallbackFailed("无音频轨道".into()))?;

    let codec_params = &track.codec_params;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::RmsFallbackFailed(format!("解码器初始化失败: {}", e)))?;

    // 展平所有声道计算全局 RMS，作为 loudnorm 的降级回退方案。
    // 不做声道分离，因为回退场景下精度要求低于 loudnorm 主路径。
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }

        if let Ok(audio_buf) = decoder.decode(&packet) {
            let spec = *audio_buf.spec();
            let cap = audio_buf.capacity() as u64;
            let mut buf = SampleBuffer::<f32>::new(cap, spec);
            buf.copy_interleaved_ref(audio_buf);
            all_samples.extend_from_slice(buf.samples());
        }
    }

    if all_samples.is_empty() {
        return Err(AudioError::RmsFallbackFailed("解码后无音频采样".into()));
    }

    let sum_sq: f64 = all_samples.iter().map(|s| (*s as f64) * (*s as f64)).sum();
    let rms = (sum_sq / all_samples.len() as f64).sqrt();

    let rms_db = if rms > 0.0 {
        20.0 * rms.log10()
    } else {
        -60.0 // 静音
    };
    Ok(rms_db)
}

/// 计算 TTS 和原声的音量调整系数
///
/// 纯函数，无文件 I/O。目标响度 -20.0 LUFS（低于全局 target_lufs -14.0），
/// 因为 TTS 和原声混合后仍需为 BGM 和其他音轨留出响度余量。
/// - TTS 调整系数 clamp 到 [0.1, 2.0]
/// - 原声调整系数 clamp 到 [0.1, 3.0]
///
/// 注意：此处的 clamp 范围与 `validate_volume()` 的 [0.0, 2.0] 不同。
/// 这里计算的是增益系数（LUFS 差值转换的线性增益），极端安静的原声
/// 需要更大的增益倍数，因此上限为 3.0。`validate_volume()` 校验的是
/// 用户配置的音量百分比，上限 2.0 即 200%。
pub fn calculate_volume_adjustment(tts_lufs: f64, original_lufs: f64) -> (f64, f64) {
    let target_lufs = -20.0;

    let tts_adjustment = 10.0_f64.powf((target_lufs - tts_lufs) / 20.0);
    let original_adjustment = 10.0_f64.powf((target_lufs - original_lufs) / 20.0);

    let tts_adj = tts_adjustment.clamp(0.1, 2.0);
    let orig_adj = original_adjustment.clamp(0.1, 3.0);

    (tts_adj, orig_adj)
}

/// 音频标准化编排入口
///
/// 优先 loudnorm 两遍标准化，失败时回退到 RMS 标准化。
/// 输出路径格式：`{output_dir}/{filename}_normalized.mp3`
pub fn normalize_audio_for_mixing(
    audio_path: &Path,
    output_dir: &Path,
    target_lufs: f64,
    max_peak: f64,
    sample_rate: u32,
    channels: u32,
) -> Result<PathBuf, AudioError> {
    if !audio_path.exists() {
        return Err(AudioError::FileNotFound(audio_path.display().to_string()));
    }

    let stem = audio_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = output_dir.join(format!("{}_normalized.mp3", stem));

    // 优先 loudnorm 两遍标准化
    match normalize_lufs(
        audio_path,
        &output_path,
        target_lufs,
        max_peak,
        sample_rate,
        channels,
    ) {
        Ok(()) => return Ok(output_path),
        Err(e) => tracing::warn!("loudnorm 两遍标准化失败，回退到 RMS 标准化: {}", e),
    }

    // 回退：RMS 标准化
    let rms_db = get_audio_rms(audio_path)?;
    let gain_db = target_lufs - rms_db;

    let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();
    let result = Command::new(&ffmpeg_bin)
        .args(["-y", "-hide_banner", "-i"])
        .arg(audio_path.as_os_str())
        .args(["-af", &format!("volume={}dB", gain_db)])
        .args([
            "-ar",
            &sample_rate.to_string(),
            "-ac",
            &channels.to_string(),
        ])
        .arg(&output_path)
        .output()
        .map_err(|e| AudioError::LoudnormNormalizeFailed(e.to_string()))?;

    if !result.status.success() {
        return Err(AudioError::RmsFallbackFailed(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_error_chinese_display() {
        let err = AudioError::LoudnormAnalysisFailed("测试".into());
        assert!(err.to_string().contains("LUFS 分析失败"));

        let err = AudioError::LoudnormNormalizeFailed("测试".into());
        assert!(err.to_string().contains("LUFS 标准化失败"));

        let err = AudioError::RmsFallbackFailed("测试".into());
        assert!(err.to_string().contains("RMS 回退失败"));

        let err = AudioError::FileNotFound("test.mp3".into());
        assert!(err.to_string().contains("文件未找到"));

        let err = AudioError::InvalidVolume("音量超出范围".into());
        assert!(err.to_string().contains("无效音量值"));
    }

    #[test]
    fn test_loudnorm_data_parse() {
        let json = r#"{
            "input_i": "-27.61",
            "input_lra": "18.06",
            "input_tp": "-4.47",
            "input_thresh": "-39.20",
            "target_offset": "0.58",
            "output_i": "-23.00",
            "output_lra": "6.80",
            "output_tp": "-1.50",
            "output_thresh": "-34.00",
            "normalization_type": "linear"
        }"#;
        let data: LoudnormData = serde_json::from_str(json).unwrap();
        assert!((data.measured_I() - (-27.61)).abs() < 0.01);
        assert!((data.offset() - 0.58).abs() < 0.01);
    }

    #[test]
    fn test_calculate_volume_adjustment() {
        // TTS -16 LUFS, Original -20 LUFS
        let (tts_adj, orig_adj) = calculate_volume_adjustment(-16.0, -20.0);
        // target = -20, so TTS needs attenuation, Original is at target
        assert!(tts_adj < 1.0, "TTS 响度高于目标，应衰减: {}", tts_adj);
        assert!(
            (orig_adj - 1.0).abs() < 0.1,
            "原声已在目标附近: {}",
            orig_adj
        );
    }

    #[test]
    fn test_calculate_volume_adjustment_clamp() {
        // 极端值测试 clamp
        let (tts_adj, orig_adj) = calculate_volume_adjustment(-60.0, -80.0);
        assert!(tts_adj <= 2.0, "TTS 不应超过 2.0 clamp: {}", tts_adj);
        assert!(orig_adj <= 3.0, "原声不应超过 3.0 clamp: {}", orig_adj);
    }

    #[test]
    fn test_normalize_lufs_nonexistent_file() {
        let tmp = std::env::temp_dir();
        let result = normalize_lufs(
            &tmp.join("nonexistent_test_audio.wav"),
            &tmp.join("output.wav"),
            -23.0,
            -1.0,
            44100,
            2,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AudioError::FileNotFound(_) => {} // 预期
            _ => panic!("应为 FileNotFound 错误"),
        }
    }

    #[test]
    fn test_normalize_audio_for_mixing_nonexistent_file() {
        let tmp = std::env::temp_dir();
        let result = normalize_audio_for_mixing(
            &tmp.join("nonexistent_test.wav"),
            &tmp,
            -20.0,
            -1.0,
            44100,
            2,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_audio_rms_formula() {
        // 静音：全零采样 → RMS = 0.0
        let silent: Vec<f32> = vec![0.0; 100];
        let rms_silent = (silent
            .iter()
            .map(|s| (*s as f64) * (*s as f64))
            .sum::<f64>()
            / silent.len() as f64)
            .sqrt();
        assert!((rms_silent - 0.0).abs() < f64::EPSILON);
        // 静音时 rms = 0，进入 else 分支，返回 -60.0

        // 恒定振幅 0.5 → RMS = 0.5 → dB ≈ -6.02
        let half: Vec<f32> = vec![0.5; 100];
        let rms_half = (half.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>()
            / half.len() as f64)
            .sqrt();
        let db_half = 20.0 * rms_half.log10();
        assert!(
            (db_half - (-6.0206)).abs() < 0.01,
            "0.5 RMS 应为 -6.02 dB, got {}",
            db_half
        );

        // 恒定振幅 1.0 → RMS = 1.0 → dB = 0.0
        let full: Vec<f32> = vec![1.0; 100];
        let rms_full = (full.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>()
            / full.len() as f64)
            .sqrt();
        let db_full = 20.0 * rms_full.log10();
        assert!(
            (db_full - 0.0).abs() < 0.01,
            "1.0 RMS 应为 0 dB, got {}",
            db_full
        );

        // 验证旧公式（i16::MAX 参考）产生错误结果
        let db_old = 20.0 * (rms_half / (i16::MAX as f64)).log10();
        assert!(
            (db_old - (-96.329)).abs() < 0.01,
            "旧公式 0.5 RMS 应为 -96.33 dB, got {}",
            db_old
        );
    }
}
