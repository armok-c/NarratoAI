use crate::config::types::*;

// RED phase: deliberately wrong defaults to make tests fail

impl Default for AppSection {
    fn default() -> Self {
        Self {
            // wrong project_version — test expects "0.1.0" but RED returns "0.0.0"
            project_version: "0.0.0".to_string(),
            vision_llm_provider: String::new(),
            vision_openai_model_name: String::new(),
            vision_openai_api_key: String::new(),
            vision_openai_base_url: String::new(),
            text_llm_provider: String::new(),
            text_openai_model_name: String::new(),
            text_openai_api_key: String::new(),
            text_openai_base_url: String::new(),
            llm_vision_timeout: 0,
            llm_text_timeout: 0,
            llm_max_retries: 0,
            hide_config: false,
        }
    }
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            tts_engine: String::new(),
            edge_voice_name: String::new(),
            edge_volume: 0,
            edge_rate: 0.0,
            edge_pitch: 0,
            azure_voice_name: String::new(),
            azure_volume: 0,
            azure_rate: 0.0,
            azure_pitch: 0,
            doubaotts_voice_type: String::new(),
            doubaotts_rate: 0.0,
        }
    }
}

impl Default for AzureSection {
    fn default() -> Self {
        Self {
            speech_key: String::new(),
            speech_region: String::new(),
        }
    }
}

impl Default for TencentSection {
    fn default() -> Self {
        Self {
            secret_id: String::new(),
            secret_key: String::new(),
            region: String::new(),
        }
    }
}

impl Default for SoulVoiceSection {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            voice_uri: String::new(),
            api_url: String::new(),
            model: String::new(),
        }
    }
}

impl Default for TtsQwenSection {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model_name: String::new(),
        }
    }
}

impl Default for IndexTTS2Section {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            reference_audio: String::new(),
            infer_mode: String::new(),
            temperature: 0.0,
            top_p: 0.0,
            top_k: 0,
            do_sample: false,
            num_beams: 0,
            repetition_penalty: 0.0,
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
            cluster: String::new(),
            volume: 0.0,
            pitch: 0.0,
            silence_duration: 0.0,
        }
    }
}

impl Default for ProxySection {
    fn default() -> Self {
        Self {
            http: String::new(),
            https: String::new(),
            enabled: false,
        }
    }
}

impl Default for FramesSection {
    fn default() -> Self {
        Self {
            frame_interval_input: 0,
            vision_batch_size: 0,
            vision_max_concurrency: 0,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSection::default(),
            ui: UiSection::default(),
            azure: AzureSection::default(),
            tencent: TencentSection::default(),
            soulvoice: SoulVoiceSection::default(),
            tts_qwen: TtsQwenSection::default(),
            indextts2: IndexTTS2Section::default(),
            doubaotts: DoubaoTTSSection::default(),
            proxy: ProxySection::default(),
            frames: FramesSection::default(),
        }
    }
}
