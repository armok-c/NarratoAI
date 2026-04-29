use std::path::PathBuf;

use uuid::Uuid;

use super::error::JianYingError;
use super::segment::{AudioSegment, VideoSegment};
use super::template::{DRAFT_CONTENT_TEMPLATE, DRAFT_META_INFO_TEMPLATE};
use super::time::trange_from_secs;
use super::track::{Track, TrackType};
use super::types::{AudioMaterialJson, SpeedJson, VideoMaterialJson};
use crate::ffmpeg::probe::probe_audio;
use crate::script::types::{OstType, Script, ScriptClip};

// ---------------------------------------------------------------------------
// ExportRequest（per D-05, D-11, D-14）
// ---------------------------------------------------------------------------

/// 导出请求——包含导出所需的所有数据（per D-05）
pub struct ExportRequest {
    /// 处理后的脚本（所有 Option 字段应有值）
    pub script: Script,
    /// 原始视频文件路径（用于 source_timerange 回退）
    pub video_origin_path: PathBuf,
    /// 草稿保存路径（per D-14）
    pub draft_path: PathBuf,
    /// 草稿名称（per D-12，空则自动生成 NarratoAI_{timestamp}）
    pub draft_name: String,
    /// 输出视频宽度（per D-11，默认 1920）
    pub width: u32,
    /// 输出视频高度（per D-11，默认 1080）
    pub height: u32,
}

impl ExportRequest {
    /// 默认分辨率
    pub fn default_resolution() -> (u32, u32) {
        (1920, 1080)
    }

    /// 获取草稿名称——空则自动生成 NarratoAI_{timestamp}
    fn draft_name_or_default(&self) -> String {
        if self.draft_name.trim().is_empty() {
            format!(
                "NarratoAI_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            )
        } else {
            self.draft_name.clone()
        }
    }
}

// ---------------------------------------------------------------------------
// DraftFolder（per pyJianYingDraft draft_folder.py）
// ---------------------------------------------------------------------------

/// 草稿文件夹——管理草稿目录和文件路径
pub struct DraftFolder {
    draft_path: PathBuf,
    draft_name: String,
}

impl DraftFolder {
    /// 创建草稿文件夹 + 元信息文件（per RESEARCH Code Examples 草稿文件夹结构）
    pub fn create_draft(
        draft_path: &PathBuf,
        draft_name: &str,
        width: u32,
        height: u32,
    ) -> Result<(Self, ScriptFile), JianYingError> {
        let draft_dir = draft_path.join(draft_name);
        std::fs::create_dir_all(&draft_dir)?;

        // 写入 draft_meta_info.json（per RESEARCH Pitfall 7）
        let draft_id = Uuid::new_v4().to_string().replace("-", "");
        let meta_content = DRAFT_META_INFO_TEMPLATE
            .replace("DRAFT_ID_PLACEHOLDER", &draft_id)
            .replace("DRAFT_NAME_PLACEHOLDER", draft_name);
        std::fs::write(draft_dir.join("draft_meta_info.json"), meta_content)?;

        let script_file = ScriptFile::new(width, height, draft_id);

        Ok((
            Self {
                draft_path: draft_path.clone(),
                draft_name: draft_name.to_string(),
            },
            script_file,
        ))
    }

    /// 草稿目录完整路径
    pub fn draft_dir(&self) -> PathBuf {
        self.draft_path.join(&self.draft_name)
    }
}

// ---------------------------------------------------------------------------
// ScriptFile（per pyJianYingDraft script_file.py）
// ---------------------------------------------------------------------------

/// 草稿内容文件——管理轨道和素材，最终序列化为 draft_content.json
pub struct ScriptFile {
    width: u32,
    height: u32,
    tracks: Vec<Track>,
    draft_id: String,
    /// 素材集合——由 add_segment_to_track 时自动收集
    video_materials: Vec<VideoMaterialJson>,
    audio_materials: Vec<AudioMaterialJson>,
    speed_materials: Vec<SpeedJson>,
}

impl ScriptFile {
    /// 创建空的 ScriptFile
    pub fn new(width: u32, height: u32, draft_id: String) -> Self {
        Self {
            width,
            height,
            tracks: Vec::new(),
            draft_id,
            video_materials: Vec::new(),
            audio_materials: Vec::new(),
            speed_materials: Vec::new(),
        }
    }

    /// 添加轨道
    pub fn add_track(&mut self, track_type: TrackType, name: &str) {
        self.tracks.push(Track::new(track_type, name));
    }

    /// 添加视频 segment 到指定名称的轨道
    ///
    /// 如果轨道名称不存在，回滚已添加的素材并返回 Validation 错误。
    pub fn add_video_segment(&mut self, seg: VideoSegment, track_name: &str) -> Result<(), JianYingError> {
        // 收集素材
        self.video_materials.push(seg.material_json());
        self.speed_materials.push(seg.speed_json());

        // 添加到轨道
        if let Some(track) = self.tracks.iter_mut().find(|t| t.name() == track_name) {
            track.add_video_segment(seg);
            Ok(())
        } else {
            // 回滚素材——segment 未被添加，对应的素材也应移除
            self.video_materials.pop();
            self.speed_materials.pop();
            Err(JianYingError::Validation {
                details: format!("视频轨道 '{}' 不存在", track_name),
            })
        }
    }

    /// 添加音频 segment 到指定名称的轨道
    ///
    /// 如果轨道名称不存在，回滚已添加的素材并返回 Validation 错误。
    pub fn add_audio_segment(&mut self, seg: AudioSegment, track_name: &str) -> Result<(), JianYingError> {
        // 收集素材
        self.audio_materials.push(seg.material_json());
        self.speed_materials.push(seg.speed_json());

        // 添加到轨道
        if let Some(track) = self.tracks.iter_mut().find(|t| t.name() == track_name) {
            track.add_audio_segment(seg);
            Ok(())
        } else {
            // 回滚素材——segment 未被添加，对应的素材也应移除
            self.audio_materials.pop();
            self.speed_materials.pop();
            Err(JianYingError::Validation {
                details: format!("音频轨道 '{}' 不存在", track_name),
            })
        }
    }

    /// 保存 draft_content.json（per pyJianYingDraft ScriptFile.save/dumps）
    pub fn save(&self, folder: &DraftFolder) -> Result<PathBuf, JianYingError> {
        let draft_dir = folder.draft_dir();
        let content_path = draft_dir.join("draft_content.json");

        // 计算总时长——最后一个片段的 end 时间
        let total_duration_us = self.calculate_total_duration()?;

        // 从模板加载基础结构并替换
        let mut content: serde_json::Value = serde_json::from_str(DRAFT_CONTENT_TEMPLATE)?;

        // 替换 canvas_config
        content["canvas_config"]["width"] = serde_json::json!(self.width);
        content["canvas_config"]["height"] = serde_json::json!(self.height);

        // 替换 id
        content["id"] = serde_json::json!(self.draft_id);

        // 替换 duration
        content["duration"] = serde_json::json!(total_duration_us);

        // 填充素材
        content["materials"]["videos"] = serde_json::json!(self.video_materials);
        content["materials"]["audios"] = serde_json::json!(self.audio_materials);
        content["materials"]["speeds"] = serde_json::json!(self.speed_materials);

        // 构建 tracks JSON
        let tracks_json: Vec<serde_json::Value> = self
            .tracks
            .iter()
            .map(|t| {
                serde_json::to_value(t.to_json()?).map_err(JianYingError::JsonSerialize)
            })
            .collect::<Result<Vec<_>, _>>()?;
        content["tracks"] = serde_json::json!(tracks_json);

        // 写入文件
        let json_str = serde_json::to_string_pretty(&content)?;
        std::fs::write(&content_path, json_str)?;

        Ok(content_path)
    }

    /// 计算时间线总时长（微秒）——扫描所有轨道中所有 segment 的 target_timerange
    fn calculate_total_duration(&self) -> Result<i64, JianYingError> {
        let max_end = self
            .tracks
            .iter()
            .filter_map(|track| {
                let json = track.to_json().ok()?;
                json.segments
                    .iter()
                    .filter_map(|seg| {
                        // VideoSegmentJson 和 AudioSegmentJson 都通过 serde flatten
                        // 包含 target_timerange
                        let target = seg.get("target_timerange")?;
                        let start = target.get("start")?.as_i64()?;
                        let duration = target.get("duration")?.as_i64()?;
                        Some(start + duration)
                    })
                    .max()
            })
            .max();

        Ok(max_end.unwrap_or(60_000_000)) // 默认 60 秒
    }
}

// ---------------------------------------------------------------------------
// export_draft 导出主函数（per D-04, D-06, D-07, D-08, D-09）
// ---------------------------------------------------------------------------

/// 导出脚本为剪映草稿（per Python 版 jianying_task.py 逻辑）
pub fn export_draft(req: &ExportRequest) -> Result<PathBuf, JianYingError> {
    // 1. 校验 ExportRequest（per D-06）
    validate_export_request(req)?;

    let draft_name = req.draft_name_or_default();
    let (folder, mut script_file) =
        DraftFolder::create_draft(&req.draft_path, &draft_name, req.width, req.height)?;

    // 2. 添加双轨（per D-07）
    script_file.add_track(TrackType::Video, "视频轨道");
    script_file.add_track(TrackType::Audio, "音频轨道");

    // 3. 遍历 ScriptClip 构建时间线（per D-08, D-09）
    let mut current_time_secs: f64 = 0.0;

    for (i, clip) in req.script.iter().enumerate() {
        let duration = clip.duration.ok_or_else(|| JianYingError::MissingField {
            field: "duration".to_string(),
            clip_index: i,
        })?;
        let target = trange_from_secs(current_time_secs, duration);

        // 视频片段（per D-09 智能回退）
        if let Some(ref video_path) = clip.video {
            let video_seg =
                VideoSegment::new(video_path, target.clone(), req.width, req.height)?;
            script_file.add_video_segment(video_seg, "视频轨道")?;
        } else {
            let source_start = parse_source_start_time(clip, i)?;
            let source = trange_from_secs(source_start, duration);
            let video_seg = VideoSegment::with_source_timerange(
                &req.video_origin_path,
                target.clone(),
                source,
                req.width,
                req.height,
            )?;
            script_file.add_video_segment(video_seg, "视频轨道")?;
        }

        // 音频片段（per D-08 OST 映射）
        if clip.ost == OstType::NarrationOnly || clip.ost == OstType::Mixed {
            if let Some(ref audio_path) = clip.audio {
                let audio_duration = probe_audio(audio_path)
                    .map_err(|e| JianYingError::ProbeError(e.to_string()))?;
                let safe_duration = duration.min(audio_duration); // per D-10
                let audio_target = trange_from_secs(current_time_secs, safe_duration);
                let audio_seg = AudioSegment::new(audio_path, audio_target)?;
                script_file.add_audio_segment(audio_seg, "音频轨道")?;
            }
        }

        current_time_secs += duration;
    }

    // 4. 保存草稿
    let draft_path = script_file.save(&folder)?;
    Ok(draft_path)
}

// ---------------------------------------------------------------------------
// 内部辅助函数
// ---------------------------------------------------------------------------

/// 校验导出请求（per D-06）
fn validate_export_request(req: &ExportRequest) -> Result<(), JianYingError> {
    if req.script.is_empty() {
        return Err(JianYingError::Validation {
            details: "脚本不能为空".to_string(),
        });
    }
    if req.draft_path.as_os_str().is_empty() {
        return Err(JianYingError::Validation {
            details: "草稿保存路径不能为空".to_string(),
        });
    }
    if req.width == 0 || req.height == 0 {
        return Err(JianYingError::Validation {
            details: "分辨率宽高必须大于 0".to_string(),
        });
    }
    Ok(())
}

/// 解析 source_time_range 获取起始秒数（格式: "HH:MM:SS,mmm-HH:MM:SS,mmm"）
fn parse_source_start_time(clip: &ScriptClip, index: usize) -> Result<f64, JianYingError> {
    let range = clip.source_time_range.as_ref().ok_or_else(|| {
        JianYingError::MissingField {
            field: "source_time_range".to_string(),
            clip_index: index,
        }
    })?;
    parse_timestamp_start(range).ok_or_else(|| JianYingError::Validation {
        details: format!("第 {} 段 source_time_range 格式无效: {}", index + 1, range),
    })
}

/// 解析时间戳字符串的起始部分为秒数
/// 格式: "HH:MM:SS,mmm-HH:MM:SS,mmm" 或 "HH:MM:SS-HH:MM:SS"
///
/// 使用 `rsplit_once('-')` 从最后一个 `-` 分割，避免负号和逗号分隔符冲突。
/// 同时验证小时、分钟、秒、毫秒非负且在有效范围内。
fn parse_timestamp_start(range: &str) -> Option<f64> {
    // 从最后一个 '-' 分割，避免 "00:-05:00" 等畸形输入被误拆
    let (start_str, _) = range.rsplit_once('-')?;
    let sub_parts: Vec<&str> = start_str.split(',').collect();
    let time_str = sub_parts.first()?;
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }
    let h: f64 = time_parts[0].parse().ok()?;
    let m: f64 = time_parts[1].parse().ok()?;
    let s: f64 = time_parts[2].parse().ok()?;
    // 验证范围——不允许负值，分钟和秒不超过 60
    if h < 0.0 || m < 0.0 || s < 0.0 || m >= 60.0 || s >= 60.0 {
        return None;
    }
    let ms: f64 = if sub_parts.len() > 1 {
        let millis: f64 = sub_parts[1].parse().ok()?;
        if millis < 0.0 {
            return None;
        }
        millis / 1000.0
    } else {
        0.0
    };
    Some(h * 3600.0 + m * 60.0 + s + ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // --- ExportRequest 测试 ---

    /// Test: ExportRequest 的默认分辨率为 1920x1080
    #[test]
    fn test_export_request_default_resolution() {
        let (w, h) = ExportRequest::default_resolution();
        assert_eq!(w, 1920);
        assert_eq!(h, 1080);
    }

    /// Test: ExportRequest 校验——空脚本返回错误
    #[test]
    fn test_validate_empty_script() {
        let req = ExportRequest {
            script: vec![],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from("/tmp/drafts"),
            draft_name: "test".to_string(),
            width: 1920,
            height: 1080,
        };
        let result = validate_export_request(&req);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("脚本不能为空"),
            "应包含'脚本不能为空': {}",
            msg
        );
    }

    /// Test: ExportRequest 校验——空路径返回错误
    #[test]
    fn test_validate_empty_path() {
        let req = ExportRequest {
            script: vec![make_test_clip(0)],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from(""),
            draft_name: "test".to_string(),
            width: 1920,
            height: 1080,
        };
        let result = validate_export_request(&req);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("草稿保存路径不能为空"),
            "应包含路径错误: {}",
            msg
        );
    }

    /// Test: ExportRequest 校验——零分辨率返回错误
    #[test]
    fn test_validate_zero_resolution() {
        let req = ExportRequest {
            script: vec![make_test_clip(0)],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from("/tmp/drafts"),
            draft_name: "test".to_string(),
            width: 0,
            height: 0,
        };
        let result = validate_export_request(&req);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("分辨率宽高必须大于 0"),
            "应包含分辨率错误: {}",
            msg
        );
    }

    /// Test: ExportRequest 校验——有效请求通过
    #[test]
    fn test_validate_valid_request() {
        let req = ExportRequest {
            script: vec![make_test_clip(0)],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from("/tmp/drafts"),
            draft_name: "test".to_string(),
            width: 1920,
            height: 1080,
        };
        assert!(validate_export_request(&req).is_ok());
    }

    /// Test: draft_name_or_default 空名称自动生成 NarratoAI_{timestamp}
    #[test]
    fn test_draft_name_auto_generate() {
        let req = ExportRequest {
            script: vec![],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from("/tmp"),
            draft_name: String::new(),
            width: 1920,
            height: 1080,
        };
        let name = req.draft_name_or_default();
        assert!(name.starts_with("NarratoAI_"), "应以前缀开头: {}", name);
        // timestamp 部分应为纯数字
        let ts_part = name.trim_start_matches("NarratoAI_");
        assert!(
            ts_part.parse::<u64>().is_ok(),
            "时间戳部分应为数字: {}",
            ts_part
        );
    }

    /// Test: draft_name_or_default 非空名称保持原样
    #[test]
    fn test_draft_name_preserved() {
        let req = ExportRequest {
            script: vec![],
            video_origin_path: PathBuf::from("video.mp4"),
            draft_path: PathBuf::from("/tmp"),
            draft_name: "我的草稿".to_string(),
            width: 1920,
            height: 1080,
        };
        assert_eq!(req.draft_name_or_default(), "我的草稿");
    }

    // --- DraftFolder 测试 ---

    /// Test: DraftFolder::create_draft 创建草稿目录和两个 JSON 文件
    #[test]
    fn test_draft_folder_creates_files() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();

        let (folder, _script_file) =
            DraftFolder::create_draft(&draft_path, "TestDraft", 1920, 1080)
                .expect("应成功创建草稿");

        let draft_dir = folder.draft_dir();
        assert!(draft_dir.exists(), "草稿目录应存在");
        assert!(
            draft_dir.join("draft_meta_info.json").exists(),
            "draft_meta_info.json 应存在"
        );
    }

    /// Test: draft_content.json 包含 canvas_config (1920x1080), fps=30.0, version=360000
    #[test]
    fn test_script_file_save_content() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();

        let (folder, mut script_file) =
            DraftFolder::create_draft(&draft_path, "TestDraft", 1920, 1080)
                .expect("应成功创建草稿");

        // 添加轨道
        script_file.add_track(TrackType::Video, "视频轨道");
        script_file.add_track(TrackType::Audio, "音频轨道");

        // 保存
        let content_path = script_file.save(&folder).expect("应成功保存");

        // 读取并验证
        let content = std::fs::read_to_string(&content_path).expect("应能读取文件");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("应为有效 JSON");

        // canvas_config
        assert_eq!(json["canvas_config"]["width"], 1920, "width 应为 1920");
        assert_eq!(
            json["canvas_config"]["height"], 1080,
            "height 应为 1080"
        );

        // fps
        assert_eq!(json["fps"], 30.0, "fps 应为 30.0");

        // version
        assert_eq!(json["version"], 360000, "version 应为 360000");

        // tracks 应存在
        assert!(json["tracks"].is_array(), "tracks 应为数组");

        // materials 应存在
        assert!(
            json["materials"]["videos"].is_array(),
            "materials.videos 应为数组"
        );
        assert!(
            json["materials"]["audios"].is_array(),
            "materials.audios 应为数组"
        );
        assert!(
            json["materials"]["speeds"].is_array(),
            "materials.speeds 应为数组"
        );
    }

    /// Test: draft_meta_info.json 包含 draft_id（32 位 hex 格式）
    #[test]
    fn test_draft_meta_info_has_draft_id() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();

        let (folder, _) = DraftFolder::create_draft(&draft_path, "TestDraft", 1920, 1080)
            .expect("应成功创建草稿");

        let meta_path = folder.draft_dir().join("draft_meta_info.json");
        let content = std::fs::read_to_string(&meta_path).expect("应能读取 meta 文件");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("应为有效 JSON");

        // draft_id 应为非空字符串
        let draft_id = json["draft_id"].as_str().expect("draft_id 应为字符串");
        assert_eq!(draft_id.len(), 32, "draft_id 应为 32 位 hex");
        assert!(
            draft_id.chars().all(|c| c.is_ascii_hexdigit()),
            "draft_id 应为 hex 字符"
        );
    }

    // --- ScriptFile 测试 ---

    /// Test: ScriptFile::new 创建空 tracks 列表
    #[test]
    fn test_script_file_new_empty() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();
        let (folder, sf) = DraftFolder::create_draft(&draft_path, "TestEmpty", 1920, 1080)
            .expect("应成功创建");
        let content_path = sf.save(&folder).expect("应成功保存");

        let content = std::fs::read_to_string(&content_path).expect("应能读取");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("应为有效 JSON");
        let tracks = json["tracks"].as_array().expect("tracks 应为数组");
        assert!(tracks.is_empty(), "新 ScriptFile 应有空的 tracks");
    }

    /// Test: ScriptFile::add_track 添加视频轨道和音频轨道
    #[test]
    fn test_script_file_add_tracks() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();
        let (folder, mut sf) =
            DraftFolder::create_draft(&draft_path, "TestTracks", 1920, 1080)
                .expect("应成功创建");

        sf.add_track(TrackType::Video, "视频轨道");
        sf.add_track(TrackType::Audio, "音频轨道");

        let content_path = sf.save(&folder).expect("应成功保存");
        let content = std::fs::read_to_string(&content_path).expect("应能读取");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("应为有效 JSON");

        let tracks = json["tracks"].as_array().expect("tracks 应为数组");
        assert_eq!(tracks.len(), 2, "应有 2 个轨道");
        assert_eq!(tracks[0]["type"], "video", "第一个应为视频轨道");
        assert_eq!(tracks[1]["type"], "audio", "第二个应为音频轨道");
    }

    /// Test: ScriptFile::save 后 draft_content.json 可被 serde_json 解析
    #[test]
    fn test_script_file_save_parseable() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let draft_path = dir.path().to_path_buf();
        let (folder, mut sf) =
            DraftFolder::create_draft(&draft_path, "TestParse", 1280, 720)
                .expect("应成功创建");

        sf.add_track(TrackType::Video, "视频轨道");

        let content_path = sf.save(&folder).expect("应成功保存");
        let content = std::fs::read_to_string(&content_path).expect("应能读取");

        // 应能被 serde_json 解析
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("应为有效 JSON");
        assert!(json.is_object(), "应为 JSON 对象");

        // canvas_config 应反映传入的分辨率
        assert_eq!(json["canvas_config"]["width"], 1280);
        assert_eq!(json["canvas_config"]["height"], 720);
    }

    // --- parse_timestamp_start 测试 ---

    /// Test: parse_timestamp_start 正确解析 "00:00:07,559-00:00:15,000" 为 7.559 秒
    #[test]
    fn test_parse_timestamp_start_with_millis() {
        let result = parse_timestamp_start("00:00:07,559-00:00:15,000");
        assert!(result.is_some(), "应解析成功");
        let secs = result.unwrap();
        assert!(
            (secs - 7.559).abs() < 0.001,
            "应为 7.559 秒，实际: {}",
            secs
        );
    }

    /// Test: parse_timestamp_start 正确解析 "00:01:30,000-00:02:00,000" 为 90.0 秒
    #[test]
    fn test_parse_timestamp_start_minutes() {
        let result = parse_timestamp_start("00:01:30,000-00:02:00,000");
        assert!(result.is_some(), "应解析成功");
        let secs = result.unwrap();
        assert!(
            (secs - 90.0).abs() < 0.001,
            "应为 90.0 秒，实际: {}",
            secs
        );
    }

    /// Test: parse_timestamp_start 无效格式返回 None
    #[test]
    fn test_parse_timestamp_start_invalid() {
        assert!(parse_timestamp_start("invalid").is_none());
        assert!(parse_timestamp_start("").is_none());
        assert!(parse_timestamp_start("--").is_none());
    }

    // --- 辅助函数 ---

    /// 创建测试用 ScriptClip
    fn make_test_clip(ost: u8) -> ScriptClip {
        ScriptClip {
            _id: 1,
            timestamp: "00:00:00,600-00:00:07,559".to_string(),
            picture: "测试画面".to_string(),
            narration: "测试解说".to_string(),
            ost: match ost {
                0 => OstType::NarrationOnly,
                1 => OstType::OriginalSound,
                2 => OstType::Mixed,
                _ => OstType::NarrationOnly,
            },
            duration: Some(6.959),
            source_time_range: Some("00:00:00,600-00:00:07,559".to_string()),
            edited_time_range: None,
            audio: Some(PathBuf::from("test_audio.mp3")),
            video: Some(PathBuf::from("test_video.mp4")),
            subtitle: None,
        }
    }
}
