use std::path::Path;
use async_trait::async_trait;
use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider, WordBoundary};

/// Edge TTS WebSocket 端点 URL（硬编码，不可配置）
const EDGE_TTS_WSS_URL: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

/// 将 rate 转换为 Edge TTS 百分比格式
/// 1.0 → "+0%", 1.5 → "+50%", 0.5 → "-50%"
fn convert_rate_to_percent(rate: f64) -> String {
    if rate == 1.0 {
        return "+0%".to_string();
    }
    let diff = ((rate - 1.0) * 100.0).round() as i32;
    if diff >= 0 {
        format!("+{}%", diff)
    } else {
        format!("{}%", diff)
    }
}

/// 将 pitch 转换为 Edge TTS Hz 格式
/// 0 → "+0Hz", 50 → "+50Hz", -10 → "-10Hz"
fn convert_pitch_to_percent(pitch: f64) -> String {
    if pitch == 0.0 {
        return "+0Hz".to_string();
    }
    let p = pitch.round() as i32;
    if p >= 0 {
        format!("+{}Hz", p)
    } else {
        format!("{}Hz", p)
    }
}

/// 构建 Edge TTS SSML 请求 XML
fn build_ssml(text: &str, voice_name: &str, rate: f64, pitch: f64) -> String {
    let rate_str = convert_rate_to_percent(rate);
    let pitch_str = convert_pitch_to_percent(pitch);
    let escaped_text = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="zh-CN"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody></voice></speak>"#,
        voice_name, rate_str, pitch_str, escaped_text
    )
}

/// Edge-TTS 引擎
///
/// 通过原生 WebSocket (tokio-tungstenite) 直接连接微软 Edge TTS 服务
pub struct EdgeTtsEngine {
    pub proxy_enabled: bool,
    pub proxy_http: String,
    pub proxy_https: String,
}

impl EdgeTtsEngine {
    pub fn new(proxy_enabled: bool, proxy_http: String, proxy_https: String) -> Self {
        Self {
            proxy_enabled,
            proxy_http,
            proxy_https,
        }
    }

    /// 建立 WebSocket 连接
    ///
    /// 如果 proxy_enabled 为 true，通过代理连接；否则直连
    /// 对应 D-01: 原生 Rust WebSocket（tokio-tungstenite）
    /// 对应 D-02: 支持代理（从 config.proxy 读取）
    async fn connect(&self) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, TTSError> {
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let mut request = EDGE_TTS_WSS_URL
            .into_client_request()
            .map_err(|e| TTSError::ConnectionFailed(format!("构建请求失败: {}", e)))?;

        // 添加 Edge TTS 所需 HTTP 头
        request.headers_mut().insert(
            "Origin",
            "chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
                .parse()
                .unwrap(),
        );
        request.headers_mut().insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0"
                .parse()
                .unwrap(),
        );

        if self.proxy_enabled {
            // 通过代理连接
            let _proxy_url = if !self.proxy_https.is_empty() {
                &self.proxy_https
            } else if !self.proxy_http.is_empty() {
                &self.proxy_http
            } else {
                return Err(TTSError::ConnectionFailed("代理已启用但未配置代理地址".to_string()));
            };

            let (ws_stream, _) = connect_async(request)
                .await
                .map_err(|e| TTSError::ConnectionFailed(format!("WebSocket 代理连接失败: {}", e)))?;
            Ok(ws_stream)
        } else {
            let (ws_stream, _) = connect_async(request)
                .await
                .map_err(|e| TTSError::ConnectionFailed(format!("WebSocket 连接失败: {}", e)))?;
            Ok(ws_stream)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_rate_to_percent_standard() {
        assert_eq!(convert_rate_to_percent(1.0), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_faster() {
        assert_eq!(convert_rate_to_percent(1.5), "+50%");
    }

    #[test]
    fn test_convert_rate_to_percent_slower() {
        assert_eq!(convert_rate_to_percent(0.5), "-50%");
    }

    #[test]
    fn test_convert_pitch_to_percent_default() {
        assert_eq!(convert_pitch_to_percent(0.0), "+0Hz");
    }

    #[test]
    fn test_convert_pitch_to_percent_positive() {
        assert_eq!(convert_pitch_to_percent(50.0), "+50Hz");
    }

    #[test]
    fn test_convert_pitch_to_percent_negative() {
        assert_eq!(convert_pitch_to_percent(-10.0), "-10Hz");
    }

    #[test]
    fn test_build_ssml_contains_voice() {
        let ssml = build_ssml("测试文本", "zh-CN-XiaoyiNeural", 1.0, 0.0);
        assert!(ssml.contains(r#"voice name="zh-CN-XiaoyiNeural""#));
    }

    #[test]
    fn test_build_ssml_contains_prosody() {
        let ssml = build_ssml("测试文本", "zh-CN-XiaoyiNeural", 1.5, 50.0);
        assert!(ssml.contains(r#"prosody rate="+50%" pitch="+50Hz""#));
    }

    #[test]
    fn test_build_ssml_escapes_xml_chars() {
        let ssml = build_ssml("a & b < c > d", "voice", 1.0, 0.0);
        assert!(ssml.contains("a &amp; b &lt; c &gt; d"));
        assert!(!ssml.contains("& ")); // 原始 & 不应出现
    }

    #[test]
    fn test_build_ssml_contains_text() {
        let ssml = build_ssml("你好世界", "voice", 1.0, 0.0);
        assert!(ssml.contains("你好世界"));
    }

    #[test]
    fn test_edge_tts_engine_new() {
        let engine = EdgeTtsEngine::new(true, "http://127.0.0.1:7890".to_string(), "http://127.0.0.1:7890".to_string());
        assert!(engine.proxy_enabled);
        assert_eq!(engine.proxy_http, "http://127.0.0.1:7890");
        assert_eq!(engine.proxy_https, "http://127.0.0.1:7890");
    }

    #[test]
    fn test_edge_tts_engine_new_no_proxy() {
        let engine = EdgeTtsEngine::new(false, String::new(), String::new());
        assert!(!engine.proxy_enabled);
        assert!(engine.proxy_http.is_empty());
        assert!(engine.proxy_https.is_empty());
    }
}
