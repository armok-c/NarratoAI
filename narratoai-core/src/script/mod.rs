pub mod edit;
pub mod error;
pub mod types;

use std::path::Path;

use crate::script::error::ScriptError;
use crate::script::types::{Script, ValidationError};

/// 将单个时间戳字符串转换为毫秒数
///
/// 接受两种格式: `HH:MM:SS,mmm` 或 `HH:MM:SS`（毫秒部分可选，默认 0）。
fn timestamp_to_millis(ts: &str) -> Option<u64> {
    let sub_parts: Vec<&str> = ts.split(',').collect();
    let time_str = sub_parts.first()?;
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }
    for tp in &time_parts {
        if tp.len() != 2 || !tp.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
    }
    let h: u64 = time_parts[0].parse().ok()?;
    let m: u64 = time_parts[1].parse().ok()?;
    let s: u64 = time_parts[2].parse().ok()?;
    if h > 23 || m > 59 || s > 59 {
        return None;
    }
    let ms: u64 = if sub_parts.len() > 1 {
        let millis_str = sub_parts[1];
        if millis_str.len() != 3 || !millis_str.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        millis_str.parse().ok()?
    } else {
        0
    };
    Some((h * 3600 + m * 60 + s) * 1000 + ms)
}

/// 校验 timestamp 格式: `HH:MM:SS,mmm-HH:MM:SS,mmm` 或 `HH:MM:SS-HH:MM:SS`
///
/// 毫秒部分可选，与 Python 管道兼容。同时校验范围和起止顺序。
/// 手写校验，避免引入 regex 依赖。
pub(crate) fn validate_timestamp(ts: &str) -> bool {
    // NOTE: Split on '-' assumes the timestamp format never contains internal hyphens.
    // The format "HH:MM:SS,mmm-HH:MM:SS,mmm" guarantees this because all components
    // are digits, colons, and commas.
    let parts: Vec<&str> = ts.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let start_millis = match timestamp_to_millis(parts[0]) {
        Some(v) => v,
        None => return false,
    };
    let end_millis = match timestamp_to_millis(parts[1]) {
        Some(v) => v,
        None => return false,
    };
    end_millis >= start_millis
}

/// 校验脚本——收集所有错误一次性返回
///
/// 对齐 Python 版 `app/utils/check_script.py` 的校验规则:
/// - 非空数组
/// - `_id` 为正整数
/// - `timestamp` 格式匹配 `HH:MM:SS,mmm-HH:MM:SS,mmm`
/// - `picture` 非空
/// - `narration` 非空
/// - OST 有效值（serde_repr 已自动处理）
pub fn validate(script: &Script) -> Result<(), ScriptError> {
    let mut errors = Vec::new();

    if script.is_empty() {
        errors.push(ValidationError {
            clip_index: 0,
            field: "script".to_string(),
            message: "脚本数组不能为空".to_string(),
        });
        return Err(ScriptError::Validation(errors));
    }

    for (i, clip) in script.iter().enumerate() {
        if clip._id <= 0 {
            errors.push(ValidationError {
                clip_index: i,
                field: "_id".to_string(),
                message: format!("必须为正整数，当前值: {}", clip._id),
            });
        }

        if !validate_timestamp(&clip.timestamp) {
            errors.push(ValidationError {
                clip_index: i,
                field: "timestamp".to_string(),
                message: format!(
                    "时间戳格式无效，正确格式: HH:MM:SS,mmm-HH:MM:SS,mmm 或 HH:MM:SS-HH:MM:SS，当前值: {}",
                    clip.timestamp
                ),
            });
        }

        if clip.picture.trim().is_empty() {
            errors.push(ValidationError {
                clip_index: i,
                field: "picture".to_string(),
                message: "必须是非空字符串".to_string(),
            });
        }

        if clip.narration.trim().is_empty() {
            errors.push(ValidationError {
                clip_index: i,
                field: "narration".to_string(),
                message: "必须是非空字符串".to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ScriptError::Validation(errors))
    }
}

/// 从 JSON 文件加载脚本，加载时自动校验
pub fn load_script(path: &Path) -> Result<Script, ScriptError> {
    let content = std::fs::read_to_string(path).map_err(ScriptError::Io)?;
    let script: Script = serde_json::from_str(&content).map_err(ScriptError::JsonParse)?;
    validate(&script)?;
    Ok(script)
}

/// 将脚本保存为美化 JSON 文件
///
/// - indent=2 缩进
/// - 非 ASCII 字符原样输出（不转义）
/// - Option::None 字段不输出（由 serde skip_serializing_if 保证）
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    validate(script)?;
    let json = serde_json::to_string_pretty(script).map_err(ScriptError::JsonParse)?;
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, json.as_bytes()).map_err(ScriptError::Io)?;
    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
        return Err(ScriptError::Io(e));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::{OstType, ScriptClip};
    use tempfile::TempDir;

    /// Test 1: load_script 从有效 JSON 文件加载，返回正确的 Script
    #[test]
    fn test_load_valid_script() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("valid.json");
        let json = r#"[
            {
                "_id": 1,
                "timestamp": "00:00:00,600-00:00:07,559",
                "picture": "画面1",
                "narration": "解说1",
                "OST": 0
            },
            {
                "_id": 2,
                "timestamp": "00:00:07,559-00:00:15,000",
                "picture": "画面2",
                "narration": "解说2",
                "OST": 1
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let script = load_script(&path).expect("有效脚本应加载成功");
        assert_eq!(script.len(), 2);
        assert_eq!(script[0]._id, 1);
        assert_eq!(script[0].ost, OstType::NarrationOnly);
        assert_eq!(script[1]._id, 2);
        assert_eq!(script[1].ost, OstType::OriginalSound);
    }

    /// Test 2: load_script 从不存在的文件加载，返回 ScriptError::Io
    #[test]
    fn test_load_nonexistent_file() {
        let path = std::path::PathBuf::from("/tmp/narratoai_nonexistent_test_file.json");
        let result = load_script(&path);
        assert!(result.is_err(), "不存在的文件应返回错误");
        match result {
            Err(ScriptError::Io(_)) => { /* 正确 */ }
            Err(e) => panic!("应为 Io 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 3: load_script 从无效 JSON 文件加载，返回 ScriptError::JsonParse
    #[test]
    fn test_load_invalid_json() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("invalid.json");
        std::fs::write(&path, "{not valid json}").expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "无效 JSON 应返回错误");
        match result {
            Err(ScriptError::JsonParse(_)) => { /* 正确 */ }
            Err(e) => panic!("应为 JsonParse 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 4: load_script 从 OST 为无效值的文件加载，serde_repr 反序列化失败
    #[test]
    fn test_load_invalid_ost() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("invalid_ost.json");
        let json = r#"[
            {
                "_id": 1,
                "timestamp": "00:00:00,600-00:00:07,559",
                "picture": "画面",
                "narration": "解说",
                "OST": 5
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "无效 OST 值应返回错误");
        // serde_repr 对无效整数值返回反序列化错误
        match result {
            Err(ScriptError::JsonParse(_)) => { /* serde_repr 反序列化失败 */ }
            Err(ScriptError::Validation(_)) => { /* 或 validate 捕获 */ }
            Err(e) => panic!("应为 JsonParse 或 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 5: load_script 对空数组返回校验错误
    #[test]
    fn test_load_empty_array() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("empty.json");
        std::fs::write(&path, "[]").expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "空数组应返回错误");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.message.contains("脚本数组不能为空")),
                    "应包含空数组错误信息"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 6: load_script 对 _id <= 0 的脚本返回校验错误
    #[test]
    fn test_load_invalid_id() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("bad_id.json");
        let json = r#"[
            {
                "_id": 0,
                "timestamp": "00:00:00,600-00:00:07,559",
                "picture": "画面",
                "narration": "解说",
                "OST": 0
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "_id <= 0 应返回错误");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.field == "_id"),
                    "应包含 _id 校验错误"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 7: load_script 对无效 timestamp 格式返回校验错误
    #[test]
    fn test_load_invalid_timestamp() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("bad_ts.json");
        let json = r#"[
            {
                "_id": 1,
                "timestamp": "bad-format",
                "picture": "画面",
                "narration": "解说",
                "OST": 0
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "无效 timestamp 应返回错误");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.field == "timestamp"),
                    "应包含 timestamp 校验错误"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 8: load_script 对空 picture/narration 返回校验错误
    #[test]
    fn test_load_empty_picture_narration() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("empty_fields.json");
        let json = r#"[
            {
                "_id": 1,
                "timestamp": "00:00:00,600-00:00:07,559",
                "picture": "  ",
                "narration": "",
                "OST": 0
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let result = load_script(&path);
        assert!(result.is_err(), "空 picture/narration 应返回错误");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.field == "picture"),
                    "应包含 picture 校验错误"
                );
                assert!(
                    errors.iter().any(|e| e.field == "narration"),
                    "应包含 narration 校验错误"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 9: validate 收集多个错误一次性返回
    #[test]
    fn test_validate_collects_multiple_errors() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("multi_error.json");
        let json = r#"[
            {
                "_id": -1,
                "timestamp": "bad",
                "picture": "",
                "narration": "  ",
                "OST": 0
            }
        ]"#;
        std::fs::write(&path, json).expect("写入临时文件失败");

        let result = load_script(&path);
        match result {
            Err(ScriptError::Validation(errors)) => {
                // 应收集 _id, timestamp, picture, narration 四个错误
                assert!(
                    errors.len() >= 4,
                    "应收集至少 4 个错误，实际: {} 个: {:?}",
                    errors.len(),
                    errors
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 10: save_script 输出美化 JSON（indent=2），Option 字段不出现
    #[test]
    fn test_save_script_pretty_json() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("output.json");

        let script = vec![ScriptClip {
            _id: 1,
            timestamp: "00:00:00,600-00:00:07,559".to_string(),
            picture: "中文画面".to_string(),
            narration: "中文解说".to_string(),
            ost: OstType::Mixed,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        }];

        save_script(&script, &path).expect("保存应成功");

        let content = std::fs::read_to_string(&path).expect("读取输出文件失败");
        // 验证美化 JSON 格式（包含缩进）
        assert!(
            content.contains("\n"),
            "美化 JSON 应包含换行"
        );
        // 验证中文字符原样输出
        assert!(
            content.contains("中文画面"),
            "中文字符应原样输出: {}",
            content
        );
        // 验证 Option 字段不出现
        assert!(
            !content.contains("duration"),
            "Option::None 字段 duration 不应出现在输出中"
        );
        assert!(
            !content.contains("sourceTimeRange"),
            "Option::None 字段 sourceTimeRange 不应出现在输出中"
        );
    }

    /// Test 11: save_script 后 load_script 可以正确重新加载（round-trip）
    #[test]
    fn test_save_load_roundtrip() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("roundtrip.json");

        let original = vec![
            ScriptClip {
                _id: 1,
                timestamp: "00:00:00,600-00:00:07,559".to_string(),
                picture: "画面一".to_string(),
                narration: "解说一".to_string(),
                ost: OstType::NarrationOnly,
                duration: Some(6.959),
                source_time_range: None,
                edited_time_range: None,
                audio: None,
                video: None,
                subtitle: None,
            },
            ScriptClip {
                _id: 2,
                timestamp: "00:00:07,559-00:00:15,000".to_string(),
                picture: "画面二".to_string(),
                narration: "解说二".to_string(),
                ost: OstType::OriginalSound,
                duration: None,
                source_time_range: Some("00:00:07,559-00:00:15,000".to_string()),
                edited_time_range: None,
                audio: None,
                video: None,
                subtitle: None,
            },
        ];

        save_script(&original, &path).expect("保存应成功");
        let loaded = load_script(&path).expect("重新加载应成功");

        assert_eq!(loaded.len(), original.len());
        assert_eq!(loaded[0]._id, original[0]._id);
        assert_eq!(loaded[0].picture, original[0].picture);
        assert_eq!(loaded[0].narration, original[0].narration);
        assert_eq!(loaded[0].ost, original[0].ost);
        assert_eq!(loaded[0].duration, original[0].duration);

        assert_eq!(loaded[1]._id, original[1]._id);
        assert_eq!(loaded[1].ost, original[1].ost);
        assert_eq!(loaded[1].source_time_range, original[1].source_time_range);
        assert!(loaded[1].duration.is_none());
    }

    /// Test 12: validate_timestamp 辅助函数正确识别有效/无效 timestamp 格式
    #[test]
    fn test_validate_timestamp() {
        // 有效格式
        assert!(
            validate_timestamp("00:00:00,600-00:00:07,559"),
            "标准格式应通过"
        );
        assert!(
            validate_timestamp("01:23:45,678-23:59:59,999"),
            "边界值应通过"
        );

        // 无效格式
        assert!(
            !validate_timestamp("bad-format"),
            "无效格式应失败"
        );
        assert!(
            validate_timestamp("00:00:00-00:00:07"),
            "无毫秒格式（Python 管道格式）应通过"
        );
        assert!(
            !validate_timestamp("0:00:00,600-00:00:07,559"),
            "HH 非两位应失败"
        );
        assert!(
            !validate_timestamp("00:00:00,60-00:00:07,559"),
            "毫秒非三位应失败"
        );
        assert!(
            !validate_timestamp("00:00:00,600"),
            "缺少结束时间应失败"
        );
        assert!(
            !validate_timestamp(""),
            "空字符串应失败"
        );

        // WR-01: 范围校验 — 不可能的时间值应失败
        assert!(
            !validate_timestamp("99:99:99,999-99:99:99,999"),
            "超出范围的时间值应失败"
        );
        assert!(
            !validate_timestamp("24:00:00,000-25:00:00,000"),
            "小时 > 23 应失败"
        );
        assert!(
            !validate_timestamp("00:60:00,000-00:61:00,000"),
            "分钟 > 59 应失败"
        );
        assert!(
            !validate_timestamp("00:00:60,000-00:00:61,000"),
            "秒 > 59 应失败"
        );

        // WR-02: 起止顺序校验 — 结束时间早于起始时间应失败
        assert!(
            !validate_timestamp("00:00:10,000-00:00:05,000"),
            "结束时间早于起始时间应失败"
        );
        // 起止相等应通过
        assert!(
            validate_timestamp("00:00:05,000-00:00:05,000"),
            "起止时间相等应通过"
        );

        // CR-01: Python 管道格式（无毫秒）+ 范围校验组合
        assert!(
            validate_timestamp("00:00:00-00:00:26"),
            "Python 管道标准格式应通过"
        );
        assert!(
            !validate_timestamp("99:99:99-99:99:99"),
            "Python 管道格式但超出范围应失败"
        );
        assert!(
            !validate_timestamp("00:00:10-00:00:05"),
            "Python 管道格式但结束早于起始应失败"
        );
    }

    /// Test 13: save_script 对无效脚本返回校验错误
    #[test]
    fn test_save_script_validates() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("invalid.json");

        let invalid_script = vec![ScriptClip {
            _id: 1,
            timestamp: "00:00:00,600-00:00:07,559".to_string(),
            picture: "  ".to_string(), // 空白 picture
            narration: "解说".to_string(),
            ost: OstType::NarrationOnly,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        }];

        let result = save_script(&invalid_script, &path);
        assert!(result.is_err(), "无效脚本应保存失败");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.field == "picture"),
                    "应包含 picture 校验错误"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }
}
