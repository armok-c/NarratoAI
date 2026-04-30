use thiserror::Error;

/// Prompt 领域错误
#[derive(Error, Debug)]
pub enum PromptError {
    #[error("模板渲染失败: {0}")]
    TemplateRender(String),

    #[error("未找到模板: {category}/{name} v{version:?}")]
    NotFound {
        category: String,
        name: String,
        version: Option<String>,
    },

    #[error("注册中心锁失败: {0}")]
    LockFailure(String),

    #[error("模板注册失败: {0}")]
    Registration(String),

    #[error("模板校验失败: {0}")]
    Validation(String),

    #[error("版本错误: {0}")]
    Version(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_render_error_message_chinese() {
        let err = PromptError::TemplateRender("test".into());
        let msg = err.to_string();
        assert!(msg.contains("模板渲染失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_not_found_error_message_chinese() {
        let err = PromptError::NotFound {
            category: "doc".into(),
            name: "test".into(),
            version: Some("v1".into()),
        };
        let msg = err.to_string();
        assert!(msg.contains("未找到模板"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_registration_error_message_chinese() {
        let err = PromptError::Registration("version exists".into());
        let msg = err.to_string();
        assert!(msg.contains("模板注册失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_validation_error_message_chinese() {
        let err = PromptError::Validation("invalid template".into());
        let msg = err.to_string();
        assert!(msg.contains("模板校验失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_version_error_message_chinese() {
        let err = PromptError::Version("bad version".into());
        let msg = err.to_string();
        assert!(msg.contains("版本错误"), "消息应包含中文: {}", msg);
    }
}
