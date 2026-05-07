use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// SDP 流水线请求参数
///
/// 精简设计：包含 run_sdp 和 generate_sdp_script 所需的所有参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpRequest {
    /// 字幕文件路径（用于 generate_sdp_script）
    pub subtitle_path: PathBuf,
    /// 视频文件路径（用于 run_sdp）
    pub video_path: PathBuf,
    /// 脚本路径（None 表示由 generate_sdp_script 自动生成 merged_subtitle.json）
    pub script_path: Option<PathBuf>,
    /// LLM temperature，默认 0.1
    pub temperature: f64,
    /// 建议提取的片段数量，默认 5
    pub custom_clips: u32,
    /// 原声音量，默认 0.7
    pub original_volume: f64,
    /// BGM 音量，默认 0.3
    pub bgm_volume: f64,
    /// 可选 BGM 文件路径
    pub bgm_path: Option<PathBuf>,
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 并发线程数，默认 4
    pub threads: u32,
}

impl Default for SdpRequest {
    fn default() -> Self {
        Self {
            subtitle_path: PathBuf::new(),
            video_path: PathBuf::new(),
            script_path: None,
            temperature: 0.1,
            custom_clips: 5,
            original_volume: 0.7,
            bgm_volume: 0.3,
            bgm_path: None,
            output_dir: None,
            threads: 4,
        }
    }
}

impl SdpRequest {
    /// 校验请求参数，返回错误描述或 Ok(())
    pub fn validate(&self) -> Result<(), String> {
        if self.subtitle_path.as_os_str().is_empty() {
            return Err("subtitle_path 不能为空".to_string());
        }
        if self.video_path.as_os_str().is_empty() {
            return Err("video_path 不能为空".to_string());
        }
        if !(0.0..=10.0).contains(&self.original_volume) {
            return Err(format!(
                "original_volume 超出有效范围 [0, 10]: {}",
                self.original_volume
            ));
        }
        if !(0.0..=10.0).contains(&self.bgm_volume) {
            return Err(format!(
                "bgm_volume 超出有效范围 [0, 10]: {}",
                self.bgm_volume
            ));
        }
        if self.threads == 0 {
            return Err("threads 必须大于 0".to_string());
        }
        Ok(())
    }
}

/// SDP 进度步骤
///
/// 精简为 3 步（与 SDE 的 9 步不同），对应短剧混剪流水线的最终合成阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdpProgressStep {
    Clip,
    Concat,
    Composite,
}

/// SDP 进度回调类型（字符串基——step_name, percent, message）
pub type SdpProgressCallback = Box<dyn Fn(&str, f32, &str) + Send + Sync>;

/// SDP 流水线中间状态——精简为 4 步所需字段
///
/// 无 TTS/字幕相关字段（区别于 SdePipelineState）：
/// - 无 tts_results
/// - 无 merged_audio_path
/// - 无 merged_subtitle_path
pub struct SdpPipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    pub subtitle_text: String,
    pub subtitle_segments: Vec<crate::subtitle::types::SubtitleSegment>,
    pub script: crate::script::types::Script,
    pub video_clips: HashMap<i64, PathBuf>,
    pub combined_video_path: Option<PathBuf>,
    pub output_video_path: Option<PathBuf>,
    pub total_duration: f64,
    pub progress: Option<SdpProgressCallback>,
}

impl SdpPipelineState {
    /// 创建新的流水线状态
    pub fn new(task_id: String, task_dir: PathBuf) -> Self {
        Self {
            task_id,
            task_dir,
            subtitle_text: String::new(),
            subtitle_segments: Vec::new(),
            script: Vec::new(),
            video_clips: HashMap::new(),
            combined_video_path: None,
            output_video_path: None,
            total_duration: 0.0,
            progress: None,
        }
    }

    /// 触发进度回调（如果已注册）
    pub fn emit_progress(&self, step: SdpProgressStep, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_request_validate_valid() {
        let req = SdpRequest {
            subtitle_path: PathBuf::from("sub.srt"),
            video_path: PathBuf::from("video.mp4"),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_sdp_request_validate_empty_path() {
        let req = SdpRequest::default();
        let err = req.validate().unwrap_err();
        assert!(
            err.contains("subtitle_path") || err.contains("video_path"),
            "应提示路径为空: {}",
            err
        );
    }

    #[test]
    fn test_sdp_request_default_values() {
        let req = SdpRequest::default();
        assert!((req.temperature - 0.1).abs() < f64::EPSILON);
        assert_eq!(req.custom_clips, 5);
        assert!((req.original_volume - 0.7).abs() < f64::EPSILON);
        assert!((req.bgm_volume - 0.3).abs() < f64::EPSILON);
        assert_eq!(req.threads, 4);
        assert!(req.script_path.is_none());
        assert!(req.bgm_path.is_none());
        assert!(req.output_dir.is_none());
    }

    #[test]
    fn test_sdp_progress_step_order() {
        let first = SdpProgressStep::Clip;
        let last = SdpProgressStep::Composite;
        assert_eq!(first as u8, 0);
        assert_eq!(last as u8, 2);
    }

    #[test]
    fn test_sdp_pipeline_state_new_initializes_defaults() {
        let state = SdpPipelineState::new("test-123".into(), PathBuf::from("/tmp/sdp"));
        assert_eq!(state.task_id, "test-123");
        assert_eq!(state.task_dir, PathBuf::from("/tmp/sdp"));
        assert!(state.subtitle_text.is_empty());
        assert!(state.subtitle_segments.is_empty());
        assert!(state.script.is_empty());
        assert!(state.video_clips.is_empty());
        assert!(state.combined_video_path.is_none());
        assert!(state.output_video_path.is_none());
        assert!((state.total_duration - 0.0).abs() < f64::EPSILON);
        assert!(state.progress.is_none());
    }

    #[test]
    fn test_subtitle_segment_creation() {
        let seg = crate::subtitle::types::SubtitleSegment {
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
