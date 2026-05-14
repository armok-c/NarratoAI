use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// LLM 响应格式选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub enum LlmResponseFormat {
    /// 普通文本格式
    Text,
    /// JSON 格式（触发 D-19 response_format=json_object 回退逻辑）
    Json,
}
