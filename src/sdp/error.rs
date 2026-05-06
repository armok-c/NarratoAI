use thiserror::Error;

/// SDP 流水线错误——覆盖 4 步流水线各环节 + IO
#[derive(Error, Debug)]
pub enum SdpError {
    #[error("字幕解析失败: {details}")]
    ParseSubtitle { details: String },

    #[error("LLM 调用失败: {details}")]
    LlmError { details: String },

    #[error("JSON 修复失败: {details}")]
    JsonRepair { details: String },

    #[error("视频处理失败: {0}")]
    VideoProcess(#[from] crate::documentary::error::PipelineError),

    #[error("参数校验失败: {details}")]
    Validation { details: String },

    #[error("IO 操作失败: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for SdpError {
    fn from(source: std::io::Error) -> Self {
        SdpError::Io { source }
    }
}

impl From<crate::script::error::ScriptError> for SdpError {
    fn from(e: crate::script::error::ScriptError) -> Self {
        match e {
            crate::script::error::ScriptError::JsonParse(err) => {
                SdpError::JsonRepair {
                    details: format!("脚本 JSON 解析失败: {}", err),
                }
            }
            crate::script::error::ScriptError::Validation(errors) => {
                SdpError::Validation {
                    details: errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; "),
                }
            }
            crate::script::error::ScriptError::Io(source) => SdpError::Io { source },
            crate::script::error::ScriptError::IndexOutOfBounds => {
                SdpError::Validation {
                    details: "脚本索引越界".into(),
                }
            }
            crate::script::error::ScriptError::InvalidTimestamp(ts) => {
                SdpError::ParseSubtitle {
                    details: format!("无效时间戳: {}", ts),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_error_parse_subtitle_chinese() {
        let err = SdpError::ParseSubtitle {
            details: "时间戳格式无效".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("字幕解析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_llm_error_chinese() {
        let err = SdpError::LlmError {
            details: "LLM 响应超时".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("LLM 调用失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_json_repair_chinese() {
        let err = SdpError::JsonRepair {
            details: "JSON 解析错误".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("JSON 修复失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_validation_chinese() {
        let err = SdpError::Validation {
            details: "video_path 不能为空".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("参数校验失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_io_chinese() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let err = SdpError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("IO 操作失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_video_process_chinese() {
        let pipe_err = crate::documentary::error::PipelineError::VideoClip {
            details: "裁剪失败".into(),
        };
        let err = SdpError::from(pipe_err);
        let msg = err.to_string();
        assert!(msg.contains("视频处理失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_from_script_error_validation() {
        use crate::script::error::ScriptError;
        use crate::script::types::ValidationError;

        let err = ScriptError::Validation(vec![ValidationError {
            clip_index: 0,
            field: "timestamp".to_string(),
            message: "格式无效".to_string(),
        }]);
        let sdp_err = SdpError::from(err);
        let msg = sdp_err.to_string();
        assert!(msg.contains("参数校验失败"), "应包含中文: {}", msg);
    }

    #[test]
    fn test_sdp_error_from_script_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let script_err = crate::script::error::ScriptError::Io(io_err);
        let sdp_err = SdpError::from(script_err);
        let msg = sdp_err.to_string();
        assert!(msg.contains("IO 操作失败"), "应包含中文: {}", msg);
    }
}
