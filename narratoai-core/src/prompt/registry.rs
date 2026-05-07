use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use regex::Regex;

use crate::prompt::error::PromptError;
use crate::prompt::types::Prompt;

/// 共享的 Prompt 注册表——Arc<RwLock<PromptRegistry>> 提供线程安全访问
pub type SharedPromptRegistry = Arc<RwLock<PromptRegistry>>;

/// Prompt 注册表——三级索引: category → name → version → Prompt
pub struct PromptRegistry {
    /// 三级索引: {category: {name: {version: Prompt}}}
    prompts: HashMap<String, HashMap<String, HashMap<String, Prompt>>>,
    /// 默认版本索引: {category: {name: default_version_string}}
    default_versions: HashMap<String, HashMap<String, String>>,
}

impl PromptRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            default_versions: HashMap::new(),
        }
    }

    /// 注册 prompt
    ///
    /// 按 category.name.version 三级插入。如果版本已存在返回 `PromptError::Registration`。
    /// 如果 `is_default` 为 true 或无现有默认版本，更新 default_versions。
    ///
    /// 注册前自动校验 `${variable}` 引用是否全部在 `ParameterDef` 中声明（WR-04）。
    pub fn register(&mut self, prompt: Prompt, is_default: bool) -> Result<(), PromptError> {
        // 校验模板变量与 ParameterDef 声明一致
        self.validate_prompt_parameters(&prompt)?;

        let category = prompt.metadata.category.clone();
        let name = prompt.metadata.name.clone();
        let version = prompt.metadata.version.clone();

        // 获取或创建版本映射
        let version_map = self
            .prompts
            .entry(category.clone())
            .or_default()
            .entry(name.clone())
            .or_default();

        // 检查版本是否已存在
        if version_map.contains_key(&version) {
            return Err(PromptError::Registration(format!(
                "版本已存在: {}.{} v{}",
                category, name, version
            )));
        }

        // 插入 prompt
        version_map.insert(version.clone(), prompt);

        // 更新默认版本
        let has_default = self
            .default_versions
            .get(&category)
            .and_then(|m| m.get(&name))
            .is_some();

        if is_default || !has_default {
            self.default_versions
                .entry(category)
                .or_default()
                .insert(name, version);
        }

        Ok(())
    }

    /// 校验模板内容中的 `${variable}` 引用是否全部在 `ParameterDef` 中声明
    ///
    /// 防止模板引用了未声明的变量（WR-04），确保 `render_prompt` 中的 `Validation` 错误路径
    /// 覆盖所有可能的变量引用，避免用户收到与 `ParameterDef` 不一致的 `TemplateRender` 错误。
    fn validate_prompt_parameters(&self, prompt: &Prompt) -> Result<(), PromptError> {
        let re = Regex::new(r"\$\{(\w+)(?:\|(\w+))?\}").map_err(|e| {
            PromptError::TemplateRender(format!("正则编译失败: {}", e))
        })?;

        let declared: HashSet<&str> = prompt
            .metadata
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        let mut undeclared: Vec<&str> = Vec::new();
        for caps in re.captures_iter(&prompt.content) {
            if let Some(name) = caps.get(1).map(|m| m.as_str()) {
                if !name.is_empty() && !declared.contains(name) {
                    undeclared.push(name);
                }
            }
        }

        if !undeclared.is_empty() {
            undeclared.sort();
            undeclared.dedup();
            return Err(PromptError::Validation(format!(
                "模板引用了未声明的参数: {}",
                undeclared.join(", ")
            )));
        }

        Ok(())
    }

    /// 按三级索引查找 prompt
    ///
    /// `version` 为 None 时使用默认版本。未找到时返回 `PromptError::NotFound`。
    pub fn get(
        &self,
        category: &str,
        name: &str,
        version: Option<&str>,
    ) -> Result<&Prompt, PromptError> {
        let version_str = match version {
            Some(v) => v.to_string(),
            None => self
                .default_versions
                .get(category)
                .and_then(|m| m.get(name))
                .cloned()
                .ok_or_else(|| PromptError::NotFound {
                    category: category.to_string(),
                    name: name.to_string(),
                    version: None,
                })?,
        };

        self.prompts
            .get(category)
            .and_then(|m| m.get(name))
            .and_then(|m| m.get(&version_str))
            .ok_or_else(|| PromptError::NotFound {
                category: category.to_string(),
                name: name.to_string(),
                version: Some(version_str),
            })
    }

    /// 搜索 prompt——在 name/tags/category 中模糊匹配（不区分大小写）
    pub fn search(&self, query: &str) -> Vec<&Prompt> {
        let query_lower = query.to_ascii_lowercase();
        let mut results = Vec::new();

        for cat_prompts in self.prompts.values() {
            for ver_prompts in cat_prompts.values() {
                for prompt in ver_prompts.values() {
                    let meta = &prompt.metadata;
                    if meta.name.to_ascii_lowercase().contains(&query_lower)
                        || meta.category.to_ascii_lowercase().contains(&query_lower)
                        || meta
                            .tags
                            .iter()
                            .any(|t| t.to_ascii_lowercase().contains(&query_lower))
                    {
                        results.push(prompt);
                    }
                }
            }
        }

        results
    }

    /// 返回所有已注册的 category 列表（排序后）
    pub fn list_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.prompts.keys().cloned().collect();
        cats.sort();
        cats
    }

    /// 返回指定 category 的所有 prompt（按 name 排序，取每个 name 的默认版本）
    ///
    /// 默认版本不存在时 fallback 到第一个可用版本并记录警告，而非静默丢弃。
    pub fn list_prompts(&self, category: &str) -> Vec<&Prompt> {
        let names_map = match self.prompts.get(category) {
            Some(m) => m,
            None => return Vec::new(),
        };

        let mut names: Vec<&String> = names_map.keys().collect();
        names.sort();

        let mut result = Vec::new();
        for name in names {
            // 取默认版本，不存在时 fallback 到第一个可用版本
            let version = self
                .default_versions
                .get(category)
                .and_then(|m| m.get(name.as_str()))
                .cloned()
                .or_else(|| {
                    let fallback = names_map.get(name).and_then(|m| {
                        let mut versions: Vec<&String> = m.keys().collect();
                        versions.sort_by(|a, b| {
                            let a_num: u64 = a.trim_start_matches('v').split('.').next()
                                .and_then(|s| s.parse().ok()).unwrap_or(0);
                            let b_num: u64 = b.trim_start_matches('v').split('.').next()
                                .and_then(|s| s.parse().ok()).unwrap_or(0);
                            a_num.cmp(&b_num)
                        });
                        versions.into_iter().next().cloned()
                    });
                    if let Some(ref v) = fallback {
                        tracing::warn!(
                            "没有默认版本: {}.{}, 使用 {}",
                            category, name, v
                        );
                    }
                    fallback
                });

            if let Some(ver) = version {
                if let Some(prompt) = names_map.get(name).and_then(|m| m.get(&ver)) {
                    result.push(prompt);
                }
            }
        }
        result
    }
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::types::{
        ModelType, OutputFormat, ParameterDef, PromptMetadata,
    };

    fn make_test_prompt(
        category: &str,
        name: &str,
        version: &str,
        content: &str,
    ) -> Prompt {
        Prompt {
            metadata: PromptMetadata {
                name: name.to_string(),
                category: category.to_string(),
                version: version.to_string(),
                model_type: ModelType::Text,
                output_format: OutputFormat::Json,
                tags: Vec::new(),
                parameters: Vec::new(),
            },
            content: content.to_string(),
        }
    }

    /// 注册一个 prompt，验证 get() 返回正确的 content
    #[test]
    fn test_register_and_get() {
        let mut reg = PromptRegistry::new();
        let prompt = Prompt {
            metadata: PromptMetadata {
                name: "frame_analysis".to_string(),
                category: "doc".to_string(),
                version: "v1.0".to_string(),
                model_type: ModelType::Text,
                output_format: OutputFormat::Json,
                tags: Vec::new(),
                parameters: vec![ParameterDef {
                    name: "frame".to_string(),
                    required: true,
                    default: None,
                    description: "帧编号".to_string(),
                }],
            },
            content: "分析帧: ${frame}".to_string(),
        };
        reg.register(prompt, true).expect("注册应成功");

        let retrieved = reg
            .get("doc", "frame_analysis", Some("v1.0"))
            .expect("查询应成功");
        assert_eq!(retrieved.content, "分析帧: ${frame}");
    }

    /// 注册两个不同 category 的 prompt，验证 list_categories 返回两个
    #[test]
    fn test_list_categories() {
        let mut reg = PromptRegistry::new();
        let p1 = make_test_prompt("doc", "narration", "v1.0", "解说模板");
        let p2 = make_test_prompt("sde", "plot", "v1.0", "剧情提取");
        reg.register(p1, true).expect("注册 doc 应成功");
        reg.register(p2, true).expect("注册 sde 应成功");

        let categories = reg.list_categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"doc".to_string()));
        assert!(categories.contains(&"sde".to_string()));
    }

    /// 查询不存在的 prompt 返回 PromptError::NotFound
    #[test]
    fn test_get_not_found() {
        let reg = PromptRegistry::new();
        let result = reg.get("doc", "nonexistent", Some("v1.0"));
        assert!(result.is_err());
        match result {
            Err(PromptError::NotFound { .. }) => {} // 正确
            _ => panic!("应为 NotFound 错误"),
        }
    }

    /// 注册重复版本返回 PromptError::Registration
    #[test]
    fn test_duplicate_registration() {
        let mut reg = PromptRegistry::new();
        let p1 = make_test_prompt("doc", "test", "v1.0", "第一个");
        reg.register(p1, true).expect("第一次注册应成功");

        let p2 = make_test_prompt("doc", "test", "v1.0", "第二个");
        let result = reg.register(p2, true);
        assert!(result.is_err());
        match result {
            Err(PromptError::Registration(_)) => {} // 正确
            _ => panic!("应为 Registration 错误"),
        }
    }

    /// search 按名称部分匹配返回结果
    #[test]
    fn test_search_by_name() {
        let mut reg = PromptRegistry::new();
        let p1 = make_test_prompt("doc", "frame_analysis", "v1.0", "帧分析");
        let p2 = make_test_prompt("doc", "narration_gen", "v1.0", "解说生成");
        reg.register(p1, true).expect("注册 frame_analysis 应成功");
        reg.register(p2, true).expect("注册 narration_gen 应成功");

        let results = reg.search("frame");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.name, "frame_analysis");
    }

    /// 默认版本 fallback 正确工作
    #[test]
    fn test_default_version_fallback() {
        let mut reg = PromptRegistry::new();
        let p1 = make_test_prompt("doc", "test", "v1.0", "旧版本");
        let p2 = make_test_prompt("doc", "test", "v2.0", "新版本");

        // 先注册 v1.0 为默认，再注册 v2.0 为默认
        reg.register(p1, true).expect("注册 v1.0 应成功");
        reg.register(p2, true).expect("注册 v2.0 应成功");

        // 不指定版本应返回 v2.0（最新默认）
        let retrieved = reg.get("doc", "test", None).expect("默认版本查询应成功");
        assert_eq!(retrieved.content, "新版本");
    }

    /// list_prompts 返回指定 category 的 prompt（按 name 排序、取默认版本）
    #[test]
    fn test_list_prompts_in_category() {
        let mut reg = PromptRegistry::new();
        let p1 = make_test_prompt("doc", "narration", "v1.0", "解说");
        let p2 = make_test_prompt("doc", "frame", "v1.0", "帧分析");
        reg.register(p1, true).expect("注册 narration 应成功");
        reg.register(p2, true).expect("注册 frame 应成功");

        let prompts = reg.list_prompts("doc");
        assert_eq!(prompts.len(), 2);
        // 按 name 排序：frame 应在前，narration 应在后
        assert_eq!(prompts[0].metadata.name, "frame");
        assert_eq!(prompts[1].metadata.name, "narration");
    }
}
