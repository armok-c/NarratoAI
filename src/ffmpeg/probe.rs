use std::path::Path;

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
/// 使用 `ffmpeg_sidecar::ffprobe_path()` 获取 ffprobe 二进制路径（per D-12），
/// 通过 `std::process::Command` 调用 ffprobe 解析 JSON 输出。
/// 使用 std::process::Command 而非 ffmpeg-sidecar API 是合理的（WARNING 1 说明），
/// 因为 ffmpeg-sidecar 不提供完整的 ffprobe 封装。
pub fn probe_video(_path: &Path) -> Result<VideoInfo, FFmpegError> {
    // RED 阶段: stub 返回错误
    unimplemented!("GREEN 阶段实现")
}

/// 通过 ffprobe 获取视频信息（异步，spawn_blocking 包装）
pub async fn probe_video_async(_path: &Path) -> Result<VideoInfo, FFmpegError> {
    // RED 阶段: stub 返回错误
    unimplemented!("GREEN 阶段实现")
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
