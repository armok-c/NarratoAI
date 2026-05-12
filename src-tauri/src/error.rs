use serde::Serialize;
use std::fmt;

/// Tauri 命令统一错误——所有领域错误通过 From<T> 转换为此类型
///
/// # 安全性
/// message 保留中文错误信息（与 Phase 1 D-18 一致）。
/// code 用于前端程序化区分错误类型。
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

/// 校验路径不包含遍历序列，防止 IPC 边界的路径遍历攻击
///
/// 使用 `std::path::Component` 进行平台感知的路径解析：
/// - 在 Windows 上正确处理盘符路径（如 `C:\Users\...`）和 UNC 路径
/// - 仅拒绝实际包含 `..` 组件（ParentDir）的路径
/// - 拒绝包含空字节的路径
pub fn validate_path(path: &std::path::PathBuf) -> Result<(), CommandError> {
    let path_str = path.to_string_lossy();
    if path_str.contains('\0') {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "路径不允许包含空字节".into(),
        });
    }
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(CommandError {
                code: "INVALID_PATH".into(),
                message: "路径不允许包含 '..' 遍历序列".into(),
            });
        }
    }
    Ok(())
}

// ---- From<T> implementations for each domain error type ----
// 每个实现使用 err.to_string() 保留中文 Display 信息（D-18）

use narratoai_core::error::ConfigError;
impl From<ConfigError> for CommandError {
    fn from(err: ConfigError) -> Self {
        CommandError {
            code: "CONFIG_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::documentary::error::PipelineError;
impl From<PipelineError> for CommandError {
    fn from(err: PipelineError) -> Self {
        CommandError {
            code: "PIPELINE_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::sde::error::SdeError;
impl From<SdeError> for CommandError {
    fn from(err: SdeError) -> Self {
        CommandError {
            code: "SDE_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::sdp::error::SdpError;
impl From<SdpError> for CommandError {
    fn from(err: SdpError) -> Self {
        CommandError {
            code: "SDP_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::script::error::ScriptError;
impl From<ScriptError> for CommandError {
    fn from(err: ScriptError) -> Self {
        CommandError {
            code: "SCRIPT_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::jianying::error::JianYingError;
impl From<JianYingError> for CommandError {
    fn from(err: JianYingError) -> Self {
        CommandError {
            code: "JIANYING_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::error::TTSError;
impl From<TTSError> for CommandError {
    fn from(err: TTSError) -> Self {
        CommandError {
            code: "TTS_ERROR".into(),
            message: err.to_string(),
        }
    }
}

use narratoai_core::error::LLMError;
impl From<LLMError> for CommandError {
    fn from(err: LLMError) -> Self {
        CommandError {
            code: "LLM_ERROR".into(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_display_format() {
        let err = CommandError {
            code: "TEST_CODE".into(),
            message: "测试消息".into(),
        };
        assert_eq!(format!("{}", err), "[TEST_CODE] 测试消息");
    }

    #[test]
    fn test_command_error_serialize() {
        let err = CommandError {
            code: "PIPELINE_ERROR".into(),
            message: "视频裁剪失败".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":\"PIPELINE_ERROR\""));
        assert!(json.contains("\"message\":\"视频裁剪失败\""));
        // T-10-01: 不含敏感字段
        assert!(!json.contains("api_key"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("token"));
    }

    #[test]
    fn test_from_pipeline_error() {
        let err = CommandError::from(PipelineError::VideoClip { details: "test.mp4".into() });
        assert_eq!(err.code, "PIPELINE_ERROR");
        assert!(err.message.contains("test.mp4"));
    }

    #[test]
    fn test_from_sde_error() {
        let err = CommandError::from(SdeError::Validation { details: "参数错误".into() });
        assert_eq!(err.code, "SDE_ERROR");
    }

    #[test]
    fn test_from_sdp_error() {
        let err = CommandError::from(SdpError::Validation { details: "校验失败".into() });
        assert_eq!(err.code, "SDP_ERROR");
    }

    #[test]
    fn test_from_script_error() {
        let err = CommandError::from(ScriptError::InvalidTimestamp("bad-ts".into()));
        assert_eq!(err.code, "SCRIPT_ERROR");
    }

    #[test]
    fn test_from_config_error() {
        let err = CommandError::from(ConfigError::NotFound { path: "config.toml".into() });
        assert_eq!(err.code, "CONFIG_ERROR");
    }

    #[test]
    fn test_from_tts_error() {
        let err = CommandError::from(TTSError::UnknownEngine { engine: "bad_engine".into() });
        assert_eq!(err.code, "TTS_ERROR");
    }

    #[test]
    fn test_from_llm_error() {
        let err = CommandError::from(LLMError::Configuration("配置错误".into()));
        assert_eq!(err.code, "LLM_ERROR");
    }

    #[test]
    fn test_from_jianying_error() {
        let err = CommandError::from(JianYingError::Validation { details: "验证失败".into() });
        assert_eq!(err.code, "JIANYING_ERROR");
    }

    #[test]
    fn test_validate_path_rejects_parent_dir() {
        let path = std::path::PathBuf::from("../../etc/passwd");
        let result = validate_path(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_PATH");
        assert!(err.message.contains(".."));
    }

    #[test]
    fn test_validate_path_rejects_null_bytes() {
        let path = std::path::PathBuf::from("test\0file.txt");
        let result = validate_path(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "INVALID_PATH");
    }

    #[test]
    fn test_validate_path_allows_normal_path() {
        let path = std::path::PathBuf::from("C:\\Users\\test\\video.mp4");
        assert!(validate_path(&path).is_ok());
    }

    #[test]
    fn test_validate_path_allows_relative_path() {
        let path = std::path::PathBuf::from("output/video.mp4");
        assert!(validate_path(&path).is_ok());
    }

    #[test]
    fn test_command_error_is_std_error() {
        let err = CommandError {
            code: "TEST".into(),
            message: "msg".into(),
        };
        let _: &dyn std::error::Error = &err;
    }
}

