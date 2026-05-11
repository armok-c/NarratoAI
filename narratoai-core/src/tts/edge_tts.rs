use crate::error::TTSError;
use crate::tts::{common, TtsOutput, TtsProvider, WordBoundary};
use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use futures::SinkExt;
use futures::StreamExt;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

const WS_RECEIVE_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

// Edge TTS protocol constants (aligned with Python edge-tts 7.2.7 + SmartEdit)

const EDGE_TTS_WSS_BASE: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";
const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
const CHROMIUM_MAJOR_VERSION: &str = "143";
const SEC_MS_GEC_VERSION: &str = "1-143.0.3650.75";
const WIN_EPOCH_OFFSET: u64 = 11644473600;
const SEC_MS_GEC_ROUNDING: u64 = 300;

static CLOCK_SKEW_MICROS: AtomicI64 = AtomicI64::new(0);

fn get_corrected_timestamp() -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let skew_micros = CLOCK_SKEW_MICROS.load(Ordering::SeqCst);
    now + (skew_micros as f64 / 1_000_000.0)
}

fn generate_sec_ms_gec() -> String {
    let ticks = get_corrected_timestamp();
    let ticks = ticks + WIN_EPOCH_OFFSET as f64;
    let ticks = ticks - (ticks % SEC_MS_GEC_ROUNDING as f64);
    let ticks = ticks * 10_000_000.0;
    let str_to_hash = format!("{:.0}{}", ticks, TRUSTED_CLIENT_TOKEN);
    let mut hasher = Sha256::new();
    hasher.update(str_to_hash.as_bytes());
    hex::encode(hasher.finalize()).to_uppercase()
}

fn generate_muid() -> String {
    hex::encode(uuid::Uuid::new_v4().as_bytes()).to_uppercase()
}

fn connect_id() -> String {
    uuid::Uuid::new_v4().as_simple().to_string()
}

fn parse_rfc2616_date(date_str: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc2822(date_str)
        .ok()
        .map(|dt| dt.timestamp() as f64)
}

fn date_to_string() -> String {
    let now = chrono::Utc::now();
    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{} {} {:02} {} {:02}:{:02}:{:02} GMT+0000 (Coordinated Universal Time)",
        weekdays[now.weekday().num_days_from_sunday() as usize],
        months[now.month0() as usize],
        now.day(),
        now.year(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn build_user_agent() -> String {
    format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
        CHROMIUM_MAJOR_VERSION, CHROMIUM_MAJOR_VERSION
    )
}

fn build_speech_config_message(timestamp: &str) -> String {
    let payload = concat!(
        "{\"context\":{\"synthesis\":{\"audio\":{\"metadataoptions\":",
        "{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"},",
        "\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"",
        "}}}}"
    );
    format!(
        "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}\r\n",
        timestamp, payload
    )
}

fn build_ssml_message(request_id: &str, timestamp: &str, ssml: &str) -> String {
    format!(
        "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}Z\r\nPath:ssml\r\n\r\n{}",
        request_id, timestamp, ssml
    )
}

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

fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some((idx, _)) = voice_name.char_indices().filter(|(_, c)| *c == '-').nth(1) {
        voice_name[..idx].to_string()
    } else {
        tracing::warn!(
            "voice name '{}' does not follow lang-Region-Name format, falling back to en-US",
            voice_name
        );
        "en-US".to_string()
    }
}

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
            c == '\t'
                || c == '\n'
                || c == '\r'
                || (c as u32 >= 0x20 && !(0x7F..=0x9F).contains(&(c as u32)))
        })
        .collect();

    format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="{}"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody></voice></speak>"#,
        common::escape_xml_attr(&voice_lang),
        common::escape_xml_attr(voice_name),
        rate_str, pitch_str, escaped_text
    )
}

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

    fn build_wss_url(&self) -> String {
        let connection_id = connect_id();
        let sec_ms_gec = generate_sec_ms_gec();
        format!(
            "{}?TrustedClientToken={}&ConnectionId={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}",
            EDGE_TTS_WSS_BASE, TRUSTED_CLIENT_TOKEN, connection_id, sec_ms_gec, SEC_MS_GEC_VERSION
        )
    }

    fn apply_wss_headers(
        request: &mut tokio_tungstenite::tungstenite::http::Request<()>,
    ) -> Result<(), TTSError> {
        use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};

        let muid = generate_muid();

        let headers: &[(&str, &str)] = &[
            ("Pragma", "no-cache"),
            ("Cache-Control", "no-cache"),
            ("Origin", "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"),
            ("Accept-Language", "en-US,en;q=0.9"),
        ];

        for (key, value) in headers {
            request.headers_mut().insert(
                HeaderName::try_from(*key).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }

        request.headers_mut().insert(
            "User-Agent",
            HeaderValue::from_str(&build_user_agent()).unwrap(),
        );

        request.headers_mut().insert(
            "Cookie",
            HeaderValue::from_str(&format!("muid={};", muid)).unwrap(),
        );

        Ok(())
    }

    async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        TTSError,
    > {
        let wss_url = self.build_wss_url();

        // Extract proxy target info before wss_url is moved into request
        let target_parts: Option<(String, u16)> = if self.proxy_enabled {
            let target_addr = wss_url.trim_start_matches("wss://");
            let target_host_only = target_addr.split('/').next().unwrap_or(target_addr);
            let (host, port_str) = target_addr
                .split_once(':')
                .map(|(h, p)| (h, p.split('/').next().unwrap_or("443")))
                .unwrap_or((target_host_only, "443"));
            let port: u16 = port_str
                .parse()
                .map_err(|_| TTSError::ConnectionFailed("目标端口格式错误".to_string()))?;
            Some((host.to_string(), port))
        } else {
            None
        };

        tokio::time::timeout(CONNECT_TIMEOUT, async move {
            use tokio_tungstenite::tungstenite::client::IntoClientRequest;

            let mut request = wss_url
                .into_client_request()
                .map_err(|e| TTSError::ConnectionFailed(format!("构建请求失败: {}", e)))?;

            Self::apply_wss_headers(&mut request)?;

            if self.proxy_enabled {
                let (target_host, target_port) = target_parts.as_ref().unwrap();

                let proxy_url = if !self.proxy_https.is_empty() {
                    &self.proxy_https
                } else if !self.proxy_http.is_empty() {
                    &self.proxy_http
                } else {
                    return Err(TTSError::ConnectionFailed("代理已启用但未配置代理地址".to_string()));
                };

                let (proxy_scheme, proxy_rest) = proxy_url
                    .split_once("://")
                    .unwrap_or(("", proxy_url));
                let proxy_addr = proxy_rest;
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
                let credentials = proxy_addr.rsplitn(2, '@').nth(1);

                let host_port = proxy_addr
                    .rsplit('@')
                    .next()
                    .unwrap_or(proxy_addr);
                let (proxy_host, proxy_port) = if host_port.starts_with('[') {
                    if let Some((addr_inner, port_str)) = host_port.rsplit_once("]:") {
                        let port: u16 = port_str
                            .parse()
                            .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
                        (&addr_inner[1..], port)
                    } else {
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

                let mut tcp = tokio::net::TcpStream::connect((proxy_host, proxy_port))
                    .await
                    .map_err(|e| TTSError::ConnectionFailed(format!("连接代理失败: {}", e)))?;

                use tokio::io::AsyncWriteExt;
                tcp.write_all(connect_req.as_bytes()).await
                    .map_err(|e| TTSError::ConnectionFailed(format!("发送 CONNECT 请求失败: {}", e)))?;
                tcp.flush().await
                    .map_err(|e| TTSError::ConnectionFailed(format!("刷新 CONNECT 请求失败: {}", e)))?;

                use tokio::io::AsyncReadExt;

                let mut response_buf = vec![0u8; 4096];
                let mut total_read = 0usize;

                loop {
                    loop {
                        if total_read >= 4
                            && response_buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n")
                        {
                            break;
                        }
                        let n = tcp.read(&mut response_buf[total_read..]).await
                            .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;
                        if n == 0 {
                            return Err(TTSError::ConnectionFailed("代理连接提前关闭".to_string()));
                        }
                        total_read += n;
                        if total_read >= response_buf.len() {
                            return Err(TTSError::ConnectionFailed("代理响应头过大".to_string()));
                        }
                    }

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
                        break;
                    } else if (100..200).contains(&status_code) {
                        let sep_end = response_buf[..total_read]
                            .windows(4)
                            .position(|w| w == b"\r\n\r\n")
                            .map(|p| p + 4)
                            .unwrap_or(0);
                        if sep_end < total_read {
                            let extra_len = total_read - sep_end;
                            response_buf.copy_within(sep_end..total_read, 0);
                            total_read = extra_len;
                            continue;
                        }
                        total_read = 0;
                        continue;
                    } else {
                        return Err(TTSError::ConnectionFailed(format!(
                            "代理 CONNECT 失败: {}",
                            status_line.trim()
                        )));
                    }
                }

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
                use tokio_tungstenite::connect_async_tls_with_config;
                use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

                let config = WebSocketConfig::default()
                    .max_message_size(Some(10 * 1024 * 1024))
                    .max_frame_size(Some(10 * 1024 * 1024));

                let (ws_stream, _) = connect_async_tls_with_config(request, Some(config), false, None)
                    .await
                    .map_err(|e| {
                        let err_msg = format!("WebSocket 连接失败: {}", e);
                        Self::classify_error(err_msg, TTSError::ConnectionFailed)
                    })?;
                Ok(ws_stream)
            }
        })
        .await
        .map_err(|_| TTSError::ConnectionFailed("Edge-TTS CONNECT 超时 (30s)".to_string()))?
    }

    fn classify_error(err_msg: String, fallback: fn(String) -> TTSError) -> TTSError {
        let lower = err_msg.to_lowercase();
        if err_msg.contains("401")
            || lower.contains("authentication")
            || lower.contains("unauthorized")
        {
            TTSError::AuthenticationFailed(err_msg)
        } else {
            fallback(err_msg)
        }
    }

    async fn synthesize_with_retry(
        &self,
        ssml: &str,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        // 首次尝试前校正时钟偏差（对应 SmartEdit correct_clock_skew）
        self.correct_clock_skew().await;

        let result = common::retry_loop(|| self.synthesize_once(ssml, output_path)).await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(output_path).await;
        }
        result
    }

    /// 通过 HTTP 请求获取服务器时间，校正时钟偏差
    async fn correct_clock_skew(&self) {
        let sec_ms_gec = generate_sec_ms_gec();
        let url = format!(
            "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}",
            TRUSTED_CLIENT_TOKEN, sec_ms_gec, SEC_MS_GEC_VERSION
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let muid = generate_muid();
        match client
            .get(&url)
            .header("User-Agent", build_user_agent())
            .header("Cookie", format!("muid={};", muid))
            .header("Accept-Encoding", "gzip, deflate, br, zstd")
            .send()
            .await
        {
            Ok(resp) => {
                if let Some(date) = resp.headers().get("Date").and_then(|d| d.to_str().ok()) {
                    if let Some(server_ts) = parse_rfc2616_date(date) {
                        let skew = server_ts - get_corrected_timestamp();
                        if skew.abs() > 1.0 {
                            CLOCK_SKEW_MICROS.fetch_add(
                                (skew * 1_000_000.0) as i64,
                                Ordering::SeqCst,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Edge-TTS clock skew correction failed: {}", e);
            }
        }
    }

    async fn synthesize_once(&self, ssml: &str, output_path: &Path) -> Result<TtsOutput, TTSError> {
        let ws_stream = self.connect().await?;
        let (mut tx, mut rx) = ws_stream.split();

        let request_id = connect_id();
        let timestamp = date_to_string();

        // 1. Send speech.config (aligns with Python edge-tts protocol)
        let config_msg = build_speech_config_message(&timestamp);
        tx.send(Message::Text(config_msg.into()))
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("send speech.config failed: {}", e)))?;

        // 2. Send SSML with X-Timestamp
        let ssml_timestamp = date_to_string();
        let ssml_msg = build_ssml_message(&request_id, &ssml_timestamp, ssml);
        tx.send(Message::Text(ssml_msg.into()))
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("send SSML failed: {}", e)))?;

        // 3. Receive response stream
        let mut audio_data: Vec<u8> = Vec::new();
        let mut word_boundaries: Vec<WordBoundary> = Vec::new();
        let mut duration: f64 = 0.0;
        let mut received_turn_end = false;
        while let Some(msg_result) = timeout(WS_RECEIVE_TIMEOUT, rx.next())
            .await
            .map_err(|_| TTSError::SynthesisFailed("receive timeout: server did not respond".to_string()))?
        {
            let msg = msg_result.map_err(|e| {
                let err_msg = format!("receive failed: {}", e);
                Self::classify_error(err_msg, TTSError::SynthesisFailed)
            })?;

            match msg {
                Message::Binary(data) => {
                    if let Some(content) = parse_edge_tts_binary(&data) {
                        match content.path.as_str() {
                            "audio" => {
                                audio_data.extend_from_slice(&content.payload);
                            }
                            "turn.end" => {
                                received_turn_end = true;
                                match std::str::from_utf8(&content.payload) {
                                    Ok(payload_str) => {
                                        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(payload_str) {
                                            duration = if let Some(d) = metadata["audio_duration"].as_f64() {
                                                d / 10_000_000.0
                                            } else if let Some(s) = metadata["audio_duration"].as_str() {
                                                s.parse::<f64>().unwrap_or(0.0) / 10_000_000.0
                                            } else {
                                                0.0
                                            };
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("turn.end payload invalid UTF-8: {}", e);
                                    }
                                }
                                break;
                            }
                            _ => {
                                tracing::debug!("Edge-TTS binary path={}", content.path);
                            }
                        }
                    } else {
                        // Unparseable binary data — treat as raw audio
                        audio_data.extend_from_slice(&data);
                    }
                }
                Message::Text(text) => {
                    let path = parse_text_frame_path(&text).unwrap_or("");
                    match path {
                        "turn.end" => {
                            received_turn_end = true;
                            if let Some(json_start) = text.find("\r\n\r\n") {
                                let json_str = &text[json_start + 4..];
                                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    duration = metadata["audioDuration"]
                                        .as_f64()
                                        .or_else(|| metadata["audio_duration"].as_f64())
                                        .map(|d| d / 10_000_000.0)
                                        .or_else(|| {
                                            metadata["audioDuration"].as_u64()
                                                .or_else(|| metadata["audio_duration"].as_u64())
                                                .map(|d| d as f64 / 10_000_000.0)
                                        })
                                        .unwrap_or(0.0);
                                }
                            }
                            break;
                        }
                        "audio.metadata" => {
                            if let Some(json_start) = text.find("\r\n\r\n") {
                                let json_str = &text[json_start + 4..];
                                if let Ok(frame) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    if let Some(metadata) = frame["Metadata"].as_array() {
                                        for item in metadata {
                                            if item["Type"].as_str() == Some("WordBoundary") {
                                                if let Some(data) = item["Data"].as_object() {
                                                    let offset = data["Offset"].as_u64().unwrap_or(0);
                                                    let dur = data["Duration"].as_u64().unwrap_or(0);
                                                    let wb_text = data["text"]["Text"].as_str().unwrap_or("").to_string();
                                                    if !wb_text.is_empty() {
                                                        word_boundaries.push(WordBoundary {
                                                            start_offset: offset,
                                                            end_offset: offset.saturating_add(dur),
                                                            text: wb_text,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            tracing::debug!("Edge-TTS text path={}", path);
                        }
                    }
                }
                Message::Close(frame) => {
                    tracing::debug!("Edge-TTS connection closed: {:?}", frame);
                    break;
                }
                _ => {
                    tracing::debug!("Edge-TTS ignoring message type");
                }
            }
        }

        if audio_data.is_empty() {
            return Err(TTSError::SynthesisFailed("no audio data received".to_string()));
        }

        if !received_turn_end {
            return Err(TTSError::SynthesisFailed(
                "WebSocket closed before turn.end, audio data incomplete".to_string(),
            ));
        }
        // Derive duration from WordBoundary when turn.end has no audio_duration
        if duration == 0.0 {
            if let Some(last) = word_boundaries.last() {
                duration = last.end_offset as f64 / 10_000_000.0;
            } else if !audio_data.is_empty() {
                // Rough estimate: MP3 48kbps mono
                duration = (audio_data.len() as f64 * 8.0) / 48000.0;
            }
            if duration == 0.0 {
                tracing::warn!(
                    "Edge-TTS turn.end with missing audio_duration, audio size: {} bytes",
                    audio_data.len()
                );
            }
        }

        tokio::fs::write(output_path, &audio_data)
            .await
            .map_err(|e| TTSError::SynthesisFailed(format!("write audio file failed: {}", e)))?;

        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries,
            duration,
        })
    }
}

// Binary message parsing

struct EdgeTtsContent {
    path: String,
    payload: Vec<u8>,
}

/// Parse binary frame with 2-byte big-endian header length prefix (new protocol)
/// Format: [u16 header_length BE][header bytes][payload bytes]
fn parse_binary_headers_and_data(data: &[u8]) -> Option<(Vec<(String, String)>, Vec<u8>)> {
    if data.len() < 2 {
        return None;
    }
    let header_length = u16::from_be_bytes([data[0], data[1]]) as usize;
    if header_length + 2 > data.len() {
        return None;
    }
    let header = std::str::from_utf8(&data[2..2 + header_length]).ok()?;
    let payload = data[2 + header_length..].to_vec();

    let headers: Vec<(String, String)> = header
        .lines()
        .filter_map(|line| {
            let (k, v) = line.split_once(':')?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect();

    Some((headers, payload))
}

/// Parse binary frame with \\r\\n\\r\\n separator (old protocol, fallback)
fn parse_text_style_binary(data: &[u8]) -> Option<(String, Vec<u8>)> {
    let separator = b"\r\n\r\n";
    let sep_pos = data.windows(4).position(|w| w == separator)?;

    let header = std::str::from_utf8(&data[..sep_pos]).ok()?;
    let payload = data[sep_pos + 4..].to_vec();

    let path = header
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("path:"))
        .and_then(|line| line.splitn(2, ':').nth(1))
        .map(|p| p.trim().to_string())?;

    Some((path, payload))
}

fn parse_edge_tts_binary(data: &[u8]) -> Option<EdgeTtsContent> {
    // Try new protocol first (2-byte length prefix)
    if let Some((headers, payload)) = parse_binary_headers_and_data(data) {
        let path = headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == "path")
            .map(|(_, v)| v.to_lowercase())?;
        return Some(EdgeTtsContent { path, payload });
    }

    // Fallback to old protocol (text-style headers)
    if let Some((path, payload)) = parse_text_style_binary(data) {
        return Some(EdgeTtsContent { path, payload });
    }

    None
}

/// Parse text frame path
fn parse_text_frame_path(text: &str) -> Option<&str> {
    let header_end = text.find("\r\n\r\n")?;
    let header_bytes = text.as_bytes();
    let header = std::str::from_utf8(&header_bytes[..header_end]).ok()?;
    header
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("path:"))
        .and_then(|line| line.splitn(2, ':').nth(1))
        .map(|p| p.trim())
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
        if text.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("text cannot be empty".to_string()));
        }
        if voice_name.trim().is_empty() {
            return Err(TTSError::SynthesisFailed("voice_name cannot be empty".to_string()));
        }
        if !rate.is_finite() || rate < 0.0 {
            return Err(TTSError::SynthesisFailed(format!(
                "rate must be finite positive: {}",
                rate
            )));
        }
        if !pitch.is_finite() {
            return Err(TTSError::SynthesisFailed(format!(
                "pitch must be finite: {}",
                pitch
            )));
        }
        let ssml = build_ssml(text, voice_name, rate, pitch);
        self.synthesize_with_retry(&ssml, output_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // rate/pitch/SSML tests (unchanged)

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
        assert_eq!(convert_rate_to_percent(0.0), "-100%");
    }

    #[test]
    fn test_convert_rate_to_percent_nan() {
        assert_eq!(convert_rate_to_percent(f64::NAN), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_negative() {
        assert_eq!(convert_rate_to_percent(-0.5), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_inf() {
        assert_eq!(convert_rate_to_percent(f64::INFINITY), "+0%");
    }

    #[test]
    fn test_convert_rate_to_percent_neg_inf() {
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
        let ssml = build_ssml("test text", "zh-CN-XiaoyiNeural", 1.0, 0.0);
        assert!(ssml.contains(r#"voice name="zh-CN-XiaoyiNeural""#));
    }

    #[test]
    fn test_build_ssml_contains_prosody() {
        let ssml = build_ssml("test text", "zh-CN-XiaoyiNeural", 1.5, 50.0);
        assert!(ssml.contains(r#"prosody rate="+50%" pitch="+50Hz""#));
    }

    #[test]
    fn test_build_ssml_escapes_xml_chars() {
        let ssml = build_ssml("a & b < c > d", "voice", 1.0, 0.0);
        assert!(ssml.contains("a &amp; b &lt; c &gt; d"));
        assert!(!ssml.contains("& "));
    }

    #[test]
    fn test_build_ssml_contains_text() {
        let ssml = build_ssml("hello world", "voice", 1.0, 0.0);
        assert!(ssml.contains("hello world"));
    }

    #[test]
    fn test_edge_tts_engine_new() {
        let engine = EdgeTtsEngine::new(
            true,
            "http://127.0.0.1:7890".to_string(),
            "http://127.0.0.1:7890".to_string(),
        );
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

    // New: Sec-MS-GEC, protocol constants, header tests

    #[test]
    fn test_generate_sec_ms_gec() {
        let t = generate_sec_ms_gec();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_muid() {
        let m = generate_muid();
        assert_eq!(m.len(), 32);
        assert!(m.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_connect_id() {
        let id = connect_id();
        assert_eq!(id.len(), 32);
        assert!(!id.contains('-'));
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_date_to_string_format() {
        let date_str = date_to_string();
        assert!(date_str.contains(" GMT+0000 (Coordinated Universal Time)"));
        let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        assert!(weekdays.iter().any(|w| date_str.starts_with(w)));
    }

    #[test]
    fn test_build_wss_url_contains_required_params() {
        let engine = EdgeTtsEngine::new(false, String::new(), String::new());
        let url = engine.build_wss_url();
        assert!(url.starts_with("wss://speech.platform.bing.com"));
        assert!(url.contains("TrustedClientToken="));
        assert!(url.contains("ConnectionId="));
        assert!(url.contains("Sec-MS-GEC="));
        assert!(url.contains("Sec-MS-GEC-Version="));
    }

    #[test]
    fn test_build_speech_config_message() {
        let msg = build_speech_config_message(
            "Wed Mar 12 2025 10:11:12 GMT+0000 (Coordinated Universal Time)",
        );
        assert!(msg.starts_with("X-Timestamp:"));
        assert!(msg.contains("Path:speech.config"));
        assert!(msg.contains("\"wordBoundaryEnabled\":\"true\""));
        assert!(msg.contains("\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\""));
    }

    #[test]
    fn test_build_ssml_message_format() {
        let msg = build_ssml_message(
            "req123",
            "Wed Mar 12 2025 10:11:12 GMT+0000 (Coordinated Universal Time)",
            "<speak>test</speak>",
        );
        assert!(msg.contains("X-RequestId:req123"));
        assert!(msg.contains("X-Timestamp:"));
        assert!(msg.contains("Z\r\n"));
        assert!(msg.contains("Path:ssml"));
    }

    #[test]
    fn test_chromium_constants_aligned() {
        assert_eq!(CHROMIUM_MAJOR_VERSION, "143");
        assert_eq!(SEC_MS_GEC_VERSION, "1-143.0.3650.75");
    }

    #[test]
    fn test_user_agent_contains_chromium_version() {
        let ua = build_user_agent();
        assert!(ua.contains(&format!("Chrome/{}.0.0.0", CHROMIUM_MAJOR_VERSION)));
        assert!(ua.contains(&format!("Edg/{}.0.0.0", CHROMIUM_MAJOR_VERSION)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_edge_tts_integration() {
        let engine = EdgeTtsEngine::new(false, String::new(), String::new());
        let dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = dir.path().join("test_output.mp3");

        let result = engine
            .synthesize(
                "This is a test.",
                "zh-CN-XiaoyiNeural",
                1.0,
                0.0,
                &output_path,
            )
            .await;

        assert!(result.is_ok(), "Edge-TTS integration test failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists(), "audio file should exist");
        assert!(output.audio_file_path.metadata().unwrap().len() > 1000);
        assert!(output.duration > 0.0);
        assert!(!output.word_boundaries.is_empty());
    }

    // parse_edge_tts_binary tests (unchanged)

    #[test]
    fn test_parse_edge_tts_binary_audio() {
        let data =
            b"Path: audio\r\nX-RequestId: abc\r\nContent-Type: audio/mpeg\r\n\r\n\xff\xf3\x00\x00";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "audio");
        assert_eq!(content.payload, b"\xff\xf3\x00\x00");
    }

    #[test]
    fn test_parse_edge_tts_binary_wordboundary() {
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
        let data = b"Path: audio\r\nX-RequestId: abc";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_edge_tts_binary_empty_payload() {
        let data = b"Path: audio\r\nX-RequestId: abc\r\n\r\n";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content.path, "audio");
        assert!(content.payload.is_empty());
    }

    #[test]
    fn test_parse_edge_tts_binary_missing_path() {
        let data = b"X-RequestId: abc\r\nContent-Type: audio/mpeg\r\n\r\npayload";
        let result = parse_edge_tts_binary(data);
        assert!(result.is_none());
    }
}
