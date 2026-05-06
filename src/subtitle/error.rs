use thiserror::Error;

/// 字幕处理错误——覆盖解析、IO 等操作
#[derive(Error, Debug)]
pub enum SubtitleError {
    #[error("字幕解析失败: {details}")]
    ParseSubtitle { details: String },

    #[error("IO 操作失败: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for SubtitleError {
    fn from(source: std::io::Error) -> Self {
        SubtitleError::Io { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_error_parse_chinese() {
        let err = SubtitleError::ParseSubtitle {
            details: "时间戳格式无效".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("字幕解析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_subtitle_error_io_chinese() {
        let err = SubtitleError::Io {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到"),
        };
        let msg = err.to_string();
        assert!(msg.contains("IO 操作失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_subtitle_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "权限不足");
        let err: SubtitleError = io_err.into();
        assert!(matches!(err, SubtitleError::Io { .. }));
    }

    #[test]
    fn test_subtitle_error_display() {
        let err = SubtitleError::ParseSubtitle {
            details: "测试错误".into(),
        };
        let display = format!("{}", err);
        assert!(display.contains("字幕解析失败"));
        assert!(display.contains("测试错误"));
    }
}
