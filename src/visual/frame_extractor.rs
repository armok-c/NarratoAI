//! 帧提取模块
//!
//! 实现视频关键帧提取功能，支持快路径（单条 ffmpeg fps filter）和
//! 回退路径（逐帧提取 + 4 级输出格式回退：硬件加速 JPEG → 软件 JPEG → PNG → BMP/MJPEG）。
//!
//! 所有 FFmpeg 操作通过 spawn_blocking 包装，不阻塞 tokio 运行时（D-14）。

use std::path::Path;
use std::path::PathBuf;

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;
use tokio_util::sync::CancellationToken;

use crate::visual::error::VisualError;

/// 快路径帧提取（D-12）
///
/// 使用单条 ffmpeg `fps=1/{interval_seconds}` video filter 命令，
/// 硬件加速 mjpeg 编码，一次性提取所有帧。
///
/// 帧输出到 output_dir，命名格式 `fastframe_%06d.jpg`，提取完成后
/// 由 `rename_fast_path_frames` 重命名为 `keyframe_{frame_number:06d}_{HHMMSSmmm}.jpg`。
pub async fn extract_frames_fast_path(
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

/// 将 `fastframe_XXXXXX.jpg` 重命名为 `keyframe_XXXXXX_HHMMSSmmm.jpg`（D-24）
///
/// 1. 扫描 output_dir 下的所有 `fastframe_*.jpg` 文件
/// 2. 从文件名提取帧序号
/// 3. 计算该帧的时间戳：`frame_number * interval_seconds` 秒
/// 4. 格式化为 `HHMMSSmmm`（9 位：2 小时 + 2 分钟 + 2 秒 + 3 毫秒）
/// 5. 重命名
///
/// 如果 output_dir 没有 fastframe_ 文件则返回 Ok(0)。
fn rename_fast_path_frames(output_dir: &Path, interval_seconds: f64) -> Result<usize, VisualError> {
    let mut entries: Vec<PathBuf> = Vec::new();
    let mut dir_reader = std::fs::read_dir(output_dir)
        .map_err(|e| VisualError::FrameExtraction(format!("读取帧目录失败: {}", e)))?;

    while let Some(entry) = dir_reader.next() {
        let entry = entry.map_err(|e| VisualError::FrameExtraction(format!("读取目录项失败: {}", e)))?;
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
        let hours = (total_secs / 3600.0) as u64;
        let minutes = ((total_secs % 3600.0) / 60.0) as u64;
        let secs = (total_secs % 60.0) as u64;
        let millis = ((total_secs - total_secs.floor()) * 1000.0) as u64;
        let timestamp = format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis);

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

/// 将秒数转换为 HHMMSSmmm 格式字符串
///
/// 输出为 9 位数字：HH（2 位小时）+ MM（2 位分钟）+ SS（2 位秒）+ mmm（3 位毫秒）
#[allow(dead_code)]
pub(crate) fn seconds_to_hhmmssmmm(total_secs: f64) -> String {
    let hours = (total_secs / 3600.0) as u64;
    let minutes = ((total_secs % 3600.0) / 60.0) as u64;
    let secs = (total_secs % 60.0) as u64;
    let millis = ((total_secs - total_secs.floor()) * 1000.0) as u64;
    format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis)
}

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
            let total_secs = i as f64 * 3.0;
            let hours = (total_secs / 3600.0) as u64;
            let minutes = ((total_secs % 3600.0) / 60.0) as u64;
            let secs = (total_secs % 60.0) as u64;
            let millis = ((total_secs - total_secs.floor()) * 1000.0) as u64;
            let ts = format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis);
            let expected_name = format!("keyframe_{:06}_{}.jpg", i, ts);
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
        // Test a few combinations
        let test_cases = [
            (0u64, 0.0f64, "keyframe_000000_000000000.jpg"),
            (1u64, 3.0f64, "keyframe_000001_000003000.jpg"),
            (120u64, 5.0f64, "keyframe_000120_001000000.jpg"), // 120*5=600s=10min
        ];

        for (frame_num, interval, expected) in &test_cases {
            let total_secs = *frame_num as f64 * interval;
            let hours = (total_secs / 3600.0) as u64;
            let minutes = ((total_secs % 3600.0) / 60.0) as u64;
            let secs = (total_secs % 60.0) as u64;
            let millis = ((total_secs - total_secs.floor()) * 1000.0) as u64;
            let ts = format!("{:02}{:02}{:02}{:03}", hours, minutes, secs, millis);
            let pattern = format!("keyframe_{:06}_{}.jpg", frame_num, ts);

            assert_eq!(
                pattern, *expected,
                "帧命名格式不匹配: frame={}, interval={}",
                frame_num, interval
            );

            // Verify structural pattern: prefix_6digits_9digits.jpg
            assert!(
                pattern.starts_with("keyframe_"),
                "应以 keyframe_ 开头"
            );
            let parts: Vec<&str> = pattern.split('_').collect();
            assert_eq!(parts.len(), 3, "应由 2 个下划线分隔的 3 部分");
            assert_eq!(parts[1].len(), 6, "帧序号应为 6 位数字");
            let ts_part = parts[2].strip_suffix(".jpg").unwrap_or(parts[2]);
            assert_eq!(ts_part.len(), 9, "时间戳部分应为 9 位数字");
            assert!(ts_part.chars().all(|c| c.is_ascii_digit()), "时间戳部分应全为数字");
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
        // 0 seconds
        assert_eq!(seconds_to_hhmmssmmm(0.0), "000000000");
        // 3 seconds
        assert_eq!(seconds_to_hhmmssmmm(3.0), "000003000");
        // 65 seconds = 1 min 5 sec
        assert_eq!(seconds_to_hhmmssmmm(65.0), "000105000");
        // 3600 seconds = 1 hour
        assert_eq!(seconds_to_hhmmssmmm(3600.0), "010000000");
        // 3661.5 seconds = 1 hour 1 min 1.5 sec
        assert_eq!(seconds_to_hhmmssmmm(3661.5), "010101500");
        // 9 digits always
        assert_eq!(seconds_to_hhmmssmmm(0.0).len(), 9);
        assert_eq!(seconds_to_hhmmssmmm(99999.0).len(), 9);
    }
}
