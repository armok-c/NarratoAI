use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider};
use crate::config::types::TtsQwenSection;
use crate::tts::common;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

/// Qwen TTS 引擎 (DashScope REST API)
///
/// API 流程:
/// 1. POST https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization
///    Authorization: Bearer {api_key}
///    请求体: {"model": "qwen3-tts-flash", "input": {"text": "...", "voice": "..."}}
///    响应: {"output": {"audio": {"url": "...", "duration": 12.5}}}
/// 2. GET audio_url → 下载音频字节
///
/// 对齐 Python 版 qwen3_tts() — voice.py:1840-1928
pub(super) struct QwenTtsEngine {
    client: reqwest::Client,
    config: TtsQwenSection,
}

impl QwenTtsEngine {
    pub(super) fn new(config: TtsQwenSection, proxy_config: &common::ProxyConfig) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config })
    }

    /// 单次合成（无重试）
    /// 步骤: 1) 构建并发送 TTS 请求  2) 从响应解析 audio_url  3) 下载音频  4) 写入文件
    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path) -> Result<TtsOutput, TTSError> {
        // 解析 voice_name: 去除 "qwen3:" 前缀 (Python 版 parse_qwen3_voice — voice.py:1831)
        let parsed_voice = common::parse_engine_prefix(voice_name, &["qwen3:"]);

        let api_key = &self.config.api_key;
        if api_key.is_empty() {
            return Err(TTSError::AuthenticationFailed("Qwen TTS API key 未配置".to_string()));
        }

        let model_name = if self.config.model_name.is_empty() {
            "qwen3-tts-flash"
        } else {
            &self.config.model_name
        };

        // Step 1: 发送 TTS 合成请求到 DashScope
        // 使用配置的 api_url（便于测试），回退到默认 DashScope 端点
        let api_url = if self.config.api_url.is_empty() {
            const DASHSCOPE_TTS_URL: &str =
                "https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization";
            DASHSCOPE_TTS_URL
        } else {
            &self.config.api_url
        };

        let payload = serde_json::json!({
            "model": model_name,
            "input": {
                "text": text.trim(),
                "voice": parsed_voice,
            },
        });

        let response = self.client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("Qwen 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!(
                "Qwen API 返回 {}: {}", status.as_u16(), body
            )));
        }

        // Step 2: 解析响应，提取 audio URL 和 duration
        let result: serde_json::Value = response.json().await
            .map_err(|e| TTSError::SynthesisFailed(format!("Qwen 响应 JSON 解析失败: {}", e)))?;

        let audio_url = result["output"]["audio"]["url"].as_str()
            .ok_or_else(|| TTSError::SynthesisFailed("Qwen 响应中无 output.audio.url".to_string()))?;
        let duration = result["output"]["audio"]["duration"].as_f64().unwrap_or(0.0);

        // Step 3: 下载音频文件
        // Python 版: requests.get(audio_url, timeout=30)
        let audio_response = self.client
            .get(audio_url)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::SynthesisFailed(format!("Qwen 音频下载失败: {}", e)))?;

        let audio_status = audio_response.status();
        if !audio_status.is_success() {
            return Err(TTSError::SynthesisFailed(format!(
                "Qwen 音频 URL 返回 {} (期望 200)", audio_status.as_u16()
            )));
        }

        let audio_bytes = audio_response.bytes().await
            .map_err(|e| TTSError::SynthesisFailed(format!("Qwen 读取音频响应失败: {}", e)))?;

        // Step 4: 写入文件
        common::write_audio_bytes(output_path, &audio_bytes).await?;

        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries: vec![],
            duration,
        })
    }
}

#[async_trait]
impl TtsProvider for QwenTtsEngine {
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
        if voice_name.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
        }
        let result = common::retry_loop(|| self.synthesize_once(text, voice_name, output_path)).await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(output_path).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use tempfile::TempDir;

    #[test]
    fn test_qwen_engine_new() {
        let config = TtsQwenSection {
            api_key: "test-key".to_string(),
            api_url: String::new(),
            model_name: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = QwenTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        assert_eq!(engine.config.api_key, "test-key");
    }

    #[tokio::test]
    async fn test_qwen_success() {
        let mock_server = MockServer::start().await;
        let audio_bytes_data = b"fake audio data";

        // Step 1: Mock POST 返回 JSON 含 audio URL
        let audio_url = format!("{}/audio.mp3", mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/v1/services/audio/tts/customization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "output": {
                    "audio": {
                        "url": audio_url,
                        "duration": 12.5,
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        // Step 2: Mock GET audio URL 返回音频字节
        Mock::given(method("GET"))
            .and(path("/audio.mp3"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(audio_bytes_data))
            .mount(&mock_server)
            .await;

        let config = TtsQwenSection {
            api_key: "test-key".to_string(),
            api_url: format!("{}/api/v1/services/audio/tts/customization", mock_server.uri()),
            model_name: "qwen3-tts-flash".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = QwenTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "qwen3:Cherry", &output_path).await;
        assert!(result.is_ok(), "Qwen 成功: {:?}", result.err());
        let output = result.unwrap();
        assert!((output.duration - 12.5).abs() < f64::EPSILON, "duration 应为 12.5, got {}", output.duration);
        assert!(output.audio_file_path.exists());
        let written = std::fs::read(&output.audio_file_path).expect("读取输出文件失败");
        assert_eq!(written, audio_bytes_data);
    }

    #[tokio::test]
    async fn test_qwen_empty_text() {
        let config = TtsQwenSection {
            api_key: "test-key".to_string(),
            api_url: String::new(),
            model_name: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = QwenTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize("", "qwen3:Cherry", 1.0, 0.0, &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_qwen_no_audio_url() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/services/audio/tts/customization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "output": {
                    "audio": {}
                }
            })))
            .mount(&mock_server)
            .await;

        let config = TtsQwenSection {
            api_key: "test-key".to_string(),
            api_url: format!("{}/api/v1/services/audio/tts/customization", mock_server.uri()),
            model_name: String::new(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = QwenTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "qwen3:Cherry", &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("output.audio.url")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[test]
    fn test_qwen_parse_prefix() {
        let parsed = common::parse_engine_prefix("qwen3:Cherry", &["qwen3:"]);
        assert_eq!(parsed, "Cherry");
    }
}
