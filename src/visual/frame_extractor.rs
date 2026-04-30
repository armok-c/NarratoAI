//! 帧提取模块
//!
//! 实现视频关键帧提取功能，支持快路径（单条 ffmpeg fps filter）和
//! 回退路径（逐帧提取 + 4 级输出格式回退：硬件加速 JPEG → 软件 JPEG → PNG → BMP/MJPEG）。
//!
//! 所有 FFmpeg 操作通过 spawn_blocking 包装，不阻塞 tokio 运行时（D-14）。

use std::path::Path;

use crate::visual::error::VisualError;

/// 快路径帧提取：单条 ffmpeg fps=1/{interval} filter 命令（D-12）
///
/// TODO: 将在 Task 2 实现
pub async fn extract_frames_fast_path(
    _video_path: &Path,
    _output_dir: &Path,
    _interval_seconds: f64,
    _quality: u32,
) -> Result<usize, VisualError> {
    todo!("将在 Task 2 实现")
}
