use crate::config::types::*;

/// 默认值来源：config.example.toml 中的注释值和 Python 版 defaults.py
impl Default for AppSection {
    fn default() -> Self {
        Self {
            project_version: "0.7.8".to_string(),
            vision_llm_provider: "openai".to_string(),
            vision_openai_model_name: "Qwen/Qwen3.5-122B-A10B".to_string(),
            vision_openai_api_key: String::new(),
            vision_openai_base_url: "https://api.siliconflow.cn/v1".to_string(),
            text_llm_provider: "openai".to_string(),
            text_openai_model_name: "Pro/zai-org/GLM-5".to_string(),
            text_openai_api_key: String::new(),
            text_openai_base_url: "https://api.siliconflow.cn/v1".to_string(),
            llm_vision_timeout: 120,
            llm_text_timeout: 180,
            llm_max_retries: 3,
            hide_config: true,
            pexels_api_keys: Vec::new(),
            pixabay_api_keys: Vec::new(),
        }
    }
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            tts_engine: "edge_tts".to_string(),
            edge_voice_name: "zh-CN-XiaoyiNeural-Female".to_string(),
            edge_volume: 80,
            edge_rate: 1.0,
            edge_pitch: 0,
            azure_voice_name: "zh-CN-XiaoyiNeural-Female".to_string(),
            azure_volume: 80,
            azure_rate: 1.0,
            azure_pitch: 0,
            doubaotts_voice_type: "BV700_V2_streaming".to_string(),
            doubaotts_rate: 1.0,
        }
    }
}

impl Default for TencentSection {
    fn default() -> Self {
        Self {
            secret_id: String::new(),
            secret_key: String::new(),
            region: "ap-beijing".to_string(),
        }
    }
}

impl Default for SoulVoiceSection {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            voice_uri: "speech:mcg3fdnx:clzkyf4vy00e5qr6hywum4u84:bzznlkuhcjzpbosexitr".to_string(),
            api_url: "https://tts.scsmtech.cn/tts".to_string(),
            model: "FunAudioLLM/CosyVoice2-0.5B".to_string(),
        }
    }
}

impl Default for TtsQwenSection {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_url: String::new(),
            model_name: "qwen3-tts-flash".to_string(),
        }
    }
}

impl Default for IndexTTS2Section {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:8081/tts".to_string(),
            reference_audio: String::new(),
            infer_mode: "普通推理".to_string(),
            temperature: 1.0,
            top_p: 0.8,
            top_k: 30,
            do_sample: true,
            num_beams: 3,
            repetition_penalty: 1.5,
        }
    }
}

impl Default for DoubaoTTSSection {
    fn default() -> Self {
        Self {
            ak: String::new(),
            sk: String::new(),
            appid: String::new(),
            token: String::new(),
            cluster: "volcano_tts".to_string(),
            volume: 1.0,
            pitch: 1.0,
            silence_duration: 0.125,
        }
    }
}

impl Default for FramesSection {
    fn default() -> Self {
        Self {
            frame_interval_input: 3,
            vision_batch_size: 10,
            vision_max_concurrency: 2,
        }
    }
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            target_lufs: -23.0,
            max_peak: -1.0,
            enable_audio_normalization: false,
            enable_smart_volume: false,
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
            sample_rate: 44100,
            channels: 2,
            bitrate: 192_000,
            crossfade_duration: 0.1,
            bgm_fade_out: 3.0,
        }
    }
}

