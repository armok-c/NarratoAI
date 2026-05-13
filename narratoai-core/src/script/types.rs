use serde::{Deserialize, Serialize};
use serde::de;
use serde_repr::{Deserialize_repr, Serialize_repr};
use ts_rs::TS;

/// OST 类型——决定视频裁剪时如何处理音频
///
/// - `NarrationOnly` (0): 仅保留解说音频，根据 TTS 时长裁剪
/// - `OriginalSound` (1): 保留原声，按脚本 timestamp 精确裁剪
/// - `Mixed` (2): 保留解说+原声（混合），根据 TTS 时长裁剪
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, TS)]
#[repr(u8)]
#[ts(export, export_to = "../src/types/generated/")]
#[ts(type = "0 | 1 | 2")]
pub enum OstType {
    NarrationOnly = 0,
    OriginalSound = 1,
    Mixed = 2,
}

/// 自定义 _id 反序列化——同时接受整数和浮点数（如 LLM 生成的 `1.0`）
fn deserialize_id<'de, D: de::Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let val = serde_json::Value::deserialize(d)?;
    match val {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i)
            } else if let Some(f) = n.as_f64() {
                let i = f as i64;
                // Reject if truncation (fractional part) or saturation occurred
                if i as f64 != f || i == i64::MAX || i == i64::MIN {
                    return Err(de::Error::custom("_id 数值超出有效范围"));
                }
                Ok(i)
            } else {
                Err(de::Error::custom("_id 必须是数字"))
            }
        }
        _ => Err(de::Error::custom("_id 必须是数字")),
    }
}

/// 脚本片段——对应 Python 版 VideoClipParams 的核心字段 + 管道扩展字段
///
/// 5 个核心字段（`_id`, `timestamp`, `picture`, `narration`, `OST`）在 JSON 中必须有值。
/// 6 个管道扩展字段用 `Option<T>`，加载时为 `None`，管道处理后可能有值。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct ScriptClip {
    #[serde(deserialize_with = "deserialize_id")]
    pub _id: i64,
    pub timestamp: String,
    pub picture: String,
    pub narration: String,
    #[serde(rename = "OST")]
    pub ost: OstType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(rename = "sourceTimeRange", skip_serializing_if = "Option::is_none")]
    pub source_time_range: Option<String>,
    #[serde(rename = "editedTimeRange", skip_serializing_if = "Option::is_none")]
    pub edited_time_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<std::path::PathBuf>,
}

/// 顶层脚本类型——JSON 数组格式，与 Python 版一致
pub type Script = Vec<ScriptClip>;

/// 校验错误——记录单个字段的校验失败信息
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub clip_index: usize,
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "第 {} 段 {}: {}",
            self.clip_index + 1,
            self.field,
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: 反序列化包含 5 个核心字段的 JSON，验证 _id 和 OST
    #[test]
    fn test_deserialize_core_fields() {
        let json = r#"{
            "_id": 1,
            "timestamp": "00:00:00,600-00:00:07,559",
            "picture": "画面描述内容",
            "narration": "解说文案内容",
            "OST": 0
        }"#;
        let clip: ScriptClip = serde_json::from_str(json).expect("应反序列化成功");
        assert_eq!(clip._id, 1);
        assert_eq!(clip.timestamp, "00:00:00,600-00:00:07,559");
        assert_eq!(clip.picture, "画面描述内容");
        assert_eq!(clip.narration, "解说文案内容");
        assert_eq!(clip.ost, OstType::NarrationOnly);
        // Option 字段应为 None
        assert!(clip.duration.is_none());
        assert!(clip.source_time_range.is_none());
        assert!(clip.edited_time_range.is_none());
        assert!(clip.audio.is_none());
        assert!(clip.video.is_none());
        assert!(clip.subtitle.is_none());
    }

    /// Test 2: 反序列化包含管道扩展字段的 JSON
    #[test]
    fn test_deserialize_pipeline_fields() {
        let json = r#"{
            "_id": 2,
            "timestamp": "00:00:07,559-00:00:15,000",
            "picture": "画面2",
            "narration": "解说2",
            "OST": 1,
            "duration": 7.441,
            "sourceTimeRange": "00:00:07,559-00:00:15,000",
            "editedTimeRange": "00:00:00,000-00:00:07,441",
            "audio": "/tmp/audio_2.mp3",
            "video": "/tmp/video_2.mp4",
            "subtitle": "/tmp/sub_2.srt"
        }"#;
        let clip: ScriptClip = serde_json::from_str(json).expect("应反序列化成功");
        assert_eq!(clip._id, 2);
        assert_eq!(clip.ost, OstType::OriginalSound);
        assert_eq!(clip.duration, Some(7.441));
        assert_eq!(
            clip.source_time_range,
            Some("00:00:07,559-00:00:15,000".to_string())
        );
        assert_eq!(
            clip.edited_time_range,
            Some("00:00:00,000-00:00:07,441".to_string())
        );
        assert_eq!(
            clip.audio,
            Some(std::path::PathBuf::from("/tmp/audio_2.mp3"))
        );
        assert_eq!(
            clip.video,
            Some(std::path::PathBuf::from("/tmp/video_2.mp4"))
        );
        assert_eq!(
            clip.subtitle,
            Some(std::path::PathBuf::from("/tmp/sub_2.srt"))
        );
    }

    /// Test 3: 缺少管道扩展字段的 JSON 反序列化后，Option 字段为 None
    #[test]
    fn test_missing_pipeline_fields_are_none() {
        let json = r#"{
            "_id": 3,
            "timestamp": "00:00:15,000-00:00:20,000",
            "picture": "画面3",
            "narration": "解说3",
            "OST": 2
        }"#;
        let clip: ScriptClip = serde_json::from_str(json).expect("应反序列化成功");
        assert!(clip.duration.is_none());
        assert!(clip.source_time_range.is_none());
        assert!(clip.edited_time_range.is_none());
        assert!(clip.audio.is_none());
        assert!(clip.video.is_none());
        assert!(clip.subtitle.is_none());
    }

    /// Test 4: OstType 序列化为整数 0/1/2，反序列化从 0/1/2 正确还原
    #[test]
    fn test_ost_type_serde_roundtrip() {
        for (variant, value) in [
            (OstType::NarrationOnly, 0u8),
            (OstType::OriginalSound, 1),
            (OstType::Mixed, 2),
        ] {
            // 序列化
            let serialized = serde_json::to_string(&variant).expect("应序列化成功");
            assert_eq!(serialized, value.to_string(), "OstType {:?} 应序列化为 {}", variant, value);

            // 反序列化
            let deserialized: OstType =
                serde_json::from_str(&serialized).expect("应反序列化成功");
            assert_eq!(deserialized, variant, "应还原为 {:?}", variant);
        }
    }

    /// Test 5: ScriptClip 序列化时 Option::None 字段不出现在 JSON 输出中
    #[test]
    fn test_serialize_skips_none_fields() {
        let clip = ScriptClip {
            _id: 1,
            timestamp: "00:00:00,600-00:00:07,559".to_string(),
            picture: "画面".to_string(),
            narration: "解说".to_string(),
            ost: OstType::NarrationOnly,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };
        let json = serde_json::to_string(&clip).expect("应序列化成功");
        assert!(
            !json.contains("duration"),
            "None 字段 duration 不应出现在 JSON 中"
        );
        assert!(
            !json.contains("sourceTimeRange"),
            "None 字段 sourceTimeRange 不应出现在 JSON 中"
        );
        assert!(
            !json.contains("editedTimeRange"),
            "None 字段 editedTimeRange 不应出现在 JSON 中"
        );
        assert!(
            !json.contains("audio"),
            "None 字段 audio 不应出现在 JSON 中"
        );
        assert!(
            !json.contains("video"),
            "None 字段 video 不应出现在 JSON 中"
        );
        assert!(
            !json.contains("subtitle"),
            "None 字段 subtitle 不应出现在 JSON 中"
        );
        // 核心字段应该出现
        assert!(json.contains("_id"));
        assert!(json.contains("timestamp"));
        assert!(json.contains("picture"));
        assert!(json.contains("narration"));
        assert!(json.contains("OST"));
    }

    /// Test 6: ScriptClip 序列化时中文字符原样输出（不转义为 \uXXXX）
    #[test]
    fn test_serialize_preserves_chinese() {
        let clip = ScriptClip {
            _id: 1,
            timestamp: "00:00:00,600-00:00:07,559".to_string(),
            picture: "一个安静的夜晚，月光洒在湖面上".to_string(),
            narration: "这是一段关于宁静的解说".to_string(),
            ost: OstType::NarrationOnly,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };
        let json = serde_json::to_string(&clip).expect("应序列化成功");
        // serde_json 默认不转义非 ASCII 字符
        assert!(
            json.contains("一个安静的夜晚"),
            "中文应原样输出，不应被转义"
        );
        assert!(
            json.contains("这是一段关于宁静的解说"),
            "中文 narration 应原样输出"
        );
        assert!(
            !json.contains("\\u"),
            "不应包含 Unicode 转义序列"
        );
    }

    /// Test 7: ValidationError Display 输出中文格式
    #[test]
    fn test_validation_error_display_chinese() {
        let err = ValidationError {
            clip_index: 2,
            field: "timestamp".to_string(),
            message: "时间戳格式无效".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("第 3 段"),
            "应包含中文段号: {}",
            msg
        );
        assert!(
            msg.contains("timestamp"),
            "应包含字段名: {}",
            msg
        );
        assert!(
            msg.contains("时间戳格式无效"),
            "应包含中文消息: {}",
            msg
        );
    }

    /// Test 8: _id 为浮点数（如 LLM 生成的 1.0）应正确反序列化为整数 1
    #[test]
    fn test_deserialize_id_from_float() {
        let json = r#"{
            "_id": 1.0,
            "timestamp": "00:00:00,600-00:00:07,559",
            "picture": "画面",
            "narration": "解说",
            "OST": 0
        }"#;
        let clip: ScriptClip = serde_json::from_str(json).expect("浮点数 _id 应反序列化成功");
        assert_eq!(clip._id, 1, "_id 1.0 应转为整数 1");
    }

    /// Test 9: _id 为非数字值应反序列化失败
    #[test]
    fn test_deserialize_id_from_string_fails() {
        let json = r#"{
            "_id": "abc",
            "timestamp": "00:00:00,600-00:00:07,559",
            "picture": "画面",
            "narration": "解说",
            "OST": 0
        }"#;
        let result: Result<ScriptClip, _> = serde_json::from_str(json);
        assert!(result.is_err(), "字符串 _id 应反序列化失败");
    }

    /// Test 10: _id 为超大整数（超出 i64 范围）应反序列化失败
    #[test]
    fn test_deserialize_id_oversized_integer() {
        let json = r#"{
            "_id": 99999999999999999999,
            "timestamp": "00:00:00,600-00:00:07,559",
            "picture": "画面",
            "narration": "解说",
            "OST": 0
        }"#;
        let result: Result<ScriptClip, _> = serde_json::from_str(json);
        assert!(result.is_err(), "超出 i64 范围的 _id 应反序列化失败");
    }

    /// Test 11: _id 为小数（如 1.9）应反序列化失败
    #[test]
    fn test_deserialize_id_fractional_fails() {
        let json = r#"{
            "_id": 1.9,
            "timestamp": "00:00:00,600-00:00:07,559",
            "picture": "画面",
            "narration": "解说",
            "OST": 0
        }"#;
        let result: Result<ScriptClip, _> = serde_json::from_str(json);
        assert!(result.is_err(), "小数 _id 应反序列化失败");
    }
}
