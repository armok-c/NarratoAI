use std::path::Path;

use uuid::Uuid;

use super::error::JianYingError;
use super::types::{AudioMaterialJson, CropSettings, SpeedJson, VideoMaterialJson};

// ---------------------------------------------------------------------------
// Speed builder（per RESEARCH Pattern 7: Speed 素材）
// ---------------------------------------------------------------------------

/// Speed 素材构建器——每个 segment 必须关联一个 Speed 对象
pub struct Speed {
    pub id: String,
}

impl Speed {
    /// 创建默认 Speed（speed=1.0, curve_speed=null）
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string().replace("-", ""),
        }
    }

    /// 导出为 SpeedJson
    pub fn to_json(&self) -> SpeedJson {
        SpeedJson {
            curve_speed: None,
            id: self.id.clone(),
            mode: 0,
            speed: 1.0,
            type_field: "speed".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// VideoMaterial builder（per RESEARCH Pattern 2）
// ---------------------------------------------------------------------------

/// 视频素材构建器——保存序列化所需状态
pub struct VideoMaterial {
    pub material_id: String,
    path: String,
    duration_us: i64,
    width: u32,
    height: u32,
    file_name: String,
}

impl VideoMaterial {
    /// 从视频文件路径构建 VideoMaterial
    ///
    /// - `path`: 视频文件路径（自动转为绝对路径）
    /// - `duration_us`: 时长（微秒）
    /// - `width`: 视频宽度
    /// - `height`: 视频高度
    pub fn new(path: &Path, duration_us: i64, width: u32, height: u32) -> Result<Self, JianYingError> {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let id = Uuid::new_v4().to_string().replace("-", "");
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(Self {
            material_id: id,
            path: abs_path.to_string_lossy().to_string(),
            duration_us,
            width,
            height,
            file_name,
        })
    }

    /// 导出为 VideoMaterialJson
    pub fn to_json(&self) -> VideoMaterialJson {
        VideoMaterialJson {
            audio_fade: None,
            category_id: String::new(),
            category_name: "local".to_string(),
            check_flag: 63487,
            crop: CropSettings::default_no_crop(),
            crop_ratio: "free".to_string(),
            crop_scale: 1.0,
            duration: self.duration_us,
            height: self.height,
            id: self.material_id.clone(),
            local_material_id: String::new(),
            material_id: self.material_id.clone(),
            material_name: self.file_name.clone(),
            media_path: String::new(),
            path: self.path.clone(),
            type_field: "video".to_string(),
            width: self.width,
        }
    }
}

// ---------------------------------------------------------------------------
// AudioMaterial builder（per RESEARCH Pattern 2）
// ---------------------------------------------------------------------------

/// 音频素材构建器——保存序列化所需状态
pub struct AudioMaterial {
    pub material_id: String,
    path: String,
    duration_us: i64,
    file_name: String,
}

impl AudioMaterial {
    /// 从音频文件路径构建 AudioMaterial
    ///
    /// - `path`: 音频文件路径（自动转为绝对路径）
    /// - `duration_us`: 时长（微秒）
    pub fn new(path: &Path, duration_us: i64) -> Result<Self, JianYingError> {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let id = Uuid::new_v4().to_string().replace("-", "");
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(Self {
            material_id: id,
            path: abs_path.to_string_lossy().to_string(),
            duration_us,
            file_name,
        })
    }

    /// 导出为 AudioMaterialJson
    pub fn to_json(&self) -> AudioMaterialJson {
        AudioMaterialJson {
            app_id: 0,
            category_id: String::new(),
            category_name: "local".to_string(),
            check_flag: 3,
            copyright_limit_type: "none".to_string(),
            duration: self.duration_us,
            effect_id: String::new(),
            formula_id: String::new(),
            id: self.material_id.clone(),
            local_material_id: self.material_id.clone(),
            music_id: self.material_id.clone(),
            name: self.file_name.clone(),
            path: self.path.clone(),
            source_platform: 0,
            type_field: "extract_music".to_string(),
            wave_points: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Test 1: VideoMaterial::new 生成 VideoMaterialJson，id 为 32 位 hex，path 为绝对路径
    #[test]
    fn test_video_material_new_generates_valid_json() {
        let path = PathBuf::from("test_video.mp4");
        let mat = VideoMaterial::new(&path, 5_000_000, 1920, 1080).expect("应成功创建 VideoMaterial");
        let json = mat.to_json();

        // id 应为 32 位 hex
        assert_eq!(json.id.len(), 32, "id 应为 32 位 hex");
        assert!(json.id.chars().all(|c| c.is_ascii_hexdigit()), "id 应为 hex 字符");

        // material_id 应与 id 相同
        assert_eq!(json.material_id, json.id, "material_id 应等于 id");

        // duration 应正确
        assert_eq!(json.duration, 5_000_000, "duration 应为 5000000 微秒");

        // width/height 应正确
        assert_eq!(json.width, 1920);
        assert_eq!(json.height, 1080);
    }

    /// Test 2: VideoMaterialJson 的 crop 字段为默认全画面裁剪
    #[test]
    fn test_video_material_crop_defaults() {
        let path = PathBuf::from("test.mp4");
        let mat = VideoMaterial::new(&path, 5_000_000, 1920, 1080).expect("应成功创建");
        let json = mat.to_json();

        let expected_crop = CropSettings::default_no_crop();
        assert_eq!(json.crop, expected_crop, "crop 应为默认无裁剪");
    }

    /// Test 3: VideoMaterialJson 的 check_flag, type, category_name
    #[test]
    fn test_video_material_fixed_fields() {
        let path = PathBuf::from("test.mp4");
        let mat = VideoMaterial::new(&path, 5_000_000, 1920, 1080).expect("应成功创建");
        let json = mat.to_json();

        assert_eq!(json.check_flag, 63487, "check_flag 应为 63487");
        assert_eq!(json.type_field, "video", "type 应为 video");
        assert_eq!(json.category_name, "local", "category_name 应为 local");
    }

    /// Test 4: AudioMaterial::new 生成 AudioMaterialJson，type = "extract_music"
    #[test]
    fn test_audio_material_new_generates_valid_json() {
        let path = PathBuf::from("test_audio.mp3");
        let mat = AudioMaterial::new(&path, 3_500_000).expect("应成功创建 AudioMaterial");
        let json = mat.to_json();

        // id 应为 32 位 hex
        assert_eq!(json.id.len(), 32, "id 应为 32 位 hex");
        assert!(json.id.chars().all(|c| c.is_ascii_hexdigit()), "id 应为 hex 字符");

        // type 应为 extract_music
        assert_eq!(json.type_field, "extract_music", "type 应为 extract_music");

        // duration 应正确
        assert_eq!(json.duration, 3_500_000, "duration 应为 3500000 微秒");
    }

    /// Test 5: AudioMaterialJson 的 check_flag = 3, wave_points = []
    #[test]
    fn test_audio_material_fixed_fields() {
        let path = PathBuf::from("test.mp3");
        let mat = AudioMaterial::new(&path, 3_500_000).expect("应成功创建");
        let json = mat.to_json();

        assert_eq!(json.check_flag, 3, "check_flag 应为 3");
        assert!(json.wave_points.is_empty(), "wave_points 应为空数组");
    }

    /// Test 6: Speed::new 生成 SpeedJson，curve_speed=null, speed=1.0, type="speed", mode=0
    #[test]
    fn test_speed_new_generates_valid_json() {
        let speed = Speed::new();
        let json = speed.to_json();

        assert_eq!(json.curve_speed, None, "curve_speed 应为 null");
        assert_eq!(json.speed, 1.0, "speed 应为 1.0");
        assert_eq!(json.type_field, "speed", "type 应为 speed");
        assert_eq!(json.mode, 0, "mode 应为 0");
        assert_eq!(json.id.len(), 32, "id 应为 32 位 hex");
    }

    /// Test 7: VideoMaterial 路径自动转为绝对路径（canonicalize fallback）
    #[test]
    fn test_video_material_path_canonicalize_fallback() {
        // 使用不存在的路径——canonicalize 会失败，应 fallback 到原始路径
        let path = PathBuf::from("nonexistent_video.mp4");
        let mat = VideoMaterial::new(&path, 5_000_000, 1920, 1080).expect("应成功创建");
        let json = mat.to_json();

        // 由于文件不存在，canonicalize 失败，应 fallback 到原始路径
        // 但路径应包含文件名
        assert!(
            json.path.contains("nonexistent_video.mp4"),
            "路径应包含文件名: {}",
            json.path
        );
    }
}
