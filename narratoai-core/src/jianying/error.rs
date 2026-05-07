use thiserror::Error;

/// 剪映导出领域错误
#[derive(Error, Debug)]
pub enum JianYingError {
    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化失败: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    #[error("导出数据校验失败: {details}")]
    Validation { details: String },

    #[error("缺少必要字段: {field}，片段索引: {clip_index}")]
    MissingField { field: String, clip_index: usize },

    #[error("FFmpeg 探测失败: {0}")]
    ProbeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_chinese_message() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = JianYingError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("文件操作失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_validation_error_chinese_message() {
        let err = JianYingError::Validation {
            details: "片段时长不能为零".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("导出数据校验失败"), "消息应包含中文: {}", msg);
        assert!(msg.contains("片段时长不能为零"), "消息应包含详情: {}", msg);
    }

    #[test]
    fn test_missing_field_error_chinese_message() {
        let err = JianYingError::MissingField {
            field: "duration".to_string(),
            clip_index: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("缺少必要字段"), "消息应包含中文: {}", msg);
        assert!(msg.contains("duration"), "消息应包含字段名: {}", msg);
        assert!(msg.contains("3"), "消息应包含片段索引: {}", msg);
    }

    #[test]
    fn test_json_serialize_error_chinese_message() {
        // 使用一个无法序列化的类型触发 serde_json 错误
        // serde_json::Value::Object 中插入非字符串 key 会失败
        // 更简单的方式：直接用 serde_json::from_str 构造一个反序列化错误
        let json_err: Result<(), _> = serde_json::from_str("not valid json");
        let err = JianYingError::from(json_err.unwrap_err());
        let msg = err.to_string();
        assert!(msg.contains("JSON 序列化失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_probe_error_chinese_message() {
        let err = JianYingError::ProbeError("无法获取音频时长".to_string());
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 探测失败"), "消息应包含中文: {}", msg);
        assert!(msg.contains("无法获取音频时长"), "消息应包含详情: {}", msg);
    }
}
