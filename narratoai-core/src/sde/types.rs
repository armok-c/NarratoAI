use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::documentary::types::TtsResult;

/// SDE 流水线请求参数
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct SdeRequest {
    pub subtitle_path: PathBuf,
    pub video_path: PathBuf,
    pub drama_name: String,
    pub temperature: f64,
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

impl Default for SdeRequest {
    fn default() -> Self {
        Self {
            subtitle_path: PathBuf::new(),
            video_path: PathBuf::new(),
            drama_name: String::new(),
            temperature: 1.0,
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

impl SdeRequest {
    /// 校验请求参数，返回错误描述或 Ok(())
    pub fn validate(&self) -> Result<(), String> {
        if self.subtitle_path.as_os_str().is_empty() {
            return Err("subtitle_path 不能为空".to_string());
        }
        if self.video_path.as_os_str().is_empty() {
            return Err("video_path 不能为空".to_string());
        }
        if !self.subtitle_path.exists() {
            return Err(format!("字幕文件不存在: {}", self.subtitle_path.display()));
        }
        if !self.video_path.exists() {
            return Err(format!("视频文件不存在: {}", self.video_path.display()));
        }
        if self.voice_rate <= 0.0 || self.voice_rate > 5.0 {
            return Err(format!("voice_rate 超出有效范围 (0, 5]: {}", self.voice_rate));
        }
        if !(-10.0..=10.0).contains(&self.voice_pitch) {
            return Err(format!("voice_pitch 超出有效范围 [-10, 10]: {}", self.voice_pitch));
        }
        if self.tts_volume <= 0.0 || self.tts_volume > 10.0 {
            return Err(format!("tts_volume 超出有效范围 (0, 10]: {}", self.tts_volume));
        }
        if !(0.0..=10.0).contains(&self.original_volume) {
            return Err(format!("original_volume 超出有效范围 [0, 10]: {}", self.original_volume));
        }
        if !(0.0..=10.0).contains(&self.bgm_volume) {
            return Err(format!("bgm_volume 超出有效范围 [0, 10]: {}", self.bgm_volume));
        }
        if self.threads == 0 {
            return Err("threads 必须大于 0".to_string());
        }
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(format!("temperature 超出有效范围 [0, 2]: {}", self.temperature));
        }
        if self.subtitle_font_size == 0 || self.subtitle_font_size > 200 {
            return Err(format!(
                "subtitle_font_size 超出有效范围 [1, 200]: {}",
                self.subtitle_font_size
            ));
        }
        // 校验 subtitle_color 为 #RRGGBB 格式
        let hex = self.subtitle_color.trim_start_matches('#');
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("subtitle_color 必须为 #RRGGBB 格式".to_string());
        }
        Ok(())
    }
}

/// SDE 进度步骤
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdeProgressStep {
    ParseSubtitle,
    PlotAnalysis,
    ScriptGenerate,
    LoadScript,
    Tts,
    Clip,
    MergeAudio,
    Concat,
    Composite,
}

/// SDE 进度回调类型（字符串基——step_name, percent, message）
pub type SdeProgressCallback = Box<dyn Fn(&str, f32, &str) + Send + Sync>;

/// 字幕段落——解析后的 SRT 条目
pub use crate::subtitle::types::SubtitleSegment;

/// SDE 流水线中间状态——贯穿 9 步
pub struct SdePipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    pub subtitle_text: String,
    pub detected_encoding: String,
    pub subtitle_segments: Vec<SubtitleSegment>,
    pub plot_analysis: String,
    pub narration_raw: String,
    pub script: crate::script::types::Script,
    pub tts_results: HashMap<i64, TtsResult>,
    pub video_clips: HashMap<i64, PathBuf>,
    pub merged_audio_path: Option<PathBuf>,
    pub merged_subtitle_path: Option<PathBuf>,
    pub combined_video_path: Option<PathBuf>,
    pub output_video_path: Option<PathBuf>,
    pub total_duration: f64,
    pub progress: Option<SdeProgressCallback>,
}

impl SdePipelineState {
    /// 创建新的流水线状态
    pub fn new(task_id: String, task_dir: PathBuf) -> Self {
        Self {
            task_id,
            task_dir,
            subtitle_text: String::new(),
            detected_encoding: String::new(),
            subtitle_segments: Vec::new(),
            plot_analysis: String::new(),
            narration_raw: String::new(),
            script: Vec::new(),
            tts_results: HashMap::new(),
            video_clips: HashMap::new(),
            merged_audio_path: None,
            merged_subtitle_path: None,
            combined_video_path: None,
            output_video_path: None,
            total_duration: 0.0,
            progress: None,
        }
    }

    /// 触发进度回调（如果已注册）
    pub fn emit_progress(&self, step: &str, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sde_request_validate_valid() {
        let dir = tempfile::TempDir::new().unwrap();
        let sub = dir.path().join("sub.srt");
        let vid = dir.path().join("video.mp4");
        std::fs::write(&sub, "").unwrap();
        std::fs::write(&vid, "").unwrap();
        let req = SdeRequest {
            subtitle_path: sub,
            video_path: vid,
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_sde_request_validate_empty_path() {
        let req = SdeRequest::default();
        let err = req.validate().unwrap_err();
        assert!(err.contains("subtitle_path"), "应提示 subtitle_path 为空: {}", err);
    }

    #[test]
    fn test_sde_request_default_values() {
        let req = SdeRequest::default();
        assert_eq!(req.temperature, 1.0);
        assert_eq!(req.tts_engine, "edge_tts");
        assert_eq!(req.voice_rate, 1.0);
        assert_eq!(req.voice_pitch, 0.0);
        assert_eq!(req.tts_volume, 1.0);
        assert_eq!(req.original_volume, 0.7);
        assert_eq!(req.bgm_volume, 0.3);
        assert_eq!(req.subtitle_enabled, true);
        assert_eq!(req.subtitle_font_size, 40);
        assert_eq!(req.subtitle_color, "#FFFFFF");
        assert_eq!(req.subtitle_position, "bottom");
        assert_eq!(req.threads, 4);
    }

    #[test]
    fn test_sde_progress_step_order() {
        // 确认 9 步以 ParseSubtitle 开头、Composite 结尾
        let first = SdeProgressStep::ParseSubtitle;
        let last = SdeProgressStep::Composite;
        assert_eq!(first as u8, 0);
        assert_eq!(last as u8, 8);
    }

    #[test]
    fn test_subtitle_segment_creation() {
        let seg = SubtitleSegment {
            index: 1,
            start_secs: 0.0,
            end_secs: 5.5,
            text: "你好".into(),
        };
        assert_eq!(seg.index, 1);
        assert!((seg.start_secs - 0.0).abs() < f64::EPSILON);
        assert!((seg.end_secs - 5.5).abs() < f64::EPSILON);
        assert_eq!(seg.text, "你好");
    }
}
