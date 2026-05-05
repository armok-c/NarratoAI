use crate::documentary::error::PipelineError;

/// SDE 流水线错误——覆盖 9 步流水线各环节 + IO
#[derive(thiserror::Error, Debug)]
pub enum SdeError {
    #[error("字幕解析失败: {details}")]
    ParseSubtitle { details: String },

    #[error("剧情分析失败: {details}")]
    PlotAnalysis { details: String },

    #[error("解说脚本生成失败: {details}")]
    ScriptGeneration { details: String },

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

impl From<std::io::Error> for SdeError {
    fn from(source: std::io::Error) -> Self {
        SdeError::Io { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sde_error_parse_subtitle_chinese() {
        let err = SdeError::ParseSubtitle {
            details: "时间戳格式无效".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("字幕解析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sde_error_plot_analysis_chinese() {
        let err = SdeError::PlotAnalysis {
            details: "LLM 响应为空".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("剧情分析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sde_error_script_generation_chinese() {
        let err = SdeError::ScriptGeneration {
            details: "模板渲染失败".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("解说脚本生成失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sde_error_json_repair_chinese() {
        let err = SdeError::JsonRepair {
            details: "JSON 解析错误".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("JSON 修复失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_sde_error_validation_chinese() {
        let err = SdeError::Validation {
            details: "video_path 不能为空".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("参数校验失败"), "消息应包含中文: {}", msg);
    }
}
