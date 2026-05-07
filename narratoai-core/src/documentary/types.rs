use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::tts::WordBoundary;

/// 纪录片流水线请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentaryRequest {
    pub video_path: PathBuf,
    pub script_path: PathBuf,
    pub tts_engine: String,
    pub voice_name: String,
    pub voice_rate: f64,
    pub voice_pitch: f64,
    pub tts_volume: f64,
    pub original_volume: f64,
    pub bgm_volume: f64,
    pub bgm_path: Option<PathBuf>,
    pub subtitle_enabled: bool,
    pub subtitle_font: Option<String>,
    pub subtitle_font_size: u32,
    pub subtitle_color: String,
    pub subtitle_position: String,
    pub output_dir: Option<PathBuf>,
    pub threads: u32,
}

impl Default for DocumentaryRequest {
    fn default() -> Self {
        Self {
            video_path: PathBuf::new(),
            script_path: PathBuf::new(),
            tts_engine: "edge_tts".to_string(),
            voice_name: String::new(),
            voice_rate: 1.0,
            voice_pitch: 0.0,
            tts_volume: 1.0,
            original_volume: 0.7,
            bgm_volume: 0.3,
            bgm_path: None,
            subtitle_enabled: true,
            subtitle_font: None,
            subtitle_font_size: 40,
            subtitle_color: "#FFFFFF".to_string(),
            subtitle_position: "bottom".to_string(),
            output_dir: None,
            threads: 4,
        }
    }
}

impl DocumentaryRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.video_path.as_os_str().is_empty() {
            return Err("video_path 不能为空".to_string());
        }
        if self.script_path.as_os_str().is_empty() {
            return Err("script_path 不能为空".to_string());
        }
        if self.voice_rate <= 0.0 || self.voice_rate > 5.0 {
            return Err(format!("voice_rate 超出有效范围 (0, 5]: {}", self.voice_rate));
        }
        if self.voice_pitch < -10.0 || self.voice_pitch > 10.0 {
            return Err(format!("voice_pitch 超出有效范围 [-10, 10]: {}", self.voice_pitch));
        }
        if self.tts_volume <= 0.0 || self.tts_volume > 10.0 {
            return Err(format!("tts_volume 超出有效范围 (0, 10]: {}", self.tts_volume));
        }
        if self.original_volume < 0.0 || self.original_volume > 10.0 {
            return Err(format!("original_volume 超出有效范围 [0, 10]: {}", self.original_volume));
        }
        if self.bgm_volume < 0.0 || self.bgm_volume > 10.0 {
            return Err(format!("bgm_volume 超出有效范围 [0, 10]: {}", self.bgm_volume));
        }
        if self.threads == 0 {
            return Err("threads 必须大于 0".to_string());
        }
        // 校验 subtitle_color 为 #RRGGBB 格式
        let hex = self.subtitle_color.trim_start_matches('#');
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("subtitle_color 必须为 #RRGGBB 格式".to_string());
        }
        Ok(())
    }
}

/// 流水线进度步骤
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStep {
    LoadScript,
    Tts,
    Clip,
    MergeAudio,
    Concat,
    Composite,
}

/// 进度回调类型（字符串基——step_name, percent, message）
pub type ProgressCallback = Box<dyn Fn(&str, f32, &str) + Send + Sync>;

/// TTS 合成结果——关联到具体脚本片段
#[derive(Debug, Clone)]
pub struct TtsResult {
    pub clip_id: i64,
    pub audio_path: PathBuf,
    pub subtitle_path: PathBuf,
    pub duration: f64,
    pub word_boundaries: Vec<WordBoundary>,
}
