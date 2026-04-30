//! Prompt 分类注册函数（D-17）
//!
//! 对齐 Phase 2 `src/llm/register.rs` 的 `register_all_providers()` 模式。

use crate::prompt::error::PromptError;
use crate::prompt::registry::PromptRegistry;
use crate::prompt::types::{ModelType, OutputFormat, ParameterDef, Prompt, PromptMetadata};

#[derive(Debug)]
pub struct RegistrationErrors(pub Vec<PromptError>);

impl std::fmt::Display for RegistrationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prompt 注册失败 ({} 个错误):", self.0.len())?;
        for (i, err) in self.0.iter().enumerate() {
            write!(f, "\n  [{}] {}", i + 1, err)?;
        }
        Ok(())
    }
}

impl std::error::Error for RegistrationErrors {}

pub fn register_all_prompts(registry: &mut PromptRegistry) -> Result<(), RegistrationErrors> {
    let mut errors = Vec::new();
    register_documentary_prompts(registry, &mut errors);
    register_short_drama_editing_prompts(registry, &mut errors);
    register_short_drama_narration_prompts(registry, &mut errors);

    if errors.is_empty() { Ok(()) }
    else { Err(RegistrationErrors(errors)) }
}

fn register_documentary_prompts(registry: &mut PromptRegistry, errors: &mut Vec<PromptError>) {
    let frame = Prompt {
        metadata: PromptMetadata {
            name: "frame_analysis".into(), category: "documentary".into(), version: "v1.0".into(),
            model_type: ModelType::Vision, output_format: OutputFormat::Json,
            tags: vec!["documentary".into(), "frame-analysis".into(), "vision".into()],
            parameters: vec![
                ParameterDef { name: "video_description".into(), required: true, default: None, description: "视频内容摘要".into() },
                ParameterDef { name: "language".into(), required: false, default: Some("zh-CN".into()), description: "输出语言".into() },
                ParameterDef { name: "frame_data".into(), required: true, default: None, description: "帧数据（Base64 编码）".into() },
            ],
        },
        content: include_str!("templates/documentary/frame_analysis_v1.0.md").into(),
    };
    if let Err(e) = registry.register(frame, true) { errors.push(e); }

    let narration = Prompt {
        metadata: PromptMetadata {
            name: "narration_generation".into(), category: "documentary".into(), version: "v2.0".into(),
            model_type: ModelType::Text, output_format: OutputFormat::NarrationScript,
            tags: vec!["documentary".into(), "narration".into(), "text".into()],
            parameters: vec![
                ParameterDef { name: "video_title".into(), required: true, default: None, description: "视频标题".into() },
                ParameterDef { name: "frame_analysis_json".into(), required: true, default: None, description: "帧分析 JSON 数据".into() },
                ParameterDef { name: "language".into(), required: false, default: Some("zh-CN".into()), description: "输出语言".into() },
                ParameterDef { name: "style".into(), required: false, default: Some("正式".into()), description: "文案风格".into() },
            ],
        },
        content: include_str!("templates/documentary/narration_generation_v2.0.md").into(),
    };
    if let Err(e) = registry.register(narration, true) { errors.push(e); }
}

fn register_short_drama_editing_prompts(registry: &mut PromptRegistry, errors: &mut Vec<PromptError>) {
    let prompt = Prompt {
        metadata: PromptMetadata {
            name: "plot_extraction".into(), category: "short_drama_editing".into(), version: "v2.0".into(),
            model_type: ModelType::Text, output_format: OutputFormat::PlotAnalysis,
            tags: vec!["short-drama-editing".into(), "plot-extraction".into(), "text".into()],
            parameters: vec![
                ParameterDef { name: "subtitle_content".into(), required: true, default: None, description: "字幕内容".into() },
                ParameterDef { name: "language".into(), required: false, default: Some("zh-CN".into()), description: "输出语言".into() },
            ],
        },
        content: include_str!("templates/short_drama_editing/plot_extraction_v2.0.md").into(),
    };
    if let Err(e) = registry.register(prompt, true) { errors.push(e); }
}

fn register_short_drama_narration_prompts(registry: &mut PromptRegistry, errors: &mut Vec<PromptError>) {
    let prompt = Prompt {
        metadata: PromptMetadata {
            name: "script_generation".into(), category: "short_drama_narration".into(), version: "v1.0".into(),
            model_type: ModelType::Text, output_format: OutputFormat::NarrationScript,
            tags: vec!["short-drama-narration".into(), "script-generation".into(), "text".into()],
            parameters: vec![
                ParameterDef { name: "plot_analysis".into(), required: true, default: None, description: "剧情分析数据".into() },
                ParameterDef { name: "language".into(), required: false, default: Some("zh-CN".into()), description: "输出语言".into() },
                ParameterDef { name: "style".into(), required: false, default: Some("幽默风趣".into()), description: "文案风格".into() },
            ],
        },
        content: include_str!("templates/short_drama_narration/script_generation_v1.0.md").into(),
    };
    if let Err(e) = registry.register(prompt, true) { errors.push(e); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_categories() {
        let mut registry = PromptRegistry::new();
        register_documentary_prompts(&mut registry, &mut Vec::new());
        register_short_drama_editing_prompts(&mut registry, &mut Vec::new());
        register_short_drama_narration_prompts(&mut registry, &mut Vec::new());
        let categories = registry.list_categories();
        assert_eq!(categories.len(), 3);
    }

    #[test]
    fn test_register_all_prompts_full() {
        let mut registry = PromptRegistry::new();
        let result = register_all_prompts(&mut registry);
        assert!(result.is_ok(), "完整注册应成功: {:?}", result.err());
        assert_eq!(registry.list_prompts("documentary").len(), 2);
        assert_eq!(registry.list_prompts("short_drama_editing").len(), 1);
        assert_eq!(registry.list_prompts("short_drama_narration").len(), 1);
    }

    #[test]
    fn test_register_all_duplicate_returns_errors() {
        let mut registry = PromptRegistry::new();
        assert!(register_all_prompts(&mut registry).is_ok());
        assert!(register_all_prompts(&mut registry).is_err());
    }

    #[test]
    fn test_registration_errors_display() {
        let errors = RegistrationErrors(vec![
            PromptError::Registration("版本已存在: test.p1 v1".into()),
        ]);
        let display = format!("{}", errors);
        assert!(display.contains("Prompt 注册失败"));
    }
}
