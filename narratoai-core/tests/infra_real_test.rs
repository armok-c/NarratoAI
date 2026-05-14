//! Level 0: 基础设施真实集成测试
//!
//! 验证基本文件系统、FFmpeg、视频探测、字幕解析是否正常工作。
//! 这些测试不消耗 LLM API 额度，仅依赖本地文件系统。
//!
//! 视频素材: E:\GitLib\视频\华星大酒店.mp4
//! 字幕素材: E:\GitLib\视频\华星大酒店.srt

use std::path::Path;
use std::process::Command;

use narratoai_core::subtitle::parser::parse_subtitle_file;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

const VIDEO_PATH: &str = "E:/GitLib/视频/华星大酒店.mp4";
const SUBTITLE_PATH: &str = "E:/GitLib/视频/华星大酒店.srt";
const TEST_OUTPUT_DIR: &str = "E:/GitLib/NarratoAI/test_output/infra";

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

/// 测试 1: 视频文件存在且可读
#[test]
fn test_video_file_exists_and_readable() {
    let path = Path::new(VIDEO_PATH);
    assert!(path.exists(), "视频文件应存在: {}", VIDEO_PATH);
    assert!(path.is_file(), "视频文件应是文件，不是目录");

    let metadata = std::fs::metadata(path).expect("读取视频文件元数据失败");
    assert!(
        metadata.len() > 1_000_000,
        "视频文件应大于 1MB，实际: {} bytes",
        metadata.len()
    );
    eprintln!("视频文件大小: {} MB", metadata.len() as f64 / 1_048_576.0);
}

/// 测试 2: 字幕文件存在且可读
#[test]
fn test_subtitle_file_exists_and_readable() {
    let path = Path::new(SUBTITLE_PATH);
    assert!(path.exists(), "字幕文件应存在: {}", SUBTITLE_PATH);
    assert!(path.is_file(), "字幕文件应是文件，不是目录");

    let metadata = std::fs::metadata(path).expect("读取字幕文件元数据失败");
    assert!(
        metadata.len() > 100,
        "字幕文件应大于 100 bytes，实际: {} bytes",
        metadata.len()
    );
    eprintln!("字幕文件大小: {} bytes", metadata.len());
}

/// 测试 3: FFmpeg/FFprobe 二进制可用（通过系统 PATH）
#[test]
fn test_ffmpeg_and_ffprobe_available() {
    let ffmpeg_check = Command::new("ffmpeg").arg("-version").output();
    assert!(ffmpeg_check.is_ok(), "应能启动 ffmpeg");
    let ffmpeg_out = ffmpeg_check.unwrap();
    assert!(ffmpeg_out.status.success(), "ffmpeg -version 应成功退出");
    let version_line = String::from_utf8_lossy(&ffmpeg_out.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();
    eprintln!("FFmpeg: {}", version_line);

    let ffprobe_check = Command::new("ffprobe").arg("-version").output();
    assert!(ffprobe_check.is_ok(), "应能启动 ffprobe");
    let ffprobe_out = ffprobe_check.unwrap();
    assert!(ffprobe_out.status.success(), "ffprobe -version 应成功退出");
    let fp_version = String::from_utf8_lossy(&ffprobe_out.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();
    eprintln!("FFprobe: {}", fp_version);
}

/// 测试 4: 用 ffprobe 探测视频信息
#[test]
fn test_video_probe() {
    let result = narratoai_core::ffmpeg::probe::probe_video(Path::new(VIDEO_PATH));
    assert!(result.is_ok(), "视频探测应成功: {:?}", result.err());

    let info = result.unwrap();
    eprintln!(
        "视频信息: duration={:.2}s, resolution={}x{}, codec={}, format={}",
        info.duration_secs, info.width, info.height, info.codec_name, info.format_name
    );

    assert!(info.duration_secs > 0.0, "视频时长应大于 0");
    assert!(info.width > 0, "视频宽度应大于 0");
    assert!(info.height > 0, "视频高度应大于 0");
}

/// 测试 5: 字幕文件解析
///
/// parse_subtitle_file 返回 (segments: Vec<SubtitleSegment>, encoding: String, raw_text: String)
#[test]
fn test_subtitle_parse() {
    let result = parse_subtitle_file(Path::new(SUBTITLE_PATH));
    assert!(result.is_ok(), "字幕解析应成功: {:?}", result.err());

    let (segments, encoding, _raw_text) = result.unwrap();
    assert!(!segments.is_empty(), "应解析出至少一条字幕");
    eprintln!("解析字幕: {} 条 (编码: {})", segments.len(), encoding);

    for (i, seg) in segments.iter().enumerate().take(3) {
        eprintln!(
            "  #{}: [{:.2}s - {:.2}s] {}",
            i + 1,
            seg.start_secs,
            seg.end_secs,
            seg.text.chars().take(40).collect::<String>()
        );
    }
}

/// 测试 6: 视频裁剪（clip）— 裁剪前 3 秒验证 FFmpeg 工作流
#[tokio::test]
async fn test_ffmpeg_clip_video_segment() {
    use narratoai_core::ffmpeg::command::run_ffmpeg;

    // 准备输出目录
    let output_dir = Path::new(TEST_OUTPUT_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("创建输出目录失败");
    }
    let output_path = output_dir.join("clip_test.mp4");

    // 用 FFmpeg 裁剪前 3 秒（-i 在前确保准确寻址）
    let video_path = Path::new(VIDEO_PATH).to_string_lossy().to_string();
    let output_str = output_path.to_string_lossy().to_string();

    let result = run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-i")
            .arg(&video_path)
            .arg("-ss")
            .arg("0")
            .arg("-t")
            .arg("3")
            .arg("-c")
            .arg("copy")
            .arg("-y")
            .output(&output_str);

        let mut child = cmd
            .spawn()
            .map_err(|e| narratoai_core::error::FFmpegError::SpawnFailed(e.to_string()))?;

        let iter = match child.iter() {
            Ok(iter) => iter,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(narratoai_core::error::FFmpegError::SpawnFailed(
                    e.to_string(),
                ));
            }
        };

        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                eprintln!("FFmpeg error: {}", e);
            }
        }

        let status = child
            .wait()
            .map_err(|e| narratoai_core::error::FFmpegError::ExecutionError(e.to_string()))?;

        if !status.success() {
            return Err(narratoai_core::error::FFmpegError::ExecutionError(format!(
                "FFmpeg exited with code {:?}",
                status.code()
            )));
        }
        Ok(())
    })
    .await;

    assert!(result.is_ok(), "FFmpeg 裁剪应成功: {:?}", result.err());
    assert!(output_path.exists(), "输出文件应存在: {:?}", output_path);

    let file_size = std::fs::metadata(&output_path)
        .expect("读取输出文件元数据失败")
        .len();
    assert!(
        file_size > 10_000,
        "裁剪输出应大于 10KB，实际: {} bytes",
        file_size
    );
    eprintln!("裁剪输出: {:?} ({} bytes)", output_path, file_size);
}
