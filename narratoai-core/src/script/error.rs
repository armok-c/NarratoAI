use crate::script::types::ValidationError;

/// 脚本领域错误
#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("JSON 解析失败: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("脚本校验失败")]
    Validation(Vec<ValidationError>),

    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("索引越界")]
    IndexOutOfBounds,

    #[error("时间戳格式无效: {0}")]
    InvalidTimestamp(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: ScriptError::JsonParse 中文消息
    #[test]
    fn test_json_parse_error_message_chinese() {
        let result: Result<crate::script::types::ScriptClip, _> =
            serde_json::from_str("{invalid json}");
        let err = result.unwrap_err();
        let script_err = ScriptError::from(err);
        let msg = script_err.to_string();
        assert!(msg.contains("JSON 解析失败"), "消息应包含中文: {}", msg);
    }

    /// Test: ScriptError::Validation 中文消息
    #[test]
    fn test_validation_error_message_chinese() {
        let errors = vec![ValidationError {
            clip_index: 0,
            field: "_id".to_string(),
            message: "必须为正整数".to_string(),
        }];
        let err = ScriptError::Validation(errors);
        let msg = err.to_string();
        assert!(msg.contains("脚本校验失败"), "消息应包含中文: {}", msg);
    }

    /// Test: ScriptError::Io 中文消息
    #[test]
    fn test_io_error_message_chinese() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ScriptError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("文件操作失败"), "消息应包含中文: {}", msg);
    }

    /// Test: ScriptError::IndexOutOfBounds 中文消息
    #[test]
    fn test_index_out_of_bounds_message_chinese() {
        let err = ScriptError::IndexOutOfBounds;
        let msg = err.to_string();
        assert!(msg.contains("索引越界"), "消息应包含中文: {}", msg);
    }

    /// Test: ScriptError::InvalidTimestamp 中文消息
    #[test]
    fn test_invalid_timestamp_message_chinese() {
        let err = ScriptError::InvalidTimestamp("bad-ts".to_string());
        let msg = err.to_string();
        assert!(msg.contains("时间戳格式无效"), "消息应包含中文: {}", msg);
        assert!(msg.contains("bad-ts"), "消息应包含原始值: {}", msg);
    }
}
