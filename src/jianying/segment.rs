use uuid::Uuid;

use super::error::JianYingError;
use super::material::{AudioMaterial, Speed, VideoMaterial};
use super::time::Timerange;
use super::types::{
    AudioSegmentJson, ClipTransform, HdrSettings, SegmentJson, UniformScale, VideoSegmentJson,
};

// ---------------------------------------------------------------------------
// SegmentOutput enum — 统一 VideoSegment/AudioSegment 的输出
// ---------------------------------------------------------------------------

/// Segment 输出枚举——让 Track 可以统一存储不同类型的 segment
pub enum SegmentOutput {
    Video(VideoSegmentJson),
    Audio(AudioSegmentJson),
}

// ---------------------------------------------------------------------------
// VideoSegment builder（per RESEARCH Pattern 5）
// ---------------------------------------------------------------------------

/// 视频片段构建器
pub struct VideoSegment {
    material: VideoMaterial,
    speed: Speed,
    target_timerange: Timerange,
    source_timerange: Option<Timerange>,
    segment_id: String,
}

impl VideoSegment {
    /// 创建视频片段——有处理后的视频文件（per D-09 有 video 时）
    pub fn new(
        path: &std::path::Path,
        target: Timerange,
        width: u32,
        height: u32,
    ) -> Result<Self, JianYingError> {
        Ok(Self {
            material: VideoMaterial::new(path, target.duration, width, height)?,
            speed: Speed::new(),
            target_timerange: target,
            source_timerange: None,
            segment_id: Uuid::new_v4().to_string().replace("-", ""),
        })
    }

    /// 创建视频片段——回退到原始视频路径（per D-09 无 video 时）
    pub fn with_source_timerange(
        path: &std::path::Path,
        target: Timerange,
        source: Timerange,
        width: u32,
        height: u32,
    ) -> Result<Self, JianYingError> {
        Ok(Self {
            material: VideoMaterial::new(path, target.duration, width, height)?,
            speed: Speed::new(),
            target_timerange: target,
            source_timerange: Some(source),
            segment_id: Uuid::new_v4().to_string().replace("-", ""),
        })
    }

    /// 构建 SegmentJson 基础字段
    fn build_base_segment(&self) -> SegmentJson {
        SegmentJson {
            enable_adjust: true,
            enable_color_correct_adjust: false,
            enable_color_curves: true,
            enable_color_match_adjust: false,
            enable_color_wheels: true,
            enable_lut: true,
            enable_smart_color_adjust: false,
            last_nonzero_volume: 1.0,
            reverse: false,
            track_attribute: 0,
            track_render_index: 0,
            visible: true,
            id: self.segment_id.clone(),
            material_id: self.material.material_id.clone(),
            target_timerange: self.target_timerange.clone(),
            common_keyframes: vec![],
            keyframe_refs: vec![],
            source_timerange: self.source_timerange.clone(),
            speed: 1.0,
            volume: 1.0,
            extra_material_refs: vec![self.speed.id.clone()],
            is_tone_modify: false,
        }
    }

    /// 导出为 VideoSegmentJson
    pub fn to_json(&self) -> VideoSegmentJson {
        VideoSegmentJson {
            base: self.build_base_segment(),
            clip: ClipTransform::default_video(),
            uniform_scale: UniformScale {
                on: true,
                value: 1.0,
            },
            hdr_settings: HdrSettings::default_video(),
        }
    }

    /// 获取素材 ID
    pub fn material_id(&self) -> &str {
        &self.material.material_id
    }

    /// 获取 Speed ID
    pub fn speed_id(&self) -> &str {
        &self.speed.id
    }

    /// 获取 VideoMaterial 的 JSON
    pub fn material_json(&self) -> super::types::VideoMaterialJson {
        self.material.to_json()
    }

    /// 获取 Speed 的 JSON
    pub fn speed_json(&self) -> super::types::SpeedJson {
        self.speed.to_json()
    }
}

// ---------------------------------------------------------------------------
// AudioSegment builder（per RESEARCH Pattern 6）
// ---------------------------------------------------------------------------

/// 音频片段构建器
pub struct AudioSegment {
    material: AudioMaterial,
    speed: Speed,
    target_timerange: Timerange,
    segment_id: String,
}

impl AudioSegment {
    /// 创建音频片段
    pub fn new(
        path: &std::path::Path,
        target: Timerange,
    ) -> Result<Self, JianYingError> {
        Ok(Self {
            material: AudioMaterial::new(path, target.duration)?,
            speed: Speed::new(),
            target_timerange: target,
            segment_id: Uuid::new_v4().to_string().replace("-", ""),
        })
    }

    /// 构建 SegmentJson 基础字段
    fn build_base_segment(&self) -> SegmentJson {
        SegmentJson {
            enable_adjust: true,
            enable_color_correct_adjust: false,
            enable_color_curves: true,
            enable_color_match_adjust: false,
            enable_color_wheels: true,
            enable_lut: true,
            enable_smart_color_adjust: false,
            last_nonzero_volume: 1.0,
            reverse: false,
            track_attribute: 0,
            track_render_index: 0,
            visible: true,
            id: self.segment_id.clone(),
            material_id: self.material.material_id.clone(),
            target_timerange: self.target_timerange.clone(),
            common_keyframes: vec![],
            keyframe_refs: vec![],
            source_timerange: None,
            speed: 1.0,
            volume: 1.0,
            extra_material_refs: vec![self.speed.id.clone()],
            is_tone_modify: false,
        }
    }

    /// 导出为 AudioSegmentJson
    pub fn to_json(&self) -> AudioSegmentJson {
        AudioSegmentJson {
            base: self.build_base_segment(),
            clip: None,
            hdr_settings: None,
        }
    }

    /// 获取素材 ID
    pub fn material_id(&self) -> &str {
        &self.material.material_id
    }

    /// 获取 Speed ID
    pub fn speed_id(&self) -> &str {
        &self.speed.id
    }

    /// 获取 AudioMaterial 的 JSON
    pub fn material_json(&self) -> super::types::AudioMaterialJson {
        self.material.to_json()
    }

    /// 获取 Speed 的 JSON
    pub fn speed_json(&self) -> super::types::SpeedJson {
        self.speed.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jianying::time::trange;
    use std::path::PathBuf;

    /// 创建临时视频文件用于测试
    fn make_video(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join(name);
        std::fs::write(&path, b"").expect("创建测试文件失败");
        (dir, path)
    }

    /// 创建临时音频文件用于测试
    fn make_audio(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join(name);
        std::fs::write(&path, b"").expect("创建测试文件失败");
        (dir, path)
    }

    /// Test 1: VideoSegment::new 生成的 JSON 包含 target_timerange={start:0,duration:5000000}
    #[test]
    fn test_video_segment_target_timerange() {
        let (_dir, path) = make_video("video.mp4");
        let target = trange("0s", "5s").expect("应解析时间范围");
        let seg = VideoSegment::new(&path, target, 1920, 1080).expect("应成功创建 VideoSegment");
        let json = seg.to_json();

        assert_eq!(json.base.target_timerange.start, 0, "start 应为 0");
        assert_eq!(
            json.base.target_timerange.duration, 5_000_000,
            "duration 应为 5000000"
        );
    }

    /// Test 2: VideoSegment JSON 包含 clip 字段（alpha=1.0, scale x/y=1.0）
    #[test]
    fn test_video_segment_clip_fields() {
        let (_dir, path) = make_video("video.mp4");
        let target = trange("0s", "5s").expect("应解析时间范围");
        let seg = VideoSegment::new(&path, target, 1920, 1080).expect("应成功创建");
        let json = seg.to_json();

        assert_eq!(json.clip.alpha, 1.0, "alpha 应为 1.0");
        assert_eq!(json.clip.scale.x, 1.0, "scale.x 应为 1.0");
        assert_eq!(json.clip.scale.y, 1.0, "scale.y 应为 1.0");
        assert!(!json.clip.flip.horizontal, "flip.horizontal 应为 false");
        assert!(!json.clip.flip.vertical, "flip.vertical 应为 false");
        assert_eq!(json.clip.rotation, 0.0, "rotation 应为 0.0");
    }

    /// Test 3: VideoSegment JSON 包含 hdr_settings（intensity=1.0, mode=1, nits=1000）
    #[test]
    fn test_video_segment_hdr_settings() {
        let (_dir, path) = make_video("video.mp4");
        let target = trange("0s", "5s").expect("应解析时间范围");
        let seg = VideoSegment::new(&path, target, 1920, 1080).expect("应成功创建");
        let json = seg.to_json();

        assert_eq!(json.hdr_settings.intensity, 1.0, "intensity 应为 1.0");
        assert_eq!(json.hdr_settings.mode, 1, "mode 应为 1");
        assert_eq!(json.hdr_settings.nits, 1000, "nits 应为 1000");
    }

    /// Test 4: VideoSegment::with_source_timerange 的 JSON 包含 source_timerange
    #[test]
    fn test_video_segment_with_source_timerange() {
        let (_dir, path) = make_video("original.mp4");
        let target = trange("0s", "5s").expect("应解析时间范围");
        let source = trange("10s", "5s").expect("应解析时间范围");
        let seg =
            VideoSegment::with_source_timerange(&path, target, source, 1920, 1080)
                .expect("应成功创建");
        let json = seg.to_json();

        // source_timerange 应被设置
        assert!(
            json.base.source_timerange.is_some(),
            "source_timerange 应为 Some"
        );
        let src_tr = json.base.source_timerange.unwrap();
        assert_eq!(src_tr.start, 10_000_000, "source start 应为 10000000");
        assert_eq!(
            src_tr.duration, 5_000_000,
            "source duration 应为 5000000"
        );
    }

    /// Test 5: AudioSegment::new 的 JSON 包含 clip=null, hdr_settings=null
    #[test]
    fn test_audio_segment_null_fields() {
        let (_dir, path) = make_audio("audio.mp3");
        let target = trange("0s", "3.5s").expect("应解析时间范围");
        let seg = AudioSegment::new(&path, target).expect("应成功创建 AudioSegment");
        let json = seg.to_json();

        assert_eq!(json.clip, None, "clip 应为 null");
        assert_eq!(json.hdr_settings, None, "hdr_settings 应为 null");
    }

    /// Test 6: AudioSegment JSON 的 volume=1.0, speed=1.0
    #[test]
    fn test_audio_segment_volume_speed() {
        let (_dir, path) = make_audio("audio.mp3");
        let target = trange("0s", "3.5s").expect("应解析时间范围");
        let seg = AudioSegment::new(&path, target).expect("应成功创建");
        let json = seg.to_json();

        assert_eq!(json.base.volume, 1.0, "volume 应为 1.0");
        assert_eq!(json.base.speed, 1.0, "speed 应为 1.0");
    }

    /// Test 7: 每个 segment 的 extra_material_refs 包含对应 Speed 的 ID
    #[test]
    fn test_segment_extra_material_refs_contains_speed_id() {
        // VideoSegment
        let (_vdir, vpath) = make_video("video.mp4");
        let vtarget = trange("0s", "5s").expect("应解析时间范围");
        let vseg = VideoSegment::new(&vpath, vtarget, 1920, 1080).expect("应成功创建");
        let vjson = vseg.to_json();
        let speed_id = vseg.speed_id();
        assert!(
            vjson.base.extra_material_refs.contains(&speed_id.to_string()),
            "VideoSegment extra_material_refs 应包含 Speed ID"
        );

        // AudioSegment
        let (_adir, apath) = make_audio("audio.mp3");
        let atarget = trange("0s", "3.5s").expect("应解析时间范围");
        let aseg = AudioSegment::new(&apath, atarget).expect("应成功创建");
        let ajson = aseg.to_json();
        let audio_speed_id = aseg.speed_id();
        assert!(
            ajson.base.extra_material_refs.contains(&audio_speed_id.to_string()),
            "AudioSegment extra_material_refs 应包含 Speed ID"
        );
    }
}
