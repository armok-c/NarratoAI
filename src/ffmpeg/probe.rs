use std::path::Path;
use std::process::Command;

use crate::error::FFmpegError;

/// 视频基本信息（从 ffprobe 解析）
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub codec_name: String,
    pub format_name: String,
}

/// 通过 ffprobe 获取视频信息（同步）
///
/// 使用 `ffmpeg_sidecar::ffprobe::ffprobe_path()` 获取 ffprobe 二进制路径（per D-12），
/// 通过 `std::process::Command` 调用 ffprobe 解析 JSON 输出。
/// 使用 std::process::Command 而非 ffmpeg-sidecar API 是合理的（WARNING 1 说明），
/// 因为 ffmpeg-sidecar 不提供完整的 ffprobe 封装。
pub fn probe_video(path: &Path) -> Result<VideoInfo, FFmpegError> {
    let ffprobe_bin = ffmpeg_sidecar::ffprobe::ffprobe_path();

    let output = match Command::new(&ffprobe_bin)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path.as_os_str())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(FFmpegError::BinaryNotFound);
            }
            return Err(FFmpegError::SpawnFailed(e.to_string()));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FFmpegError::ExecutionError(format!(
            "ffprobe exited with code {:?}: {}",
            output.status.code(),
            stderr,
        )));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| FFmpegError::OutputParseError(e.to_string()))?;

    let duration_secs = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| FFmpegError::OutputParseError(
            "Missing or invalid duration in ffprobe output".into()
        ))?;

    let format_name = json["format"]["format_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    // 从 streams 数组中查找第一个 video stream
    let video_stream = json["streams"].as_array().and_then(|streams| {
        streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
    });

    let (width, height, codec_name) = match video_stream {
        Some(stream) => (
            stream["width"].as_u64().unwrap_or(0) as u32,
            stream["height"].as_u64().unwrap_or(0) as u32,
            stream["codec_name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        ),
        None => (0, 0, "unknown".to_string()),
    };

    if width == 0 || height == 0 {
        return Err(FFmpegError::OutputParseError(
            "No video stream found in ffprobe output".into(),
        ));
    }

    Ok(VideoInfo {
        duration_secs,
        width,
        height,
        codec_name,
        format_name,
    })
}

/// 通过 ffprobe 获取视频信息（异步，spawn_blocking 包装）
pub async fn probe_video_async(path: &Path) -> Result<VideoInfo, FFmpegError> {
    let path_buf = path.to_path_buf();
    tokio::task::spawn_blocking(move || probe_video(&path_buf))
        .await
        .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 VideoInfo 结构体 Debug + Clone 正确
    #[test]
    fn test_video_info_debug_clone() {
        let info = VideoInfo {
            duration_secs: 120.5,
            width: 1920,
            height: 1080,
            codec_name: "h264".to_string(),
            format_name: "mp4".to_string(),
        };

        // Debug
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("h264"), "Debug should contain codec name");

        // Clone
        let cloned = info.clone();
        assert_eq!(cloned.duration_secs, 120.5);
        assert_eq!(cloned.width, 1920);
        assert_eq!(cloned.height, 1080);
        assert_eq!(cloned.codec_name, "h264");
    }
}
