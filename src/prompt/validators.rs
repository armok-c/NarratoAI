use crate::prompt::error::PromptError;
use crate::prompt::types::OutputFormat;

/// 验证 LLM 输出的格式和内容（D-20）
///
/// 对齐 Python 版 `app/services/prompts/validators.py` 的 PromptOutputValidator。
///
/// 支持三种格式校验（T-04-05）：
/// - `OutputFormat::Json`：用 serde_json 验证是否为有效 JSON 对象
/// - `OutputFormat::NarrationScript`：检查解说文案长度和段落结构
/// - `OutputFormat::PlotAnalysis`：检查剧情分析内容和中文存在性
/// - `OutputFormat::Text`：检查输出是否非空
pub fn validate_output(output: &str, format: &OutputFormat) -> Result<(), PromptError> {
    match format {
        OutputFormat::Json => validate_json(output),
        OutputFormat::NarrationScript => validate_narration_script(output),
        OutputFormat::PlotAnalysis => validate_plot_analysis(output),
        OutputFormat::Text => validate_text(output),
    }
}

/// 验证文本格式输出——检查非空
fn validate_text(output: &str) -> Result<(), PromptError> {
    if output.trim().is_empty() {
        return Err(PromptError::Validation("文本格式输出为空".into()));
    }
    Ok(())
}

/// 验证 JSON 格式输出
///
/// 使用 `serde_json::from_str::<serde_json::Value>` 验证是否为有效 JSON。
/// 然后检查顶层是否为对象类型。
fn validate_json(output: &str) -> Result<(), PromptError> {
    let trimmed = output.trim();

    if trimmed.is_empty() {
        return Err(PromptError::Validation("JSON 输出为空".into()));
    }

    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| {
            PromptError::Validation(format!("JSON 格式无效: {}", e))
        })?;

    if !value.is_object() && !value.is_array() {
        return Err(PromptError::Validation(
            "JSON 顶层应为对象或数组".into(),
        ));
    }

    Ok(())
}

/// 验证解说文案格式
///
/// - 输出非空
/// - 至少包含 3 个段落（按 \n\n 分割）
/// - 输出长度 >= 50 字符
fn validate_narration_script(output: &str) -> Result<(), PromptError> {
    let normalized = output.trim().replace("\r\n", "\n");

    if normalized.is_empty() {
        return Err(PromptError::Validation("解说文案为空".into()));
    }

    if normalized.chars().count() < 50 {
        return Err(PromptError::Validation(format!(
            "解说文案过短: {} 字符（需要 >= 50）",
            normalized.chars().count()
        )));
    }

    let paragraphs: Vec<&str> = normalized
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    if paragraphs.len() < 3 {
        return Err(PromptError::Validation(format!(
            "解说文案段落数不足: {} 段（需要 >= 3）",
            paragraphs.len()
        )));
    }

    Ok(())
}

/// 验证剧情分析格式
///
/// - 输出长度 >= 100 字符
/// - 包含中文字符（Unicode 标量值 > 0x4E00）
fn validate_plot_analysis(output: &str) -> Result<(), PromptError> {
    let trimmed = output.trim();

    if trimmed.is_empty() {
        return Err(PromptError::Validation("剧情分析内容为空".into()));
    }

    if trimmed.chars().count() < 100 {
        return Err(PromptError::Validation(format!(
            "剧情分析内容不足: {} 字符（需要 >= 100）",
            trimmed.chars().count()
        )));
    }

    // 检查是否包含中文字符（覆盖 CJK Unified Ideographs、Extension A、Compatibility Ideographs）
    let has_chinese = trimmed.chars().any(|c| {
        ('\u{4E00}'..='\u{9FFF}').contains(&c)
            || ('\u{3400}'..='\u{4DBF}').contains(&c)
            || ('\u{F900}'..='\u{FAFF}').contains(&c)
    });
    if !has_chinese {
        return Err(PromptError::Validation(
            "剧情分析应包含中文内容".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === JSON 验证 ===

    #[test]
    fn test_valid_json_object_passes() {
        let result = validate_output(r#"{"key": "value", "num": 42}"#, &OutputFormat::Json);
        assert!(result.is_ok(), "有效 JSON 对象应通过: {:?}", result.err());
    }

    #[test]
    fn test_valid_json_array_passes() {
        let result = validate_output(r#"[1, 2, 3]"#, &OutputFormat::Json);
        assert!(result.is_ok(), "有效 JSON 数组应通过: {:?}", result.err());
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let result = validate_output("{invalid json}", &OutputFormat::Json);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("JSON 格式无效"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    #[test]
    fn test_empty_json_returns_error() {
        let result = validate_output("", &OutputFormat::Json);
        assert!(result.is_err());
    }

    #[test]
    fn test_primitive_json_returns_error() {
        // 42 是有效 JSON 但不是 object 或 array
        let result = validate_output("42", &OutputFormat::Json);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_json_passes() {
        let json = r#"{"level1": {"level2": [1, 2, 3]}}"#;
        let result = validate_output(json, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    // === 解说文案验证 ===

    #[test]
    fn test_valid_narration_script_passes() {
        let script = "第一段解说内容。这里详细描述画面中的场景和人物活动。\n\n\
                      第二段解说内容。继续讲述后续发展。\n\n\
                      第三段解说内容。总结本段内容并引出下一部分。";
        let result = validate_output(script, &OutputFormat::NarrationScript);
        assert!(result.is_ok(), "有效解说文案应通过: {:?}", result.err());
    }

    #[test]
    fn test_short_narration_returns_error() {
        let result = validate_output("短", &OutputFormat::NarrationScript);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("解说文案过短"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    #[test]
    fn test_empty_narration_returns_error() {
        let result = validate_output("", &OutputFormat::NarrationScript);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("解说文案为空"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    #[test]
    fn test_narration_too_few_paragraphs() {
        let script = "第一段解说内容：这段文字包含足够的字符数以满足最低长度要求。第二段内容继续描述更多细节和场景发展。第三段总结全篇内容要点。";
        let result = validate_output(script, &OutputFormat::NarrationScript);
        // The script has only 1 paragraph (no \n\n separators) so it should fail
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("段落数不足"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    // === 剧情分析验证 ===

    #[test]
    fn test_valid_plot_analysis_passes() {
        let analysis = "这是一个完整的剧情分析。第一幕展示了主人公的日常生活和内心矛盾。\
                        第二幕引入冲突事件，推动剧情发展。第三幕达到高潮，角色关系发生根本转变。\
                        结尾留下悬念，为后续发展埋下伏笔。整个故事结构完整，人物塑造丰满。";
        let result = validate_output(analysis, &OutputFormat::PlotAnalysis);
        assert!(result.is_ok(), "有效剧情分析应通过: {:?}", result.err());
    }

    #[test]
    fn test_empty_plot_analysis_returns_error() {
        let result = validate_output("", &OutputFormat::PlotAnalysis);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("剧情分析内容为空"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    #[test]
    fn test_short_plot_analysis_returns_error() {
        let result = validate_output("短内容", &OutputFormat::PlotAnalysis);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("剧情分析内容不足"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }

    #[test]
    fn test_plot_analysis_without_chinese_returns_error() {
        let analysis = "This is a plot analysis that is long enough but does not contain any Chinese characters. \
                        It should fail validation because the story analysis must contain Chinese text. \
                        This is a required field for the target audience.";
        let result = validate_output(analysis, &OutputFormat::PlotAnalysis);
        assert!(result.is_err());
        match result {
            Err(PromptError::Validation(msg)) => {
                assert!(msg.contains("中文内容"), "应包含中文提示: {}", msg);
            }
            _ => panic!("应为 Validation 错误"),
        }
    }
}
