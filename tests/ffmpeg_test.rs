use std::path::PathBuf;
use std::process::Command;

use narratoai_core::ffmpeg::command::clip_video;
use narratoai_core::ffmpeg::hwaccel::{detect_hw_encoders, get_profile, HwAccelProfile, recommend_profile};
use narratoai_core::ffmpeg::probe::probe_video;
use tempfile::TempDir;

/// 获取 ffmpeg 二进制路径。
///
/// 优先检测系统 PATH 上的 ffmpeg，然后检查 ffmpeg-sidecar 缓存路径。
/// 两个都不可用时 panic 提示用户安装。
/// 不调用 auto_download() 以避免网络阻塞。
fn find_ffmpeg() -> PathBuf {
    // 1. 先检查 PATH
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return PathBuf::from("ffmpeg");
        }
    }

    // 2. 检查 ffmpeg-sidecar 缓存路径
    let cached = ffmpeg_sidecar::paths::ffmpeg_path();
    if cached.exists() {
        return cached;
    }

    panic!(
        "FFmpeg 不可用。请安装 FFmpeg 并确保在 PATH 上，\n\
         或运行以下命令让 ffmpeg-sidecar 自动下载:\n\
         use ffmpeg_sidecar::download::auto_download;\n\
         auto_download().unwrap();"
    );
}

/// 创建用于测试的 2 秒视频文件
///
/// 使用 ffmpeg 内建 lavfi 源生成视频和音频，不依赖外部输入文件。
/// 返回 (TempDir, 视频文件路径) — TempDir 在测试结束时自动清理。
fn create_test_video() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("创建临时目录失败");
    let output = dir.path().join("test_input.mp4");
    let ffmpeg = find_ffmpeg();

    let status = Command::new(&ffmpeg)
        .args([
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=2:size=320x240:rate=25",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=2",
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-y",
            &output.to_string_lossy(),
        ])
        .status()
        .expect("启动 FFmpeg 进程失败");

    assert!(
        status.success(),
        "FFmpeg 生成测试视频命令失败。命令: ffmpeg -f lavfi -i testsrc=duration=2:size=320x240:rate=25 \
         -f lavfi -i sine=frequency=440:duration=2 -c:v libx264 -c:a aac -y {}",
        output.display()
    );

    (dir, output)
}

/// 使用 ffprobe 探测测试视频，验证返回有效的 VideoInfo
#[test]
fn test_probe_system_video() {
    let (_dir, video_path) = create_test_video();

    let info = probe_video(&video_path).expect("probe_video 应成功返回 VideoInfo");

    assert!(
        info.duration_secs > 0.0,
        "视频时长应大于 0，实际: {}",
        info.duration_secs
    );
    assert!(info.width > 0, "宽度应大于 0，实际: {}", info.width);
    assert!(info.height > 0, "高度应大于 0，实际: {}", info.height);
    assert!(
        !info.codec_name.is_empty(),
        "编码器名称不应为空"
    );
    assert!(
        info.format_name.contains("mp4")
            || info.format_name.contains("mov")
            || info.format_name.contains("matroska"),
        "格式应包含 mp4/mov/matroska，实际: {}",
        info.format_name
    );
}

/// 通过 clip_video 异步裁剪视频，验证输出文件有效
#[tokio::test]
async fn test_clip_video_async() {
    let (_dir, video_path) = create_test_video();
    let output_dir = TempDir::new().expect("创建输出目录失败");
    let output_path = output_dir.path().join("test_clip.mp4");

    let result = clip_video(&video_path, &output_path, 0.0, 1.0, None).await;
    assert!(
        result.is_ok(),
        "clip_video 应成功完成，错误: {:?}",
        result.err()
    );

    assert!(
        output_path.exists(),
        "裁剪后的输出文件应存在: {}",
        output_path.display()
    );
    let file_size = output_path
        .metadata()
        .expect("获取文件元数据失败")
        .len();
    assert!(
        file_size > 0,
        "裁剪后的输出文件大小应大于 0，实际: {}",
        file_size
    );
}

/// 检测系统可用硬件编码器
///
/// 验证 detect_hw_encoders 至少返回空列表（而不是错误）。
/// 如果系统有硬件编码器，列表应包含已知编码器名称。
#[test]
fn test_hw_accel_detection() {
    let result = detect_hw_encoders();
    assert!(result.is_ok(), "detect_hw_encoders 不应失败");
    let encoders = result.unwrap();
    for encoder in &encoders {
        assert!(!encoder.is_empty(), "编码器名称不应为空");
    }
}

/// recommend_profile 返回 5 个 HwAccelProfile 变体之一且 profile 信息有效
#[test]
fn test_recommend_profile() {
    let profile = recommend_profile();

    // 验证返回值是 5 个 HwAccelProfile 变体之一
    let valid = matches!(
        profile,
        HwAccelProfile::HighPerformance
            | HwAccelProfile::Compatibility
            | HwAccelProfile::WindowsNvidia
            | HwAccelProfile::MacosVideotoolbox
            | HwAccelProfile::UniversalSoftware
    );
    assert!(valid, "recommend_profile 应返回有效变体，实际: {:?}", profile);

    // 验证 get_profile 返回有效数据
    let info = get_profile(profile);
    assert!(
        info.is_some(),
        "推荐 profile {:?} 应有对应信息",
        profile
    );
    let info = info.unwrap();
    assert!(!info.name.is_empty(), "Profile name 不应为空");
    assert!(!info.encoder.is_empty(), "Profile encoder 不应为空");
}

/// 验证所有 5 个 HwAccelProfile 变体的 profile 数据完整
#[test]
fn test_profiles_completeness() {
    let profiles = [
        HwAccelProfile::HighPerformance,
        HwAccelProfile::Compatibility,
        HwAccelProfile::WindowsNvidia,
        HwAccelProfile::MacosVideotoolbox,
        HwAccelProfile::UniversalSoftware,
    ];

    for profile in profiles {
        let info =
            get_profile(profile).unwrap_or_else(|| panic!("Profile {:?} 应有数据", profile));
        assert!(
            !info.name.is_empty(),
            "{:?} name 不应为空",
            profile
        );
        assert!(
            !info.encoder.is_empty(),
            "{:?} encoder 不应为空",
            profile
        );
        assert_eq!(
            info.pixel_format, "yuv420p",
            "{:?} pixel_format 应为 yuv420p，实际: {}",
            profile, info.pixel_format
        );
        assert!(
            info.compatibility_level >= 1 && info.compatibility_level <= 5,
            "{:?} compatibility_level 应在 1-5 范围，实际: {}",
            profile,
            info.compatibility_level
        );
    }
}
