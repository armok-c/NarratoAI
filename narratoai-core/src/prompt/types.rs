use serde::{Deserialize, Serialize};

/// 模型类型——区分视觉模型和文本模型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "text")]
    Text,
}

/// 输出格式——Prompt 渲染后的预期格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputFormat {
    #[serde(rename = "narration_script")]
    NarrationScript,
    #[serde(rename = "plot_analysis")]
    PlotAnalysis,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "text")]
    Text,
}

/// 参数定义——模板的可配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterDef {
    pub name: String,
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    pub description: String,
}

/// Prompt 元数据——模板的描述信息和索引字段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptMetadata {
    pub name: String,
    pub category: String,
    pub version: String,
    pub model_type: ModelType,
    pub output_format: OutputFormat,
    pub tags: Vec<String>,
    pub parameters: Vec<ParameterDef>,
}

/// Prompt 模板——元数据 + 模板内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prompt {
    pub metadata: PromptMetadata,
    pub content: String,
}
