use crate::config::types::AppConfig;
use crate::error::LLMError;
use crate::llm::openai_compatible::{OpenAiCompatibleProvider, ProviderConfig};
use crate::llm::registry::Registry;
use std::sync::Arc;

/// Vision provider 在注册中心中的默认名称
pub const VISION_PROVIDER_NAME: &str = "openai_vision";
/// Text provider 在注册中心中的默认名称
pub const TEXT_PROVIDER_NAME: &str = "openai_text";

/// 注册错误的包装类型，实现 std::error::Error 以便在统一错误处理链中传播
#[derive(Debug)]
pub struct RegistrationErrors(pub Vec<LLMError>);

impl std::fmt::Display for RegistrationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "provider 注册失败 ({} 个错误):", self.0.len())?;
        for (i, err) in self.0.iter().enumerate() {
            write!(f, "\n  [{}] {}", i + 1, err)?;
        }
        Ok(())
    }
}

impl std::error::Error for RegistrationErrors {}

/// 注册所有活跃 provider（D-07）
///
/// 在库初始化时调用。读取 AppConfig 的 vision/text 配置段和 proxy 配置，
/// 创建 OpenAiCompatibleProvider 实例并注册到 Registry。
///
/// 注册逻辑：
/// - 若 vision_openai_api_key 非空 → 注册 provider，名称取自 vision_llm_provider（若空则使用 VISION_PROVIDER_NAME）
/// - 若 text_openai_api_key 非空 → 注册 provider，名称取自 text_llm_provider（若空则使用 TEXT_PROVIDER_NAME）
/// - provider 名称与 D-08 的默认 provider 选择逻辑配合
/// - 使用配置中的 provider 名称作为注册键，以支持未来多 provider 实现共存
///
/// 代理支持（D-16）：proxy.enabled=true 时透传 proxy.http / proxy.https 到 provider 构造函数
///
/// # Errors
///
/// 返回所有注册失败的 LLMError 列表。即使部分 provider 注册失败，成功注册的
/// provider 仍然保留在 registry 中。调用者可以检查返回的错误以判断注册是否完全成功。
pub fn register_all_providers(
    config: &AppConfig,
    registry: &mut Registry,
) -> Result<(), RegistrationErrors> {
    let proxy_http = if config.proxy.enabled && !config.proxy.http.is_empty() {
        Some(config.proxy.http.clone())
    } else {
        None
    };
    let proxy_https = if config.proxy.enabled && !config.proxy.https.is_empty() {
        Some(config.proxy.https.clone())
    } else {
        None
    };

    let mut errors: Vec<LLMError> = Vec::new();

    // 注册 vision provider（D-15: timeout from config）
    if !config.app.vision_openai_api_key.is_empty()
        && !config.app.vision_openai_model_name.is_empty()
    {
        let vision_name = if config.app.vision_llm_provider.is_empty() {
            VISION_PROVIDER_NAME
        } else {
            &config.app.vision_llm_provider
        };
        match OpenAiCompatibleProvider::new(ProviderConfig {
            api_key: config.app.vision_openai_api_key.clone(),
            model_name: config.app.vision_openai_model_name.clone(),
            base_url: config.app.vision_openai_base_url.clone(),
            max_retries: config.app.llm_max_retries,
            timeout_secs: config.app.llm_vision_timeout,
            proxy_http: proxy_http.clone(),
            proxy_https: proxy_https.clone(),
        }) {
            Ok(provider) => registry.register(vision_name, Arc::new(provider)),
            Err(e) => {
                tracing::error!("vision provider 注册失败: {}", e);
                errors.push(e);
            }
        }
    }

    // 注册 text provider（D-15: timeout from config）
    if !config.app.text_openai_api_key.is_empty() && !config.app.text_openai_model_name.is_empty() {
        let text_name = if config.app.text_llm_provider.is_empty() {
            TEXT_PROVIDER_NAME
        } else {
            &config.app.text_llm_provider
        };
        match OpenAiCompatibleProvider::new(ProviderConfig {
            api_key: config.app.text_openai_api_key.clone(),
            model_name: config.app.text_openai_model_name.clone(),
            base_url: config.app.text_openai_base_url.clone(),
            max_retries: config.app.llm_max_retries,
            timeout_secs: config.app.llm_text_timeout,
            proxy_http,
            proxy_https,
        }) {
            Ok(provider) => registry.register(text_name, Arc::new(provider)),
            Err(e) => {
                tracing::error!("text provider 注册失败: {}", e);
                errors.push(e);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(RegistrationErrors(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::AppConfig;

    /// 验证空 API key 时不注册任何 provider
    #[test]
    fn test_empty_api_key_registers_nothing() {
        let config = AppConfig::default(); // 默认配置 API key 为空
        let mut registry = Registry::new();
        let result = register_all_providers(&config, &mut registry);
        assert!(result.is_ok(), "空 API key 不应产生错误");
        assert!(
            registry.list_providers().is_empty(),
            "空 API key 不应注册 provider"
        );
    }

    /// 验证 vision 和 text 配置都提供时两个 provider 都注册
    #[test]
    fn test_both_providers_registered() {
        let config = AppConfig {
            app: crate::config::types::AppSection {
                vision_llm_provider: "openai_vision".to_string(),
                vision_openai_api_key: "sk-vision".to_string(),
                vision_openai_model_name: "gpt-4-vision".to_string(),
                vision_openai_base_url: "https://api.openai.com/v1".to_string(),
                text_llm_provider: "openai_text".to_string(),
                text_openai_api_key: "sk-text".to_string(),
                text_openai_model_name: "gpt-4".to_string(),
                text_openai_base_url: "https://api.openai.com/v1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut registry = Registry::new();
        let result = register_all_providers(&config, &mut registry);
        assert!(
            result.is_ok(),
            "provider 注册应全部成功: {:?}",
            result.err()
        );
        let providers = registry.list_providers();
        assert!(providers.contains(&"openai_vision".to_string()));
        assert!(providers.contains(&"openai_text".to_string()));
    }
}
