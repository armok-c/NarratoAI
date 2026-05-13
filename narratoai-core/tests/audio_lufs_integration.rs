//! 音频 LUFS 标准化和 RMS 集成测试
//!
//! 覆盖 normalizer.rs 中 normalize_lufs 和 get_audio_rms 的集成场景。

use std::f64::consts::PI;
use std::path::Path;

use narratoai_core::audio::normalizer::{analyze_lufs, get_audio_rms, normalize_lufs};

fn generate_sine_wave(
    output_dir: &Path,
    name: &str,
    frequency: f64,
    duration_secs: f64,
    sample_rate: u32,
    amplitude: f64,
) -> std::path::PathBuf {
    let path = output_dir.join(name);
    let num_samples = (sample_rate as f64 * duration_secs) as usize;
    let data_size = (num_samples * 2) as u32;
    let file_size = 36 + data_size;
    let mut buffer = Vec::with_capacity(44 + data_size as usize);
    buffer.extend_from_slice(b"RIFF");
    buffer.extend_from_slice(&file_size.to_le_bytes());
    buffer.extend_from_slice(b"WAVE");
    buffer.extend_from_slice(b"fmt ");
    buffer.extend_from_slice(&16u32.to_le_bytes());
    buffer.extend_from_slice(&1u16.to_le_bytes());
    buffer.extend_from_slice(&1u16.to_le_bytes());
    buffer.extend_from_slice(&sample_rate.to_le_bytes());
    buffer.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    buffer.extend_from_slice(&2u16.to_le_bytes());
    buffer.extend_from_slice(&16u16.to_le_bytes());
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

fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    if !ffmpeg_available() {
        eprintln!("⚠ FFmpeg 不可用，跳过 LUFS 两遍标准化测试");
        return;
    }

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_output")
        .join("audio_lufs");
    std::fs::create_dir_all(&output_dir).expect("创建测试输出目录失败");

    let sample_rate = 44100u32;
    let input = generate_sine_wave(&output_dir, "lufs_input.wav", 440.0, 2.0, sample_rate, 0.8);
    let output = output_dir.join("lufs_output.wav");

    normalize_lufs(&input, &output, -23.0, -1.0, sample_rate, 1).expect("LUFS 标准化应成功");
    assert!(output.exists(), "标准化输出文件应存在");

    let data = analyze_lufs(&output).expect("分析标准化后文件应成功");
    let measured = data.measured_I();
    assert!(
        (measured - (-23.0)).abs() <= 3.0,
        "integrated LUFS ({measured:.1}) 应在 [-26, -20] 范围内（目标 -23 ± 3 LUFS 容差）"
    );
}

#[test]
fn test_get_audio_rms_with_real_file() {
    let dir = std::env::temp_dir().join("narratoai_audio_rms_test");
    std::fs::create_dir_all(&dir).expect("创建临时目录失败");

    let path = generate_sine_wave(&dir, "rms_input.wav", 440.0, 2.0, 44100, 0.8);
    let result = get_audio_rms(&path);

    assert!(result.is_ok(), "get_audio_rms 应成功: {:?}", result.err());
    let rms_db = result.unwrap();
    // 0.8 振幅正弦波: RMS = 0.8/sqrt(2) ≈ 0.566 → dB ≈ -4.95
    // 容差 [-10, 0] 覆盖实际值
    assert!(
        (-10.0..=0.0).contains(&rms_db),
        "RMS dB ({rms_db:.2}) 应在 [-10, 0] 范围内"
    );
}
