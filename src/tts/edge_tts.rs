use std::path::Path;
use std::time::Duration;
use async_trait::async_trait;
use futures::SinkExt;
use futures::StreamExt;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider, WordBoundary};

const WS_RECEIVE_TIMEOUT: Duration = Duration::from_secs(120);

/// Edge TTS WebSocket 端点 URL（硬编码，不可配置）
///
/// # 安全说明
/// `TrustedClientToken` 是公开令牌，广泛见于各类开源工具（如 edge-tts Python 库）。
/// 此令牌用于微软 Edge 浏览器语音服务的客户端标识，不是秘密凭据。
/// 若微软轮换此令牌，服务将中断直到本常量更新。
const EDGE_TTS_WSS_URL: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

/// 将 rate 转换为 Edge TTS 百分比格式
/// 1.0 → "+0%", 1.5 → "+50%", 0.5 → "-50%"
fn convert_rate_to_percent(rate: f64) -> String {
    if !rate.is_finite() || rate < 0.0 {
        return "+0%".to_string();
    }
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
fn convert_pitch_to_hz(pitch: f64) -> String {
    if !pitch.is_finite() {
        return "+0Hz".to_string();
    }
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

/// 从 Edge TTS 音色名称中提取语言标签
///
/// "zh-CN-XiaoyiNeural" → "zh-CN"
/// "en-US-JennyNeural"  → "en-US"
/// "ja-JP-NanamiNeural" → "ja-JP"
fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some((idx, _)) = voice_name
        .char_indices()
        .filter(|(_, c)| *c == '-')
        .nth(1)
    {
        voice_name[..idx].to_string()
    } else {
        String::new()
    }
}

/// 构建 Edge TTS SSML 请求 XML
fn build_ssml(text: &str, voice_name: &str, rate: f64, pitch: f64) -> String {
    let rate_str = convert_rate_to_percent(rate);
    let pitch_str = convert_pitch_to_hz(pitch);
    let voice_lang = voice_name_to_lang(voice_name);
    let escaped_text: String = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .chars()
        .filter(|&c| {
            // 保留标准可见字符、回车、换行、制表符；过滤 XML 1.0 禁止的控制字符
            c == '\t' || c == '\n' || c == '\r' || (c as u32 >= 0x20 && !(0x7F..=0x9F).contains(&(c as u32)))
        })
        .collect();

    format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="{}"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody></voice></speak>"#,
        voice_lang, voice_name, rate_str, pitch_str, escaped_text
    )
}

/// Edge-TTS 引擎
///
/// 通过原生 WebSocket (tokio-tungstenite) 直接连接微软 Edge TTS 服务
#[derive(Debug)]
pub(super) struct EdgeTtsEngine {
    pub(super) proxy_enabled: bool,
    pub(super) proxy_http: String,
    pub(super) proxy_https: String,
}

impl EdgeTtsEngine {
    pub(super) fn new(proxy_enabled: bool, proxy_http: String, proxy_https: String) -> Self {
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
        // SAFETY: 硬编码字面量包含的字符都是有效的 HeaderValue（ASCII 可见字符），
        // parse() 不会失败。使用 unwrap() 代替 expect() 以避免 expect 消息误导。
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
            let proxy_url = if !self.proxy_https.is_empty() {
                &self.proxy_https
            } else if !self.proxy_http.is_empty() {
                &self.proxy_http
            } else {
                return Err(TTSError::ConnectionFailed("代理已启用但未配置代理地址".to_string()));
            };

            // Parse proxy URL for host:port (case-insensitive scheme, strip credentials, split host:port)
            let (proxy_scheme, proxy_rest) = proxy_url
                .split_once("://")
                .unwrap_or(("", proxy_url));
            let proxy_addr = proxy_rest;
            // SOCKS5 is not supported — HTTP CONNECT tunnel requires HTTP proxy
            let proxy_scheme_lower = proxy_scheme.to_lowercase();
            if proxy_scheme_lower == "socks5" {
                return Err(TTSError::ConnectionFailed(
                    "本引擎不支持 SOCKS5 代理，请使用 HTTP/HTTPS 代理".to_string()
                ));
            }
            let default_proxy_port = match proxy_scheme_lower.as_str() {
                "https" | "wss" => 443u16,
                _ => 80u16,
            };
            // Extract user:pass credentials before stripping (rsplitn to handle @ in password)
            let credentials = proxy_addr.rsplitn(2, '@').nth(1);

            // Strip any user:pass@ prefix
            let host_port = proxy_addr
                .rsplit('@')
                .next()
                .unwrap_or(proxy_addr);
            let (proxy_host, proxy_port) = if host_port.starts_with('[') {
                // IPv6 bracket notation: [::1]:7890 or [::1] (no explicit port)
                if let Some((addr_inner, port_str)) = host_port.rsplit_once("]:") {
                    let port: u16 = port_str
                        .parse()
                        .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
                    (&addr_inner[1..], port)
                } else {
                    // No explicit port — fall back to default_proxy_port (consistent with IPv4)
                    let addr = host_port.trim_start_matches('[').trim_end_matches(']');
                    (addr, default_proxy_port)
                }
            } else {
                match host_port.split_once(':') {
                    Some((host, port_str)) => {
                        let port: u16 = port_str
                            .parse()
                            .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
                        (host, port)
                    }
                    None => (host_port, default_proxy_port),
                }
            };

            // Parse target WSS URL manually
            let target_addr = EDGE_TTS_WSS_URL.trim_start_matches("wss://");
            let target_host_only = target_addr.split('/').next().unwrap_or(target_addr);
            let (target_host, target_port_str) = target_addr
                .split_once(':')
                .map(|(h, p)| (h, p.split('/').next().unwrap_or("443")))
                .unwrap_or((target_host_only, "443"));
            let target_port: u16 = target_port_str
                .parse()
                .map_err(|_| TTSError::ConnectionFailed("目标端口格式错误".to_string()))?;

            // Construct HTTP CONNECT request string with optional Proxy-Authorization
            use base64::Engine;
            let connect_req = if let Some(creds) = credentials {
                let encoded = base64::engine::general_purpose::STANDARD.encode(creds);
                format!(
                    "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nProxy-Authorization: Basic {}\r\n\r\n",
                    target_host, target_port, target_host, target_port, encoded
                )
            } else {
                format!(
                    "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
                    target_host, target_port, target_host, target_port
                )
            };

            // HTTP CONNECT tunnel
            let mut tcp = tokio::net::TcpStream::connect((proxy_host, proxy_port))
                .await
                .map_err(|e| TTSError::ConnectionFailed(format!("连接代理失败: {}", e)))?;

            // Write CONNECT request directly to TcpStream (no BufReader — avoids read-ahead
            // data loss on into_inner() that would corrupt TLS handshake)
            use tokio::io::AsyncWriteExt;
            tcp.write_all(connect_req.as_bytes()).await
                .map_err(|e| TTSError::ConnectionFailed(format!("发送 CONNECT 请求失败: {}", e)))?;
            tcp.flush().await
                .map_err(|e| TTSError::ConnectionFailed(format!("刷新 CONNECT 请求失败: {}", e)))?;

            // Read CONNECT response into a fixed buffer to preserve raw TcpStream
            // (BufReader::into_inner() discards the internal buffer, which can lose TLS
            // handshake bytes that arrived in the same TCP segment as the CONNECT response)
            //
            // Loop: read one complete response -> parse -> if 1xx interim continue reading next
            // (handles proxies that send interim 1xx and final 2xx in separate TCP segments,
            // per RFC 7231 Section 4.3.6)
            //
            // IMPORTANT: The buffer is allocated once (outside the inner loop) to prevent data
            // loss when 1xx and 2xx responses coalesce in the same TCP segment. When a 1xx
            // response is found, any leftover bytes after the \r\n\r\n separator are compacted
            // to the front of the buffer and re-parsed rather than discarded.
            use tokio::io::AsyncReadExt;

            let mut response_buf = vec![0u8; 4096];
            let mut total_read = 0usize;

            loop {
                // Read until \r\n\r\n (one complete response)
                loop {
                    let n = tcp.read(&mut response_buf[total_read..]).await
                        .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;
                    if n == 0 {
                        return Err(TTSError::ConnectionFailed("代理连接提前关闭".to_string()));
                    }
                    total_read += n;
                    if response_buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                    if total_read >= response_buf.len() {
                        return Err(TTSError::ConnectionFailed("代理响应头过大".to_string()));
                    }
                }

                // Parse this response
                let response_str = std::str::from_utf8(&response_buf[..total_read])
                    .map_err(|_| TTSError::ConnectionFailed("代理响应包含无效 UTF-8".to_string()))?;

                let status_line = response_str.lines().next()
                    .ok_or_else(|| TTSError::ConnectionFailed("代理响应为空".to_string()))?;
                let status_code: u16 = status_line
                    .trim()
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| TTSError::ConnectionFailed(format!("无法解析状态码: {}", status_line)))?;

                if (200..300).contains(&status_code) {
                    // 2xx success — CONNECT established
                    break;
                } else if (100..200).contains(&status_code) {
                    // 1xx interim — check for leftover 2xx data in same buffer
                    // (proxies may coalesce 1xx + 2xx in a single TCP segment)
                    let sep_end = response_buf[..total_read]
                        .windows(4)
                        .position(|w| w == b"\r\n\r\n")
                        .map(|p| p + 4)
                        .unwrap_or(0);
                    if sep_end < total_read {
                        // Leftover data exists — compact to buffer start and re-parse
                        let extra_len = total_read - sep_end;
                        response_buf.copy_within(sep_end..total_read, 0);
                        total_read = extra_len;
                        continue;
                    }
                    // No leftover — reset and read next response
                    total_read = 0;
                    continue;
                } else {
                    return Err(TTSError::ConnectionFailed(format!(
                        "代理 CONNECT 失败: {}",
                        status_line.trim()
                    )));
                }
            }

            // TLS + WebSocket upgrade over tunnel (raw TcpStream — no buffer data loss)
            let tls_connector = native_tls::TlsConnector::builder()
                .build()
                .map_err(|e| TTSError::ConnectionFailed(format!("TLS 构建失败: {}", e)))?;

            let (ws_stream, _) = tokio_tungstenite::client_async_tls_with_config(
                request,
                tcp,
                None,
                Some(tokio_tungstenite::Connector::NativeTls(tls_connector)),
            )
                .await
                .map_err(|e| {
                    let err_msg = format!("WebSocket 代理连接失败: {}", e);
                    Self::classify_error(err_msg, TTSError::ConnectionFailed)
                })?;
            Ok(ws_stream)
        } else {
            let (ws_stream, _) = connect_async(request)
                .await
                .map_err(|e| {
                    let err_msg = format!("WebSocket 连接失败: {}", e);
                    Self::classify_error(err_msg, TTSError::ConnectionFailed)
                })?;
            Ok(ws_stream)
        }
    }

    /// 根据错误消息启发式判断是否是认证错误
    fn classify_error(err_msg: String, fallback: fn(String) -> TTSError) -> TTSError {
        let lower = err_msg.to_lowercase();
        if err_msg.contains("401") || lower.contains("authentication") || lower.contains("unauthorized") {
            TTSError::AuthenticationFailed(err_msg)
        } else {
            fallback(err_msg)
        }
    }

    /// 执行带重试的 TTS 合成
    ///
    /// 对应 D-03: 3 次重试，间隔 1 秒
    async fn synthesize_with_retry(
        &self,
        ssml: &str,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        let max_attempts = 4; // 1 initial + 3 retries (D-03: 3 次重试)
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            tracing::info!("Edge-TTS 合成尝试 {}/{}", attempt, max_attempts);

            match self.synthesize_once(ssml, output_path).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    if matches!(&e, TTSError::AuthenticationFailed(_)) {
                        return Err(e);
                    }
                    tracing::warn!("Edge-TTS 合成尝试 {} 失败: {}", attempt, e);
                    last_error = Some(e);
                    if attempt < max_attempts {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        // 清理重试失败后可能残留的损坏输出文件
        let _ = tokio::fs::remove_file(output_path).await;

        Err(TTSError::RetryExhausted(format!(
            "Edge-TTS 重试 {} 次后仍失败: {}",
            max_attempts - 1,
            last_error.map_or("未知错误".to_string(), |e| e.to_string())
        )))
    }

    /// 单次 TTS 合成（无重试）
    async fn synthesize_once(
        &self,
        ssml: &str,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        let mut ws_stream = self.connect().await?;

        // 生成 SSML 请求中的 request ID
        let request_id = uuid::Uuid::new_v4().to_string();

        // 构建并发送 SSML WebSocket 消息
        // 格式: X-RequestId:{uuid}\r\nContent-Type:application/ssml+xml\r\n\r\n{ssml}
        // 对应 D-10: Edge-TTS 流式接收 audio + WordBoundary/SentenceBoundary chunk
        let stt_message = format!(
            "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\n\r\n{}",
            request_id, ssml
        );

        ws_stream
            .send(Message::Text(stt_message.into()))
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("发送 SSML 失败: {}", e)))?;

        // 接收并处理响应流
        let mut audio_data: Vec<u8> = Vec::new();
        let mut word_boundaries: Vec<WordBoundary> = Vec::new();
        let mut duration: f64 = 0.0;
        let mut received_turn_end = false;
        while let Some(msg_result) = timeout(WS_RECEIVE_TIMEOUT, ws_stream.next()).await
            .map_err(|_| TTSError::SynthesisFailed("接收消息超时: 服务器无响应".to_string()))?
        {
            let msg =
                msg_result.map_err(|e| {
                    let err_msg = format!("接收消息失败: {}", e);
                    Self::classify_error(err_msg, TTSError::SynthesisFailed)
                })?;

            match msg {
                Message::Binary(data) => {
                    // 解析二进制消息头（格式: Path: {path}\r\n...\r\n\r\n{payload}）
                    // 对应 D-10: WordBoundary/SentenceBoundary 事件通过路径判断
                    if let Some(content) = parse_edge_tts_binary(&data) {
                        if content.path == "audio" {
                            audio_data.extend_from_slice(&content.payload);
                        } else if content.path == "turn.start" {
                            // turn.start metadata — word boundaries arrive as separate wordboundary messages
                        } else if content.path == "wordboundary" {
                            // Parse individual WordBoundary event
                            match std::str::from_utf8(&content.payload) {
                                Ok(payload_str) => {
                                    if let Ok(wb_json) = serde_json::from_str::<serde_json::Value>(payload_str) {
                                        if let (Some(offset), Some(duration_100ns), Some(text)) = (
                                            wb_json["offset"].as_u64(),
                                            wb_json["duration"].as_u64(),
                                            wb_json["text"].as_str(),
                                        ) {
                                            word_boundaries.push(WordBoundary {
                                                start_offset: offset,
                                                end_offset: offset.saturating_add(duration_100ns),
                                                text: text.to_string(),
                                            });
                                        } else {
                                            tracing::warn!(
                                                "Failed to parse word boundary fields from JSON: {:?}",
                                                wb_json
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("wordboundary payload 包含无效 UTF-8: {} ({} bytes)", e, content.payload.len());
                                }
                            }
                        } else if content.path == "turn.end" {
                            received_turn_end = true;
                            // 解析结束事件中的音频时长
                            match std::str::from_utf8(&content.payload) {
                                Ok(payload_str) => {
                                    if let Ok(metadata) =
                                        serde_json::from_str::<serde_json::Value>(payload_str)
                                    {
                                        duration = if let Some(d) = metadata["audio_duration"].as_f64() {
                                            d / 10_000_000.0
                                        } else if let Some(s) = metadata["audio_duration"].as_str() {
                                            s.parse::<f64>().unwrap_or(0.0) / 10_000_000.0
                                        } else if let Some(ticks) = metadata["audio_duration"]["ticks"].as_u64() {
                                            ticks as f64 / 10_000_000.0
                                        } else {
                                            0.0
                                        };
                                    } else {
                                        tracing::warn!(
                                            "turn.end JSON 解析失败，duration 设为 0: {} bytes",
                                            content.payload.len()
                                        );
                                        duration = 0.0;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("turn.end payload 包含无效 UTF-8: {} ({} bytes)", e, content.payload.len());
                                    duration = 0.0;
                                }
                            }
                            break;
                        }
                    } else {
                        let preview = if data.len() > 256 {
                            format!(
                                "{} bytes (first 256 shown: {:?})",
                                data.len(),
                                &data[..256]
                            )
                        } else {
                            format!("{:?}", data)
                        };
                        tracing::warn!("无法解析 Edge-TTS 二进制消息: {}", preview);
                    }
                }
                Message::Text(text) => {
                    // 文本消息通常是调试或连接信息，忽略
                    tracing::debug!("Edge-TTS 文本消息: {}", text);
                }
                Message::Close(frame) => {
                    tracing::debug!("Edge-TTS 连接关闭: {:?}", frame);
                    break;
                }
                _ => {
                    tracing::debug!("Edge-TTS 忽略消息类型");
                }
            }
        }

        // 写入音频文件（对应 D-04: MP3 格式）
        if audio_data.is_empty() {
            return Err(TTSError::SynthesisFailed("未收到音频数据".to_string()));
        }

        if !received_turn_end {
            return Err(TTSError::SynthesisFailed(
                "WebSocket 连接在收到 turn.end 前关闭，音频数据不完整".to_string(),
            ));
        }
        if duration == 0.0 {
            tracing::warn!(
                "Edge-TTS turn.end received but audio_duration missing/unparseable, audio data size: {} bytes",
                audio_data.len()
            );
        }

        tokio::fs::write(output_path, &audio_data)
            .await
            .map_err(|e| {
                TTSError::SynthesisFailed(format!("写入音频文件失败: {}", e))
            })?;

        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries,
            duration,
        })
    }
}

/// 解析 Edge-TTS 二进制消息
///
/// 消息格式: Path: {path}\r\nX-RequestId: {uuid}\r\nContent-Type: {mime}\r\n\r\n{payload}
struct EdgeTtsContent {
    path: String,
    payload: Vec<u8>,
}

fn parse_edge_tts_binary(data: &[u8]) -> Option<EdgeTtsContent> {
    // 查找 header-payload 分隔符
    let separator = b"\r\n\r\n";
    let sep_pos = data.windows(4).position(|w| w == separator)?;

    let header = std::str::from_utf8(&data[..sep_pos]).ok()?;
    let payload = data[sep_pos + 4..].to_vec();

    // 从 header 中提取 Path 字段（RFC 7230 Section 3.2: 字段名大小写不敏感）
    let path = header
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("path:"))
        .and_then(|line| line.splitn(2, ':').nth(1))
        .map(|p| p.trim().to_string())?;

    Some(EdgeTtsContent { path, payload })
}

#[async_trait]
impl TtsProvider for EdgeTtsEngine {
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        rate: f64,
        pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        if text.is_empty() {
            return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
        }
        if voice_name.is_empty() {
            return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
        }
        if !rate.is_finite() || rate < 0.0 {
            return Err(TTSError::SynthesisFailed(
                format!("rate 必须为有限正数: {}", rate),
            ));
        }
        if !pitch.is_finite() {
            return Err(TTSError::SynthesisFailed(
                format!("pitch 必须为有限数值: {}", pitch),
            ));
        }
        let ssml = build_ssml(text, voice_name, rate, pitch);
        self.synthesize_with_retry(&ssml, output_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
    fn test_convert_rate_to_percent_zero() {
        // rate=0.0 should produce -100% (one full step slower)
        assert_eq!(convert_rate_to_percent(0.0), "-100%");
    }

    #[test]
    fn test_convert_rate_to_percent_nan() {
        // NaN should fall back to +0%
        assert_eq!(convert_rate_to_percent(f64::NAN), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_negative() {
        // Negative rate should fall back to +0%
        assert_eq!(convert_rate_to_percent(-0.5), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_inf() {
        // Infinity should fall back to +0%
        assert_eq!(convert_rate_to_percent(f64::INFINITY), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_neg_inf() {
        // Negative infinity should fall back to +0%
        assert_eq!(convert_rate_to_percent(f64::NEG_INFINITY), "+0%");
    }

    #[test]
    fn test_convert_pitch_to_hz_default() {
        assert_eq!(convert_pitch_to_hz(0.0), "+0Hz");
    }

    #[test]
    fn test_convert_pitch_to_hz_positive() {
        assert_eq!(convert_pitch_to_hz(50.0), "+50Hz");
    }

    #[test]
    fn test_convert_pitch_to_hz_negative() {
        assert_eq!(convert_pitch_to_hz(-10.0), "-10Hz");
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

    /// 集成测试：需要网络连接访问微软 Edge TTS 服务
    /// 运行方式: cargo test --lib edge_tts::tests::test_edge_tts_integration -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_edge_tts_integration() {
        let engine = EdgeTtsEngine::new(false, String::new(), String::new());
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("test_output.mp3");

        let result = engine
            .synthesize("这是一个测试。", "zh-CN-XiaoyiNeural", 1.0, 0.0, &output_path)
            .await;

        assert!(result.is_ok(), "Edge-TTS 集成测试失败: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists(), "音频文件应存在");
        assert!(
            output.audio_file_path.metadata().unwrap().len() > 1000,
            "音频文件大小应大于 1KB"
        );
        assert!(output.duration > 0.0, "时长应大于 0");
        // 中文语音应有词边界数据
        assert!(!output.word_boundaries.is_empty(), "应有词边界数据");
    }

    // ============================================================
    // parse_edge_tts_binary 单元测试
    // ============================================================

    #[test]
    fn test_parse_edge_tts_binary_audio() {
        // Audio message
        let data = b"Path: audio\r\nX-RequestId: abc\r\nContent-Type: audio/mpeg\r\n\r\n\xff\xf3\x00\x00";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "audio");
        assert_eq!(content.payload, b"\xff\xf3\x00\x00");
    }

    #[test]
    fn test_parse_edge_tts_binary_wordboundary() {
        // WordBoundary message
        let payload = r#"{"offset":0,"duration":5000000,"text":"hello"}"#;
        let data = format!(
            "Path: wordboundary\r\nX-RequestId: xyz\r\nContent-Type: application/json\r\n\r\n{}",
            payload
        );
        let result = parse_edge_tts_binary(data.as_bytes());
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "wordboundary");
        assert_eq!(content.payload, payload.as_bytes());
    }

    #[test]
    fn test_parse_edge_tts_binary_turn_end() {
        // Turn.End message
        let payload = r#"{"audio_duration":52500000}"#;
        let data = format!(
            "Path: turn.end\r\nX-RequestId: xyz\r\nContent-Type: application/json\r\n\r\n{}",
            payload
        );
        let result = parse_edge_tts_binary(data.as_bytes());
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "turn.end");
    }

    #[test]
    fn test_parse_edge_tts_binary_missing_separator() {
        // No \r\n\r\n separator
        let data = b"Path: audio\r\nX-RequestId: abc";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_edge_tts_binary_empty_payload() {
        // Empty payload after separator
        let data = b"Path: audio\r\nX-RequestId: abc\r\n\r\n";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "audio");
        assert!(content.payload.is_empty());
    }

    #[test]
    fn test_parse_edge_tts_binary_missing_path() {
        // No Path: header field
        let data = b"X-RequestId: abc\r\nContent-Type: audio/mpeg\r\n\r\npayload";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_none());
    }
}
