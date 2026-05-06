use crate::documentary::error::PipelineError;

/// SDP 流水线错误——覆盖 6 个环节
#[derive(thiserror::Error, Debug)]
pub enum SdpError {
    #[error("字幕解析失败: {details}")]
    ParseSubtitle { details: String },

    #[error("LLM 调用失败: {details}")]
    LlmError { details: String },

    #[error("JSON 修复失败: {details}")]
    JsonRepair { details: String },

    #[error("视频处理失败: {0}")]
    VideoProcess(#[from] PipelineError),

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

impl From<crate::subtitle::error::SubtitleError> for SdpError {
    fn from(e: crate::subtitle::error::SubtitleError) -> Self {
        match e {
            crate::subtitle::error::SubtitleError::ParseSubtitle { details } => {
                SdpError::ParseSubtitle { details }
            }
            crate::subtitle::error::SubtitleError::Io { source } => SdpError::Io { source },
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
            details: "API 调用超时".into(),
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
    fn test_sdp_error_video_process_chinese() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "clip failed");
        let pipeline_err = PipelineError::VideoClip {
            details: format!("裁剪失败: {}", io_err),
        };
        let err = SdpError::from(pipeline_err);
        let msg = err.to_string();
        assert!(msg.contains("视频处理失败"), "消息应包含中文: {}", msg);
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
        let err = SdpError::Io {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到"),
        };
        let msg = err.to_string();
        assert!(msg.contains("IO 操作失败"), "消息应包含中文: {}", msg);
    }
}
