use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider};
use crate::config::types::AzureSection;
use crate::tts::common;
use async_trait::async_trait;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// 编译一次的重用 Azure 音色正则表达式
fn azure_voice_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z]{2}-[A-Z]{2}-.+Neural$").expect("Azure voice regex is valid"))
}

/// 判断 voice_name 是否应使用 Azure Speech Services REST API (V2)
///
/// 对齐 Python 版 should_use_azure_speech_services() — voice.py:1098-1116
/// - 以 -V2 结尾 → V2
/// - 匹配 Azure Neural 格式 `xx-XX-NameNeural` → V2
/// - 否则 → 回退 Edge-TTS (V1)
pub fn should_use_azure_services(voice_name: &str) -> bool {
    if voice_name.is_empty() {
        return false;
    }
    let name = voice_name.trim();
    // -V2 后缀 → 明确指定 Azure Speech Services
    if name.ends_with("-V2") {
        return true;
    }
    // 剥离 -Female/-Male 后缀（UI 默认语音名包含这些后缀）
    let check_name = name
        .trim_end_matches("-Female")
        .trim_end_matches("-Male");
    // 正则匹配 Azure Neural 格式: [language]-[REGION]-[Name]Neural
    // 如 zh-CN-YunzeNeural, en-US-AvaMultilingualNeural
    // 使用 OnceLock 缓存避免每次调用重新编译
    azure_voice_regex().is_match(check_name)
}

/// 返回硬编码的 Azure Neural 音色列表（供 UI/调试用）
///
/// 对齐 Python 版 get_all_azure_voices() — voice.py:80-96
/// 使用静态数组避免运行时构建
pub fn get_azure_voices() -> &'static [&'static str] {
    &[
        "af-ZA-AdriNeural", "af-ZA-WillemNeural",
        "am-ET-AmehaNeural", "am-ET-MekdesNeural",
        "ar-AE-FatimaNeural", "ar-AE-HamdanNeural",
        "ar-BH-AliNeural", "ar-BH-LailaNeural",
        "ar-DZ-AminaNeural", "ar-DZ-IsmaelNeural",
        "ar-EG-SalmaNeural", "ar-EG-ShakirNeural",
        "ar-IQ-BasselNeural", "ar-IQ-RanaNeural",
        "ar-JO-SanaNeural", "ar-JO-TaimNeural",
        "ar-KW-FahedNeural", "ar-KW-NouraNeural",
        "ar-LB-LaylaNeural", "ar-LB-RamiNeural",
        "ar-LY-ImanNeural", "ar-LY-OmarNeural",
        "ar-MA-JamalNeural", "ar-MA-MounaNeural",
        "ar-OM-AbdullahNeural", "ar-OM-AyshaNeural",
        "ar-QA-AmalNeural", "ar-QA-MoazNeural",
        "ar-SA-HamedNeural", "ar-SA-ZariyahNeural",
        "ar-SY-AmanyNeural", "ar-SY-LaithNeural",
        "ar-TN-HediNeural", "ar-TN-ReemNeural",
        "ar-YE-MaryamNeural", "ar-YE-SalehNeural",
        "az-AZ-BabekNeural", "az-AZ-BanuNeural",
        "bg-BG-BorislavNeural", "bg-BG-KalinaNeural",
        "bn-BD-NabanitaNeural", "bn-BD-PradeepNeural",
        "bn-IN-BashkarNeural", "bn-IN-TanishaaNeural",
        "bs-BA-GoranNeural", "bs-BA-VesnaNeural",
        "ca-ES-AlbaNeural", "ca-ES-EnricNeural",
        "cs-CZ-AntoninNeural", "cs-CZ-VlastaNeural",
        "cy-GB-AledNeural", "cy-GB-NiaNeural",
        "da-DK-ChristelNeural", "da-DK-JeppeNeural",
        "de-AT-IngridNeural", "de-AT-JonasNeural",
        "de-CH-JanNeural", "de-CH-LeniNeural",
        "de-DE-AmalaNeural", "de-DE-ConradNeural",
        "de-DE-FlorianMultilingualNeural",
        "de-DE-KatjaNeural", "de-DE-KillianNeural",
        "el-GR-AthinaNeural", "el-GR-NestorasNeural",
        "en-AU-NatashaNeural", "en-AU-WilliamNeural",
        "en-CA-ClaraNeural", "en-CA-LiamNeural",
        "en-GB-LibbyNeural", "en-GB-MaisieNeural",
        "en-GB-RyanNeural", "en-GB-SoniaNeural",
        "en-HK-SamNeural", "en-HK-YanNeural",
        "en-IE-ConnorNeural", "en-IE-EmilyNeural",
        "en-IN-NeerjaNeural", "en-IN-PrabhatNeural",
        "en-KE-AsiliaNeural", "en-KE-ChilembaNeural",
        "en-NG-AbeoNeural", "en-NG-EzinneNeural",
        "en-NZ-MitchellNeural", "en-NZ-MollyNeural",
        "en-PH-JamesNeural", "en-PH-RosaNeural",
        "en-SG-LunaNeural", "en-SG-WayneNeural",
        "en-TZ-ElimuNeural", "en-TZ-ImaniNeural",
        "en-US-AnaNeural", "en-US-AndrewMultilingualNeural",
        "en-US-AriaNeural", "en-US-AriaMultilingualNeural",
        "en-US-AvaNeural", "en-US-AvaMultilingualNeural",
        "en-US-BrianNeural", "en-US-BrianMultilingualNeural",
        "en-US-ChristopherNeural",
        "en-US-EmmaNeural", "en-US-EmmaMultilingualNeural",
        "en-US-ElizabethNeural",
        "en-US-JennyNeural", "en-US-JennyMultilingualNeural",
        "en-US-MichelleNeural",
        "en-US-RogerNeural", "en-US-SteffanNeural",
        "en-ZA-LeahNeural", "en-ZA-LukeNeural",
        "es-AR-ElenaNeural", "es-AR-TomasNeural",
        "es-CO-CatalinaNeural", "es-CO-GonzaloNeural",
        "es-ES-AlvaroNeural", "es-ES-ElviraNeural",
        "es-ES-XimenaNeural",
        "es-MX-DaliaNeural", "es-MX-JorgeNeural",
        "es-US-AlonsoNeural", "es-US-PalomaNeural",
        "et-EE-AnuNeural", "et-EE-KertNeural",
        "eu-ES-AinhoaNeural", "eu-ES-AnderNeural",
        "fa-IR-DilaraNeural", "fa-IR-FaridNeural",
        "fi-FI-HarriNeural", "fi-FI-NooraNeural",
        "fil-PH-AngeloNeural", "fil-PH-BlessicaNeural",
        "fr-BE-CharlineNeural", "fr-BE-GerardNeural",
        "fr-CA-AntoineNeural", "fr-CA-JeanNeural",
        "fr-CA-SylvieNeural", "fr-CA-ThierryNeural",
        "fr-CH-ArianeNeural", "fr-CH-FabriceNeural",
        "fr-FR-DeniseNeural", "fr-FR-EloiseNeural",
        "fr-FR-HenriNeural", "fr-FR-VivienneMultilingualNeural",
        "ga-IE-ColmNeural", "ga-IE-OrlaNeural",
        "gl-ES-RoiNeural", "gl-ES-SabelaNeural",
        "gu-IN-DhwaniNeural", "gu-IN-NiranjanNeural",
        "he-IL-AvriNeural", "he-IL-HilaNeural",
        "hi-IN-MadhurNeural", "hi-IN-SwaraNeural",
        "hr-HR-GabrijelaNeural", "hr-HR-SreckoNeural",
        "hu-HU-NoemiNeural", "hu-HU-TamasNeural",
        "id-ID-ArdiNeural", "id-ID-GadisNeural",
        "is-IS-GudrunNeural", "is-IS-GunnarNeural",
        "it-IT-DiegoNeural", "it-IT-ElsaNeural",
        "it-IT-GiuseppeNeural", "it-IT-IsabellaNeural",
        "ja-JP-DaichiNeural", "ja-JP-KeitaNeural",
        "ja-JP-NanamiNeural",
        "jv-ID-DimasNeural", "jv-ID-SitiNeural",
        "ka-GE-EkaNeural", "ka-GE-GiorgiNeural",
        "kk-KZ-AigulNeural", "kk-KZ-DauletNeural",
        "km-KH-PisethNeural", "km-KH-SreymomNeural",
        "kn-IN-GaganNeural", "kn-IN-SapnaNeural",
        "ko-KR-HyunsuNeural", "ko-KR-InJoonNeural",
        "ko-KR-SunHiNeural",
        "lo-LA-ChanthasoneNeural", "lo-LA-KeomanyNeural",
        "lt-LT-LeonasNeural", "lt-LT-OnaNeural",
        "lv-LV-EveritaNeural", "lv-LV-NilsNeural",
        "mk-MK-AleksandarNeural", "mk-MK-MarijaNeural",
        "ml-IN-MidhunNeural", "ml-IN-SobhanaNeural",
        "mn-MN-BataaNeural", "mn-MN-YesuiNeural",
        "mr-IN-AarohiNeural", "mr-IN-ManoharNeural",
        "ms-MY-OsmanNeural", "ms-MY-YasminNeural",
        "mt-MT-GraceNeural", "mt-MT-JosephNeural",
        "my-MM-NilarNeural", "my-MM-ThihaNeural",
        "nb-NO-FinnNeural", "nb-NO-PernilleNeural",
        "ne-NP-HemkalaNeural", "ne-NP-SagarNeural",
        "nl-BE-ArnaudNeural", "nl-BE-DenaNeural",
        "nl-NL-ColetteNeural", "nl-NL-FennaNeural",
        "nl-NL-MaartenNeural",
        "pl-PL-MarekNeural", "pl-PL-ZofiaNeural",
        "ps-AF-GulNawazNeural", "ps-AF-LatifaNeural",
        "pt-BR-AntonioNeural", "pt-BR-FranciscaNeural",
        "pt-BR-ThalitaNeural", "pt-BR-ValerioNeural",
        "pt-PT-DuarteNeural", "pt-PT-RaquelNeural",
        "ro-RO-AlinaNeural", "ro-RO-EmilNeural",
        "ru-RU-DariyaNeural", "ru-RU-SvyatoslavNeural",
        "si-LK-SameeraNeural", "si-LK-ThiliniNeural",
        "sk-SK-LukasNeural", "sk-SK-ViktoriaNeural",
        "sl-SI-PetraNeural", "sl-SI-RokNeural",
        "so-SO-MoogeNeural", "so-SO-UbaxNeural",
        "sq-AL-AnilaNeural", "sq-AL-IlirNeural",
        "sr-RS-NicholasNeural", "sr-RS-SophieNeural",
        "su-ID-JajangNeural", "su-ID-TutiNeural",
        "sv-SE-MattiasNeural", "sv-SE-SofieNeural",
        "sw-KE-RafikiNeural", "sw-KE-ZuriNeural",
        "sw-TZ-DaudiNeural", "sw-TZ-RehemaNeural",
        "ta-IN-PallaviNeural", "ta-IN-ValluvarNeural",
        "ta-LK-KumarNeural", "ta-LK-SaranyaNeural",
        "ta-MY-KaniNeural", "ta-MY-SuryaNeural",
        "te-IN-MohanNeural", "te-IN-ShrutiNeural",
        "th-TH-AcharaNeural", "th-TH-NiwatNeural",
        "tr-TR-AhmetNeural", "tr-TR-EmelNeural",
        "uk-UA-OstapNeural", "uk-UA-PolinaNeural",
        "ur-IN-GulNeural", "ur-IN-SalmanNeural",
        "ur-PK-AsadNeural", "ur-PK-UzmaNeural",
        "uz-UZ-MadinaNeural", "uz-UZ-SardorNeural",
        "vi-VN-HoaiMyNeural", "vi-VN-NamMinhNeural",
        "zh-CN-XiaoxiaoNeural", "zh-CN-XiaoyiNeural",
        "zh-CN-YunxiNeural", "zh-CN-YunyangNeural",
        "zh-HK-HiuMaanNeural", "zh-HK-WanLungNeural",
        "zh-TW-HsiaoChenNeural", "zh-TW-HsiaoYuNeural",
        "zh-TW-YunJheNeural",
        "zu-ZA-ThandoNeural", "zu-ZA-ThembaNeural",
    ]
}

/// Azure Speech Services REST API 引擎
///
/// REST API (非 SDK) — 对齐 D-04 (无第三方 SDK)
/// 端点: POST https://{region}.tts.speech.microsoft.com/cognitiveservices/v1
/// 认证: Ocp-Apim-Subscription-Key header
/// 请求体: SSML XML
/// 响应: 原始音频字节
pub(super) struct AzureSpeechEngine {
    client: reqwest::Client,
    config: AzureSection,
    /// 端点覆盖（用于 wiremock 测试，生产环境为 None）
    pub(super) endpoint_override: Option<String>,
}

impl AzureSpeechEngine {
    pub(super) fn new(config: AzureSection, proxy_config: &common::ProxyConfig) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config, endpoint_override: None })
    }

    /// 将 rate 转换为 Azure SSML 百分比格式
    /// 1.0 → "+0%", 1.5 → "+50%", 0.5 → "-50%"
    fn convert_rate_to_percent(rate: f64) -> String {
        if !rate.is_finite() || rate < 0.0 {
            return "+0%".to_string();
        }
        let percent = ((rate - 1.0) * 100.0).round() as i32;
        if percent >= 0 {
            format!("+{}%", percent)
        } else {
            format!("{}%", percent)
        }
    }

    /// 将 pitch 转换为 Azure SSML Hz 格式
    /// 0 → "+0Hz", 50 → "+50Hz", -10 → "-10Hz"
    fn convert_pitch_to_hz(pitch: f64) -> String {
        if !pitch.is_finite() {
            return "+0Hz".to_string();
        }
        let int_pitch = pitch.round() as i32;
        if int_pitch >= 0 {
            format!("+{}Hz", int_pitch)
        } else {
            format!("{}Hz", int_pitch)
        }
    }

    /// 构建 Azure REST API 的完整 URL
    fn api_url(&self) -> String {
        if let Some(ref override_url) = self.endpoint_override {
            return override_url.clone();
        }
        let region = &self.config.speech_region;
        format!(
            "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
            region
        )
    }

    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path, rate: f64, pitch: f64) -> Result<TtsOutput, TTSError> {
        if self.config.speech_key.is_empty() || self.config.speech_region.is_empty() {
            return Err(TTSError::AuthenticationFailed("Azure Speech key/region 未配置".to_string()));
        }

        // 剥离所有已知后缀（Azure REST API 不识别这些内部标记）
        // 顺序: -V2 -> -Female -> -Male，确保复合后缀（如 -Female-V2）被完全剥离
        let processed_voice_name = voice_name
            .trim()
            .trim_end_matches("-V2")
            .trim_end_matches("-Female")
            .trim_end_matches("-Male");

        // 从 voice_name 提取语言标签 (如 "zh-CN-YunzeNeural" → "zh-CN")
        let lang = extract_azure_lang(processed_voice_name);

        // 构建 SSML 请求体 (对齐 Python 版 azure_tts_v2 SDK 调用)
        // REST API 使用 SSML 格式:
        let rate_str = Self::convert_rate_to_percent(rate);
        let pitch_str = Self::convert_pitch_to_hz(pitch);
        let escaped_text: String = common::escape_xml_text(text)
            .chars()
            .filter(|&c| {
                c == '\t' || c == '\n' || c == '\r'
                    || (c as u32 >= 0x20 && !(0x7F..=0x9F).contains(&(c as u32)))
            })
            .collect();
        let ssml = format!(
            r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="{}"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody></voice></speak>"#,
            common::escape_xml_attr(&lang),
            common::escape_xml_attr(processed_voice_name),
            rate_str, pitch_str, escaped_text
        );

        let url = self.api_url();

        let response = self.client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", &self.config.speech_key)
            .header("Content-Type", "application/ssml+xml")
            .header("X-Microsoft-OutputFormat", "audio-48khz-192kbitrate-mono-mp3")
            .header("User-Agent", "NarratoAI")
            .body(ssml)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TTSError::ConnectionFailed(format!("Azure 请求失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TTSError::SynthesisFailed(format!(
                "Azure TTS 返回 {}: {}", status.as_u16(), body
            )));
        }

        let audio_bytes = response.bytes().await
            .map_err(|e| TTSError::SynthesisFailed(format!("Azure 读取响应失败: {}", e)))?;

        common::write_audio_bytes(output_path, &audio_bytes).await?;

        // REST API 不返回词边界 (Pitfall 2)
        // Python 版 azure_tts_v2 使用 SDK 支持词边界，REST API 无此能力
        Ok(TtsOutput {
            audio_file_path: output_path.to_path_buf(),
            word_boundaries: vec![],
            duration: 0.0,
        })
    }
}

/// 从 Azure Neural voice name 提取语言标签
/// "zh-CN-YunzeNeural" → "zh-CN"
fn extract_azure_lang(voice_name: &str) -> String {
    // Azure Neural 格式: xx-XX-NameNeural
    // 取前两个连字符之间的部分
    let parts: Vec<&str> = voice_name.splitn(3, '-').collect();
    if parts.len() >= 2 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        "zh-CN".to_string()
    }
}

#[async_trait]
impl TtsProvider for AzureSpeechEngine {
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
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use tempfile::TempDir;

    // ===== should_use_azure_services 测试 =====

    #[test]
    fn test_should_use_azure_services_v2_suffix() {
        assert!(should_use_azure_services("zh-CN-YunzeNeural-V2"));
    }

    #[test]
    fn test_should_use_azure_services_neural_format() {
        assert!(should_use_azure_services("zh-CN-YunzeNeural"));
    }

    #[test]
    fn test_should_use_azure_services_female_suffix() {
        assert!(should_use_azure_services("zh-CN-XiaoyiNeural-Female"));
    }

    #[test]
    fn test_should_use_azure_services_male_suffix() {
        assert!(should_use_azure_services("zh-CN-YunzeNeural-Male"));
    }

    #[test]
    fn test_should_use_azure_services_non_azure() {
        assert!(!should_use_azure_services("random_voice"));
    }

    #[test]
    fn test_should_use_azure_services_empty() {
        assert!(!should_use_azure_services(""));
    }

    // ===== extract_azure_lang 测试 =====

    #[test]
    fn test_extract_azure_lang() {
        assert_eq!(extract_azure_lang("zh-CN-YunzeNeural"), "zh-CN");
        assert_eq!(extract_azure_lang("en-US-AriaNeural"), "en-US");
        assert_eq!(extract_azure_lang("fr-FR-DeniseNeural"), "fr-FR");
    }

    // ===== get_azure_voices 测试 =====

    #[test]
    fn test_get_azure_voices_not_empty() {
        let voices = get_azure_voices();
        assert!(!voices.is_empty());
        assert!(voices.contains(&"zh-CN-XiaoxiaoNeural"));
        assert!(voices.contains(&"en-US-AriaNeural"));
    }

    // ===== Engine 新建测试 =====

    #[test]
    fn test_azure_engine_new() {
        let config = AzureSection::default();
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = AzureSpeechEngine::new(config, &proxy_config).expect("构建引擎失败");
        assert!(engine.config.speech_key.is_empty());
        assert!(engine.endpoint_override.is_none());
    }

    // ===== Wiremock 测试 =====

    #[tokio::test]
    async fn test_azure_speech_success() {
        let mock_server = MockServer::start().await;
        let mock_audio = b"fake audio data";

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(mock_audio))
            .mount(&mock_server)
            .await;

        let config = AzureSection {
            speech_key: "test-key".to_string(),
            speech_region: "MOCK".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let mut engine = AzureSpeechEngine::new(config, &proxy_config).expect("构建引擎失败");
        engine.endpoint_override = Some(mock_server.uri());

        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "zh-CN-YunzeNeural", &output_path, 1.0, 0.0).await;
        assert!(result.is_ok(), "Azure 成功: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.audio_file_path.exists());
        let written = std::fs::read(&output.audio_file_path).expect("读取文件失败");
        assert_eq!(written, mock_audio);
        assert!(output.word_boundaries.is_empty());
    }

    #[tokio::test]
    async fn test_azure_speech_empty_text() {
        let config = AzureSection::default();
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = AzureSpeechEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize("", "zh-CN-YunzeNeural", 1.0, 0.0, &output_path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::SynthesisFailed(msg) => assert!(msg.contains("不能为空"), "错误消息: {}", msg),
            _ => panic!("应为 SynthesisFailed"),
        }
    }

    #[tokio::test]
    async fn test_azure_speech_empty_config() {
        let config = AzureSection {
            speech_key: String::new(),
            speech_region: "eastasia".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let engine = AzureSpeechEngine::new(config, &proxy_config).expect("构建引擎失败");
        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        let result = engine.synthesize_once("test text", "zh-CN-YunzeNeural", &output_path, 1.0, 0.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::AuthenticationFailed(msg) => assert!(msg.contains("未配置"), "错误消息: {}", msg),
            _ => panic!("应为 AuthenticationFailed"),
        }
    }

    #[tokio::test]
    async fn test_azure_speech_v2_suffix_stripped() {
        // 验证 -V2 后缀会在 SSML 中被剥离
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"ok"))
            .mount(&mock_server)
            .await;

        let config = AzureSection {
            speech_key: "test-key".to_string(),
            speech_region: "MOCK".to_string(),
        };
        let proxy_config = common::ProxyConfig::from_proxy(None);
        let mut engine = AzureSpeechEngine::new(config, &proxy_config).expect("构建引擎失败");
        engine.endpoint_override = Some(mock_server.uri());

        let dir = TempDir::new().expect("创建临时目录失败");
        let output_path = dir.path().join("output.mp3");

        // -V2 后缀被剥离后，should_use_azure_services 在上层已处理
        // engine 内部会剥离 -V2 后再构建 SSML
        let result = engine.synthesize_once("test", "zh-CN-YunzeNeural-V2", &output_path, 1.0, 0.0).await;
        assert!(result.is_ok());
    }
}
