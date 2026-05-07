/// 视觉分析领域错误
///
/// 涵盖帧提取、视觉分析和批量处理三种错误类型（D-21/D-22）。
#[derive(thiserror::Error, Debug)]
pub enum VisualError {
    /// 帧提取失败（FFmpeg 调用、文件操作等）
    #[error("帧提取失败: {0}")]
    FrameExtraction(String),

    /// 视觉分析失败（LLM 调用、JSON 解析等）
    #[error("视觉分析失败: {0}")]
    Analysis(String),

    /// 部分批次失败——部分帧分析成功但存在错误（D-14: 收集错误继续执行）
    #[error("部分批次失败: 已分析 {analyzed_count}/{total_count} 批次，{errors}")]
    BatchPartial {
        analyzed_count: usize,
        total_count: usize,
        errors: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: VisualError::FrameExtraction 中文消息
    #[test]
    fn test_frame_extraction_error_message_chinese() {
        let err = VisualError::FrameExtraction("FFmpeg 返回错误".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("帧提取失败"),
            "消息应包含中文: {}",
            msg
        );
    }

    /// Test: VisualError::Analysis 中文消息
    #[test]
    fn test_analysis_error_message_chinese() {
        let err = VisualError::Analysis("JSON 解析超时".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("视觉分析失败"),
            "消息应包含中文: {}",
            msg
        );
    }

    /// Test: VisualError::BatchPartial 中文消息
    #[test]
    fn test_batch_partial_error_message_chinese() {
        let err = VisualError::BatchPartial {
            analyzed_count: 3,
            total_count: 5,
            errors: "批次 2 失败".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("部分批次失败"),
            "消息应包含中文: {}",
            msg
        );
        assert!(
            msg.contains("3/5"),
            "消息应包含分析计数: {}",
            msg
        );
        assert!(
            msg.contains("批次 2 失败"),
            "消息应包含错误详情: {}",
            msg
        );
    }
}
