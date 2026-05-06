use std::collections::HashMap;
use std::path::Path;

use crate::llm::provider::LlmProvider;
use crate::llm::types::LlmResponseFormat;
use crate::prompt::manager::PromptManager;
use crate::sde::error::SdeError;
use crate::sde::types::SdePipelineState;
use crate::script::types::OstType;
use crate::script::types::Script;

/// 检查文本中是否包含 SRT 时间戳模式
#[cfg(test)]
fn has_timecodes(text: &str) -> bool {
    text.contains("-->") || text.contains(':')
}

/// 步骤 2a：分析字幕剧情（D-14, D-15, D-18, D-19, D-20）
///
/// 1. 渲染 prompt（short_drama_narration/plot_analysis v1.0）
/// 2. 调用 LLM 生成剧情分析文本
/// 3. 保存中间产物 plot_analysis.txt
pub async fn step_analyze_plot(
    state: &mut SdePipelineState,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<(), SdeError> {
    let mut vars = HashMap::new();
    vars.insert("subtitle_content", &state.subtitle_text as &str);

    let prompt = prompt_manager
        .render_prompt(
            "short_drama_narration",
            "plot_analysis",
            Some("v1.0"),
            &vars,
        )
        .map_err(|e| SdeError::PlotAnalysis {
            details: format!("渲染 prompt 失败: {}", e),
        })?;

    let result = provider
        .generate_text(
            &prompt,
            Some("你是一位专业的剧本分析师和剧情概括助手。"),
            Some(temperature as f32),
            Some(4096),
            None,
        )
        .await
        .map_err(|e| SdeError::PlotAnalysis {
            details: format!("LLM 调用失败: {}", e),
        })?;

    // 保存中间产物
    let plot_path = state.task_dir.join("plot_analysis.txt");
    tokio::fs::write(&plot_path, &result)
        .await
        .map_err(|e| SdeError::Io { source: e })?;

    state.plot_analysis = result;
    Ok(())
}

/// 步骤 2b：生成解说脚本（D-14, D-15, D-16, D-18, D-19, D-20）
///
/// 1. 渲染 prompt（short_drama_narration/script_generation v2.0）
/// 2. 调用 LLM 生成 JSON 格式的脚本
/// 3. 保存中间产物 narration_raw.json
pub async fn step_generate_script(
    state: &mut SdePipelineState,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
    drama_name: &str,
    subtitle_text: &str,
) -> Result<(), SdeError> {
    let mut vars = HashMap::new();
    vars.insert("drama_name", drama_name);
    vars.insert("plot_analysis", &state.plot_analysis as &str);
    vars.insert("subtitle_content", subtitle_text);

    let prompt = prompt_manager
        .render_prompt(
            "short_drama_narration",
            "script_generation",
            Some("v2.0"),
            &vars,
        )
        .map_err(|e| SdeError::ScriptGeneration {
            details: format!("渲染 prompt 失败: {}", e),
        })?;

    let result = provider
        .generate_text(
            &prompt,
            Some("你是一位顶级的短剧解说up主，严格按JSON格式输出。"),
            Some(temperature as f32),
            Some(8192),
            Some(LlmResponseFormat::Json),
        )
        .await
        .map_err(|e| SdeError::ScriptGeneration {
            details: format!("LLM 调用失败: {}", e),
        })?;

    // 保存中间产物
    let raw_path = state.task_dir.join("narration_raw.json");
    tokio::fs::write(&raw_path, &result)
        .await
        .map_err(|e| SdeError::Io { source: e })?;

    state.narration_raw = result;
    Ok(())
}

/// 6 步 JSON 修复回退策略，对齐 Python ref generate_short_summary.py parse_and_fix_json。
///
/// 1. 直接解析 → 成功则返回原文本
/// 2. 代码块提取：去掉 ```json / ``` 标记后重试
/// 3. 首对象提取：找到第一个 { 或 [ 到匹配的 } 或 ]
/// 4. 双大括号修复：{{...}} → {...}
/// 5. 尾逗号移除：",}" → "}" 和 ",]" → "]"
/// 6. 单引号转双引号：仅当文本不包含双引号时
///
/// 全部失败 → 返回原始字符串（调用方 parse_script 负责清晰报错）
pub fn repair_json(raw: &str) -> String {
    let text = raw.trim();

    if text.is_empty() {
        return String::new();
    }

    // Step 1: Direct parse
    if serde_json::from_str::<serde_json::Value>(text).is_ok() {
        return text.to_string();
    }

    // Step 2: Code block extraction
    let without_fence = strip_code_fence(text);
    if without_fence != text {
        if serde_json::from_str::<serde_json::Value>(&without_fence).is_ok() {
            return without_fence;
        }
        return apply_repair_steps(without_fence);
    }

    // No code fence found, try steps 3-6 on original text
    apply_repair_steps(text.to_string())
}

/// 应用 JSON 修复步骤 3-6（步骤 3: 首对象提取, 4: 双大括号, 5: 尾逗号, 6: 单引号）
fn apply_repair_steps(input: String) -> String {
    let mut result = input;

    // Step 3: First JSON object extraction
    if let Some(extracted) = extract_first_json_object(&result) {
        if serde_json::from_str::<serde_json::Value>(&extracted).is_ok() {
            return extracted;
        }
        result = extracted;
    }

    // Step 4: Double braces
    let fixed_braces = fix_double_braces(&result);
    if serde_json::from_str::<serde_json::Value>(&fixed_braces).is_ok() {
        return fixed_braces;
    }
    result = fixed_braces;

    // Step 5: Trailing comma
    let fixed_commas = fix_trailing_commas(&result);
    if serde_json::from_str::<serde_json::Value>(&fixed_commas).is_ok() {
        return fixed_commas;
    }
    result = fixed_commas;

    // Step 6: Single quotes to double quotes
    if !result.contains('"') && result.contains('\'') {
        let fixed = result.replace('\'', "\"");
        if serde_json::from_str::<serde_json::Value>(&fixed).is_ok() {
            return fixed;
        }
    }

    result
}

/// 剥离代码块标记 ```json ... ```
fn strip_code_fence(text: &str) -> String {
    let text = text.trim();
    // Match ```json ... ``` pattern
    if let Some(start) = text.find("```") {
        let after_start = &text[start + 3..];
        // Skip optional json/json5 tag
        let after_tag = if after_start.starts_with("json5") {
            &after_start[5..]
        } else if after_start.starts_with("json") {
            &after_start[4..]
        } else {
            after_start
        };
        if let Some(end) = after_tag.rfind("```") {
            return after_tag[..end].trim().to_string();
        }
        // No closing fence — return content after the opening fence
        return after_tag.trim().to_string();
    }
    text.to_string()
}

/// 修复双大括号 {{...}} → {...}
fn fix_double_braces(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with("{{") && trimmed.ends_with("}}") && trimmed.len() > 4 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

/// 修复尾逗号：",}" → "}" 和 ",]" → "]"
fn fix_trailing_commas(text: &str) -> String {
    text.replace(",}", "}").replace(",]", "]")
}

/// 从文本中提取第一个完整的 JSON 对象（支持 { } 和 [ ]）
fn extract_first_json_object(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();

    // Find first { or [
    let start_pos = chars.iter().position(|&c| c == '{' || c == '[')?;
    let opening = chars[start_pos];
    let closing = match opening {
        '{' => '}',
        '[' => ']',
        _ => unreachable!(),
    };

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for i in start_pos..chars.len() {
        let c = chars[i];

        if escape_next {
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            continue;
        }

        if !in_string {
            if c == opening {
                depth += 1;
            } else if c == closing {
                depth -= 1;
                if depth == 0 {
                    let result: String = chars[start_pos..=i].iter().collect();
                    return Some(result);
                }
            }
        }
    }

    None
}

/// 将修复后的 JSON 解析为 Script
///
/// 1. 调用 repair_json(raw_json)
/// 2. serde_json::from_str 解析为 Value
/// 3. 获取数组（顶层数组或 items/clips 字段）
/// 4. 逐项反序列化为 ScriptClip（缺失 OST 默认 0，无效项跳过）
/// 5. 通过 crate::script::validate 校验
/// 6. 保存最终脚本到 task_dir/script_final.json
pub fn parse_script(raw_json: &str, task_dir: &Path) -> Result<Script, SdeError> {
    let repaired = repair_json(raw_json);

    let value: serde_json::Value = serde_json::from_str(&repaired).map_err(|e| {
        SdeError::JsonRepair {
            details: format!("JSON 解析失败: {}", e),
        }
    })?;

    // 获取数组：顶层数组或 items/clips 字段
    let items = value
        .as_array()
        .or_else(|| value.get("items").and_then(|v| v.as_array()))
        .or_else(|| value.get("clips").and_then(|v| v.as_array()))
        .ok_or_else(|| SdeError::JsonRepair {
            details: "JSON 中未找到有效数组（顶层数组、items 或 clips 字段）".into(),
        })?;

    // 逐项反序列化
    let mut clips = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let mut obj = item.clone();

        // 缺失 OST 字段 → 默认 OST=0 (NarrationOnly)
        if obj.get("OST").is_none() && obj.get("ost").is_none() {
            if let Some(val) = obj.as_object_mut() {
                val.insert(
                    "OST".to_string(),
                    serde_json::Value::Number(0.into()),
                );
            }
        }

        match serde_json::from_value::<crate::script::types::ScriptClip>(obj) {
            Ok(clip) => clips.push(clip),
            Err(e) => {
                tracing::warn!("跳过无效脚本片段 #{}: {}", i + 1, e);
            }
        }
    }

    // 空数组 → 错误
    if clips.is_empty() {
        return Err(SdeError::ScriptGeneration {
            details: "未解析到有效的脚本片段".into(),
        });
    }

    // 通过 validate 校验
    crate::script::validate(&clips).map_err(|e| SdeError::Validation {
        details: format!("脚本校验失败: {}", e),
    })?;

    // 保存最终脚本
    let script_path = task_dir.join("script_final.json");
    let json_str = serde_json::to_string_pretty(&clips).map_err(|e| SdeError::JsonRepair {
        details: format!("序列化脚本失败: {}", e),
    })?;
    std::fs::write(&script_path, &json_str).map_err(|e| SdeError::Io { source: e })?;

    Ok(clips)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== repair_json tests ==========

    #[test]
    fn test_repair_json_direct() {
        let input = r#"{"items": [{"_id": 1, "picture": "test", "narration": "test", "timestamp": "00:00:00,000-00:00:05,000", "OST": 0}]}"#;
        let result = repair_json(input);
        assert_eq!(result, input, "valid JSON should pass through unchanged");
    }

    #[test]
    fn test_repair_json_code_fence() {
        let input = "```json\n{\"items\": [{\"_id\": 1, \"picture\": \"test\", \"narration\": \"test\", \"timestamp\": \"00:00-00:05\", \"OST\": 0}]}\n```";
        let result = repair_json(input);
        assert!(
            serde_json::from_str::<serde_json::Value>(&result).is_ok(),
            "code fence should be stripped: {}",
            result
        );
        assert!(!result.contains("```"), "should not contain backticks");
    }

    #[test]
    fn test_repair_json_double_braces() {
        let input = "{{\"items\": []}}";
        let result = repair_json(input);
        assert_eq!(result, "{\"items\": []}", "double braces should be reduced");
    }

    #[test]
    fn test_repair_json_trailing_comma() {
        let input = r#"{"items": [{"_id": 1, "picture": "a", "narration": "b", "timestamp": "00:00-00:05", "OST": 0,}]}"#;
        let result = repair_json(input);
        assert!(
            serde_json::from_str::<serde_json::Value>(&result).is_ok(),
            "trailing comma should be removed: {}",
            result
        );
        assert!(!result.contains(",]"), "should not contain trailing comma in array");
    }

    #[test]
    fn test_repair_json_single_quotes() {
        let input = "{'items': [{'id': 1}]}";
        let result = repair_json(input);
        assert!(
            serde_json::from_str::<serde_json::Value>(&result).is_ok(),
            "single quotes should be replaced: {}",
            result
        );
        assert!(!result.contains('\''), "should not contain single quotes");
    }

    #[test]
    fn test_repair_json_all_fail() {
        let input = "completely invalid json {{{";
        let result = repair_json(input);
        // Should return the original string
        assert_eq!(result, input);
    }

    #[test]
    fn test_repair_json_empty() {
        let result = repair_json("");
        assert_eq!(result, "", "empty input should return empty");
    }

    // ========== extract_first_json_object tests ==========

    #[test]
    fn test_extract_first_json_object_braces() {
        let text = "prefix {\"key\": \"value\"} suffix";
        let result = extract_first_json_object(text);
        assert_eq!(result, Some("{\"key\": \"value\"}".to_string()));
    }

    #[test]
    fn test_extract_first_json_object_brackets() {
        let text = "prefix [1, 2, 3] suffix";
        let result = extract_first_json_object(text);
        assert_eq!(result, Some("[1, 2, 3]".to_string()));
    }

    #[test]
    fn test_extract_first_json_object_nested() {
        let text = "prefix {\"a\": {\"b\": 1}} suffix";
        let result = extract_first_json_object(text);
        assert_eq!(result, Some("{\"a\": {\"b\": 1}}".to_string()));
    }

    #[test]
    fn test_extract_first_json_object_string_with_braces() {
        let text = r#"prefix {"a": "text with {braces}"} suffix"#;
        let result = extract_first_json_object(text);
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("text with {braces}"));
    }

    #[test]
    fn test_extract_first_json_object_no_json() {
        let text = "plain text without braces";
        let result = extract_first_json_object(text);
        assert!(result.is_none());
    }

    // ========== parse_script tests ==========

    fn make_items_json(items: &str) -> String {
        format!(r#"{{"items": {}}}"#, items)
    }

    #[test]
    fn test_parse_script_valid_items() {
        let json = make_items_json(r#"[{"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559", "OST": 0}]"#);
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = parse_script(&json, dir.path());
        assert!(result.is_ok(), "valid items should parse: {:?}", result.err());
        let clips = result.unwrap();
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0]._id, 1);
        assert_eq!(clips[0].ost, OstType::NarrationOnly);
    }

    #[test]
    fn test_parse_script_top_level_array() {
        let json = r#"[{"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559", "OST": 0}]"#;
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = parse_script(json, dir.path());
        assert!(result.is_ok(), "top-level array should parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_script_missing_ost_default() {
        let json = r#"{"items": [{"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559"}]}"#;
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = parse_script(json, dir.path()).expect("missing OST should default");
        assert_eq!(clips[0].ost, OstType::NarrationOnly, "missing OST should default to 0");
    }

    #[test]
    fn test_parse_script_empty_clips() {
        let json = r#"{"items": []}"#;
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = parse_script(json, dir.path());
        assert!(result.is_err(), "empty clips should error");
        match result {
            Err(SdeError::ScriptGeneration { .. }) => {}
            _ => panic!("Expected ScriptGeneration error, got: {:?}", result),
        }
    }

    #[test]
    fn test_parse_script_invalid_clip_skipped() {
        let json = r#"{"items": [
            {"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559", "OST": 0},
            {"_id": "bad", "picture": "画面2", "narration": "解说2", "timestamp": "00:00-00:05", "OST": 0}
        ]}"#;
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = parse_script(json, dir.path()).expect("invalid clips should be skipped");
        assert_eq!(clips.len(), 1, "only valid clip should remain");
        assert_eq!(clips[0]._id, 1);
    }

    #[test]
    fn test_parse_script_invalid_json() {
        let json = "not json at all";
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = parse_script(json, dir.path());
        assert!(result.is_err(), "invalid JSON should error");
        match result {
            Err(SdeError::JsonRepair { .. }) => {}
            _ => panic!("Expected JsonRepair error, got: {:?}", result),
        }
    }

    #[test]
    fn test_parse_script_saves_file() {
        let json = make_items_json(r#"[{"_id": 1, "picture": "画面1", "narration": "解说1", "timestamp": "00:00:00,600-00:00:07,559", "OST": 0}]"#);
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let _clips = parse_script(&json, dir.path()).expect("parse should succeed");
        let script_path = dir.path().join("script_final.json");
        assert!(script_path.exists(), "script_final.json should be saved");
        let content = std::fs::read_to_string(&script_path).expect("read saved file");
        assert!(content.contains("解说1"), "saved file should contain narration: {}", content);
    }

    // ========== strip_code_fence tests ==========

    #[test]
    fn test_strip_code_fence_standard() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        let result = strip_code_fence(input);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_strip_code_fence_no_tag() {
        let input = "```\n{\"key\": \"value\"}\n```";
        let result = strip_code_fence(input);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_strip_code_fence_no_fence() {
        let input = "plain text";
        let result = strip_code_fence(input);
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_strip_code_fence_open_only() {
        let input = "```json\n{\"key\": \"value\"}";
        let result = strip_code_fence(input);
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    // ========== fix_double_braces tests ==========

    #[test]
    fn test_fix_double_braces_valid() {
        assert_eq!(fix_double_braces("{{\"key\": 1}}"), "{\"key\": 1}");
    }

    #[test]
    fn test_fix_double_braces_not_double() {
        assert_eq!(fix_double_braces("{\"key\": 1}"), "{\"key\": 1}");
    }

    // ========== fix_trailing_commas tests ==========

    #[test]
    fn test_fix_trailing_commas_object() {
        assert_eq!(fix_trailing_commas("{\"a\": 1,}"), "{\"a\": 1}");
    }

    #[test]
    fn test_fix_trailing_commas_array() {
        assert_eq!(fix_trailing_commas("[1, 2,]"), "[1, 2]");
    }

    // ========== has_timecodes test ==========

    #[test]
    fn test_has_timecodes() {
        assert!(has_timecodes("00:00:01,000 --> 00:00:05,000"));
        assert!(has_timecodes("contains --> arrow"));
        assert!(!has_timecodes("plain text without timecodes"));
    }
}
