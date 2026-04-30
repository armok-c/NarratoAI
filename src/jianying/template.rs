/// draft_content.json 模板（per pyJianYingDraft assets/draft_content_template.json）
///
/// 包含剪映识别草稿所需的全部固定字段。id 和 duration 将在 builder 中替换为实际值。
pub const DRAFT_CONTENT_TEMPLATE: &str = r#"{
    "canvas_config": {"height": 1080, "ratio": "original", "width": 1920},
    "color_space": 0,
    "config": {"maintrack_adsorb": true, "video_mute": false},
    "duration": 60000000,
    "fps": 30.0,
    "id": "DRAFT_ID_PLACEHOLDER",
    "materials": {
        "audios": [],
        "speeds": [],
        "videos": [],
        "audio_effects": [],
        "audio_fades": [],
        "canvases": [],
        "effects": [],
        "masks": [],
        "material_animations": [],
        "transitions": [],
        "video_effects": []
    },
    "platform": {
        "app_id": 3704,
        "app_version": "5.9.0",
        "device_id": "",
        "hard_disk_id": "",
        "mac_address": ""
    },
    "tracks": [],
    "version": 360000
}"#;

/// draft_meta_info.json 模板（per pyJianYingDraft assets/draft_meta_info.json）
///
/// 草稿元信息。draft_id 和 draft_name 将在 builder 中替换为实际值。
pub const DRAFT_META_INFO_TEMPLATE: &str = r#"{
    "draft_cloud_capcut_purchase_info": "",
    "draft_cloud_last_action_download": false,
    "draft_cloud_materials": [],
    "draft_cloud_purchase_info": "",
    "draft_cloud_template_id": "",
    "draft_cloud_tutorial_info": "",
    "draft_cloud_videocut_purchase_info": "",
    "draft_cover": "",
    "draft_deeplink_url": "",
    "draft_enterprise_info": {
        "draft_enterprise_extra": "",
        "enterprise_id": "",
        "file_path": "",
        "local_materials": [],
        "related_composition_paths": []
    },
    "draft_id": "DRAFT_ID_PLACEHOLDER",
    "draft_is_ai_shorts": false,
    "draft_is_article_video_draft": false,
    "draft_is_from_deeplink": false,
    "draft_is_invisible": false,
    "draft_materials_copied": false,
    "draft_name": "DRAFT_NAME_PLACEHOLDER",
    "draft_new_version": "",
    "draft_removable_storage_device": "",
    "draft_root_path": "",
    "draft_segment_extra_info": null,
    "draft_timeline_materials_size_": 0,
    "draft_type": 0
}"#;

#[cfg(test)]
mod tests {
    /// Test 9: DRAFT_CONTENT_TEMPLATE 可被 serde_json::from_str 解析
    #[test]
    fn test_draft_content_template_parseable() {
        let value: serde_json::Value =
            serde_json::from_str(super::DRAFT_CONTENT_TEMPLATE).expect("draft_content 模板应可解析");
        // 验证关键字段存在
        assert_eq!(value["version"], 360000, "version 应为 360000");
        assert_eq!(value["color_space"], 0, "color_space 应为 0");
        assert_eq!(value["fps"], 30.0, "fps 应为 30.0");
        assert!(value["materials"]["audios"].is_array(), "materials.audios 应为数组");
        assert!(value["materials"]["speeds"].is_array(), "materials.speeds 应为数组");
        assert!(value["materials"]["videos"].is_array(), "materials.videos 应为数组");
        assert!(value["tracks"].is_array(), "tracks 应为数组");
        assert_eq!(value["platform"]["app_id"], 3704, "platform.app_id 应为 3704");
        assert_eq!(value["canvas_config"]["width"], 1920, "canvas_config.width 应为 1920");
        assert_eq!(value["canvas_config"]["height"], 1080, "canvas_config.height 应为 1080");
    }

    /// Test 10: DRAFT_META_INFO_TEMPLATE 可被 serde_json::from_str 解析
    #[test]
    fn test_draft_meta_info_template_parseable() {
        let value: serde_json::Value =
            serde_json::from_str(super::DRAFT_META_INFO_TEMPLATE).expect("draft_meta_info 模板应可解析");
        assert!(value["draft_id"].is_string(), "draft_id 应为字符串");
        assert!(value["draft_name"].is_string(), "draft_name 应为字符串");
        assert_eq!(value["draft_type"], 0, "draft_type 应为 0");
        assert_eq!(value["draft_is_invisible"], false, "draft_is_invisible 应为 false");
    }
}
