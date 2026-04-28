use serde::{Deserialize, Serialize};

/// LLM 响应格式选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmResponseFormat {
    /// 普通文本格式
    Text,
    /// JSON 格式（触发 D-19 response_format=json_object 回退逻辑）
    Json,
}

/// 视觉分析批量处理配置
#[derive(Debug, Clone)]
pub struct VisionBatchConfig {
    /// 每批最多图片数，默认 10
    pub batch_size: usize,
    /// 最大并发批次数，默认 1
    pub max_concurrency: usize,
}

impl Default for VisionBatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            max_concurrency: 1,
        }
    }
}
