use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 统一进度事件 payload（D-10）
///
/// 所有流水线共用此结构体，前端通过 pipeline_type 字段路由到不同 UI 面板。
/// 每个命令调用生成唯一 UUID task_id（D-15），嵌入每个事件。
#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub pipeline_type: String,     // "documentary" | "sde" | "sdp"
    pub task_id: String,           // UUID v4
    pub step_name: String,
    pub percent: f32,
    pub message: String,
    pub step_index: u32,
    pub total_steps: u32,
    pub status: String,            // "running" | "success" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// 流水线命令统一返回包装（D-07）
///
/// 包含输出视频路径。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub output_video_path: PathBuf,
}

/// 脚本信息（get_script_info 命令返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub clip_count: usize,
    pub total_duration_secs: f64,
}
