use crate::config::types::IndexTTS2Section;
use crate::error::TTSError;
use crate::tts::common;
use crate::tts::{TtsOutput, TtsProvider};
use async_trait::async_trait;
use reqwest::multipart;
use std::path::Path;
use std::time::Duration;

/// IndexTTS2 语音克隆引擎。
///
/// 本地自托管服务 (Docker/本地部署)。
/// API: POST {api_url} (默认 http://127.0.0.1:8081/tts)
/// 请求: multipart/form-data 含 prompt_audio(参考音频文件) + 文本参数
/// 响应: 原始音频字节
/// 超时: 120s (推理时间长)
///
/// 对齐 Python 版 indextts2_tts() — voice.py:2162-2258
pub(super) struct IndexTts2Engine {
    client: reqwest::Client,
    config: IndexTTS2Section,
}

impl IndexTts2Engine {
    pub(super) fn new(
        config: IndexTTS2Section,
        proxy_config: &common::ProxyConfig,
    ) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config })
    }

    async fn synthesize_once(
        &self,
        text: &str,
        voice_name: &str,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        // 解析 voice_name: 去除 "indextts2:" 前缀 = 参考音频路径
        // Python 版 parse_indextts2_voice — voice.py:2151
        let ref_audio_path_str = common::parse_engine_prefix(voice_name, &["indextts2:"]);

        // 如果 voice_name 不包含前缀，使用配置中的默认 reference_audio
        let ref_audio_path =
            if ref_audio_path_str == voice_name && !self.config.reference_audio.is_empty() {
                Path::new(&self.config.reference_audio)
            } else {
                Path::new(ref_audio_path_str)
            };

        // 验证参考音频文件存在且是文件（提供清晰的错误信息）
        match tokio::fs::metadata(ref_audio_path).await {
            Ok(meta) if meta.is_file() => {}
            Ok(_) => {
                return Err(TTSError::SynthesisFailed(format!(
                    "参考音频路径不是文件: {}",
                    ref_audio_path.display()
                )))
            }
            Err(e) => {
                return Err(TTSError::SynthesisFailed(format!(
                    "无法访问参考音频文件 '{}': {}",
                    ref_audio_path.display(),
                    e
                )))
            }
        }

        // 读取参考音频到内存（重试安全：每次重新读取，不持有文件句柄）
        // 对齐 Python 版: files={'prompt_audio': open(reference_audio_path, 'rb')}
        // Rust 版使用 Part::bytes() 避免路径编码问题 (Pitfall 4)
        let reference_audio_bytes = tokio::fs::read(ref_audio_path)
            .await
            .map_err(|e| TTSError::SynthesisFailed(format!("读取参考音频失败: {}", e)))?;

        let api_url = if self.config.api_url.is_empty() {
            "http://127.0.0.1:8081/tts"
        } else {
            &self.config.api_url
        };

        // 构建 multipart 表单
        // 对齐 Python 版 data 字典 (voice.py:2197-2206):
        //   {'text': text, 'infer_mode': infer_mode, 'temperature': ..., 'top_p': ...}
        let form = multipart::Form::new()
            .part(
                "prompt_audio",
                multipart::Part::bytes(reference_audio_bytes)
                    .file_name("reference.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| TTSError::SynthesisFailed(format!("MIME 设置失败: {}", e)))?,
            )
            .text("text", text.trim().to_string())
            .text("infer_mode", self.config.infer_mode.clone())
            .text("temperature", self.config.temperature.to_string())
            .text("top_p", self.config.top_p.to_string())
            .text("top_k", self.config.top_k.to_string())
            .text("do_sample", self.config.do_sample.to_string())
            .text("num_beams", self.config.num_beams.to_string())
            .text(
                "repetition_penalty",
                self.config.repetition_penalty.to_string(),
            );

        // 超时 120s — IndexTTS2 推理时间可能较长 (Python 版 timeout=120)
        let response = self
            .client
            .post(api_url)
            .multipart(form)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("IndexTTS2 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!(
                "IndexTTS2 API 返回 {}: {}",
                status.as_u16(),
                body
            )));
        }

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| TTSError::SynthesisFailed(format!("IndexTTS2 读取响应失败: {}", e)))?;

        common::write_audio_bytes(output_path, &audio_bytes).await?;

        // IndexTTS2 不支持词边界
        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries: vec![],
            duration: 0.0,
        })
    }
}

#[async_trait]
impl TtsProvider for IndexTts2Engine {
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
        // 检查 voice_name 或配置中是否有参考音频路径
        let has_prefix = voice_name.starts_with("indextts2:");
        if !has_prefix && self.config.reference_audio.is_empty() {
            return Err(TTSError::SynthesisFailed(
                "IndexTTS2 需要参考音频路径（在 voice_name 中或 indextts2.reference_audio 配置中提供）".to_string()
            ));
        }
        let result =
            common::retry_loop(|| self.synthesize_once(text, voice_name, output_path)).await;
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_indextts2_engine_new() {
        let config = IndexTTS2Section {
            api_url: "http://127.0.0.1:8081/tts".to_string(),
            reference_audio: "/path/to/ref.wav".to_string(),
            infer_mode: "普通推理".to_string(),
            temperature: 1.0,
            top_p: 0.8,
            top_k: 30,
            do_sample: true,
            num_beams: 3,
            repetition_penalty: 10.0,
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = IndexTts2Engine::new(config, &proxy_config).expect("构建引擎失败");
        assert_eq!(engine.config.api_url, "http://127.0.0.1:8081/tts");
        assert_eq!(engine.config.reference_audio, "/path/to/ref.wav");
    }

    #[tokio::test]
    async fn test_indextts2_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tts"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(b"fake audio data from indextts2"),
            )
            .mount(&mock_server)
            .await;

        // Create a temporary reference audio file
        let dir = TempDir::new().expect("创建临时目录失败");
        let ref_audio_path = dir.path().join("reference.wav");
        std::fs::write(&ref_audio_path, b"fake reference audio").expect("写入参考音频文件失败");

        let config = IndexTTS2Section {
            api_url: mock_server.uri() + "/tts",
            reference_audio: ref_audio_path.to_str().unwrap().to_string(),
            infer_mode: "普通推理".to_string(),
            temperature: 1.0,
            top_p: 0.8,
            top_k: 30,
            do_sample: true,
            num_beams: 3,
            repetition_penalty: 10.0,
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = IndexTts2Engine::new(config, &proxy_config).expect("构建引擎失败");
        let output_path = dir.path().join("output.mp3");

        let ref_path_str = format!("indextts2:{}", ref_audio_path.to_str().unwrap());
        let result = engine
            .synthesize_once("test text", &ref_path_str, &output_path)
            .await;
        assert!(result.is_ok(), "IndexTTS2 成功: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists());
        assert!(output.word_boundaries.is_empty());

        let written = std::fs::read(&output_path).expect("读取输出文件失败");
        assert_eq!(written, b"fake audio data from indextts2");
    }

    #[tokio::test]
    async fn test_indextts2_empty_text() {
        let config = IndexTTS2Section {
            api_url: String::new(),
            reference_audio: "/path/to/ref.wav".to_string(),
            infer_mode: String::new(),
            temperature: 1.0,
            top_p: 0.8,
            top_k: 30,
            do_sample: true,
            num_beams: 3,
            repetition_penalty: 10.0,
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = IndexTts2Engine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine
            .synthesize("", "indextts2:/path/to/ref.wav", 1.0, 0.0, &output_path)
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_indextts2_missing_ref_audio() {
        let config = IndexTTS2Section {
            api_url: String::new(),
            reference_audio: String::new(),
            infer_mode: String::new(),
            temperature: 1.0,
            top_p: 0.8,
            top_k: 30,
            do_sample: true,
            num_beams: 3,
            repetition_penalty: 10.0,
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = IndexTts2Engine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        // Voice name without prefix and config has no reference_audio
        let result = engine
            .synthesize("test text", "voice_without_prefix", 1.0, 0.0, &output_path)
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("需要参考音频路径")),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[test]
    fn test_indextts2_parse_prefix() {
        let parsed = common::parse_engine_prefix("indextts2:/path/to/ref.wav", &["indextts2:"]);
        assert_eq!(parsed, "/path/to/ref.wav");
    }

    #[test]
    fn test_default_config() {
        let config = IndexTTS2Section::default();
        assert_eq!(config.repetition_penalty, 1.5);
    }
}
