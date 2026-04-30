use std::collections::HashMap;
use std::sync::RwLock;

use crate::prompt::error::PromptError;
use crate::prompt::registry::SharedPromptRegistry;
use crate::prompt::template;
use crate::prompt::types::{OutputFormat, Prompt};
use crate::prompt::validators;

/// PromptManager 外观（D-20）
///
/// 提供 Prompt 系统的统一管理接口，对齐 Python 版
/// `app/services/prompts/manager.py` 的 PromptManager。
///
/// 方法：
/// - get_prompt: 获取 Prompt 的克隆
/// - render_prompt: 获取并渲染 Prompt 模板
/// - register_prompt: 注册到 Registry
/// - search_prompts: 搜索匹配的 Prompt
/// - validate_output: 校验 LLM 输出格式
pub struct PromptManager {
    registry: SharedPromptRegistry,
}

impl PromptManager {
    /// 包装现有 registry 创建新的 Manager
    pub fn new(registry: SharedPromptRegistry) -> Self {
        Self { registry }
    }

    /// 获取 Prompt 的克隆（确保调用方获得独立副本）
    pub fn get_prompt(
        &self,
        category: &str,
        name: &str,
        version: Option<&str>,
    ) -> Result<Prompt, PromptError> {
        let registry = self.registry.read().map_err(|e| {
            PromptError::TemplateRender(format!("注册中心读取锁失败: {}", e))
        })?;
        let prompt = registry.get(category, name, version)?;
        Ok(prompt.clone())
    }

    /// 获取并渲染 Prompt 模板（D-05）
    ///
    /// 1. 从 registry 获取 Prompt 定义
    /// 2. 调用 template::render() 渲染模板
    pub fn render_prompt(
        &self,
        category: &str,
        name: &str,
        version: Option<&str>,
        vars: &HashMap<&str, &str>,
    ) -> Result<String, PromptError> {
        let prompt = self.get_prompt(category, name, version)?;
        template::render(&prompt.content, vars)
    }

    /// 注册 Prompt 到 Registry（D-17）
    pub fn register_prompt(
        &self,
        prompt: Prompt,
        is_default: bool,
    ) -> Result<(), PromptError> {
        let mut registry = self.registry.write().map_err(|e| {
            PromptError::TemplateRender(format!("注册中心写入锁失败: {}", e))
        })?;
        registry.register(prompt, is_default)
    }

    /// 搜索匹配的 Prompt（D-02）
    pub fn search_prompts(&self, query: &str) -> Vec<Prompt> {
        let registry = self
            .registry
            .read()
            .unwrap_or_else(|e| e.into_inner());
        registry.search(query).into_iter().cloned().collect()
    }

    /// 校验 LLM 输出格式（D-20）
    ///
    /// 委托给 `validators::validate_output()`。
    pub fn validate_output(
        &self,
        output: &str,
        format: &OutputFormat,
    ) -> Result<(), PromptError> {
        validators::validate_output(output, format)
    }

    /// 列出所有已注册的分类
    pub fn list_categories(&self) -> Vec<String> {
        let registry = self
            .registry
            .read()
            .unwrap_or_else(|e| e.into_inner());
        registry.list_categories()
    }

    /// 列出指定分类下的所有 Prompt 名称
    pub fn list_prompts(&self, category: &str) -> Vec<String> {
        let registry = self
            .registry
            .read()
            .unwrap_or_else(|e| e.into_inner());
        registry.list_prompts(category)
            .iter()
            .map(|p| p.metadata.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::registry::PromptRegistry;
    use crate::prompt::types::{ModelType, OutputFormat, ParameterDef, PromptMetadata};
    use std::sync::{Arc, RwLock};

    fn make_test_prompt(category: &str, name: &str, version: &str, content: &str) -> Prompt {
        Prompt {
            metadata: PromptMetadata {
                category: category.to_string(),
                name: name.to_string(),
                version: version.to_string(),
                model_type: ModelType::Text,
                output_format: OutputFormat::Json,
                tags: vec![],
                parameters: vec![],
            },
            content: content.to_string(),
        }
    }

    fn make_manager() -> PromptManager {
        let registry = Arc::new(RwLock::new(PromptRegistry::new()));
        PromptManager::new(registry)
    }

    #[test]
    fn test_get_prompt_returns_correct_prompt() {
        let manager = make_manager();
        let prompt = make_test_prompt("test", "greeting", "v1.0", "Hello ${name}!");
        manager.register_prompt(prompt, true).unwrap();

        let retrieved = manager.get_prompt("test", "greeting", None).unwrap();
        assert_eq!(retrieved.metadata.name, "greeting");
        assert_eq!(retrieved.metadata.version, "v1.0");
        assert_eq!(retrieved.content, "Hello ${name}!");
    }

    #[test]
    fn test_get_prompt_nonexistent_returns_error() {
        let manager = make_manager();
        let result = manager.get_prompt("nonexistent", "test", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_prompt_renders_correctly() {
        let manager = make_manager();
        let prompt = make_test_prompt("test", "greeting", "v1.0", "Hello ${name}!");
        manager.register_prompt(prompt, true).unwrap();

        let vars: HashMap<&str, &str> = [("name", "World")].into_iter().collect();
        let rendered = manager
            .render_prompt("test", "greeting", None, &vars)
            .unwrap();
        assert_eq!(rendered, "Hello World!");
    }

    #[test]
    fn test_render_prompt_missing_variable() {
        let manager = make_manager();
        let prompt = make_test_prompt("test", "greeting", "v1.0", "Hello ${name}!");
        manager.register_prompt(prompt, true).unwrap();

        let vars: HashMap<&str, &str> = HashMap::new();
        let result = manager.render_prompt("test", "greeting", None, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_prompt_delegates_to_registry() {
        let manager = make_manager();
        let prompt = make_test_prompt("test", "newprompt", "v1.0", "content");
        manager.register_prompt(prompt, true).unwrap();

        let categories = manager.list_categories();
        assert!(categories.contains(&"test".to_string()));
    }

    #[test]
    fn test_register_duplicate_returns_error() {
        let manager = make_manager();
        let p1 = make_test_prompt("test", "dup", "v1.0", "content");
        let p2 = make_test_prompt("test", "dup", "v1.0", "content2");
        manager.register_prompt(p1, true).unwrap();
        let result = manager.register_prompt(p2, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_prompts_returns_matches() {
        let manager = make_manager();
        manager
            .register_prompt(
                make_test_prompt("documentary", "frame_analysis", "v1.0", "content"),
                true,
            )
            .unwrap();
        manager
            .register_prompt(
                make_test_prompt("documentary", "narration_gen", "v2.0", "content"),
                true,
            )
            .unwrap();

        let results = manager.search_prompts("frame");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_prompts_no_match() {
        let manager = make_manager();
        manager
            .register_prompt(
                make_test_prompt("doc", "test", "v1.0", "content"),
                true,
            )
            .unwrap();

        let results = manager.search_prompts("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_validate_output_delegates_correctly() {
        let manager = make_manager();

        // 有效 JSON 应通过
        let result = manager.validate_output(r#"{"key": "value"}"#, &OutputFormat::Json);
        assert!(result.is_ok());

        // 无效 JSON 应失败
        let result = manager.validate_output("invalid", &OutputFormat::Json);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_categories() {
        let manager = make_manager();
        manager
            .register_prompt(
                make_test_prompt("cat1", "p1", "v1.0", "c"),
                true,
            )
            .unwrap();
        manager
            .register_prompt(
                make_test_prompt("cat2", "p2", "v1.0", "c"),
                true,
            )
            .unwrap();

        let cats = manager.list_categories();
        assert_eq!(cats.len(), 2);
        assert!(cats.contains(&"cat1".to_string()));
    }

    #[test]
    fn test_list_prompts() {
        let manager = make_manager();
        manager
            .register_prompt(
                make_test_prompt("doc", "frame_analysis", "v1.0", "c"),
                true,
            )
            .unwrap();
        manager
            .register_prompt(
                make_test_prompt("doc", "narration_gen", "v2.0", "c"),
                true,
            )
            .unwrap();

        let prompts = manager.list_prompts("doc");
        assert_eq!(prompts.len(), 2);
    }
}
