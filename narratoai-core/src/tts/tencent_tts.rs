use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider, WordBoundary};
use crate::config::types::TencentSection;
use crate::tts::common;
use async_trait::async_trait;
use base64::Engine;
use hmac::KeyInit;
use hmac::Mac;
use sha2::Digest;
use std::path::Path;
use std::time::Duration;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

/// 腾讯云 TTS 引擎（TC3-HMAC-SHA256 签名认证）
///
/// API: POST https://tts.tencentcloudapi.com
/// 认证: TC3-HMAC-SHA256 (手动实现，无 SDK — D-04)
/// 服务名: "tts"
/// 请求体: JSON (Text / SessionId / VoiceType / Speed / SampleRate / Codec / ...)
/// 响应: JSON {"Response": {"Audio": "<base64>", "Subtitles": [...]}}
/// 词边界: 从 Subtitles 数组生成 (BeginTime/EndTime in ms -> 100ns 单位)
///
/// 对齐 Python 版 tencent_tts() — voice.py:1936-2036
pub(super) struct TencentTtsEngine {
    client: reqwest::Client,
    config: TencentSection,
    /// 端点覆盖（用于 wiremock 测试，生产环境为 None）
    pub(super) endpoint_override: Option<String>,
}

impl TencentTtsEngine {
    pub(super) fn new(config: TencentSection, proxy_config: &common::ProxyConfig) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config, endpoint_override: None })
    }

    /// 实际执行一次 TTS 合成的内部方法
    async fn synthesize_at_url(&self, text: &str, voice_name: &str, output_path: &Path, api_url: &str, rate: f64) -> Result<TtsOutput, TTSError> {
        if self.config.secret_id.is_empty() || self.config.secret_key.is_empty() {
            return Err(TTSError::AuthenticationFailed("腾讯云 secret_id/secret_key 未配置".to_string()));
        }

        // 解析 voice_name: 去除 "tencent:" 前缀 = voice_type 数字
        // Python 版 parse_tencent_voice — voice.py:1821
        let voice_type_str = common::parse_engine_prefix(voice_name, &["tencent:"]);
        let voice_type: i64 = voice_type_str.parse().map_err(|_| {
            TTSError::SynthesisFailed(format!(
                "腾讯云 voice_type 必须是正整数, 收到: '{}'", voice_type_str
            ))
        })?;
        if voice_type <= 0 {
            return Err(TTSError::SynthesisFailed(format!(
                "腾讯云 voice_type 必须是正整数, 收到: '{}'", voice_type_str
            )));
        }

        // 速度映射: rate=1.0 -> speed=0, rate=1.5 -> speed=1.0, rate=0.5 -> speed=-1.0
        // Python 版: speed_value = max(-2.0, min(2.0, (speed - 1.0) * 2))
        let speed_value = ((rate - 1.0) * 2.0).clamp(-2.0, 2.0);

        let region = if self.config.region.is_empty() {
            "ap-beijing"
        } else {
            &self.config.region
        };

        // 构建请求体
        // 对齐 Python 版: req.Text = text 等 (voice.py:1986-1995)
        let session_id = uuid::Uuid::new_v4().to_string();
        let request_body = serde_json::json!({
            "Text": text,
            "SessionId": session_id,
            "VoiceType": voice_type,
            "Speed": speed_value,
            "SampleRate": 16000,
            "Codec": "mp3",
            "ProjectId": 0,
            "ModelType": 1,
            "PrimaryLanguage": 1,
            "EnableSubtitle": true,
        });

        let request_body_bytes = serde_json::to_vec(&request_body)
            .map_err(|e| TTSError::SynthesisFailed(format!("JSON 序列化失败: {}", e)))?;

        // ===== TC3-HMAC-SHA256 签名 =====
        // 对齐腾讯云官方 TC3 签名算法 (D-04: 手动实现)
        //
        // 时间戳和日期在同一时刻获取 (Pitfall 3: 防止时间偏差)
        let now = chrono::Utc::now();
        let timestamp = now.timestamp().to_string();
        let date = now.format("%Y-%m-%d").to_string();

        // 从 api_url 中提取 host（用于 TC3 签名和 Host 头），确保与目标服务器一致
        let host = api_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("tts.tencentcloudapi.com");

        // Step 1: 构建 CanonicalRequest
        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_query_string = "";
        let payload_hash = hex::encode(sha2::Sha256::digest(&request_body_bytes));
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\n", host
        );
        let signed_headers = "content-type;host";
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            payload_hash,
        );

        // Step 2: 构建 StringToSign
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/tts/tc3_request", date);
        let hashed_canonical_request = hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            timestamp,
            credential_scope,
            hashed_canonical_request,
        );

        // Step 3: 派生签名密钥
        // SecretKey -> SecretDate -> SecretService -> SecretSigning
        let secret_key = format!("TC3{}", self.config.secret_key);
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .map_err(|e| TTSError::SynthesisFailed(format!("HMAC 初始化失败: {}", e)))?;
        mac.update(date.as_bytes());
        let secret_date = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&secret_date)
            .map_err(|e| TTSError::SynthesisFailed(format!("HMAC 初始化失败: {}", e)))?;
        mac.update(b"tts");
        let secret_service = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&secret_service)
            .map_err(|e| TTSError::SynthesisFailed(format!("HMAC 初始化失败: {}", e)))?;
        mac.update(b"tc3_request");
        let secret_signing = mac.finalize().into_bytes();

        // Step 4: 计算 Signature
        let mut mac = HmacSha256::new_from_slice(&secret_signing)
            .map_err(|e| TTSError::SynthesisFailed(format!("HMAC 初始化失败: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Step 5: 构建 Authorization header
        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.config.secret_id,
            credential_scope,
            signed_headers,
            signature,
        );

        // ===== 发送请求 =====
        let response = self.client
            .post(api_url)
            .header("Authorization", &authorization)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", host)
            .header("X-TC-Action", "TextToVoice")
            .header("X-TC-Timestamp", &timestamp)
            .header("X-TC-Version", "2019-08-23")
            .header("X-TC-Region", region)
            .body(request_body_bytes)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("Tencent 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!(
                "Tencent API 返回 {}: {}", status.as_u16(), body_text
            )));
        }

        // ===== 解析响应 =====
        let result: serde_json::Value = response.json().await
            .map_err(|e| TTSError::SynthesisFailed(format!("Tencent 响应 JSON 解析失败: {}", e)))?;

        // 提取 Audio (base64) 和 Subtitles
        // Python 版: resp.Audio 和 resp.Subtitles (voice.py:2008-2026)
        let resp = &result["Response"];

        // 检查错误
        if let Some(err_msg) = resp["Error"]["Message"].as_str() {
            return Err(TTSError::SynthesisFailed(format!("腾讯云 API 错误: {}", err_msg)));
        }

        let audio_base64 = resp["Audio"].as_str()
            .ok_or_else(|| TTSError::SynthesisFailed("Tencent 响应中无 Audio 字段".to_string()))?;

        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(audio_base64)
            .map_err(|e| TTSError::SynthesisFailed(format!("Base64 解码失败: {}", e)))?;

        // 生成 WordBoundary 列表（从 Subtitles 数组）
        // Python 版: resp.Subtitles 中每个 sub 有 BeginTime, EndTime, Text
        // 单位转换: ms -> 100ns (x10000)
        let mut word_boundaries: Vec<WordBoundary> = Vec::new();
        if let Some(subtitles) = resp["Subtitles"].as_array() {
            for sub in subtitles {
                if let (Some(start_ms), Some(end_ms), Some(text_val)) = (
                    sub["BeginTime"].as_i64().or_else(|| sub["BeginTime"].as_f64().map(|f| f as i64)),
                    sub["EndTime"].as_i64().or_else(|| sub["EndTime"].as_f64().map(|f| f as i64)),
                    sub["Text"].as_str(),
                ) {
                    if start_ms < 0 || end_ms < 0 {
                        tracing::warn!(
                            "Tencent subtitle has negative timestamp: start={}, end={}, text={}",
                            start_ms, end_ms, text_val
                        );
                    }
                    let start_offset = if start_ms >= 0 { (start_ms as u64) * 10000 } else { 0 };
                    let end_offset = if end_ms >= 0 { (end_ms as u64) * 10000 } else { 0 };
                    word_boundaries.push(WordBoundary {
                        start_offset,
                        end_offset,
                        text: text_val.to_string(),
                    });
                }
            }
        }

        common::write_audio_bytes(output_path, &audio_bytes).await?;

        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries,
            duration: 0.0, // Subtitles 不直接包含总时长
        })
    }

    /// 使用生产 API URL 执行一次 TTS 合成
    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path, rate: f64) -> Result<TtsOutput, TTSError> {
        if let Some(ref override_url) = self.endpoint_override {
            self.synthesize_at_url(text, voice_name, output_path, override_url, rate).await
        } else {
            const PRODUCTION_URL: &str = "https://tts.tencentcloudapi.com";
            self.synthesize_at_url(text, voice_name, output_path, PRODUCTION_URL, rate).await
        }
    }
}

#[async_trait]
impl TtsProvider for TencentTtsEngine {
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        rate: f64,
        _pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        if text.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
        }
        if voice_name.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
        }
        let result = common::retry_loop(|| self.synthesize_once(text, voice_name, output_path, rate)).await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(output_path).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use tempfile::TempDir;

    #[test]
    fn test_tencent_engine_new() {
        let config = TencentSection {
            secret_id: "test-id".to_string(),
            secret_key: "test-key".to_string(),
            region: "ap-beijing".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = TencentTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        assert_eq!(engine.config.secret_id, "test-id");
        assert!(engine.endpoint_override.is_none());
    }

    #[tokio::test]
    async fn test_tencent_success() {
        let mock_server = MockServer::start().await;

        let fake_audio = b"fake audio data for testing";
        let encoded = base64::engine::general_purpose::STANDARD.encode(fake_audio);

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "Response": {
                    "Audio": encoded,
                    "Subtitles": [
                        {"BeginTime": 0, "EndTime": 1500, "Text": "你好"}
                    ]
                }
            })))
            .mount(&mock_server)
            .await;

        let config = TencentSection {
            secret_id: "test-id".to_string(),
            secret_key: "test-key".to_string(),
            region: "ap-beijing".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let mut engine = TencentTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        engine.endpoint_override = Some(mock_server.uri());

        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("你好世界", "tencent:101001", &output_path, 1.0).await;
        assert!(result.is_ok(), "Tencent 成功: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists());
        let written = std::fs::read(&output.audio_file_path).expect("读取文件失败");
        assert_eq!(written, fake_audio);
        assert_eq!(output.word_boundaries.len(), 1);
        assert_eq!(output.word_boundaries[0].start_offset, 0);
        assert_eq!(output.word_boundaries[0].end_offset, 15000000); // 1500ms * 10000
        assert_eq!(output.word_boundaries[0].text, "你好");
    }

    #[tokio::test]
    async fn test_tencent_empty_text() {
        let config = TencentSection {
            secret_id: "test-id".to_string(),
            secret_key: "test-key".to_string(),
            region: "ap-beijing".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = TencentTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize("", "tencent:101001", 1.0, 0.0, &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空"), "错误消息: {}", msg),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_tencent_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "Response": {
                    "Error": {"Message": "InvalidParameter"}
                }
            })))
            .mount(&mock_server)
            .await;

        let config = TencentSection {
            secret_id: "test-id".to_string(),
            secret_key: "test-key".to_string(),
            region: "ap-beijing".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let mut engine = TencentTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        engine.endpoint_override = Some(mock_server.uri());

        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test", "tencent:101001", &output_path, 1.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("InvalidParameter"), "错误消息: {}", msg),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_tencent_empty_audio() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "Response": {
                    "Subtitles": []
                }
            })))
            .mount(&mock_server)
            .await;

        let config = TencentSection {
            secret_id: "test-id".to_string(),
            secret_key: "test-key".to_string(),
            region: "ap-beijing".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let mut engine = TencentTtsEngine::new(config, &proxy_config).expect("构建引擎失败");
        engine.endpoint_override = Some(mock_server.uri());

        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test", "tencent:101001", &output_path, 1.0).await;
        let err = result.unwrap_err();
        match err {
            TTSError::SynthesisFailed(ref msg) => assert!(msg.contains("Audio"), "错误消息: {}", msg),
            _ => panic!("应为 SynthesisFailed, got: {:?}", err),
        }
    }

    #[test]
    fn test_tencent_parse_voice_prefix() {
        let parsed = common::parse_engine_prefix("tencent:101001", &["tencent:"]);
        assert_eq!(parsed, "101001");
    }

    /// 验证 HMAC 派生链编译正确（不执行网络请求，仅验证 Rust 类型）
    #[test]
    fn test_tencent_tc3_signature() {
        // 模拟 TC3 HMAC 派生链以验证类型正确
        let secret_key = "TC3test-secret-key";
        let date = "2026-04-30";

        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC 初始化应成功");
        mac.update(date.as_bytes());
        let secret_date = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&secret_date)
            .expect("HMAC 初始化应成功");
        mac.update(b"tts");
        let _secret_service = mac.finalize().into_bytes();

        // 验证 secret_date 长度为 32 bytes (SHA-256)
        assert_eq!(secret_date.len(), 32);
    }
}
