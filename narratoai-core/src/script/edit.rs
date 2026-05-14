use crate::script::error::ScriptError;
use crate::script::types::{OstType, Script, ValidationError};
use crate::script::validate_timestamp;

/// 修改指定下标的 narration，返回新的 Script 实例 (per D-15, D-16)
pub fn update_narration(script: &Script, index: usize, text: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    if text.trim().is_empty() {
        return Err(ScriptError::Validation(vec![ValidationError {
            clip_index: index,
            field: "narration".to_string(),
            message: "必须是非空字符串".to_string(),
        }]));
    }
    let mut new_script = script.clone();
    new_script[index].narration = text.to_string();
    Ok(new_script)
}

/// 修改指定下标的 OST 类型，返回新的 Script 实例 (per D-15)
pub fn set_ost(script: &Script, index: usize, ost: OstType) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    let mut new_script = script.clone();
    new_script[index].ost = ost;
    Ok(new_script)
}

/// 修改指定下标的 timestamp，返回新的 Script 实例 (per D-15)
pub fn update_timestamp(script: &Script, index: usize, ts: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    if !validate_timestamp(ts) {
        return Err(ScriptError::InvalidTimestamp(ts.to_string()));
    }
    let mut new_script = script.clone();
    new_script[index].timestamp = ts.to_string();
    Ok(new_script)
}

/// 修改指定下标的 picture，返回新的 Script 实例 (per D-15)
pub fn update_picture(script: &Script, index: usize, pic: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    if pic.trim().is_empty() {
        return Err(ScriptError::Validation(vec![ValidationError {
            clip_index: index,
            field: "picture".to_string(),
            message: "必须是非空字符串".to_string(),
        }]));
    }
    let mut new_script = script.clone();
    new_script[index].picture = pic.to_string();
    Ok(new_script)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::{OstType, ScriptClip};

    fn make_test_script() -> Script {
        vec![
            ScriptClip {
                _id: 1,
                timestamp: "00:00:00,000-00:00:10,000".to_string(),
                picture: "画面一".to_string(),
                narration: "解说文案一".to_string(),
                ost: OstType::NarrationOnly,
                duration: None,
                source_time_range: None,
                edited_time_range: None,
                audio: None,
                video: None,
                subtitle: None,
            },
            ScriptClip {
                _id: 2,
                timestamp: "00:00:10,000-00:00:20,000".to_string(),
                picture: "画面二".to_string(),
                narration: "播放原片2".to_string(),
                ost: OstType::OriginalSound,
                duration: Some(10.0),
                source_time_range: None,
                edited_time_range: None,
                audio: None,
                video: None,
                subtitle: None,
            },
        ]
    }

    /// Test 1: update_narration 返回新 Script，指定下标 narration 已更新
    #[test]
    fn test_update_narration_success() {
        let script = make_test_script();
        let result = update_narration(&script, 0, "新文案").expect("应成功");
        assert_eq!(result[0].narration, "新文案");
    }

    /// Test 2: update_narration 索引越界返回 IndexOutOfBounds
    #[test]
    fn test_update_narration_out_of_bounds() {
        let script = make_test_script();
        let result = update_narration(&script, 999, "text");
        assert!(result.is_err());
        match result {
            Err(ScriptError::IndexOutOfBounds) => { /* 正确 */ }
            Err(e) => panic!("应为 IndexOutOfBounds，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 3: set_ost 返回新 Script，指定下标 OST 已更新
    #[test]
    fn test_set_ost_success() {
        let script = make_test_script();
        let result = set_ost(&script, 0, OstType::OriginalSound).expect("应成功");
        assert_eq!(result[0].ost, OstType::OriginalSound);
    }

    /// Test 4: set_ost 索引越界返回 IndexOutOfBounds
    #[test]
    fn test_set_ost_out_of_bounds() {
        let script = make_test_script();
        let result = set_ost(&script, 999, OstType::Mixed);
        assert!(result.is_err());
        match result {
            Err(ScriptError::IndexOutOfBounds) => { /* 正确 */ }
            Err(e) => panic!("应为 IndexOutOfBounds，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 5: update_timestamp 返回新 Script，指定下标 timestamp 已更新
    #[test]
    fn test_update_timestamp_success() {
        let script = make_test_script();
        let result = update_timestamp(&script, 0, "00:01:00,000-00:02:00,000").expect("应成功");
        assert_eq!(result[0].timestamp, "00:01:00,000-00:02:00,000");
    }

    /// Test 6: update_timestamp 索引越界返回 IndexOutOfBounds
    #[test]
    fn test_update_timestamp_out_of_bounds() {
        let script = make_test_script();
        let result = update_timestamp(&script, 999, "...");
        assert!(result.is_err());
        match result {
            Err(ScriptError::IndexOutOfBounds) => { /* 正确 */ }
            Err(e) => panic!("应为 IndexOutOfBounds，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 7: update_picture 返回新 Script，指定下标 picture 已更新
    #[test]
    fn test_update_picture_success() {
        let script = make_test_script();
        let result = update_picture(&script, 0, "新画面描述").expect("应成功");
        assert_eq!(result[0].picture, "新画面描述");
    }

    /// Test 8: update_picture 索引越界返回 IndexOutOfBounds
    #[test]
    fn test_update_picture_out_of_bounds() {
        let script = make_test_script();
        let result = update_picture(&script, 999, "...");
        assert!(result.is_err());
        match result {
            Err(ScriptError::IndexOutOfBounds) => { /* 正确 */ }
            Err(e) => panic!("应为 IndexOutOfBounds，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 9: 连续编辑——先 update_narration 再 set_ost，两个修改都保留
    #[test]
    fn test_chained_edits() {
        let script = make_test_script();
        let step1 = update_narration(&script, 0, "链式新文案").expect("第一步应成功");
        let step2 = set_ost(&step1, 0, OstType::Mixed).expect("第二步应成功");

        assert_eq!(step2[0].narration, "链式新文案", "第一个修改应保留");
        assert_eq!(step2[0].ost, OstType::Mixed, "第二个修改应生效");
    }

    /// Test 10a: update_timestamp 拒绝无效时间戳格式
    #[test]
    fn test_update_timestamp_invalid_format() {
        let script = make_test_script();
        let result = update_timestamp(&script, 0, "bad-format");
        assert!(result.is_err(), "无效时间戳应被拒绝");
        match result {
            Err(ScriptError::InvalidTimestamp(ts)) => {
                assert_eq!(ts, "bad-format", "错误消息应包含原始值");
            }
            Err(e) => panic!("应为 InvalidTimestamp，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 10b: update_timestamp 拒绝超出范围的时间值
    #[test]
    fn test_update_timestamp_out_of_range() {
        let script = make_test_script();
        let result = update_timestamp(&script, 0, "99:99:99,999-99:99:99,999");
        assert!(result.is_err(), "超出范围的时间值应被拒绝");
    }

    /// Test 10c: update_narration 拒绝空字符串
    #[test]
    fn test_update_narration_empty_rejected() {
        let script = make_test_script();
        let result = update_narration(&script, 0, "");
        assert!(result.is_err(), "空 narration 应被拒绝");
        match result {
            Err(ScriptError::Validation(errors)) => {
                assert!(
                    errors.iter().any(|e| e.field == "narration"),
                    "应包含 narration 校验错误"
                );
            }
            Err(e) => panic!("应为 Validation 错误，得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    /// Test 10d: update_narration 拒绝纯空白字符串
    #[test]
    fn test_update_narration_whitespace_rejected() {
        let script = make_test_script();
        let result = update_narration(&script, 0, "   ");
        assert!(result.is_err(), "纯空白 narration 应被拒绝");
    }

    /// Test 10e: update_picture 拒绝空字符串
    #[test]
    fn test_update_picture_empty_rejected() {
        let script = make_test_script();
        let result = update_picture(&script, 0, "");
        assert!(result.is_err(), "空 picture 应被拒绝");
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

    /// Test 10f: update_picture 拒绝纯空白字符串
    #[test]
    fn test_update_picture_whitespace_rejected() {
        let script = make_test_script();
        let result = update_picture(&script, 0, "   ");
        assert!(result.is_err(), "纯空白 picture 应被拒绝");
    }

    /// Test 11: 不可变性验证——编辑后原始 Script 的所有字段值不变
    #[test]
    fn test_immutability() {
        let script = make_test_script();
        let original_narration = script[0].narration.clone();
        let original_ost = script[0].ost;
        let original_timestamp = script[0].timestamp.clone();
        let original_picture = script[0].picture.clone();

        let _ = update_narration(&script, 0, "修改后文案");
        let _ = set_ost(&script, 0, OstType::Mixed);
        let _ = update_timestamp(&script, 0, "00:00:00,000-00:00:00,001");
        let _ = update_picture(&script, 0, "修改后画面");

        assert_eq!(
            script[0].narration, original_narration,
            "原始 narration 不应变"
        );
        assert_eq!(script[0].ost, original_ost, "原始 ost 不应变");
        assert_eq!(
            script[0].timestamp, original_timestamp,
            "原始 timestamp 不应变"
        );
        assert_eq!(script[0].picture, original_picture, "原始 picture 不应变");
    }
}
