use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 顶层配置，与 config.toml 结构一一对应
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSection,
    #[serde(default)]
    pub ui: UiSection,
    #[serde(default)]
    pub azure: AzureSection,
    #[serde(default)]
    pub tencent: TencentSection,
    #[serde(default)]
    pub soulvoice: SoulVoiceSection,
    #[serde(default)]
    pub tts_qwen: TtsQwenSection,
    #[serde(default)]
    pub indextts2: IndexTTS2Section,
    #[serde(default)]
    pub doubaotts: DoubaoTTSSection,
    #[serde(default)]
    pub proxy: ProxySection,
    #[serde(default)]
    pub frames: FramesSection,
    #[serde(default)]
    pub audio: AudioSection,
}

/// [app] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct AppSection {
    #[serde(default)]
    pub project_version: String,
    #[serde(default)]
    pub vision_llm_provider: String,
    #[serde(default)]
    pub vision_openai_model_name: String,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub vision_openai_api_key: String,
    #[serde(default)]
    pub vision_openai_base_url: String,
    #[serde(default)]
    pub text_llm_provider: String,
    #[serde(default)]
    pub text_openai_model_name: String,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub text_openai_api_key: String,
    #[serde(default)]
    pub text_openai_base_url: String,
    #[serde(default)]
    pub llm_vision_timeout: u64,
    #[serde(default)]
    pub llm_text_timeout: u64,
    #[serde(default)]
    pub llm_max_retries: u32,
    #[serde(default)]
    pub hide_config: bool,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub pexels_api_keys: Vec<String>,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub pixabay_api_keys: Vec<String>,
}

/// [ui] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct UiSection {
    #[serde(default)]
    pub tts_engine: String,
    #[serde(default)]
    pub edge_voice_name: String,
    #[serde(default)]
    pub edge_volume: u32,
    #[serde(default)]
    pub edge_rate: f64,
    #[serde(default)]
    pub edge_pitch: i32,
    #[serde(default)]
    pub azure_voice_name: String,
    #[serde(default)]
    pub azure_volume: u32,
    #[serde(default)]
    pub azure_rate: f64,
    #[serde(default)]
    pub azure_pitch: i32,
    #[serde(default)]
    pub doubaotts_voice_type: String,
    #[serde(default)]
    pub doubaotts_rate: f64,
}

/// [azure] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct AzureSection {
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub speech_key: String,
    #[serde(default)]
    pub speech_region: String,
}

/// [tencent] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct TencentSection {
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub secret_id: String,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub secret_key: String,
    #[serde(default)]
    pub region: String,
}

/// [soulvoice] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct SoulVoiceSection {
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub api_key: String,
    #[serde(default)]
    pub voice_uri: String,
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub model: String,
}

/// [tts_qwen] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct TtsQwenSection {
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub api_key: String,
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub model_name: String,
}

/// [indextts2] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct IndexTTS2Section {
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub reference_audio: String,
    #[serde(default)]
    pub infer_mode: String,
    #[serde(default)]
    pub temperature: f64,
    #[serde(default)]
    pub top_p: f64,
    #[serde(default)]
    pub top_k: u32,
    #[serde(default)]
    pub do_sample: bool,
    #[serde(default)]
    pub num_beams: u32,
    #[serde(default)]
    pub repetition_penalty: f64,
}

/// [doubaotts] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct DoubaoTTSSection {
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub ak: String,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub sk: String,
    #[serde(default)]
    pub appid: String,
    #[serde(default, skip_serializing)]
    #[ts(skip)]
    pub token: String,
    #[serde(default)]
    pub cluster: String,
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub volume: f64,
    #[serde(default)]
    pub pitch: f64,
    #[serde(default)]
    pub silence_duration: f64,
}

/// [proxy] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct ProxySection {
    #[serde(default)]
    pub http: String,
    #[serde(default)]
    pub https: String,
    #[serde(default)]
    pub enabled: bool,
}

/// [frames] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct FramesSection {
    #[serde(default)]
    pub frame_interval_input: u64,
    #[serde(default)]
    pub vision_batch_size: u32,
    #[serde(default)]
    pub vision_max_concurrency: u32,
}

/// [audio] 配置段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
#[serde(deny_unknown_fields)]
pub struct AudioSection {
    #[serde(default)]
    pub target_lufs: f64,
    #[serde(default)]
    pub max_peak: f64,
    #[serde(default)]
    pub enable_audio_normalization: bool,
    #[serde(default)]
    pub enable_smart_volume: bool,
    #[serde(default)]
    pub tts_volume: f64,
    #[serde(default)]
    pub original_volume: f64,
    #[serde(default)]
    pub bgm_volume: f64,
    #[serde(default)]
    pub sample_rate: u32,
    #[serde(default)]
    pub channels: u32,
    #[serde(default)]
    pub bitrate: u32,
    #[serde(default)]
    pub crossfade_duration: f64,
    #[serde(default)]
    pub bgm_fade_out: f64,
    #[serde(default = "default_volume_profile")]
    pub volume_profile: String,
}

fn default_volume_profile() -> String {
    "balanced".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从包含所有 10 个 section 的完整 TOML 字符串解析
    #[test]
    fn test_load_full_config() {
        let toml_str = r#"
[app]
project_version = "0.7.8"
vision_llm_provider = "openai"
vision_openai_model_name = "Qwen/Qwen3.5-122B-A10B"
vision_openai_api_key = "sk-test-key"
vision_openai_base_url = "https://api.siliconflow.cn/v1"
text_llm_provider = "openai"
text_openai_model_name = "Pro/zai-org/GLM-5"
text_openai_api_key = "sk-test-key-text"
text_openai_base_url = "https://api.siliconflow.cn/v1"
llm_vision_timeout = 120
llm_text_timeout = 180
llm_max_retries = 3
hide_config = true

[ui]
tts_engine = "edge_tts"
edge_voice_name = "zh-CN-XiaoyiNeural-Female"
edge_volume = 80
edge_rate = 1.0
edge_pitch = 0
azure_voice_name = "zh-CN-XiaoyiNeural-Female"
azure_volume = 80
azure_rate = 1.0
azure_pitch = 0
doubaotts_voice_type = "BV700_V2_streaming"
doubaotts_rate = 1.0

[azure]
speech_key = "az-key"
speech_region = "eastasia"

[tencent]
secret_id = "tencent-id"
secret_key = "tencent-key"
region = "ap-beijing"

[soulvoice]
api_key = "sv-key"
voice_uri = "speech:test"
api_url = "https://tts.scsmtech.cn/tts"
model = "FunAudioLLM/CosyVoice2-0.5B"

[tts_qwen]
api_key = "qw-key"
model_name = "qwen3-tts-flash"

[indextts2]
api_url = "http://127.0.0.1:8081/tts"
reference_audio = "/path/to/audio.wav"
infer_mode = "普通推理"
temperature = 1.0
top_p = 0.8
top_k = 30
do_sample = true
num_beams = 3
repetition_penalty = 10.0

[doubaotts]
ak = "test-ak"
sk = "test-sk"
appid = "db-appid"
token = "db-token"
cluster = "volcano_tts"
volume = 1.0
pitch = 1.0
silence_duration = 0.125

[proxy]
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
enabled = true

[frames]
frame_interval_input = 3
vision_batch_size = 10
vision_max_concurrency = 2

[audio]
target_lufs = -14.0
max_peak = -1.0
enable_audio_normalization = true
enable_smart_volume = true
tts_volume = 0.8
original_volume = 1.3
bgm_volume = 0.3
sample_rate = 44100
channels = 2
bitrate = 192000
crossfade_duration = 0.1
bgm_fade_out = 3.0
"#;

        let config: AppConfig = toml::from_str(toml_str).expect("完整 TOML 应解析成功");
        assert_eq!(config.app.project_version, "0.7.8");
        assert_eq!(config.app.vision_llm_provider, "openai");
        assert_eq!(config.app.llm_vision_timeout, 120);
        assert_eq!(config.ui.tts_engine, "edge_tts");
        assert_eq!(config.ui.edge_volume, 80);
        assert_eq!(config.ui.doubaotts_voice_type, "BV700_V2_streaming");
        assert_eq!(config.ui.doubaotts_rate, 1.0);
        assert_eq!(config.azure.speech_key, "az-key");
        assert_eq!(config.tencent.region, "ap-beijing");
        assert_eq!(config.soulvoice.model, "FunAudioLLM/CosyVoice2-0.5B");
        assert_eq!(config.soulvoice.voice_uri, "speech:test");
        assert_eq!(config.tts_qwen.model_name, "qwen3-tts-flash");
        assert_eq!(config.indextts2.temperature, 1.0);
        assert_eq!(config.doubaotts.cluster, "volcano_tts");
        assert_eq!(config.doubaotts.volume, 1.0);
        assert_eq!(config.doubaotts.ak, "test-ak");
        assert_eq!(config.doubaotts.sk, "test-sk");
        assert!(config.proxy.enabled);
        assert_eq!(config.frames.frame_interval_input, 3);
        assert_eq!(config.audio.target_lufs, -14.0);
        assert_eq!(config.audio.enable_audio_normalization, true);
        assert_eq!(config.audio.volume_profile, "balanced");
        assert_eq!(config.audio.tts_volume, 0.8);
        assert_eq!(config.audio.sample_rate, 44100);
        assert!(config.app.pexels_api_keys.is_empty());
        assert!(config.app.pixabay_api_keys.is_empty());
    }

    /// 从空 TOML 字符串解析，应全部使用默认值
    #[test]
    fn test_load_empty_toml() {
        let config: AppConfig = toml::from_str("").expect("空 TOML 应成功解析");
        // Default values must not panic and should be sensible
        let _ = config.app.project_version;
        let _ = config.ui.tts_engine;
        let _ = config.frames.frame_interval_input;
    }

    /// 缺失多个 section 的 TOML，应使用默认值
    #[test]
    fn test_load_missing_section() {
        let toml_str = r#"
[app]
project_version = "0.7.8"
"#;
        let config: AppConfig = toml::from_str(toml_str).expect("仅含 app section 的 TOML 应解析成功");
        // 缺失的 section 应使用默认值
        assert_eq!(config.app.project_version, "0.7.8");
        assert_eq!(config.azure.speech_key, "", "缺失的 azure section 应使用 Default");
        assert!(!config.proxy.enabled, "缺失的 proxy section 应使用 Default");
    }

    /// 验证关键默认值是否正确
    #[test]
    fn test_default_values() {
        let config = AppConfig::default();

        // AppSection defaults
        assert_eq!(config.app.project_version, "0.7.8");
        assert_eq!(config.app.vision_llm_provider, "openai");
        assert_eq!(config.app.llm_vision_timeout, 120);
        assert_eq!(config.app.llm_text_timeout, 180);
        assert_eq!(config.app.llm_max_retries, 3);
        assert!(config.app.hide_config);

        // UiSection defaults
        assert_eq!(config.ui.tts_engine, "edge_tts");
        assert_eq!(config.ui.edge_volume, 80);

        // FramesSection defaults
        assert_eq!(config.frames.frame_interval_input, 3);
        assert_eq!(config.frames.vision_batch_size, 10);
        assert_eq!(config.frames.vision_max_concurrency, 2);

        // AudioSection defaults
        assert_eq!(config.audio.volume_profile, "balanced");
    }

    /// validate 方法在当前阶段不应拒绝空 API key
    #[test]
    fn test_validate_empty_api_key_ok() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok(), "Phase 1 不应校验 API key");
    }
}
