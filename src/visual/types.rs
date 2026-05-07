use serde::{Deserialize, Serialize};

/// 单帧观察结果（对齐 Python 版 frame_observation 字典）
///
/// 所有字段通过 serde 反序列化 LLM JSON 响应。
/// 未知字段被静默忽略，兼容 LLM 返回额外字段的场景（D-15）。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameObservation {
    /// 帧序号（从 0 开始）
    pub frame_number: u64,
    /// 帧时间码（格式: HH:MM:SS.mmm）
    pub timestamp: String,
    /// 场景描述
    pub scene_description: String,
    /// 检测到的物体/人物列表
    pub objects: Vec<String>,
    /// 画面动作/行为描述
    pub actions: Vec<String>,
    /// 字幕/文字内容（如果有）
    pub on_screen_text: Option<String>,
    /// 视觉显著性评分 0.0-1.0
    pub visual_salience: Option<f64>,
}

/// 批次分析结果（对齐 Python 版 frame_batch_result）
///
/// 汇总多个单帧观察结果、总体摘要和错误统计（D-14/D-15）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAnalysisResult {
    /// 帧观察列表
    pub observations: Vec<FrameObservation>,
    /// 总体活动摘要（跨帧的综合分析）
    pub overall_activity_summary: Option<String>,
    /// 元信息
    pub total_frames: usize,
    pub analyzed_batches: usize,
    pub errors: Vec<String>,
}

/// 从 JSON 字符串剥离 markdown 代码块包裹
///
/// 某些 LLM 即使设置 response_format=JsonObject 仍返回 ` ```json ` 包裹，
/// 此函数提取代码块内的纯净 JSON。
pub(crate) fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let after_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_start());
    let content = after_prefix.unwrap_or(trimmed);
    content
        .strip_suffix("```")
        .map(|s| {
            if s.ends_with(|c: char| c.is_whitespace()) {
                s.trim_end()
            } else {
                content
            }
        })
        .unwrap_or(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: 有效完整 JSON 反序列化（全部字段）
    #[test]
    fn test_deserialize_full_json() {
        let json = r#"{
            "frame_number": 0,
            "timestamp": "00:00:00.000",
            "scene_description": "一个安静的湖泊，湖面如镜，倒映着远处的雪山",
            "objects": ["湖水", "雪山", "天空", "云彩"],
            "actions": ["湖面微波荡漾"],
            "on_screen_text": "西藏 纳木错",
            "visual_salience": 0.85
        }"#;
        let obs: FrameObservation = serde_json::from_str(json).expect("应反序列化成功");
        assert_eq!(obs.frame_number, 0);
        assert_eq!(obs.timestamp, "00:00:00.000");
        assert_eq!(obs.scene_description, "一个安静的湖泊，湖面如镜，倒映着远处的雪山");
        assert_eq!(obs.objects, vec!["湖水", "雪山", "天空", "云彩"]);
        assert_eq!(obs.actions, vec!["湖面微波荡漾"]);
        assert_eq!(obs.on_screen_text, Some("西藏 纳木错".to_string()));
        assert_eq!(obs.visual_salience, Some(0.85));
    }

    /// Test 2: 仅必需字段 JSON 反序列化（无 optional 字段）
    #[test]
    fn test_deserialize_required_fields_only() {
        let json = r#"{
            "frame_number": 1,
            "timestamp": "00:00:03.000",
            "scene_description": "城市街道",
            "objects": ["汽车", "行人"],
            "actions": ["车辆行驶"]
        }"#;
        let obs: FrameObservation = serde_json::from_str(json).expect("应反序列化成功");
        assert_eq!(obs.frame_number, 1);
        assert_eq!(obs.on_screen_text, None);
        assert_eq!(obs.visual_salience, None);
    }

    /// Test 3: 代码块包裹 JSON 剥离后反序列化（```json...```）
    #[test]
    fn test_strip_code_fence_json_prefix() {
        let raw = "```json\n{\"frame_number\": 2, \"timestamp\": \"00:00:06.000\", \"scene_description\": \"室内\", \"objects\": [\"桌子\"], \"actions\": [\"摆放\"]}\n```";
        let cleaned = strip_code_fence(raw);
        let obs: FrameObservation = serde_json::from_str(cleaned).expect("剥离代码块后应反序列化成功");
        assert_eq!(obs.frame_number, 2);
        assert_eq!(obs.scene_description, "室内");
    }

    /// Test 4: 反向代码块标记 ``` 剥离
    #[test]
    fn test_strip_code_fence_plain() {
        let raw = "```\n{\"frame_number\": 3, \"timestamp\": \"00:00:09.000\", \"scene_description\": \"室外\", \"objects\": [\"树\"], \"actions\": [\"摇摆\"]}\n```";
        let cleaned = strip_code_fence(raw);
        let obs: FrameObservation = serde_json::from_str(cleaned).expect("剥离 ``` 后应反序列化成功");
        assert_eq!(obs.frame_number, 3);
        assert_eq!(obs.scene_description, "室外");
    }

    /// Test 5: missing field 场景——缺失 scene_description → 应返回 serde 错误
    #[test]
    fn test_missing_required_field_fails() {
        let json = r#"{
            "frame_number": 4,
            "timestamp": "00:00:12.000",
            "objects": ["汽车"],
            "actions": ["行驶"]
        }"#;
        let result: Result<FrameObservation, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "缺失必需字段 scene_description 应反序列化失败"
        );
    }

    /// Test 6: camelCase 未知字段名 → 应被静默忽略（无 deny_unknown_fields）
    #[test]
    fn test_unknown_fields_silently_ignored() {
        let json = r#"{
            "frame_number": 5,
            "timestamp": "00:00:15.000",
            "scene_description": "海滩",
            "objects": ["海浪"],
            "actions": ["拍打"],
            "sceneDescription": "额外字段"
        }"#;
        let result: Result<FrameObservation, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "未知字段（如 camelCase sceneDescription）应被静默忽略"
        );
        let obs = result.unwrap();
        assert_eq!(obs.scene_description, "海滩");
    }

    /// Test 7: 空 objects 列表
    #[test]
    fn test_empty_objects_list() {
        let json = r#"{
            "frame_number": 6,
            "timestamp": "00:00:18.000",
            "scene_description": "纯色画面",
            "objects": [],
            "actions": []
        }"#;
        let obs: FrameObservation = serde_json::from_str(json).expect("空列表应反序列化成功");
        assert!(obs.objects.is_empty());
        assert!(obs.actions.is_empty());
    }

    /// Test 8: on_screen_text 为 null
    #[test]
    fn test_on_screen_text_null() {
        let json = r#"{
            "frame_number": 7,
            "timestamp": "00:00:21.000",
            "scene_description": "会议室",
            "objects": ["投影仪"],
            "actions": ["演讲"],
            "on_screen_text": null
        }"#;
        let obs: FrameObservation = serde_json::from_str(json).expect("null 可选字段应反序列化成功");
        assert_eq!(obs.on_screen_text, None);
    }

    /// Test 9: BatchAnalysisResult 完整序列化/反序列化 round-trip
    #[test]
    fn test_batch_analysis_result_roundtrip() {
        let original = BatchAnalysisResult {
            observations: vec![
                FrameObservation {
                    frame_number: 0,
                    timestamp: "00:00:00.000".to_string(),
                    scene_description: "开场画面".to_string(),
                    objects: vec!["背景".to_string()],
                    actions: vec!["淡入".to_string()],
                    on_screen_text: Some("标题".to_string()),
                    visual_salience: Some(0.9),
                },
            ],
            overall_activity_summary: Some("视频开场".to_string()),
            total_frames: 1,
            analyzed_batches: 1,
            errors: vec![],
        };

        let json = serde_json::to_string(&original).expect("应序列化成功");
        let deserialized: BatchAnalysisResult =
            serde_json::from_str(&json).expect("应反序列化成功");

        assert_eq!(deserialized.observations.len(), 1);
        assert_eq!(deserialized.observations[0].frame_number, 0);
        assert_eq!(
            deserialized.overall_activity_summary,
            Some("视频开场".to_string())
        );
        assert_eq!(deserialized.total_frames, 1);
        assert_eq!(deserialized.analyzed_batches, 1);
        assert!(deserialized.errors.is_empty());
    }

    /// Test 10: 超大 visual_salience 值（> 1.0）→ 应反序列化成功（不做范围校验）
    #[test]
    fn test_visual_salience_out_of_range() {
        let json = r#"{
            "frame_number": 8,
            "timestamp": "00:00:24.000",
            "scene_description": "测试帧",
            "objects": ["测试对象"],
            "actions": ["测试动作"],
            "visual_salience": 2.5
        }"#;
        let obs: FrameObservation = serde_json::from_str(json).expect("超出范围的值应反序列化成功");
        assert_eq!(obs.visual_salience, Some(2.5));
    }
}
