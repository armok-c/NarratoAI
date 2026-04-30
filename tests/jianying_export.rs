//! 剪映草稿导出集成测试
//!
//! 端到端验证 export_draft 生成的剪映草稿 JSON 格式正确性。
//! 覆盖 JYNG-01（JSON 格式）和 JYNG-02（时间线轨道映射）两个需求。
//!
//! 测试分类：
//! - 不需要 ffmpeg 的测试：直接运行
//! - 需要 ffmpeg 的测试：标记 #[ignore]

use std::path::PathBuf;

use narratoai_core::jianying::builder::{export_draft, ExportRequest};
use narratoai_core::jianying::error::JianYingError;
use narratoai_core::script::types::{OstType, ScriptClip};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 测试辅助函数
// ---------------------------------------------------------------------------

/// 创建测试用 ScriptClip
///
/// video 和 audio 参数为文件名（如 "clip1.mp4"），会在 base_dir 下创建空文件
/// 并使用绝对路径。
fn make_clip(
    id: i64,
    ost: OstType,
    duration: f64,
    video: Option<&str>,
    audio: Option<&str>,
    source_time_range: Option<&str>,
    base_dir: &std::path::Path,
) -> ScriptClip {
    let video_path = video.map(|name| {
        let path = base_dir.join(name);
        std::fs::write(&path, b"").expect("创建测试视频文件失败");
        path
    });
    let audio_path = audio.map(|name| {
        let path = base_dir.join(name);
        std::fs::write(&path, b"").expect("创建测试音频文件失败");
        path
    });
    ScriptClip {
        _id: id,
        timestamp: format!(
            "00:00:{:02},000-00:00:{:02},000",
            id * 5,
            id * 5 + 5
        ),
        picture: format!("测试画面{}", id),
        narration: format!("测试解说{}", id),
        ost,
        duration: Some(duration),
        source_time_range: source_time_range.map(|s| s.to_string()),
        edited_time_range: None,
        audio: audio_path,
        video: video_path,
        subtitle: None,
    }
}

/// 读取 draft_content.json 并解析为 serde_json::Value
fn load_draft_content(draft_path: &std::path::Path) -> serde_json::Value {
    let content_path = draft_path.join("draft_content.json");
    let content = std::fs::read_to_string(&content_path)
        .unwrap_or_else(|e| panic!("应能读取 draft_content.json: {}", e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("draft_content.json 应为有效 JSON: {}", e))
}

/// 读取 draft_meta_info.json 并解析为 serde_json::Value
fn load_draft_meta(draft_path: &std::path::Path) -> serde_json::Value {
    let meta_path = draft_path.join("draft_meta_info.json");
    let content = std::fs::read_to_string(&meta_path)
        .unwrap_or_else(|e| panic!("应能读取 draft_meta_info.json: {}", e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("draft_meta_info.json 应为有效 JSON: {}", e))
}

/// 创建 OST=1（OriginalSound）的导出请求并执行导出
///
/// OST=1 不需要 probe_audio，因此不需要实际音频文件
fn export_ost1_clips(clips: Vec<ScriptClip>, dir: &TempDir) -> (PathBuf, serde_json::Value) {
    // 创建原始视频文件（绝对路径）
    let video_origin = dir.path().join("original_video.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: clips,
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestDraft".to_string(),
        width: 1920,
        height: 1080,
    };
    let content_path = export_draft(&req).expect("OST=1 导出应成功");
    let draft_dir = content_path.parent().expect("draft_content.json 应有父目录").to_path_buf();
    let json = load_draft_content(&draft_dir);
    (draft_dir, json)
}

// ---------------------------------------------------------------------------
// Test 1: OST=0（NarrationOnly）生成视频段 + 音频段
// ---------------------------------------------------------------------------

/// OST=0 片段应生成视频段和音频段。
/// 需要 probe_audio 获取音频时长——标记 #[ignore]（需要实际音频文件）。
#[test]
#[ignore = "需要实际音频文件和 ffmpeg 来运行 probe_audio"]
fn test_ost_narration_only_timeline() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(
            1,
            OstType::NarrationOnly,
            5.0,
            Some("clip1.mp4"),
            Some("audio1.mp3"),
            None,
            dir.path(),
        ),
        make_clip(
            2,
            OstType::NarrationOnly,
            4.0,
            Some("clip2.mp4"),
            Some("audio2.mp3"),
            None,
            dir.path(),
        ),
        make_clip(
            3,
            OstType::NarrationOnly,
            3.0,
            Some("clip3.mp4"),
            Some("audio3.mp3"),
            None,
            dir.path(),
        ),
    ];

    let video_origin = dir.path().join("original.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: clips,
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestOST0".to_string(),
        width: 1920,
        height: 1080,
    };

    let draft_dir = export_draft(&req).expect("导出应成功");
    let json = load_draft_content(draft_dir.parent().expect("导出路径应有父目录"));

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");
    assert_eq!(tracks.len(), 2, "应有 2 个轨道（video + audio）");

    // 视频轨道应有 3 个 segments
    let video_track = &tracks[0];
    assert_eq!(video_track["type"], "video");
    let video_segs = video_track["segments"].as_array().expect("video segments 应为数组");
    assert_eq!(video_segs.len(), 3, "视频轨道应有 3 个 segments");

    // 音频轨道应有 3 个 segments
    let audio_track = &tracks[1];
    assert_eq!(audio_track["type"], "audio");
    let audio_segs = audio_track["segments"].as_array().expect("audio segments 应为数组");
    assert_eq!(audio_segs.len(), 3, "音频轨道应有 3 个 segments");

    // 验证每个音频 segment 的 target_timerange.duration <= 对应视频 duration
    for (i, audio_seg) in audio_segs.iter().enumerate() {
        let audio_duration = audio_seg["target_timerange"]["duration"]
            .as_i64()
            .expect("音频段应有 duration");
        let video_duration = video_segs[i]["target_timerange"]["duration"]
            .as_i64()
            .expect("视频段应有 duration");
        assert!(
            audio_duration <= video_duration,
            "音频段 {} 时长 {} 不应超过视频段时长 {}",
            i,
            audio_duration,
            video_duration
        );
    }
}

// ---------------------------------------------------------------------------
// Test 2: OST=1（OriginalSound）仅生成视频段
// ---------------------------------------------------------------------------

/// OST=1 片段仅生成视频段，无音频段。不需要 probe_audio。
#[test]
fn test_ost_original_sound_video_only() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("clip1.mp4"), None, None, dir.path()),
        make_clip(2, OstType::OriginalSound, 4.0, Some("clip2.mp4"), None, None, dir.path()),
    ];

    let (draft_dir, json) = export_ost1_clips(clips, &dir);

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");
    assert_eq!(tracks.len(), 2, "应有 2 个轨道（video + audio）");

    // 视频轨道应有 2 个 segments
    let video_track = &tracks[0];
    assert_eq!(video_track["type"], "video");
    let video_segs = video_track["segments"].as_array().expect("video segments 应为数组");
    assert_eq!(video_segs.len(), 2, "视频轨道应有 2 个 segments");

    // 音频轨道应为空
    let audio_track = &tracks[1];
    assert_eq!(audio_track["type"], "audio");
    let audio_segs = audio_track["segments"].as_array().expect("audio segments 应为数组");
    assert!(
        audio_segs.is_empty(),
        "OST=1 不应生成音频段，实际有 {} 个",
        audio_segs.len()
    );

    // 验证草稿目录存在
    assert!(draft_dir.exists(), "草稿目录应存在");
}

// ---------------------------------------------------------------------------
// Test 3: OST=2（Mixed）混合脚本
// ---------------------------------------------------------------------------

/// 混合脚本中 OST=0 和 OST=2 的片段有音频，OST=1 的没有。
/// 需要 probe_audio——标记 #[ignore]。
#[test]
#[ignore = "需要实际音频文件和 ffmpeg 来运行 probe_audio"]
fn test_ost_mixed_timeline() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        // OST=0（NarrationOnly）——有音频
        make_clip(
            1,
            OstType::NarrationOnly,
            5.0,
            Some("clip1.mp4"),
            Some("audio1.mp3"),
            None,
            dir.path(),
        ),
        // OST=1（OriginalSound）——无音频
        make_clip(2, OstType::OriginalSound, 4.0, Some("clip2.mp4"), None, None, dir.path()),
        // OST=2（Mixed）——有音频
        make_clip(
            3,
            OstType::Mixed,
            3.0,
            Some("clip3.mp4"),
            Some("audio3.mp3"),
            None,
            dir.path(),
        ),
    ];

    let video_origin = dir.path().join("original.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: clips,
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestMixed".to_string(),
        width: 1920,
        height: 1080,
    };

    let draft_dir = export_draft(&req).expect("导出应成功");
    let json = load_draft_content(draft_dir.parent().expect("导出路径应有父目录"));

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");

    // 视频轨道应有 3 个 segments
    let video_segs = tracks[0]["segments"].as_array().expect("video segments 应为数组");
    assert_eq!(video_segs.len(), 3, "视频轨道应有 3 个 segments");

    // 音频轨道应有 2 个 segments（OST=0 和 OST=2）
    let audio_segs = tracks[1]["segments"].as_array().expect("audio segments 应为数组");
    assert_eq!(
        audio_segs.len(),
        2,
        "音频轨道应有 2 个 segments（OST=0 和 OST=2）"
    );
}

// ---------------------------------------------------------------------------
// Test 4: 视频回退——clip.video 为 None 时使用 source_timerange
// ---------------------------------------------------------------------------

/// clip.video 为 None 时，视频段使用 video_origin_path + source_timerange。
/// 不需要 probe_audio。
#[test]
fn test_video_fallback_source_range() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(
            1,
            OstType::OriginalSound,
            5.0,
            None,       // 无视频文件
            None,
            Some("00:00:10,000-00:00:15,000"), // 有 source_time_range
            dir.path(),
        ),
    ];

    let (_draft_dir, json) = export_ost1_clips(clips, &dir);

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");
    let video_track = &tracks[0];
    let video_segs = video_track["segments"].as_array().expect("video segments 应为数组");
    assert_eq!(video_segs.len(), 1, "应有 1 个视频 segment");

    let seg = &video_segs[0];

    // source_timerange 应存在且有值
    let source_tr = seg
        .get("source_timerange")
        .expect("source_timerange 应存在");
    assert!(
        source_tr.is_object(),
        "source_timerange 应为对象: {:?}",
        source_tr
    );
    assert!(
        source_tr.get("start").is_some(),
        "source_timerange 应有 start 字段"
    );
    assert!(
        source_tr.get("duration").is_some(),
        "source_timerange 应有 duration 字段"
    );

    // source_timerange.start 应为 10 秒 = 10,000,000 微秒
    let start_us = source_tr["start"].as_i64().expect("start 应为整数");
    assert_eq!(
        start_us, 10_000_000,
        "source start 应为 10,000,000 微秒（10 秒），实际: {}",
        start_us
    );

    // 视频素材的 path 应包含原始视频文件名
    let materials = json["materials"]["videos"].as_array().expect("videos 应为数组");
    assert_eq!(materials.len(), 1, "应有 1 个视频素材");
    let mat_path = materials[0]["path"].as_str().expect("path 应为字符串");
    assert!(
        mat_path.contains("original_video.mp4"),
        "素材路径应包含原始视频文件名: {}",
        mat_path
    );
}

// ---------------------------------------------------------------------------
// Test 5: JSON 结构完整性——version, fps, canvas_config
// ---------------------------------------------------------------------------

/// draft_content.json 包含 version=360000, fps=30.0, canvas_config { width, height }
#[test]
fn test_draft_content_structure() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v.mp4"), None, None, dir.path()),
    ];

    let (_, json) = export_ost1_clips(clips, &dir);

    // version
    assert_eq!(json["version"], 360000, "version 应为 360000");

    // fps
    assert_eq!(json["fps"], 30.0, "fps 应为 30.0");

    // canvas_config
    assert_eq!(
        json["canvas_config"]["width"], 1920,
        "canvas_config.width 应为 1920"
    );
    assert_eq!(
        json["canvas_config"]["height"], 1080,
        "canvas_config.height 应为 1080"
    );

    // color_space（模板固定字段）
    assert_eq!(json["color_space"], 0, "color_space 应为 0");

    // platform（模板固定字段）
    assert_eq!(
        json["platform"]["app_id"], 3704,
        "platform.app_id 应为 3704"
    );
}

// ---------------------------------------------------------------------------
// Test 6: materials 包含 videos, audios, speeds 三类素材
// ---------------------------------------------------------------------------

/// materials.videos + materials.audios + materials.speeds 非空（有内容时），
/// 每个 material 有唯一 id。
#[test]
fn test_materials_contain_all_types() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v1.mp4"), None, None, dir.path()),
        make_clip(2, OstType::OriginalSound, 4.0, Some("v2.mp4"), None, None, dir.path()),
    ];

    let (_, json) = export_ost1_clips(clips, &dir);

    let videos = json["materials"]["videos"]
        .as_array()
        .expect("materials.videos 应为数组");
    let audios = json["materials"]["audios"]
        .as_array()
        .expect("materials.audios 应为数组");
    let speeds = json["materials"]["speeds"]
        .as_array()
        .expect("materials.speeds 应为数组");

    // 有 2 个视频片段，应有 2 个视频素材
    assert_eq!(
        videos.len(),
        2,
        "应有 2 个视频素材，实际: {}",
        videos.len()
    );

    // OST=1 不生成音频素材
    assert!(
        audios.is_empty(),
        "OST=1 不应有音频素材，实际: {}",
        audios.len()
    );

    // 每个视频段有一个 Speed 素材
    assert_eq!(
        speeds.len(),
        2,
        "应有 2 个 Speed 素材（每个视频段一个），实际: {}",
        speeds.len()
    );

    // 每个视频素材应有唯一 id
    let video_ids: Vec<&str> = videos
        .iter()
        .map(|v| v["id"].as_str().expect("id 应为字符串"))
        .collect();
    assert_eq!(
        video_ids.len(),
        2,
        "应有 2 个视频素材 ID"
    );

    // id 应各不相同
    assert_ne!(video_ids[0], video_ids[1], "视频素材 ID 应各不相同");

    // 每个 id 应为 32 位 hex
    for id in &video_ids {
        assert_eq!(id.len(), 32, "素材 ID 应为 32 位 hex: {}", id);
        assert!(
            id.chars().all(|c| c.is_ascii_hexdigit()),
            "素材 ID 应为 hex 字符: {}",
            id
        );
    }
}

// ---------------------------------------------------------------------------
// Test 7: ID 引用一致性
// ---------------------------------------------------------------------------

/// 每个 segment 的 material_id 在 materials 中有对应素材，
/// extra_material_refs[0] 在 speeds 中有对应素材。
#[test]
fn test_id_reference_consistency() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v1.mp4"), None, None, dir.path()),
        make_clip(2, OstType::OriginalSound, 4.0, Some("v2.mp4"), None, None, dir.path()),
    ];

    let (_, json) = export_ost1_clips(clips, &dir);

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");
    let video_segs = tracks[0]["segments"]
        .as_array()
        .expect("video segments 应为数组");
    let materials = &json["materials"];

    // 收集所有素材 ID
    let video_ids: Vec<String> = materials["videos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_string())
        .collect();

    let speed_ids: Vec<String> = materials["speeds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_string())
        .collect();

    // 验证每个视频 segment 的 material_id 和 extra_material_refs
    for (i, seg) in video_segs.iter().enumerate() {
        // material_id 应在 video_ids 中
        let mat_id = seg["material_id"]
            .as_str()
            .unwrap_or_else(|| panic!("segment {} 应有 material_id", i));
        assert!(
            video_ids.contains(&mat_id.to_string()),
            "segment {} 的 material_id '{}' 在 materials.videos 中未找到",
            i,
            mat_id
        );

        // extra_material_refs[0] 应在 speed_ids 中
        let extra_refs = seg["extra_material_refs"]
            .as_array()
            .unwrap_or_else(|| panic!("segment {} 应有 extra_material_refs", i));
        assert!(
            !extra_refs.is_empty(),
            "segment {} 的 extra_material_refs 不应为空",
            i
        );
        let speed_ref = extra_refs[0]
            .as_str()
            .unwrap_or_else(|| panic!("segment {} 的 extra_material_refs[0] 应为字符串", i));
        assert!(
            speed_ids.contains(&speed_ref.to_string()),
            "segment {} 的 extra_material_refs[0] '{}' 在 materials.speeds 中未找到",
            i,
            speed_ref
        );
    }
}

// ---------------------------------------------------------------------------
// Test 8: 时间连续性
// ---------------------------------------------------------------------------

/// 视频段 target_timerange 连续——第一段 start=0，后续段 start = 前一段 start + duration
#[test]
fn test_timeline_continuity() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v1.mp4"), None, None, dir.path()),
        make_clip(2, OstType::OriginalSound, 4.0, Some("v2.mp4"), None, None, dir.path()),
        make_clip(3, OstType::OriginalSound, 3.5, Some("v3.mp4"), None, None, dir.path()),
    ];

    let (_, json) = export_ost1_clips(clips, &dir);

    let tracks = json["tracks"].as_array().expect("tracks 应为数组");
    let video_segs = tracks[0]["segments"]
        .as_array()
        .expect("video segments 应为数组");
    assert_eq!(video_segs.len(), 3, "应有 3 个视频 segments");

    // 第一段 start = 0
    let first_start = video_segs[0]["target_timerange"]["start"]
        .as_i64()
        .expect("第一段应有 start");
    assert_eq!(first_start, 0, "第一段的 start 应为 0");

    // 第二段 start = 5 秒 = 5,000,000 微秒
    let second_start = video_segs[1]["target_timerange"]["start"]
        .as_i64()
        .expect("第二段应有 start");
    assert_eq!(
        second_start, 5_000_000,
        "第二段的 start 应为 5,000,000（5 秒），实际: {}",
        second_start
    );

    // 第三段 start = 9 秒 = 9,000,000 微秒
    let third_start = video_segs[2]["target_timerange"]["start"]
        .as_i64()
        .expect("第三段应有 start");
    assert_eq!(
        third_start, 9_000_000,
        "第三段的 start 应为 9,000,000（9 秒），实际: {}",
        third_start
    );

    // 验证连续性：每段 start = 前一段 start + duration
    let durations: Vec<i64> = video_segs
        .iter()
        .map(|seg| seg["target_timerange"]["duration"].as_i64().unwrap())
        .collect();

    assert_eq!(durations[0], 5_000_000, "第一段 duration 应为 5 秒");
    assert_eq!(durations[1], 4_000_000, "第二段 duration 应为 4 秒");
    assert_eq!(durations[2], 3_500_000, "第三段 duration 应为 3.5 秒");

    // 验证 start 累加关系
    assert_eq!(
        second_start, first_start + durations[0],
        "第二段 start 应等于第一段 start + duration"
    );
    assert_eq!(
        third_start, second_start + durations[1],
        "第三段 start 应等于第二段 start + duration"
    );
}

// ---------------------------------------------------------------------------
// Test 9: 缺少 duration 返回 MissingField 错误
// ---------------------------------------------------------------------------

/// 脚本中某片段缺少 duration 字段应返回 MissingField 错误
#[test]
fn test_export_missing_duration() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let mut clip = make_clip(1, OstType::OriginalSound, 5.0, Some("v.mp4"), None, None, dir.path());
    clip.duration = None; // 清除 duration

    let video_origin = dir.path().join("original.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: vec![clip],
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestMissing".to_string(),
        width: 1920,
        height: 1080,
    };

    let result = export_draft(&req);
    assert!(result.is_err(), "缺少 duration 应返回错误");

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    match err {
        JianYingError::MissingField { field, clip_index } => {
            assert_eq!(field, "duration", "缺失字段应为 duration");
            assert_eq!(clip_index, 0, "片段索引应为 0");
        }
        _ => panic!(
            "应为 MissingField 错误，实际: {} ({})",
            err_msg,
            std::any::type_name_of_val(&err)
        ),
    }
}

// ---------------------------------------------------------------------------
// Test 10: 空脚本返回 Validation 错误
// ---------------------------------------------------------------------------

/// 空脚本应返回 Validation 错误
#[test]
fn test_export_empty_script() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let req = ExportRequest {
        script: vec![],
        video_origin_path: dir.path().join("original.mp4"),
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestEmpty".to_string(),
        width: 1920,
        height: 1080,
    };

    let result = export_draft(&req);
    assert!(result.is_err(), "空脚本应返回错误");

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    match err {
        JianYingError::Validation { details } => {
            assert!(
                details.contains("脚本不能为空"),
                "应包含'脚本不能为空'，实际: {}",
                details
            );
        }
        _ => panic!(
            "应为 Validation 错误，实际: {} ({})",
            err_msg,
            std::any::type_name_of_val(&err)
        ),
    }
}

// ---------------------------------------------------------------------------
// Test 11: 草稿命名——draft_name 为空时自动生成
// ---------------------------------------------------------------------------

/// draft_name 为空时自动生成 NarratoAI_ 前缀的名称
#[test]
fn test_draft_name_auto_generate() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v.mp4"), None, None, dir.path()),
    ];

    let video_origin = dir.path().join("original.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: clips,
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: String::new(), // 空名称
        width: 1920,
        height: 1080,
    };

    let result = export_draft(&req);
    assert!(result.is_ok(), "空 draft_name 应自动生成名称: {:?}", result);

    // 验证草稿目录名以 NarratoAI_ 开头
    // export_draft 返回 draft_content.json 的路径，需要取父目录名
    let draft_content_path = result.unwrap();
    let draft_dir = draft_content_path
        .parent()
        .expect("应有父目录");
    let draft_dir_name = draft_dir
        .file_name()
        .expect("草稿目录应有名称")
        .to_string_lossy();
    assert!(
        draft_dir_name.starts_with("NarratoAI_"),
        "草稿目录名应以 NarratoAI_ 开头，实际: {}",
        draft_dir_name
    );
}

// ---------------------------------------------------------------------------
// Test 12: draft_meta_info.json 包含 draft_id 字段
// ---------------------------------------------------------------------------

/// draft_meta_info.json 包含 draft_id 字段（32 位 hex 格式）
#[test]
fn test_draft_meta_info_json() {
    let dir = TempDir::new().expect("创建临时目录失败");

    let clips = vec![
        make_clip(1, OstType::OriginalSound, 5.0, Some("v.mp4"), None, None, dir.path()),
    ];

    let video_origin = dir.path().join("original.mp4");
    std::fs::write(&video_origin, b"").expect("创建原始视频文件失败");

    let req = ExportRequest {
        script: clips,
        video_origin_path: video_origin,
        draft_path: dir.path().to_path_buf(),
        draft_name: "TestMetaInfo".to_string(),
        width: 1920,
        height: 1080,
    };

    let draft_content_path = export_draft(&req).expect("导出应成功");
    let draft_dir = draft_content_path.parent().expect("应有父目录");

    let meta = load_draft_meta(draft_dir);

    // draft_id 应为非空字符串
    let draft_id = meta["draft_id"]
        .as_str()
        .expect("draft_id 应为字符串");
    assert!(
        !draft_id.is_empty(),
        "draft_id 不应为空"
    );

    // draft_id 应为 32 位 hex（UUID 去掉连字符）
    assert_eq!(
        draft_id.len(),
        32,
        "draft_id 应为 32 位 hex，实际长度: {}",
        draft_id.len()
    );
    assert!(
        draft_id.chars().all(|c| c.is_ascii_hexdigit()),
        "draft_id 应全部为 hex 字符: {}",
        draft_id
    );

    // draft_name 应为 "TestMetaInfo"
    let draft_name = meta["draft_name"]
        .as_str()
        .expect("draft_name 应为字符串");
    assert_eq!(
        draft_name, "TestMetaInfo",
        "draft_name 应为 TestMetaInfo"
    );
}
