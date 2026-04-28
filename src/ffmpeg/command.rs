use std::path::Path;
use std::sync::Arc;

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;

use crate::error::FFmpegError;

/// FFmpeg 进度回调类型
/// 参数: Option<f64> 表示 0.0 到 1.0 的进度分数（None 表示未知），&str 表示步骤描述
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

        let iter = match child.iter() {
            Ok(iter) => iter,
            Err(e) => {
                // Kill the orphaned child process before returning error
                let _ = child.kill();
                let _ = child.wait();
                return Err(FFmpegError::SpawnFailed(e.to_string()));
            }
        };

        for event in iter {
            match event {
                FfmpegEvent::Progress(p) => {
                    if let Some(ref cb) = progress {
                        let secs = parse_time_to_secs(&p.time);
                        let fraction = secs.map(|s| if duration > 0.0 { s / duration } else { 0.0 });
                        cb(fraction, "视频裁剪中");
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
    use std::sync::Barrier;

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
        let temp_dir = std::env::temp_dir();
        let input = temp_dir.join("narratoai_test_nonexistent_video.mp4");
        let output = temp_dir.join("narratoai_test_nonexistent_output.mp4");

        let result = clip_video(&input, &output, 0.0, 10.0, None).await;

        assert!(
            result.is_err(),
            "不存在的输入文件应该返回 Err"
        );
    }

    /// 验证 spawn_blocking 不阻塞 tokio runtime
    /// 使用 Barrier 验证两个任务真正并行执行，而非依赖时序断言
    #[tokio::test]
    async fn test_spawn_blocking_non_blocking() {
        let barrier = Arc::new(Barrier::new(3)); // 2 tasks + main

        let b1 = barrier.clone();
        let task1 = tokio::task::spawn_blocking(move || {
            b1.wait();
            1
        });

        let b2 = barrier.clone();
        let task2 = tokio::task::spawn_blocking(move || {
            b2.wait();
            2
        });

        // Wait for both tasks to reach the barrier.
        // If spawn_blocking runs tasks serially (only 1 blocking thread),
        // task2 would never start (task1 holds the thread while blocked on barrier),
        // causing a deadlock. With concurrent execution, both reach the barrier.
        barrier.wait();

        let (r1, r2) = tokio::join!(task1, task2);

        assert!(r1.is_ok(), "Task 1 应该成功完成");
        assert!(r2.is_ok(), "Task 2 应该成功完成");
        assert_eq!(r1.unwrap(), 1);
        assert_eq!(r2.unwrap(), 2);
    }
}
