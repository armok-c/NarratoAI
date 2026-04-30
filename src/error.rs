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

/// TTS 领域错误
#[derive(Error, Debug)]
pub enum TTSError {
    #[error("未知 TTS 引擎: {engine}")]
    UnknownEngine { engine: String },

    #[error("TTS 连接失败: {0}")]
    ConnectionFailed(String),

    #[error("TTS 认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("TTS 合成失败: {0}")]
    SynthesisFailed(String),

    #[error("TTS 重试耗尽: {0}")]
    RetryExhausted(String),
}

/// LLM 服务领域错误
#[derive(Error, Debug)]
pub enum LLMError {
    #[error("LLM 调用失败: {0}")]
    General(String),

    #[error("未找到大模型供应商: {0}")]
    ProviderNotFound(String),

    #[error("配置错误: {0}")]
    Configuration(String),

    #[error("API 调用失败: {0}")]
    APICall(String),

    #[error("输出验证失败: {0}")]
    Validation(String),

    #[error("不支持的模型: {0}")]
    ModelNotSupported(String),

    #[error("API 速率限制: {0}")]
    RateLimit(String),

    #[error("API 认证失败: {0}")]
    Authentication(String),

    #[error("内容被安全过滤器阻止: {0}")]
    ContentFilter(String),
}

/// 将 async-openai 的 OpenAIError 映射到 LLMError
impl From<async_openai::error::OpenAIError> for LLMError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        match err {
            async_openai::error::OpenAIError::ApiError(api_err) => {
                let msg = api_err.message;
                match api_err.code.as_deref() {
                    Some("insufficient_quota") => LLMError::RateLimit(msg),
                    Some("invalid_api_key") | Some("invalid_header") => {
                        LLMError::Authentication(msg)
                    }
                    _ => {
                        // 回退到基于消息的启发式判断
                        let lower = msg.to_lowercase();
                        if lower.contains("rate limit")
                            || lower.contains("too many requests")
                            || lower.contains("rate_limit")
                        {
                            LLMError::RateLimit(msg)
                        } else if lower.contains("authentication")
                            || lower.contains("unauthorized")
                            || lower.contains("invalid api key")
                            || lower.contains("invalid_api_key")
                            || lower.contains("invalid_token")
                        {
                            LLMError::Authentication(msg)
                        } else {
                            LLMError::APICall(msg)
                        }
                    }
                }
            }
            async_openai::error::OpenAIError::Reqwest(e) => {
                if e.is_timeout() || e.to_string().to_lowercase().contains("timeout") {
                    LLMError::APICall(format!("请求超时: {}", e))
                } else if e.is_status() {
                    let status = e.status().unwrap_or_default();
                    match status.as_u16() {
                        401 => LLMError::Authentication(e.to_string()),
                        429 => LLMError::RateLimit(e.to_string()),
                        _ => LLMError::APICall(e.to_string()),
                    }
                } else {
                    LLMError::APICall(e.to_string())
                }
            }
            _ => LLMError::APICall(err.to_string()),
        }
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

    #[test]
    fn test_tts_error_unknown_engine_message_chinese() {
        let err = TTSError::UnknownEngine { engine: "test".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("未知 TTS 引擎"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_tts_error_connection_failed_message_chinese() {
        let err = TTSError::ConnectionFailed("test".to_string());
        let msg = err.to_string();
        assert!(msg.contains("TTS 连接失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_tts_error_authentication_failed_message_chinese() {
        let err = TTSError::AuthenticationFailed("test".to_string());
        let msg = err.to_string();
        assert!(msg.contains("TTS 认证失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_tts_error_synthesis_failed_message_chinese() {
        let err = TTSError::SynthesisFailed("test".to_string());
        let msg = err.to_string();
        assert!(msg.contains("TTS 合成失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_tts_error_retry_exhausted_message_chinese() {
        let err = TTSError::RetryExhausted("test".to_string());
        let msg = err.to_string();
        assert!(msg.contains("TTS 重试耗尽"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_general_message_chinese() {
        let err = LLMError::General("test".into());
        let msg = err.to_string();
        assert!(msg.contains("LLM 调用失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_provider_not_found_message_chinese() {
        let err = LLMError::ProviderNotFound("unknown_provider".into());
        let msg = err.to_string();
        assert!(msg.contains("未找到大模型供应商"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_configuration_message_chinese() {
        let err = LLMError::Configuration("bad config".into());
        let msg = err.to_string();
        assert!(msg.contains("配置错误"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_api_call_message_chinese() {
        let err = LLMError::APICall("timeout".into());
        let msg = err.to_string();
        assert!(msg.contains("API 调用失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_validation_message_chinese() {
        let err = LLMError::Validation("invalid output".into());
        let msg = err.to_string();
        assert!(msg.contains("输出验证失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_model_not_supported_message_chinese() {
        let err = LLMError::ModelNotSupported("gpt-5".into());
        let msg = err.to_string();
        assert!(msg.contains("不支持的模型"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_rate_limit_message_chinese() {
        let err = LLMError::RateLimit("too many requests".into());
        let msg = err.to_string();
        assert!(msg.contains("API 速率限制"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_authentication_message_chinese() {
        let err = LLMError::Authentication("invalid key".into());
        let msg = err.to_string();
        assert!(msg.contains("API 认证失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_llm_error_content_filter_message_chinese() {
        let err = LLMError::ContentFilter("blocked content".into());
        let msg = err.to_string();
        assert!(msg.contains("内容被安全过滤器阻止"), "消息应包含中文: {}", msg);
    }
}
