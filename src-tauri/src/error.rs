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
pub fn validate_path(path: &std::path::PathBuf) -> Result<(), CommandError> {
    let path_str = path.to_string_lossy();
    // 检查遍历序列（混合分隔符归一化后检查）
    let normalized = path_str.replace('\\', "/");
    if normalized.contains("..") {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "路径不允许包含 '..' 遍历序列".into(),
        });
    }
    // 检查反斜杠（统一使用正斜杠）
    if path_str.contains('\\') {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "路径不允许包含反斜杠".into(),
        });
    }
    // 检查 Windows 绝对路径（盘符 C: 或 UNC \\）
    let bytes = path_str.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "不允许使用绝对路径".into(),
        });
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

