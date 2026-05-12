use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

/// 统一进度事件 payload（D-10）
///
/// 所有流水线共用此结构体，前端通过 pipeline_type 字段路由到不同 UI 面板。
/// 每个命令调用生成唯一 UUID task_id（D-15），嵌入每个事件。
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
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
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct CommandResponse {
    pub output_video_path: PathBuf,
}

/// 脚本信息（get_script_info 命令返回）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct ScriptInfo {
    pub clip_count: usize,
    pub total_duration_secs: f64,
    pub unparseable_clips: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_progress_payload_serialize_full() {
        let payload = ProgressPayload {
            pipeline_type: "documentary".into(),
            task_id: "test-uuid".into(),
            step_name: "tts".into(),
            percent: 50.0,
            message: "TTS 生成中".into(),
            step_index: 1,
            total_steps: 6,
            status: "running".into(),
            error_code: Some("TTS_ERROR".into()),
            error_message: Some("连接超时".into()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"pipeline_type\":\"documentary\""));
        assert!(json.contains("\"error_code\":\"TTS_ERROR\""));
        assert!(json.contains("\"error_message\":\"连接超时\""));
    }

    #[test]
    fn test_progress_payload_skip_optional_when_none() {
        let payload = ProgressPayload {
            pipeline_type: "sde".into(),
            task_id: "uuid".into(),
            step_name: "clip".into(),
            percent: 30.0,
            message: "裁剪中".into(),
            step_index: 5,
            total_steps: 9,
            status: "running".into(),
            error_code: None,
            error_message: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("error_code"));
        assert!(!json.contains("error_message"));
    }

    #[test]
    fn test_progress_payload_no_sensitive_fields() {
        // T-10-03: ProgressPayload 不含任何敏感字段
        let payload = ProgressPayload {
            pipeline_type: "documentary".into(),
            task_id: "uuid".into(),
            step_name: "tts".into(),
            percent: 0.0,
            message: String::new(),
            step_index: 0,
            total_steps: 6,
            status: "running".into(),
            error_code: None,
            error_message: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("api_key"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("token"));
        assert!(!json.contains("password"));
    }

    #[test]
    fn test_command_response_roundtrip() {
        let resp = CommandResponse {
            output_video_path: PathBuf::from("/output/video.mp4"),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let de: CommandResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(de.output_video_path, PathBuf::from("/output/video.mp4"));
    }

    #[test]
    fn test_script_info_roundtrip() {
        let info = ScriptInfo {
            clip_count: 42,
            total_duration_secs: 1800.5,
            unparseable_clips: 3,
        };
        let json = serde_json::to_string(&info).unwrap();
        let de: ScriptInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de.clip_count, 42);
        assert_eq!(de.total_duration_secs, 1800.5);
        assert_eq!(de.unparseable_clips, 3);
    }
}
