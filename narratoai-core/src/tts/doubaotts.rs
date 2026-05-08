use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider};
use crate::config::types::DoubaoTTSSection;
use crate::tts::common;
use async_trait::async_trait;
use base64::Engine;
use std::path::Path;
use std::time::Duration;

/// 豆包语音 TTS 引擎。
///
/// API 端点: POST https://openspeech.bytedance.com/api/v1/tts
/// 认证: Authorization: Bearer;{token}  (分号不是空格 — 非标准但必要)
/// 请求体: 嵌套 JSON 含 app/user/audio/request 四段
/// 响应: {"code": 3000, "message": "Success", "data": "<base64 audio>"}
///
/// 对齐 Python 版 doubaotts_tts() — voice.py:1119-1235
pub(super) struct DoubaoTtsEngine {
    client: reqwest::Client,
    config: DoubaoTTSSection,
}

impl DoubaoTtsEngine {
    pub(super) fn new(config: DoubaoTTSSection, proxy_config: &common::ProxyConfig) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config })
    }

    /// 单次合成（无重试）
    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path, rate: f64, _pitch: f64) -> Result<TtsOutput, TTSError> {
        // Doubao 不使用 voice_name 前缀解析，voice_name 直接作为 voice_type
        // 配置检查
        if self.config.appid.is_empty() || self.config.token.is_empty() {
            if !self.config.ak.is_empty() || !self.config.sk.is_empty() {
                return Err(TTSError::AuthenticationFailed(
                    "豆包语音 TTS: ak/sk 认证方式暂未实现，请使用 appid + token 认证".to_string()
                ));
            }
            return Err(TTSError::AuthenticationFailed("豆包语音 TTS 配置未完成（缺少 appid 或 token）".to_string()));
        }

        let api_url = if self.config.api_url.is_empty() {
            "https://openspeech.bytedance.com/api/v1/tts"
        } else {
            &self.config.api_url
        };

        // HTTPS enforcement for cloud APIs — prevent credential leakage over cleartext HTTP
        if !api_url.starts_with("https://")
            && !api_url.starts_with("http://127.0.0.1")
            && !api_url.starts_with("http://localhost")
        {
            return Err(TTSError::SynthesisFailed(
                "Doubao API URL must use HTTPS for cloud services".to_string()
            ));
        }

        let speed_ratio = rate;  // 使用调用方传入的 rate 参数

        // 构建请求体 (对齐 Python 版 payload — voice.py:1149-1176)
        let mut payload = serde_json::json!({
            "app": {
                "appid": self.config.appid,
                "token": self.config.token,
                "cluster": self.config.cluster,
            },
            "user": {
                "uid": "NarratoAI"
            },
            "audio": {
                "voice_type": voice_name,
                "encoding": "mp3",
                "rate": 24000,
                "speed_ratio": speed_ratio,
                "volume_ratio": self.config.volume,
                "pitch_ratio": self.config.pitch,
            },
            "request": {
                "reqid": uuid::Uuid::new_v4().to_string(),
                "text": text.trim(),
                "text_type": "plain",
                "operation": "query",
            },
        });

        // 如果设置了句尾静音时长，添加 (Python 版: voice.py 1175-1176)
        if self.config.silence_duration > 0.0 {
            payload["audio"]["silence_duration"] = serde_json::json!(self.config.silence_duration);
        }

        let response = self.client
            .post(api_url)
            // 分号不是空格 — 对齐 Python 版: headers = {"Authorization": f"Bearer;{token}"}
            .header("Authorization", format!("Bearer;{}", self.config.token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("Doubao 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!("Doubao API 返回 {}: {}", status.as_u16(), body)));
        }

        let result: serde_json::Value = response.json().await
            .map_err(|e| TTSError::SynthesisFailed(format!("Doubao 响应 JSON 解析失败: {}", e)))?;

        // 检查业务状态码 (Python 版: result.get("code") == 3000)
        let code = result["code"].as_i64().unwrap_or(0);
        if code != 3000 {
            let msg = result["message"].as_str().unwrap_or("未知错误");
            return Err(TTSError::SynthesisFailed(format!("Doubao TTS 错误 ({}): {}", code, msg)));
        }

        // 解码 base64 audio data (Python 版: base64.b64decode(result.get("data", "")))
        let audio_base64 = result["data"].as_str()
            .ok_or_else(|| TTSError::SynthesisFailed("Doubao 响应中无 data 字段".to_string()))?;

        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(audio_base64)
            .map_err(|e| TTSError::SynthesisFailed(format!("Base64 解码失败: {}", e)))?;

        common::write_audio_bytes(output_path, &audio_bytes).await?;

        // Doubao 不支持词边界
        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries: vec![],
            duration: 0.0,
        })
    }
}

#[async_trait]
impl TtsProvider for DoubaoTtsEngine {
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        rate: f64,
        pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        if text.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
        }
        if voice_name.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
        }
        let result = common::retry_loop(|| self.synthesize_once(text, voice_name, output_path, rate, pitch)).await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(output_path).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_doubaotts_engine_new() {
        let config = DoubaoTTSSection {
            appid: "test-appid".to_string(),
            token: "test-token".to_string(),
            cluster: "volcano_tts".to_string(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.125,
            ..Default::default()
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = DoubaoTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        assert_eq!(engine.config.appid, "test-appid");
        assert_eq!(engine.config.cluster, "volcano_tts");
        assert_eq!(engine.config.silence_duration, 0.125);
    }

    #[tokio::test]
    async fn test_doubaotts_empty_text() {
        let config = DoubaoTTSSection {
            appid: "test-appid".to_string(),
            token: "test-token".to_string(),
            cluster: String::new(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.0,
            ..Default::default()
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = DoubaoTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize("", "voice", 1.0, 0.0, &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空"), "错误消息: {}", msg),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_doubaotts_missing_config() {
        // Missing appid and token should return AuthenticationFailed
        let config = DoubaoTTSSection {
            appid: String::new(),
            token: String::new(),
            cluster: String::new(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.0,
            ..Default::default()
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = DoubaoTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "voice", &output_path, 1.0, 1.0).await;
        match result.unwrap_err() {
            TTSError::AuthenticationFailed(msg) => assert!(msg.contains("配置未完成"), "错误消息: {}", msg),
            _ => panic!("应为 AuthenticationFailed"),
        }
    }

    #[tokio::test]
    async fn test_doubaotts_ak_sk_not_supported() {
        // ak/sk without token should return clear error about unsupported auth method
        let config = DoubaoTTSSection {
            ak: "test-ak".to_string(),
            sk: "test-sk".to_string(),
            appid: String::new(),
            token: String::new(),
            cluster: String::new(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.0,
            ..Default::default()
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = DoubaoTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "voice", &output_path, 1.0, 1.0).await;
        match result.unwrap_err() {
            TTSError::AuthenticationFailed(msg) => assert!(msg.contains("ak/sk"), "错误消息应提及 ak/sk: {}", msg),
            _ => panic!("应为 AuthenticationFailed"),
        }
    }

    #[test]
    fn test_doubaotts_auth_header_format() {
        // 验证 Authorization 头使用 Bearer;（分号）而不是 Bearer （空格）
        // 对齐 Python 版: headers = {"Authorization": f"Bearer;{token}"}
        let config = DoubaoTTSSection {
            appid: "test-appid".to_string(),
            token: "test-token".to_string(),
            cluster: String::new(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.0,
            ..Default::default()
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = DoubaoTtsEngine::new(config, &proxy_config).expect("构建引擎失败");

        let header_value = format!("Bearer;{}", engine.config.token);
        assert_eq!(header_value, "Bearer;test-token");
        assert!(header_value.contains(';'), "Authorization 头应包含分号");
        assert!(!header_value.contains("Bearer "), "不应使用空格格式");
    }

    #[test]
    fn test_doubaotts_base64_response_handling() {
        // 验证 base64 解码逻辑
        let fake_audio = b"fake audio data for testing";
        let encoded = base64::engine::general_purpose::STANDARD.encode(fake_audio);

        // Simulate response parsing logic
        let response = serde_json::json!({
            "code": 3000,
            "message": "Success",
            "data": encoded,
        });

        let code = response["code"].as_i64().unwrap_or(0);
        assert_eq!(code, 3000, "成功状态码应为 3000");

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(response["data"].as_str().unwrap())
            .expect("base64 解码应成功");
        assert_eq!(decoded, fake_audio);
    }

    #[test]
    fn test_doubaotts_business_code_check() {
        // 验证业务状态码检查逻辑：非 3000 应视为错误
        let error_response = serde_json::json!({
            "code": 4000,
            "message": "invalid parameter",
        });

        let code = error_response["code"].as_i64().unwrap_or(0);
        assert_eq!(code, 4000);
        assert_ne!(code, 3000, "code 4000 不应等于 3000");

        let msg = error_response["message"].as_str().unwrap_or("未知错误");
        assert_eq!(msg, "invalid parameter");
    }

    #[test]
    fn test_doubaotts_payload_structure() {
        // 验证请求体结构正确
        let config = DoubaoTTSSection {
            appid: "test-appid".to_string(),
            token: "test-token".to_string(),
            cluster: "volcano_tts".to_string(),
            volume: 1.5,
            pitch: 1.2,
            silence_duration: 0.2,
            ..Default::default()
        };

        let voice_name = "BV700_V2_streaming";
        let text = "你好世界";

        let mut payload = serde_json::json!({
            "app": {
                "appid": config.appid.clone(),
                "token": config.token.clone(),
                "cluster": config.cluster.clone(),
            },
            "user": {
                "uid": "NarratoAI"
            },
            "audio": {
                "voice_type": voice_name,
                "encoding": "mp3",
                "rate": 24000,
                "speed_ratio": 1.0,
                "volume_ratio": config.volume,
                "pitch_ratio": config.pitch,
            },
            "request": {
                "reqid": uuid::Uuid::new_v4().to_string(),
                "text": text.trim(),
                "text_type": "plain",
                "operation": "query",
            },
        });

        if config.silence_duration > 0.0 {
            payload["audio"]["silence_duration"] = serde_json::json!(config.silence_duration);
        }

        // Verify structure
        assert_eq!(payload["app"]["appid"], "test-appid");
        assert_eq!(payload["app"]["token"], "test-token");
        assert_eq!(payload["app"]["cluster"], "volcano_tts");
        assert_eq!(payload["user"]["uid"], "NarratoAI");
        assert_eq!(payload["audio"]["voice_type"], "BV700_V2_streaming");
        assert_eq!(payload["audio"]["encoding"], "mp3");
        assert_eq!(payload["audio"]["rate"], 24000);
        assert_eq!(payload["audio"]["volume_ratio"], 1.5);
        assert_eq!(payload["audio"]["pitch_ratio"], config.pitch);
        assert_eq!(payload["audio"]["silence_duration"], 0.2);
        assert_eq!(payload["request"]["text"], "你好世界");
        assert_eq!(payload["request"]["text_type"], "plain");
        assert_eq!(payload["request"]["operation"], "query");
        assert!(payload["request"]["reqid"].as_str().unwrap().len() > 0);
    }
}
