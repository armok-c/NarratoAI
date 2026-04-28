use std::path::Path;
use std::sync::Arc;

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;

use crate::error::FFmpegError;

/// FFmpeg 进度回调类型
/// 参数: Option<f64> 表示进度百分比（None 表示未知），&str 表示步骤描述
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send + Sync>;

/// run_ffmpeg 的通用 spawn_blocking 包装器（后续 Phase 6 可复用）
pub async fn run_ffmpeg<F, R>(blocking_fn: F) -> Result<R, FFmpegError>
where
    F: FnOnce() -> Result<R, FFmpegError> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(blocking_fn)
        .await
        .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
}

/// 解析 FFmpeg 时间格式 "HH:MM:SS.mm" 为秒数
fn parse_time_to_secs(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        3 => {
            let hours: f64 = parts[0].parse().ok()?;
            let minutes: f64 = parts[1].parse().ok()?;
            let secs: f64 = parts[2].parse().ok()?;
            Some(hours * 3600.0 + minutes * 60.0 + secs)
        }
        2 => {
            let minutes: f64 = parts[0].parse().ok()?;
            let secs: f64 = parts[1].parse().ok()?;
            Some(minutes * 60.0 + secs)
        }
        _ => None,
    }
}

/// 异步视频裁剪
///
/// 通过 ffmpeg-sidecar 执行 FFmpeg 视频裁剪操作，通过 spawn_blocking 异步化。
/// 支持进度回调（per D-15）。
pub async fn clip_video(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    progress: Option<ProgressCallback>,
) -> Result<(), FFmpegError> {
    let input_path = input.to_string_lossy().to_string();
    let output_path = output.to_string_lossy().to_string();
    let progress = progress.map(Arc::new);

    tokio::task::spawn_blocking(move || {
        let mut cmd = FfmpegCommand::new();
        cmd.seek(start.to_string())
            .input(&input_path)
            .duration(duration.to_string())
            .output(&output_path)
            .overwrite();

        let mut child = cmd
            .spawn()
            .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

        let iter = child
            .iter()
            .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

        for event in iter {
            match event {
                FfmpegEvent::Progress(p) => {
                    if let Some(ref cb) = progress {
                        let secs = parse_time_to_secs(&p.time);
                        cb(secs, "视频裁剪中");
                    }
                }
                FfmpegEvent::Error(e) => {
                    tracing::error!("FFmpeg error: {}", e);
                }
                _ => {}
            }
        }

        // 检查 ffmpeg 进程退出状态
        let status = child
            .wait()
            .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?;

        if !status.success() {
            return Err(FFmpegError::ExecutionError(format!(
                "FFmpeg 进程退出码: {:?}",
                status.code()
            )));
        }

        Ok(())
    })
    .await
    .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// 验证 ProgressCallback 类型可以被 Box 化且 Send + Sync
    #[test]
    fn test_progress_callback_type() {
        let cb: ProgressCallback = Box::new(|pct, desc| {
            let _ = (pct, desc);
        });
        // 验证可以调用
        cb(Some(0.5), "测试进度");
        cb(None, "未知进度");
    }

    /// 对不存在的输入文件调用 clip_video 应返回 FFmpegError
    #[tokio::test]
    async fn test_clip_video_invalid_input() {
        let input = Path::new("/tmp/nonexistent_video_12345.mp4");
        let output = Path::new("/tmp/nonexistent_output.mp4");

        let result = clip_video(input, output, 0.0, 10.0, None).await;

        assert!(
            result.is_err(),
            "不存在的输入文件应该返回 Err"
        );
    }

    /// 验证 spawn_blocking 不阻塞 tokio runtime
    /// 创建两个并发的 spawn_blocking 任务（各 sleep 100ms），验证并行执行
    #[tokio::test]
    async fn test_spawn_blocking_non_blocking() {
        let start = Instant::now();

        let task1 = tokio::task::spawn_blocking(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            1
        });
        let task2 = tokio::task::spawn_blocking(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            2
        });

        let (r1, r2) = tokio::join!(task1, task2);
        let elapsed = start.elapsed();

        assert!(r1.is_ok(), "Task 1 应该成功完成");
        assert!(r2.is_ok(), "Task 2 应该成功完成");
        assert_eq!(r1.unwrap(), 1);
        assert_eq!(r2.unwrap(), 2);

        // 如果 spawn_blocking 正常工作，两个任务应并行执行
        // 总耗时应显著小于 200ms（串行执行的理论值）
        assert!(
            elapsed.as_millis() < 180,
            "spawn_blocking 任务应并行执行, 预期 < 180ms, 实际: {:?}",
            elapsed
        );
    }
}
