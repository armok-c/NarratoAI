use thiserror::Error;

/// 配置领域错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件读取失败: {0}")]
    IoError(String),

    #[error("配置文件解析失败: {0}")]
    ParseError(String),

    #[error("配置校验失败: {0}")]
    ValidationError(String),

    #[error("配置文件不存在: {path}")]
    NotFound { path: String },

    #[error("配置监听失败: {0}")]
    WatchError(String),
}

/// FFmpeg 领域错误
#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg 进程启动失败: {0}")]
    SpawnFailed(String),

    #[error("FFmpeg 执行超时: {0}")]
    Timeout(String),

    #[error("FFmpeg 输出解析错误: {0}")]
    OutputParseError(String),

    #[error("FFmpeg 执行错误: {0}")]
    ExecutionError(String),

    #[error("未找到 FFmpeg 二进制文件")]
    BinaryNotFound,
}

/// 为 notify 错误实现 From trait，便于 ConfigManager 统一错误处理
impl From<notify::Error> for ConfigError {
    fn from(err: notify::Error) -> Self {
        ConfigError::WatchError(err.to_string())
    }
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
    fn test_ffmpeg_error_spawn_failed_message_chinese() {
        let err = FFmpegError::SpawnFailed("test".into());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 进程启动失败"), "消息应包含中文: {}", msg);
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
