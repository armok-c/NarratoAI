use crate::error::TTSError;
use crate::tts::TtsOutput;
use std::path::Path;

/// 代理配置（从 ProxySection 提取的简化结构体）
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub http: String,
    pub https: String,
    pub enabled: bool,
}

impl ProxyConfig {
    /// 从可选的 ProxySection 引用创建 ProxyConfig
    pub fn from_proxy(proxy: Option<&crate::config::types::ProxySection>) -> Self {
        match proxy {
            Some(p) => Self {
                http: p.http.clone(),
                https: p.https.clone(),
                enabled: p.enabled,
            },
            None => Self {
                http: String::new(),
                https: String::new(),
                enabled: false,
            },
        }
    }

    /// 将代理配置应用到 reqwest ClientBuilder
    pub fn apply_to_client(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        if !self.enabled {
            return builder;
        }
        let mut builder = builder;
        if !self.https.is_empty() {
            if let Ok(proxy) = reqwest::Proxy::https(&self.https) {
                builder = builder.proxy(proxy);
            }
        }
        if !self.http.is_empty() {
            if let Ok(proxy) = reqwest::Proxy::http(&self.http) {
                builder = builder.proxy(proxy);
            }
        }
        builder
    }
}

/// 构建 reqwest HTTP 客户端
///
/// 使用 ProxyConfig 配置代理，不设置全局超时（各引擎在具体请求上设置 timeout）
pub fn build_client(proxy_config: &ProxyConfig) -> Result<reqwest::Client, TTSError> {
    let builder = reqwest::Client::builder();
    let builder = proxy_config.apply_to_client(builder);
    builder.build().map_err(|e| {
        TTSError::ConnectionFailed(format!(
            "构建 HTTP 客户端失败（代理配置可能已丢失）: {}", e
        ))
    })
}

/// 带重试的 TTS 合成执行器
///
/// 执行指定闭包 `f` 最多 4 次（1 次初始 + 3 次重试），间隔 1 秒。
/// - `AuthenticationFailed` 直接返回不重试
/// - 重试耗尽返回 `TTSError::RetryExhausted` 带最后错误信息
/// - 调用方负责在重试耗尽后清理可能残留的损坏输出文件
pub async fn retry_loop<F, Fut>(f: F) -> Result<TtsOutput, TTSError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<TtsOutput, TTSError>>,
{
    let max_attempts = 4; // 1 initial + 3 retries
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match f().await {
            Ok(output) => return Ok(output),
            Err(e) => {
                if matches!(&e, TTSError::AuthenticationFailed(_)) {
                    return Err(e);
                }
                last_error = Some(e);
                if attempt < max_attempts {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(TTSError::RetryExhausted(format!(
        "TTS 重试 {} 次后仍失败: {}",
        max_attempts - 1,
        last_error.map_or("未知错误".to_string(), |e| e.to_string())
    )))
}

/// 将音频字节写入文件
pub async fn write_audio_bytes(path: &Path, data: &[u8]) -> Result<(), TTSError> {
    tokio::fs::write(path, data)
        .await
        .map_err(|e| TTSError::SynthesisFailed(format!("写入音频文件失败: {}", e)))
}

/// XML 属性值转义（defense-in-depth）
///
/// 对插入 SSML XML 属性值中的外部字符串进行转义，
/// 防止双引号或 XML 特殊字符破坏 SSML 结构。
pub fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

/// 解析 TTS 引擎语音名前缀
///
/// 检查 `voice_name` 是否以 `prefixes` 中的某个字符串开头。
/// 匹配则返回去除前缀后的子串；不匹配则返回原始的 `voice_name`。
///
/// 示例:
/// ```
/// parse_engine_prefix("soulvoice:speech:model:voice:id", &["soulvoice:"]) → "speech:model:voice:id"
/// parse_engine_prefix("zh-CN-XiaoyiNeural", &["soulvoice:", "tencent:"]) → "zh-CN-XiaoyiNeural"
/// ```
pub fn parse_engine_prefix<'a>(voice_name: &'a str, prefixes: &[&str]) -> &'a str {
    for prefix in prefixes {
        if let Some(rest) = voice_name.strip_prefix(prefix) {
            return rest;
        }
    }
    voice_name
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_engine_prefix_matches_soulvoice() {
        let result = parse_engine_prefix("soulvoice:speech:model:voice:id", &["soulvoice:"]);
        assert_eq!(result, "speech:model:voice:id");
    }

    #[test]
    fn test_parse_engine_prefix_matches_tencent() {
        let result = parse_engine_prefix("tencent:101001", &["tencent:"]);
        assert_eq!(result, "101001");
    }

    #[test]
    fn test_parse_engine_prefix_no_match() {
        let result = parse_engine_prefix("zh-CN-XiaoyiNeural", &["soulvoice:", "tencent:"]);
        assert_eq!(result, "zh-CN-XiaoyiNeural");
    }

    #[test]
    fn test_proxy_config_from_none() {
        let cfg = ProxyConfig::from_proxy(None);
        assert!(!cfg.enabled);
        assert!(cfg.http.is_empty());
        assert!(cfg.https.is_empty());
    }

    #[test]
    fn test_proxy_config_from_some() {
        let section = crate::config::types::ProxySection {
            http: "http://127.0.0.1:7890".to_string(),
            https: "http://127.0.0.1:7890".to_string(),
            enabled: true,
        };
        let cfg = ProxyConfig::from_proxy(Some(&section));
        assert!(cfg.enabled);
        assert_eq!(cfg.http, "http://127.0.0.1:7890");
        assert_eq!(cfg.https, "http://127.0.0.1:7890");
    }

    #[test]
    fn test_write_audio_bytes() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("test_audio.mp3");
        let data = b"fake audio data";

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(write_audio_bytes(&path, data));

        assert!(result.is_ok());
        assert!(path.exists());
        let written = std::fs::read(&path).expect("读取写入的文件失败");
        assert_eq!(written, data);
    }
}
