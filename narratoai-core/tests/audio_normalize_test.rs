//! Level 2: 音频标准化集成测试 (D-31)
//!
//! 生成测试正弦波 → normalize_audio_for_mixing() → ffprobe loudnorm 验证
//!
//! 前置条件：
//! - FFmpeg/ffprobe 已安装并在 PATH 中
//! - 测试运行时检查 FFmpeg 可用性，不可用时静默跳过
//!
//! 测试产出在 E:\GitLib\NarratoAI\test_output\audio_normalize\ 目录

use std::f64::consts::PI;
use std::path::Path;

use narratoai_core::audio::normalizer::{analyze_lufs, normalize_audio_for_mixing};

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 生成指定参数的 WAV 文件
///
/// 返回写入的文件路径。生成 16-bit PCM 单声道 WAV。
fn generate_sine_wave(
    output_dir: &Path,
    name: &str,
    frequency: f64,
    duration_secs: f64,
    sample_rate: u32,
    amplitude: f64, // 0.0 ~ 1.0
) -> std::path::PathBuf {
    let path = output_dir.join(name);
    let num_samples = (sample_rate as f64 * duration_secs) as usize;

    // WAV header: 44 bytes + PCM data
    let data_size = (num_samples * 2) as u32; // 16-bit = 2 bytes/sample
    let file_size = 36 + data_size;

    let mut buffer = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    buffer.extend_from_slice(b"RIFF");
    buffer.extend_from_slice(&file_size.to_le_bytes());
    buffer.extend_from_slice(b"WAVE");

    // fmt chunk
    buffer.extend_from_slice(b"fmt ");
    buffer.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buffer.extend_from_slice(&1u16.to_le_bytes()); // PCM = 1
    buffer.extend_from_slice(&1u16.to_le_bytes()); // mono
    buffer.extend_from_slice(&sample_rate.to_le_bytes());
    buffer.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buffer.extend_from_slice(&2u16.to_le_bytes()); // block align
    buffer.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    buffer.extend_from_slice(b"data");
    buffer.extend_from_slice(&data_size.to_le_bytes());

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (amplitude * (2.0 * PI * frequency * t).sin() * 32767.0) as i16;
        buffer.extend_from_slice(&sample.to_le_bytes());
    }

    std::fs::write(&path, &buffer).expect("写入 WAV 文件失败");
    path
}

/// 检查 FFmpeg 是否可用
fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[test]
fn test_normalize_to_minus_14_lufs() {
    if !ffmpeg_available() {
        eprintln!("⚠ FFmpeg 不可用，跳过 D-31 LUFS 集成测试");
        return;
    }

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_output")
        .join("audio_normalize");
    std::fs::create_dir_all(&output_dir).expect("创建测试输出目录失败");

    // Step 1: 生成测试正弦波（440Hz, 2秒, 0.8 振幅 → 约 -4 dBFS）
    let sample_rate = 44100u32;
    let wav_path = generate_sine_wave(
        &output_dir,
        "test_tone_440hz.wav",
        440.0,
        2.0,
        sample_rate,
        0.8,
    );
    assert!(wav_path.exists(), "测试 WAV 应存在");

    // Step 2: 执行音频标准化（目标 -14.0 LUFS, YouTube/流媒体标准）
    let normalized = normalize_audio_for_mixing(
        &wav_path,
        &output_dir,
        -14.0, // target_lufs
        -1.0,  // max_peak
        sample_rate,
        1, // channels (mono)
    )
    .expect("标准化应成功");
    assert!(normalized.exists(), "标准化文件应存在");

    // Step 3: 用 ffprobe loudnorm 分析标准化后的音频
    let loudnorm = analyze_lufs(&normalized).expect("ffprobe loudnorm 分析应成功");

    // Step 4: 验证 integrated LUFS ≈ -14.0（±2 LUFS 容差）
    let integrated_lufs = loudnorm.measured_I();

    let expected = -14.0;
    let tolerance = 2.0;
    let diff = (integrated_lufs - expected).abs();

    println!(
        "标准化结果: integrated_lufs={:.1}, 目标={:.1}, 偏差={:.1} LUFS",
        integrated_lufs, expected, diff
    );

    assert!(
        diff <= tolerance,
        "integrated_lufs ({:.1}) 应接近 {} (±{} LUFS)，偏差 {:.1} 超出容差",
        integrated_lufs,
        expected,
        tolerance,
        diff
    );

    // 验证 normalize_audio_for_mixing 返回值正确（非空路径）
    assert!(
        !normalized.to_string_lossy().is_empty(),
        "标准化输出路径不应为空"
    );
}

#[test]
fn test_normalize_handles_nonexistent_input() {
    let tmp = std::env::temp_dir();
    let result = normalize_audio_for_mixing(
        Path::new("/nonexistent/test.wav"),
        &tmp,
        -14.0,
        -1.0,
        44100,
        1,
    );
    assert!(result.is_err(), "不存在的输入文件应返回错误");
}
