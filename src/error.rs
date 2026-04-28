use thiserror::Error;

/// 配置领域错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Watch error: {0}")]
    WatchError(String),
}

/// FFmpeg 领域错误
#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("Spawn failed: {0}")]
    SpawnFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Output parse error: {0}")]
    OutputParseError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("FFmpeg binary not found")]
    BinaryNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_io_error_message_chinese() {
        let err = ConfigError::IoError("test".into());
        let msg = err.to_string();
        assert!(msg.contains("配置文件读取失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_ffmpeg_error_spawn_failed_message_chinese() {
        let err = FFmpegError::SpawnFailed("test".into());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 进程启动失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_config_error_parse_error_message_chinese() {
        let err = ConfigError::ParseError("parse issue".into());
        let msg = err.to_string();
        assert!(msg.contains("配置文件解析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_config_error_validation_error_message_chinese() {
        let err = ConfigError::ValidationError("invalid".into());
        let msg = err.to_string();
        assert!(msg.contains("配置校验失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_config_error_not_found_message_chinese() {
        let err = ConfigError::NotFound { path: "/test/path".into() };
        let msg = err.to_string();
        assert!(msg.contains("配置文件不存在"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_config_error_watch_error_message_chinese() {
        let err = ConfigError::WatchError("watch issue".into());
        let msg = err.to_string();
        assert!(msg.contains("配置监听失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_ffmpeg_error_timeout_message_chinese() {
        let err = FFmpegError::Timeout("timed out".into());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 执行超时"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_ffmpeg_error_output_parse_message_chinese() {
        let err = FFmpegError::OutputParseError("bad output".into());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 输出解析错误"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_ffmpeg_error_execution_error_message_chinese() {
        let err = FFmpegError::ExecutionError("exec failed".into());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 执行错误"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_ffmpeg_error_binary_not_found_message_chinese() {
        let err = FFmpegError::BinaryNotFound;
        let msg = err.to_string();
        assert!(msg.contains("未找到 FFmpeg 二进制文件"), "消息应包含中文: {}", msg);
    }
}
