use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::time::Timerange;

// ---------------------------------------------------------------------------
// Clip 变换相关结构体（per RESEARCH Pattern 5: VideoSegment.clip）
// ---------------------------------------------------------------------------

/// 翻转状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlipState {
    pub horizontal: bool,
    pub vertical: bool,
}

/// 缩放值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleValue {
    pub x: f64,
    pub y: f64,
}

/// 位移值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformValue {
    pub x: f64,
    pub y: f64,
}

/// 统一缩放
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniformScale {
    pub on: bool,
    pub value: f64,
}

/// VideoSegment 的 clip 字段（per RESEARCH Pattern 5）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipTransform {
    pub alpha: f64,
    pub flip: FlipState,
    pub rotation: f64,
    pub scale: ScaleValue,
    pub transform: TransformValue,
}

impl ClipTransform {
    /// VideoSegment 默认 clip 值
    pub fn default_video() -> Self {
        Self {
            alpha: 1.0,
            flip: FlipState {
                horizontal: false,
                vertical: false,
            },
            rotation: 0.0,
            scale: ScaleValue { x: 1.0, y: 1.0 },
            transform: TransformValue { x: 0.0, y: 0.0 },
        }
    }
}

// ---------------------------------------------------------------------------
// HdrSettings（per RESEARCH Pattern 5: VideoSegment.hdr_settings）
// ---------------------------------------------------------------------------

/// HDR 设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HdrSettings {
    pub intensity: f64,
    pub mode: u32,
    pub nits: u32,
}

impl HdrSettings {
    /// VideoSegment 默认 HDR 值
    pub fn default_video() -> Self {
        Self {
            intensity: 1.0,
            mode: 1,
            nits: 1000,
        }
    }
}

// ---------------------------------------------------------------------------
// CropSettings（per RESEARCH Pattern 2: VideoMaterial.crop）
// ---------------------------------------------------------------------------

/// 素材裁剪设置——4 个角的归一化坐标
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CropSettings {
    pub upper_left_x: f64,
    pub upper_left_y: f64,
    pub upper_right_x: f64,
    pub upper_right_y: f64,
    pub lower_left_x: f64,
    pub lower_left_y: f64,
    pub lower_right_x: f64,
    pub lower_right_y: f64,
}

impl CropSettings {
    /// 默认无裁剪
    pub fn default_no_crop() -> Self {
        Self {
            upper_left_x: 0.0,
            upper_left_y: 0.0,
            upper_right_x: 1.0,
            upper_right_y: 0.0,
            lower_left_x: 0.0,
            lower_left_y: 1.0,
            lower_right_x: 1.0,
            lower_right_y: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// SegmentJson（per RESEARCH Pattern 4: BaseSegment + MediaSegment 合并）
// ---------------------------------------------------------------------------

/// 合并 BaseSegment + MediaSegment 字段的 segment JSON（per RESEARCH Pattern 4）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SegmentJson {
    // BaseSegment 字段
    pub enable_adjust: bool,
    pub enable_color_correct_adjust: bool,
    pub enable_color_curves: bool,
    pub enable_color_match_adjust: bool,
    pub enable_color_wheels: bool,
    pub enable_lut: bool,
    pub enable_smart_color_adjust: bool,
    pub last_nonzero_volume: f64,
    pub reverse: bool,
    pub track_attribute: u32,
    pub track_render_index: u32,
    pub visible: bool,
    pub id: String,
    pub material_id: String,
    pub target_timerange: Timerange,
    pub common_keyframes: Vec<Value>,
    pub keyframe_refs: Vec<Value>,
    // MediaSegment 字段
    pub source_timerange: Option<Timerange>,
    pub speed: f64,
    pub volume: f64,
    pub extra_material_refs: Vec<String>,
    pub is_tone_modify: bool,
}

// ---------------------------------------------------------------------------
// VideoSegmentJson（per RESEARCH Pattern 5: VisualSegment + hdr_settings）
// ---------------------------------------------------------------------------

/// VideoSegment JSON = SegmentJson + clip + uniform_scale + hdr_settings
///
/// 注意: `#[serde(flatten)]` 将 SegmentJson 的所有字段展开到同一 JSON 对象层级。
/// 当前仅用于序列化（导出），不用于反序列化，因此不存在冲突风险。
/// 如果未来需要反序列化（如导入剪映草稿），需注意 `#[serde(flatten)]` 在遇到
/// 未知字段时可能导致字段名冲突或静默丢弃数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VideoSegmentJson {
    #[serde(flatten)]
    pub base: SegmentJson,
    pub clip: ClipTransform,
    pub uniform_scale: UniformScale,
    pub hdr_settings: HdrSettings,
}

// ---------------------------------------------------------------------------
// AudioSegmentJson（per RESEARCH Pattern 6: clip=null, hdr_settings=null）
// ---------------------------------------------------------------------------

/// AudioSegment JSON = SegmentJson + clip=null + hdr_settings=null
///
/// 注意: 与 VideoSegmentJson 相同的 `#[serde(flatten)]` 设计约束。
/// 仅用于序列化（导出），不应尝试反序列化包含未知字段的 JSON。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AudioSegmentJson {
    #[serde(flatten)]
    pub base: SegmentJson,
    pub clip: Option<()>,
    pub hdr_settings: Option<()>,
}

// ---------------------------------------------------------------------------
// VideoMaterialJson（per RESEARCH Pattern 2）
// ---------------------------------------------------------------------------

/// 视频素材 JSON（per pyJianYingDraft VideoMaterial.export_json()）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMaterialJson {
    pub audio_fade: Option<()>,
    pub category_id: String,
    pub category_name: String,
    pub check_flag: u32,
    pub crop: CropSettings,
    pub crop_ratio: String,
    pub crop_scale: f64,
    pub duration: i64,
    pub height: u32,
    pub id: String,
    pub local_material_id: String,
    pub material_id: String,
    pub material_name: String,
    pub media_path: String,
    pub path: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub width: u32,
}

// ---------------------------------------------------------------------------
// AudioMaterialJson（per RESEARCH Pattern 2）
// ---------------------------------------------------------------------------

/// 音频素材 JSON（per pyJianYingDraft AudioMaterial.export_json()）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMaterialJson {
    pub app_id: u32,
    pub category_id: String,
    pub category_name: String,
    pub check_flag: u32,
    pub copyright_limit_type: String,
    pub duration: i64,
    pub effect_id: String,
    pub formula_id: String,
    pub id: String,
    pub local_material_id: String,
    pub music_id: String,
    pub name: String,
    pub path: String,
    pub source_platform: u32,
    #[serde(rename = "type")]
    pub type_field: String,
    pub wave_points: Vec<Value>,
}

// ---------------------------------------------------------------------------
// SpeedJson（per RESEARCH Pattern 7）
// ---------------------------------------------------------------------------

/// Speed 素材（per pyJianYingDraft Speed.export_json()）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeedJson {
    pub curve_speed: Option<()>,
    pub id: String,
    pub mode: u32,
    pub speed: f64,
    #[serde(rename = "type")]
    pub type_field: String,
}

// ---------------------------------------------------------------------------
// TrackJson（per RESEARCH Pattern 3）
// ---------------------------------------------------------------------------

/// 轨道 JSON 输出（per pyJianYingDraft Track.export_json()）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackJson {
    pub attribute: u32,
    pub flag: u32,
    pub id: String,
    pub is_default_name: bool,
    pub name: String,
    pub segments: Vec<Value>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: SegmentJson 序列化包含所有 BaseSegment 字段
    #[test]
    fn test_segment_json_has_base_fields() {
        let seg = SegmentJson {
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
            id: "seg-id".to_string(),
            material_id: "mat-id".to_string(),
            target_timerange: Timerange {
                start: 0,
                duration: 5_000_000,
            },
            common_keyframes: vec![],
            keyframe_refs: vec![],
            source_timerange: None,
            speed: 1.0,
            volume: 1.0,
            extra_material_refs: vec![],
            is_tone_modify: false,
        };
        let json = serde_json::to_string(&seg).expect("应序列化成功");
        // 验证关键 BaseSegment 字段
        assert!(json.contains("enable_adjust"), "应包含 enable_adjust");
        assert!(json.contains("visible"), "应包含 visible");
        assert!(json.contains("\"id\":\"seg-id\""), "应包含 id");
        assert!(json.contains("\"material_id\":\"mat-id\""), "应包含 material_id");
        assert!(json.contains("\"target_timerange\""), "应包含 target_timerange");
        // 验证 MediaSegment 字段
        assert!(json.contains("\"source_timerange\":null"), "应包含 source_timerange: null");
        assert!(json.contains("\"speed\":1.0"), "应包含 speed: 1.0");
        assert!(json.contains("\"volume\":1.0"), "应包含 volume: 1.0");
    }

    /// Test 2: SegmentJson 的 extra_material_refs 包含 Speed UUID
    #[test]
    fn test_segment_json_extra_material_refs() {
        let seg = SegmentJson {
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
            id: "seg-id".to_string(),
            material_id: "mat-id".to_string(),
            target_timerange: Timerange {
                start: 0,
                duration: 5_000_000,
            },
            common_keyframes: vec![],
            keyframe_refs: vec![],
            source_timerange: None,
            speed: 1.0,
            volume: 1.0,
            extra_material_refs: vec!["speed-uuid-123".to_string()],
            is_tone_modify: false,
        };
        let json = serde_json::to_string(&seg).expect("应序列化成功");
        assert!(
            json.contains("\"extra_material_refs\":[\"speed-uuid-123\"]"),
            "extra_material_refs 应包含 Speed UUID: {}",
            json
        );
    }

    /// Test 3: ClipTransform 序列化
    #[test]
    fn test_clip_transform_serialization() {
        let clip = ClipTransform::default_video();
        let json = serde_json::to_string(&clip).expect("应序列化成功");
        let expected = r#"{"alpha":1.0,"flip":{"horizontal":false,"vertical":false},"rotation":0.0,"scale":{"x":1.0,"y":1.0},"transform":{"x":0.0,"y":0.0}}"#;
        assert_eq!(json, expected, "ClipTransform 序列化应精确匹配");
    }

    /// Test 4: HdrSettings 序列化
    #[test]
    fn test_hdr_settings_serialization() {
        let hdr = HdrSettings::default_video();
        let json = serde_json::to_string(&hdr).expect("应序列化成功");
        assert_eq!(json, r#"{"intensity":1.0,"mode":1,"nits":1000}"#);
    }

    /// Test 5: VideoMaterialJson 包含所有字段
    #[test]
    fn test_video_material_json_all_fields() {
        let mat = VideoMaterialJson {
            audio_fade: None,
            category_id: "".to_string(),
            category_name: "local".to_string(),
            check_flag: 63487,
            crop: CropSettings::default_no_crop(),
            crop_ratio: "free".to_string(),
            crop_scale: 1.0,
            duration: 5_000_000,
            height: 1080,
            id: "vid-uuid".to_string(),
            local_material_id: "".to_string(),
            material_id: "vid-uuid".to_string(),
            material_name: "video.mp4".to_string(),
            media_path: "".to_string(),
            path: r"C:\full\path\to\video.mp4".to_string(),
            type_field: "video".to_string(),
            width: 1920,
        };
        let json = serde_json::to_string(&mat).expect("应序列化成功");
        assert!(json.contains("\"path\":"), "应包含 path");
        assert!(json.contains("\"width\":1920"), "应包含 width");
        assert!(json.contains("\"height\":1080"), "应包含 height");
        assert!(json.contains("\"duration\":5000000"), "应包含 duration");
        assert!(json.contains("\"type\":\"video\""), "应包含 type: video");
        assert!(json.contains("\"crop\":"), "应包含 crop");
        assert!(json.contains("\"check_flag\":63487"), "应包含 check_flag");
    }

    /// Test 6: AudioMaterialJson 包含所有字段
    #[test]
    fn test_audio_material_json_all_fields() {
        let mat = AudioMaterialJson {
            app_id: 0,
            category_id: "".to_string(),
            category_name: "local".to_string(),
            check_flag: 3,
            copyright_limit_type: "none".to_string(),
            duration: 3_500_000,
            effect_id: "".to_string(),
            formula_id: "".to_string(),
            id: "aud-uuid".to_string(),
            local_material_id: "aud-uuid".to_string(),
            music_id: "aud-uuid".to_string(),
            name: "audio.mp3".to_string(),
            path: r"C:\full\path\to\audio.mp3".to_string(),
            source_platform: 0,
            type_field: "extract_music".to_string(),
            wave_points: vec![],
        };
        let json = serde_json::to_string(&mat).expect("应序列化成功");
        assert!(json.contains("\"name\":\"audio.mp3\""), "应包含 name");
        assert!(json.contains("\"duration\":3500000"), "应包含 duration");
        assert!(json.contains("\"type\":\"extract_music\""), "应包含 type");
        assert!(json.contains("\"wave_points\":[]"), "应包含 wave_points");
        assert!(json.contains("\"check_flag\":3"), "应包含 check_flag");
    }

    /// Test 7: SpeedJson 序列化
    #[test]
    fn test_speed_json_serialization() {
        let speed = SpeedJson {
            curve_speed: None,
            id: "speed-uuid-abc".to_string(),
            mode: 0,
            speed: 1.0,
            type_field: "speed".to_string(),
        };
        let json = serde_json::to_string(&speed).expect("应序列化成功");
        assert!(json.contains("\"curve_speed\":null"), "curve_speed 应为 null");
        assert!(json.contains("\"id\":\"speed-uuid-abc\""), "应包含 id");
        assert!(json.contains("\"mode\":0"), "应包含 mode: 0");
        assert!(json.contains("\"speed\":1.0"), "应包含 speed: 1.0");
        assert!(json.contains("\"type\":\"speed\""), "应包含 type: speed");
    }

    /// Test 8: TrackJson 序列化
    #[test]
    fn test_track_json_serialization() {
        let track = TrackJson {
            attribute: 0,
            flag: 0,
            id: "track-uuid".to_string(),
            is_default_name: false,
            name: "视频轨道".to_string(),
            segments: vec![],
            type_field: "video".to_string(),
        };
        let json = serde_json::to_string(&track).expect("应序列化成功");
        assert!(json.contains("\"attribute\":0"), "应包含 attribute");
        assert!(json.contains("\"id\":\"track-uuid\""), "应包含 id");
        assert!(json.contains("\"name\":\"视频轨道\""), "应包含 name");
        assert!(json.contains("\"segments\":[]"), "应包含 segments");
        assert!(json.contains("\"type\":\"video\""), "应包含 type: video");
    }
}
