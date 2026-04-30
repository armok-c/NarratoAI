//! 帧提取模块
//!
//! 实现视频关键帧提取功能，支持快路径（单条 ffmpeg fps filter）和
//! 回退路径（逐帧提取 + 4 级输出格式回退：硬件加速 JPEG → 软件 JPEG → PNG → BMP/MJPEG）。
//!
//! 所有 FFmpeg 操作通过 spawn_blocking 包装，不阻塞 tokio 运行时（D-14）。

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;
use tokio_util::sync::CancellationToken;

use crate::ffmpeg::command::ProgressCallback;
use crate::visual::error::VisualError;

// ---------------------------------------------------------------------------
// 公共入口
// ---------------------------------------------------------------------------

/// 帧提取主入口（D-11/D-12）
///
/// 先尝试快路径（单条 ffmpeg fps filter 命令），如果快路径返回 0 帧
/// 则自动清理快路径残留文件并回退到逐帧提取（4 级输出格式回退）。
///
/// `interval_seconds`: 帧提取间隔，默认 3.0 秒
/// `quality`: 输出 JPEG 质量，`q:v` 值范围 2-31，越小质量越高，默认 5
/// `output_dir`: 帧文件输出目录（不存在时自动创建）
/// `progress`: 进度回调
/// `cancel`: 取消令牌
pub async fn extract_frames(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: Option<u32>,
    progress: Option<ProgressCallback>,
    cancel: Option<CancellationToken>,
) -> Result<usize, VisualError> {
    let quality_val = quality.unwrap_or(5);
    if quality_val < 2 || quality_val > 31 {
        return Err(VisualError::FrameExtraction(
            format!("JPEG quality 值 ({}) 超出有效范围 2-31", quality_val)
        ));
    }

    if interval_seconds.is_nan() || interval_seconds <= 0.0 {
        return Err(VisualError::FrameExtraction("帧提取间隔必须 > 0".into()));
    }

    // Create output dir if not exists
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| VisualError::FrameExtraction(format!("创建输出目录失败: {}", e)))?;
    }

    // Fast path: single ffmpeg fps filter command
    match extract_frames_fast_path(
        video_path,
        output_dir,
        interval_seconds,
        quality_val,
        cancel.clone(),
    )
    .await
    {
        Ok(count) if count > 0 => {
            if let Some(ref cb) = progress {
                cb(Some(1.0), "帧提取完成");
            }
            return Ok(count);
        }
        _ => {
            // Clean up fast-path artifacts
            cleanup_fast_path_files(output_dir);
            // Fallback to per-frame extraction
            extract_frames_fallback(
                video_path,
                output_dir,
                interval_seconds,
                quality_val,
                progress,
                cancel,
            )
            .await
        }
    }
}

// ---------------------------------------------------------------------------
// 快路径
// ---------------------------------------------------------------------------

/// 快路径帧提取（D-12）
///
/// 使用单条 ffmpeg `fps=1/{interval_seconds}` video filter 命令，
/// 硬件加速 mjpeg 编码，一次性提取所有帧。
///
/// 帧输出到 output_dir，命名格式 `fastframe_%06d.jpg`，提取完成后
/// 由 `rename_fast_path_frames` 重命名为 `keyframe_{frame_number:06}_{HHMMSSmmm}.jpg`。
pub(crate) async fn extract_frames_fast_path(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: u32,
    cancel: Option<CancellationToken>,
) -> Result<usize, VisualError> {
    let video = video_path.to_path_buf();
    let output = output_dir.to_path_buf();
    let cancel = cancel.unwrap_or_else(CancellationToken::new);

    tokio::task::spawn_blocking(move || -> Result<usize, VisualError> {
        let pattern = output.join("fastframe_%06d.jpg");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| VisualError::FrameExtraction("帧输出路径无效".into()))?
            .to_string();

        let video_str = video
            .to_str()
            .ok_or_else(|| VisualError::FrameExtraction("视频路径无效".into()))?
            .to_string();

        let mut cmd = FfmpegCommand::new();
        cmd.arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .input(&video_str)
            .arg("-vf")
            .arg(format!("fps=1/{}", interval_seconds))
            .codec_video("mjpeg")
            .args(["-q:v", &quality.to_string()])
            .arg("-start_number")
            .arg("0")
            .overwrite()
            .output(&pattern_str);

        let mut child = cmd
            .spawn()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg 启动失败: {}", e)))?;

        let iter = child
            .iter()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg 事件迭代失败: {}", e)))?;

        for event in iter {
            if cancel.is_cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                return Err(VisualError::FrameExtraction("帧提取被取消".into()));
            }
            if let FfmpegEvent::Error(e) = event {
                tracing::warn!("FFmpeg 快路径错误: {}", e);
            }
        }

        let _ = child.wait();
        rename_fast_path_frames(&output, interval_seconds)
    })
    .await
    .map_err(|e| VisualError::FrameExtraction(format!("任务崩溃: {}", e)))?
}

// ---------------------------------------------------------------------------
// 回退路径（逐帧提取 + 4 级回退）
// ---------------------------------------------------------------------------

/// 逐帧提取回退路径
///
/// 当快路径返回 0 帧或失败时使用。通过 ffprobe 获取视频时长，
/// 计算总帧数，然后逐帧调用 `extract_single_frame` 进行 4 级回退提取。
///
/// 单帧失败不会中止整体流程（D-14: 收集错误继续执行）。
async fn extract_frames_fallback(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: u32,
    progress: Option<ProgressCallback>,
    cancel: Option<CancellationToken>,
) -> Result<usize, VisualError> {
    let video = video_path.to_path_buf();
    let output = output_dir.to_path_buf();
    let cancel = cancel.unwrap_or_else(CancellationToken::new);
    let progress = progress.map(Arc::new);

    tokio::task::spawn_blocking(move || -> Result<usize, VisualError> {
        let video_str = video
            .to_str()
            .ok_or_else(|| VisualError::FrameExtraction("视频路径无效".into()))?;

        // Get video duration via ffprobe
        let duration = get_video_duration(video_str)?;
        if duration <= 0.0 {
            return Err(VisualError::FrameExtraction("视频时长为 0".into()));
        }

        let total_frames = (duration / interval_seconds).ceil() as usize;
        if total_frames == 0 {
            return Ok(0);
        }

        let mut extracted_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for i in 0..total_frames {
            if cancel.is_cancelled() {
                tracing::info!("帧提取被取消，已提取 {} 帧", extracted_count);
                return Ok(extracted_count);
            }

            let timestamp_secs = i as f64 * interval_seconds;
            let frame_name = format!(
                "keyframe_{:06}_{}.jpg",
                i,
                seconds_to_hhmmssmmm(timestamp_secs)
            );
            let frame_path = output.join(&frame_name);

            match extract_single_frame(video_str, &frame_path, timestamp_secs, quality) {
                Ok(_) => {
                    extracted_count += 1;
                }
                Err(e) => {
                    let err_msg = format!("帧 {} ({}s): {}", i, timestamp_secs, e);
                    tracing::warn!("{}", err_msg);
                    errors.push(err_msg);
                }
            }

            // Report progress
            if let Some(ref cb) = progress {
                let pct = (i as f64 + 1.0) / total_frames as f64;
                cb(
                    Some(pct.min(1.0)),
                    &format!("正在提取关键帧 ({}/{})", i + 1, total_frames),
                );
            }
        }

        if extracted_count == 0 && !errors.is_empty() {
            return Err(VisualError::FrameExtraction(format!(
                "所有帧提取均失败 ({} 个错误)",
                errors.len()
            )));
        }

        Ok(extracted_count)
    })
    .await
    .map_err(|e| VisualError::FrameExtraction(format!("任务崩溃: {}", e)))?
}

/// 单帧提取 — 4 级输出格式回退（D-10）
///
/// 按以下顺序尝试，成功后返回：
/// 1. 硬件加速 JPEG（`-hwaccel auto`）
/// 2. 软件解码 JPEG（`-avoid_negative_ts make_zero`）
/// 3. PNG 格式输出，再通过 image crate 转换为 JPEG
/// 4. BMP/MJPEG 格式输出，再通过 image crate 转换为 JPEG
fn extract_single_frame(
    video_path: &str,
    output_path: &Path,
    timestamp_secs: f64,
    quality: u32,
) -> Result<(), VisualError> {
    let timestamp = format!("{:.3}", timestamp_secs);
    let output_str = output_path
        .to_str()
        .ok_or_else(|| VisualError::FrameExtraction("输出路径无效".into()))?;

    // Level 1: Hardware accelerated JPEG
    let level1_ok = run_ffmpeg(&[
        "-hwaccel",
        "auto",
        "-ss",
        &timestamp,
        "-i",
        video_path,
        "-vframes",
        "1",
        "-q:v",
        &quality.to_string(),
        "-pix_fmt",
        "yuv420p",
        "-y",
        output_str,
    ]);
    if level1_ok && file_is_valid(output_path) {
        return Ok(());
    }

    // Level 2: Software JPEG
    let level2_ok = run_ffmpeg(&[
        "-ss",
        &timestamp,
        "-i",
        video_path,
        "-vframes",
        "1",
        "-q:v",
        &quality.to_string(),
        "-pix_fmt",
        "yuv420p",
        "-avoid_negative_ts",
        "make_zero",
        "-y",
        output_str,
    ]);
    if level2_ok && file_is_valid(output_path) {
        return Ok(());
    }

    // Level 3: PNG -> JPEG conversion
    let png_path = output_path.with_extension("png");
    let png_str = png_path
        .to_str()
        .ok_or_else(|| VisualError::FrameExtraction("PNG 输出路径无效".into()))?;
    let level3_ok = run_ffmpeg(&[
        "-ss",
        &timestamp,
        "-i",
        video_path,
        "-vframes",
        "1",
        "-pix_fmt",
        "yuv420p",
        "-f",
        "image2",
        "-y",
        png_str,
    ]);
    if level3_ok && file_is_valid(&png_path) {
        match convert_image_to_jpeg(&png_path, output_path, quality) {
            Ok(_) => {
                let _ = std::fs::remove_file(&png_path);
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("PNG 转 JPEG 失败: {}", e);
                let _ = std::fs::remove_file(&png_path);
            }
        }
    }

    // Level 4: BMP -> JPEG conversion
    let bmp_path = output_path.with_extension("bmp");
    let bmp_str = bmp_path
        .to_str()
        .ok_or_else(|| VisualError::FrameExtraction("BMP 输出路径无效".into()))?;
    let level4_ok = run_ffmpeg(&[
        "-ss",
        &timestamp,
        "-i",
        video_path,
        "-vframes",
        "1",
        "-f",
        "image2",
        "-pix_fmt",
        "rgb24",
        "-y",
        bmp_str,
    ]);
    if level4_ok && file_is_valid(&bmp_path) {
        match convert_image_to_jpeg(&bmp_path, output_path, quality) {
            Ok(_) => {
                let _ = std::fs::remove_file(&bmp_path);
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("BMP 转 JPEG 失败: {}", e);
                let _ = std::fs::remove_file(&bmp_path);
            }
        }
    }

    let _ = std::fs::remove_file(output_path);
    Err(VisualError::FrameExtraction(format!(
        "所有 4 级回退均失败 (timestamp={}s)",
        timestamp_secs
    )))
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 将 `fastframe_XXXXXX.jpg` 重命名为 `keyframe_XXXXXX_HHMMSSmmm.jpg`（D-24）
///
/// 如果 output_dir 没有 fastframe_ 文件则返回 Ok(0)。
fn rename_fast_path_frames(
    output_dir: &Path,
    interval_seconds: f64,
) -> Result<usize, VisualError> {
    let mut entries: Vec<PathBuf> = Vec::new();
    let mut dir_reader = std::fs::read_dir(output_dir)
        .map_err(|e| VisualError::FrameExtraction(format!("读取帧目录失败: {}", e)))?;

    while let Some(entry) = dir_reader.next() {
        let entry = entry
            .map_err(|e| VisualError::FrameExtraction(format!("读取目录项失败: {}", e)))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if file_name.starts_with("fastframe_") && file_name.ends_with(".jpg") {
            entries.push(path);
        }
    }

    if entries.is_empty() {
        return Ok(0);
    }

    // Sort by frame number (lexical sort works since frame number is zero-padded)
    entries.sort();

    let mut renamed_count = 0;
    for path in &entries {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Extract frame number from "fastframe_XXXXXX.jpg"
        let frame_number = file_name
            .strip_prefix("fastframe_")
            .and_then(|s| s.strip_suffix(".jpg"))
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| {
                VisualError::FrameExtraction(format!("无法从文件名解析帧序号: {}", file_name))
            })?;

        let total_secs = frame_number as f64 * interval_seconds;
        let timestamp = seconds_to_hhmmssmmm(total_secs);

        let new_name = format!("keyframe_{:06}_{}.jpg", frame_number, timestamp);
        let new_path = output_dir.join(&new_name);

        std::fs::rename(path, &new_path).map_err(|e| {
            VisualError::FrameExtraction(format!(
                "重命名失败 {} -> {}: {}",
                file_name, new_name, e
            ))
        })?;
        renamed_count += 1;
    }

    Ok(renamed_count)
}

/// 将秒数转换为 HHMMSSmmm 格式字符串（9 位数字）
#[allow(dead_code)]
pub(crate) fn seconds_to_hhmmssmmm(total_secs: f64) -> String {
    let hours = (total_secs / 3600.0) as u64;
    let minutes = ((total_secs % 3600.0) / 60.0) as u64;
    let secs = (total_secs % 60.0) as u64;
    let millis = ((total_secs - total_secs.floor()) * 1000.0) as u64;
    format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis)
}

/// 删除 output_dir 下的所有 `fastframe_*.jpg` 文件
fn cleanup_fast_path_files(output_dir: &Path) {
    if let Ok(mut reader) = std::fs::read_dir(output_dir) {
        while let Some(Ok(entry)) = reader.next() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if name.starts_with("fastframe_") {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// 通过 ffprobe 获取视频时长（秒）
fn get_video_duration(video_path: &str) -> Result<f64, VisualError> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            video_path,
        ])
        .output()
        .map_err(|e| {
            VisualError::FrameExtraction(format!("ffprobe 执行失败: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VisualError::FrameExtraction(format!(
            "ffprobe 返回错误: {}",
            stderr.trim()
        )));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str
        .parse::<f64>()
        .map_err(|e| VisualError::FrameExtraction(format!("解析视频时长失败: {}", e)))
}

/// 检查文件是否存在且大小 > 0
fn file_is_valid(path: &Path) -> bool {
    path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
}

/// 运行 ffmpeg 命令并检查退出状态
fn run_ffmpeg(args: &[&str]) -> bool {
    std::process::Command::new("ffmpeg")
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 使用 image crate 将任意格式图片转换为 JPEG
///
/// `ffmpeg_quality` 是 FFmpeg 的 `-q:v` 值（范围 2-31，越小质量越高），
/// 此函数将其映射到 image crate 的 JPEG quality（范围 1-100，越高越好）。
fn convert_image_to_jpeg(input: &Path, output: &Path, ffmpeg_quality: u32) -> Result<(), VisualError> {
    let img = image::open(input).map_err(|e| {
        VisualError::FrameExtraction(format!("图片解码失败: {}", e))
    })?;

    // Map FFmpeg quality (2-31, lower=better) to image crate quality (1-100, higher=better)
    let image_quality = ((31 - ffmpeg_quality.clamp(2, 31)) as f32 / 29.0 * 99.0 + 1.0) as u8;

    let file = std::fs::File::create(output).map_err(|e| {
        VisualError::FrameExtraction(format!("创建输出文件失败: {}", e))
    })?;
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, image_quality);
    img.write_with_encoder(encoder).map_err(|e| {
        VisualError::FrameExtraction(format!("JPEG 编码失败: {}", e))
    })
}

/// 从 fast frame 文件名解析帧序号
#[allow(dead_code)]
pub(crate) fn parse_frame_number_from_name(filename: &str) -> Option<u64> {
    filename
        .strip_prefix("fastframe_")
        .and_then(|s| s.strip_suffix(".jpg"))
        .and_then(|s| s.parse::<u64>().ok())
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 rename_fast_path_frames 将 fastframe_ 文件正确重命名为 keyframe_ 格式
    #[test]
    fn test_rename_fast_path_frames() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_rename");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        // Create mock fastframe files
        for i in 0..3 {
            let name = format!("fastframe_{:06}.jpg", i);
            let path = temp_dir.join(&name);
            std::fs::write(&path, b"mock frame data").expect("应能写入测试文件");
        }

        let result = rename_fast_path_frames(&temp_dir, 3.0).expect("应重命名成功");
        assert_eq!(result, 3, "应重命名 3 个文件");

        // Verify all renamed files exist with correct naming pattern
        for i in 0..3 {
            let expected_name =
                format!("keyframe_{:06}_{}.jpg", i, seconds_to_hhmmssmmm(i as f64 * 3.0));
            assert!(
                temp_dir.join(&expected_name).exists(),
                "应存在文件: {}",
                expected_name
            );
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 验证关键帧命名格式符合 `keyframe_\d{6}_\d{9}\.jpg`
    #[test]
    fn test_frame_naming_pattern() {
        let test_cases = [
            (0u64, 0.0f64, "keyframe_000000_000000000.jpg"),
            (1u64, 3.0f64, "keyframe_000001_000003000.jpg"),
            (120u64, 5.0f64, "keyframe_000120_001000000.jpg"), // 120*5=600s=10min
        ];

        for (frame_num, interval, expected) in &test_cases {
            let total_secs = *frame_num as f64 * interval;
            let pattern = format!(
                "keyframe_{:06}_{}.jpg",
                frame_num,
                seconds_to_hhmmssmmm(total_secs)
            );

            assert_eq!(
                pattern, *expected,
                "帧命名格式不匹配: frame={}, interval={}",
                frame_num, interval
            );

            // Verify structural pattern: prefix_6digits_9digits.jpg
            assert!(pattern.starts_with("keyframe_"), "应以 keyframe_ 开头");
            let parts: Vec<&str> = pattern.split('_').collect();
            assert_eq!(parts.len(), 3, "应由 2 个下划线分隔的 3 部分");
            assert_eq!(parts[1].len(), 6, "帧序号应为 6 位数字");
            let ts_part = parts[2].strip_suffix(".jpg").unwrap_or(parts[2]);
            assert_eq!(ts_part.len(), 9, "时间戳部分应为 9 位数字");
            assert!(
                ts_part.chars().all(|c| c.is_ascii_digit()),
                "时间戳部分应全为数字"
            );
        }
    }

    /// 空目录应返回 Ok(0)
    #[test]
    fn test_fast_path_noop_when_no_frames() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_noop");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        let result = rename_fast_path_frames(&temp_dir, 3.0).expect("空目录应成功");
        assert_eq!(result, 0, "空目录应返回 0");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 验证 seconds_to_hhmmssmmm 格式转换正确性
    #[test]
    fn test_seconds_to_hhmmssmmm_format() {
        assert_eq!(seconds_to_hhmmssmmm(0.0), "000000000");
        assert_eq!(seconds_to_hhmmssmmm(3.0), "000003000");
        assert_eq!(seconds_to_hhmmssmmm(65.0), "000105000"); // 1 min 5 sec
        assert_eq!(seconds_to_hhmmssmmm(3600.0), "010000000"); // 1 hour
        assert_eq!(seconds_to_hhmmssmmm(3661.5), "010101500"); // 1h 1m 1.5s
        assert_eq!(seconds_to_hhmmssmmm(0.0).len(), 9);
        assert_eq!(seconds_to_hhmmssmmm(99999.0).len(), 9);
    }

    /// 验证 parse_frame_number_from_name 正确解析帧序号
    #[test]
    fn test_parse_frame_number() {
        assert_eq!(parse_frame_number_from_name("fastframe_000000.jpg"), Some(0));
        assert_eq!(parse_frame_number_from_name("fastframe_000001.jpg"), Some(1));
        assert_eq!(parse_frame_number_from_name("fastframe_000120.jpg"), Some(120));
        assert_eq!(parse_frame_number_from_name("fastframe_999999.jpg"), Some(999999));
        // Invalid names
        assert_eq!(parse_frame_number_from_name("fastframe_abc.jpg"), None);
        assert_eq!(parse_frame_number_from_name("other_000000.jpg"), None);
        assert_eq!(parse_frame_number_from_name("fastframe_.jpg"), None);
    }

    /// 验证 output_dir_creation — extract_frames 创建 output_dir
    #[tokio::test]
    async fn test_output_dir_creation() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_dir_creation");
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Call extract_frames with a non-existent video path
        // The function should create the output dir before attempting extraction
        let video_path = temp_dir.join("nonexistent.mp4");
        let result = extract_frames(
            &video_path,
            &temp_dir,
            3.0,
            Some(5),
            None,
            Some(CancellationToken::new()),
        )
        .await;

        // Directory should exist (even though extraction will fail)
        assert!(temp_dir.exists(), "输出目录应被创建");
        // But extraction should fail (video doesn't exist)
        assert!(result.is_err(), "不存在的视频文件应返回错误");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 验证 cleanup_fast_path_files 删除 fastframe_ 文件
    #[test]
    fn test_cleanup_fast_path_files() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_cleanup");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        // Create some fastframe files and a normal file
        std::fs::write(temp_dir.join("fastframe_000000.jpg"), b"data").unwrap();
        std::fs::write(temp_dir.join("fastframe_000001.jpg"), b"data").unwrap();
        std::fs::write(temp_dir.join("normal_file.txt"), b"keep me").unwrap();

        cleanup_fast_path_files(&temp_dir);

        // fastframe files should be deleted
        assert!(!temp_dir.join("fastframe_000000.jpg").exists());
        assert!(!temp_dir.join("fastframe_000001.jpg").exists());
        // normal file should remain
        assert!(temp_dir.join("normal_file.txt").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 验证 get_video_duration 在不存在的视频上返回错误
    #[test]
    fn test_get_video_duration_fails_on_nonexistent() {
        let result = get_video_duration("/nonexistent/video.mp4");
        assert!(result.is_err(), "不存在的视频应返回错误");
    }

    /// 验证 file_is_valid 对空文件和不存在文件的处理
    #[test]
    fn test_file_is_valid() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_valid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        let existing = temp_dir.join("exists.jpg");
        std::fs::write(&existing, b"data").unwrap();
        assert!(file_is_valid(&existing));

        let empty_file = temp_dir.join("empty.jpg");
        std::fs::write(&empty_file, b"").unwrap();
        assert!(!file_is_valid(&empty_file));

        let nonexistent = temp_dir.join("nonexistent.jpg");
        assert!(!file_is_valid(&nonexistent));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
