use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider};
use crate::config::types::SoulVoiceSection;
use crate::tts::common;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

/// SoulVoice TTS 引擎
///
/// 通过 REST API 将文本合成为语音。
/// API 端点: 配置中的 api_url 或默认值 https://tts.scsmtech.cn/tts
/// 认证: Authorization: Bearer {api_key}
/// 请求体: { "text": "...", "model": "...", "voice": "...", "speed": 1.0 }
///
/// 对应 Python 版 app/services/voice.py -> soulvoice_tts()
pub(super) struct SoulVoiceEngine {
    client: reqwest::Client,
    config: SoulVoiceSection,
}

impl SoulVoiceEngine {
    pub(super) fn new(config: SoulVoiceSection, proxy_config: &common::ProxyConfig) -> Self {
        let client = common::build_client(proxy_config);
        Self { client, config }
    }

    /// 单次 TTS 合成（无重试）
    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path) -> Result<TtsOutput, TTSError> {
        // 解析 voice_name: 去除 "soulvoice:" 前缀
        // 对齐 Python 版 parse_soulvoice_voice() 函数
        let parsed_voice = common::parse_engine_prefix(voice_name, &["soulvoice:"]);

        let api_key = &self.config.api_key;
        if api_key.is_empty() {
            return Err(TTSError::AuthenticationFailed("SoulVoice API key 未配置".to_string()));
        }

        let api_url = if self.config.api_url.is_empty() {
            "https://tts.scsmtech.cn/tts"
        } else {
            &self.config.api_url
        };

        let model = if self.config.model.is_empty() {
            "FunAudioLLM/CosyVoice2-0.5B"
        } else {
            &self.config.model
        };

        // 请求体: {"text": "...", "model": "...", "voice": "...", "speed": 1.0}
        // 对齐 Python 版: headers={'Authorization': f'Bearer {api_key}', 'Content-Type': 'application/json'}
        // data = {'text': text.strip(), 'model': default_model, 'voice': parsed_voice, 'speed': speed}
        let request_body = serde_json::json!({
            "text": text.trim(),
            "model": model,
            "voice": parsed_voice,
            "speed": 1.0,
        });

        let response = self.client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("SoulVoice 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!(
                "SoulVoice API 返回 {}: {}", status.as_u16(), body_text
            )));
        }

        let audio_bytes = response.bytes().await
            .map_err(|e| TTSError::SynthesisFailed(format!("SoulVoice 读取响应失败: {}", e)))?;

        common::write_audio_bytes(output_path, &audio_bytes).await?;

        // SoulVoice 不支持词边界
        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries: vec![],
            duration: 0.0,
        })
    }
}

#[async_trait]
impl TtsProvider for SoulVoiceEngine {
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        _rate: f64,
        _pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        if text.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
        }
        common::retry_loop(|| self.synthesize_once(text, voice_name, output_path)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use tempfile::TempDir;

    #[test]
    fn test_soulvoice_engine_new() {
        let config = SoulVoiceSection {
            api_key: "test-key".to_string(),
            api_url: String::new(),
            model: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = SoulVoiceEngine::new(config, &proxy_config);
        assert_eq!(engine.config.api_key, "test-key");
    }

    #[tokio::test]
    async fn test_soulvoice_success() {
        let mock_server = MockServer::start().await;
        let mock_response = b"fake audio data";

        Mock::given(method("POST"))
            .and(path("/tts"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(mock_response)
            )
            .mount(&mock_server)
            .await;

        let config = SoulVoiceSection {
            api_key: "test-key".to_string(),
            api_url: mock_server.uri() + "/tts",
            model: "test-model".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = SoulVoiceEngine::new(config, &proxy_config);
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        // Override retry_loop by calling synthesize_once directly to avoid retry complexity
        let result = engine.synthesize_once("test text", "soulvoice:speech:voice:id", &output_path).await;
        assert!(result.is_ok(), "soulvoice 成功: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists());
        assert!(output.word_boundaries.is_empty());
    }

    #[tokio::test]
    async fn test_soulvoice_empty_text() {
        let config = SoulVoiceSection {
            api_key: "test-key".to_string(),
            api_url: String::new(),
            model: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = SoulVoiceEngine::new(config, &proxy_config);
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize("", "voice", 1.0, 0.0, &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_soulvoice_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tts"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&mock_server)
            .await;

        let config = SoulVoiceSection {
            api_key: "bad-key".to_string(),
            api_url: mock_server.uri() + "/tts",
            model: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = SoulVoiceEngine::new(config, &proxy_config);
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "soulvoice:voice", &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("401") || msg.contains("unauthorized")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[test]
    fn test_soulvoice_parse_prefix() {
        let parsed = common::parse_engine_prefix("soulvoice:speech:model:voice:id", &["soulvoice:"]);
        assert_eq!(parsed, "speech:model:voice:id");
    }
}
