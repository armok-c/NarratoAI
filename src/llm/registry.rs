use std::collections::HashMap;
use std::sync::Arc;

use crate::error::LLMError;
use crate::llm::provider::LlmProvider;

/// Provider 注册中心
///
/// 运行时 trait object 注册模式（D-05）：HashMap 持有 Arc<dyn LlmProvider>。
/// 按 provider 名字符串（小写）索引。
pub struct Registry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
}

impl Registry {
    /// 创建空注册中心
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// 注册 provider，name 自动转为小写（D-02）
    pub fn register(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(name.to_lowercase(), provider);
    }

    /// 按名称查询 provider。未找到时返回 LLMError::ProviderNotFound（D-14）
    #[must_use]
    pub fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>, LLMError> {
        self.providers
            .get(&name.to_lowercase())
            .cloned()
            .ok_or_else(|| LLMError::ProviderNotFound(name.to_string()))
    }

    /// 返回已注册的所有 provider 名称列表
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
