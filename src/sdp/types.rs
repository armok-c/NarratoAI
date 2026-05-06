use std::collections::HashMap;
use std::path::PathBuf;

use crate::sde::types::SubtitleSegment;

/// SDP 流水线请求参数
#[derive(Debug, Clone)]
pub struct SdpRequest {
    pub subtitle_path: PathBuf,
    pub video_path: PathBuf,
    pub script_path: Option<PathBuf>,
    pub temperature: f64,
    pub custom_clips: u32,
    pub original_volume: f64,
    pub bgm_volume: f64,
    pub bgm_path: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
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
        if self.video_path.as_os_str().is_empty() {
            return Err("video_path 不能为空".to_string());
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
        Ok(())
    }
}

/// SDP 进度步骤（4 步流水线中的 3 个进度阶段）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdpProgressStep {
    Clip,
    Concat,
    Composite,
}

/// SDP 进度回调类型
pub type SdpProgressCallback = Box<dyn Fn(SdpProgressStep, f32, &str) + Send + Sync>;

/// SDP 流水线中间状态——贯穿 4 步
///
/// 与 SDE 相比，SDP 不需要 TTS 结果、字幕烧录、音频合并等相关字段。
pub struct SdpPipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    pub subtitle_text: String,
    pub subtitle_segments: Vec<SubtitleSegment>,
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
            video_path: PathBuf::from("video.mp4"),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_sdp_request_validate_empty_path() {
        let req = SdpRequest::default();
        let err = req.validate().unwrap_err();
        assert!(err.contains("video_path"), "应提示 video_path 为空: {}", err);
    }

    #[test]
    fn test_sdp_request_default_values() {
        let req = SdpRequest::default();
        assert_eq!(req.temperature, 0.1);
        assert_eq!(req.custom_clips, 5);
        assert_eq!(req.original_volume, 0.7);
        assert_eq!(req.bgm_volume, 0.3);
        assert_eq!(req.threads, 4);
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
        let state = SdpPipelineState::new("test-id".into(), PathBuf::from("/tmp/test"));
        assert_eq!(state.task_id, "test-id");
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
        let seg = SubtitleSegment {
            index: 1,
            start_secs: 0.0,
            end_secs: 5.5,
            text: "测试字幕".into(),
        };
        assert_eq!(seg.index, 1);
        assert!((seg.start_secs - 0.0).abs() < f64::EPSILON);
        assert!((seg.end_secs - 5.5).abs() < f64::EPSILON);
        assert_eq!(seg.text, "测试字幕");
    }

    #[test]
    fn test_sdp_request_validate_volume_ranges() {
        let req = SdpRequest {
            video_path: PathBuf::from("video.mp4"),
            original_volume: 15.0,
            ..Default::default()
        };
        assert!(req.validate().is_err(), "original_volume 超出范围应失败");

        let req2 = SdpRequest {
            video_path: PathBuf::from("video.mp4"),
            bgm_volume: -1.0,
            ..Default::default()
        };
        assert!(req2.validate().is_err(), "bgm_volume 超出范围应失败");
    }

    #[test]
    fn test_sdp_pipeline_state_emit_progress() {
        use std::sync::{Arc, Mutex};

        let state = SdpPipelineState::new("test-id".into(), PathBuf::from("/tmp/test"));
        // 无回调用时，emit_progress 不应崩溃
        state.emit_progress(SdpProgressStep::Clip, 0.0, "测试");

        // 注册回调后应能正常触发
        let last_step = Arc::new(Mutex::new(None::<SdpProgressStep>));
        let last_step_clone = last_step.clone();
        let cb: SdpProgressCallback = Box::new(move |step, _, _| {
            *last_step_clone.lock().unwrap() = Some(step);
        });

        // 创建带回调的新状态
        let mut state2 = SdpPipelineState::new("test-id-2".into(), PathBuf::from("/tmp/test2"));
        state2.progress = Some(cb);
        state2.emit_progress(SdpProgressStep::Concat, 50.0, "拼接中");

        let observed = *last_step.lock().unwrap();
        assert_eq!(observed, Some(SdpProgressStep::Concat));
    }
}
