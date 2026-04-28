use std::path::Path;

use crate::error::FFmpegError;

/// FFmpeg 进度回调类型
/// 参数: Option<f64> 表示进度百分比（None 表示未知），&str 表示步骤描述
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send + Sync>;

/// run_ffmpeg 的通用 spawn_blocking 包装器（后续 Phase 6 可复用）
pub async fn run_ffmpeg<F, R>(_blocking_fn: F) -> Result<R, FFmpegError>
where
    F: FnOnce() -> Result<R, FFmpegError> + Send + 'static,
    R: Send + 'static,
{
    // RED 阶段: 不使用 spawn_blocking, 直接执行（失败测试）
    // 这将导致并发测试失败，因为任务是串行的
    unimplemented!("GREEN 阶段实现")
}

/// 异步视频裁剪
pub async fn clip_video(
    _input: &Path,
    _output: &Path,
    _start: f64,
    _duration: f64,
    _progress: Option<ProgressCallback>,
) -> Result<(), FFmpegError> {
    // RED 阶段: 返回一个错误类型，测试期望 SpawnFailed
    unimplemented!("GREEN 阶段实现")
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

    /// 对不存在的输入文件调用 clip_video 应返回 SpawnFailed 错误
    #[tokio::test]
    async fn test_clip_video_invalid_input() {
        let input = Path::new("/tmp/nonexistent_video_12345.mp4");
        let output = Path::new("/tmp/nonexistent_output.mp4");

        let result = clip_video(input, output, 0.0, 10.0, None).await;

        assert!(
            result.is_err(),
            "不存在的输入文件应该返回 Err"
        );
        match result {
            Err(FFmpegError::SpawnFailed(_)) => {} // 预期
            Err(other) => panic!(
                "预期 SpawnFailed 错误, 但得到: {:?}",
                other
            ),
            Ok(_) => panic!("预期错误, 但得到 Ok"),
        }
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
