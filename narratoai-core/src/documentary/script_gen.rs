use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::documentary::error::PipelineError;
use crate::llm::provider::LlmProvider;
use crate::llm::types::LlmResponseFormat;
use crate::prompt::manager::PromptManager;
use crate::script::types::ScriptClip;

/// 脚本生成请求参数
#[derive(Serialize, Deserialize)]
pub struct ScriptGenRequest {
    pub video_path: PathBuf,
    pub video_theme: Option<String>,
    pub custom_prompt: Option<String>,
    pub frame_interval: Option<f64>,
    pub vision_batch_size: Option<usize>,
    pub vision_model: Option<String>,
    pub text_model: Option<String>,
    pub max_concurrency: Option<usize>,
    // progress 回调不参与 IPC 序列化；Tauri 命令层在反序列化后手动注入。
    // 若序列化回 JSON，此字段会被静默丢弃——这是有意为之的设计。
    #[serde(skip)]
    pub progress: Option<Box<dyn Fn(f32, &str) + Send + Sync>>,
}

/// 帧分析结果
#[derive(Debug, Clone)]
pub struct FrameAnalysisResult {
    pub analysis_json_path: PathBuf,
    pub keyframe_files: Vec<PathBuf>,
    pub batches: Vec<crate::visual::types::BatchAnalysisResult>,
}

/// RAII guard: removes the directory on drop unless cancelled (i.e., on success)
struct CleanupOnDrop {
    path: std::path::PathBuf,
    active: bool,
}

impl CleanupOnDrop {
    fn new(path: std::path::PathBuf) -> Self {
        Self { path, active: true }
    }
    fn cancel(&mut self) {
        self.active = false;
    }
}

impl Drop for CleanupOnDrop {
    fn drop(&mut self) {
        if self.active {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

/// 分析视频帧——提取关键帧并通过视觉 LLM 分析
pub async fn analyze_video(
    request: &ScriptGenRequest,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
) -> Result<FrameAnalysisResult, PipelineError> {
    let interval = request.frame_interval.unwrap_or(3.0);

    // Create output directory for keyframes
    // On failure, clean up automatically via guard; on success, retain for user inspection
    let output_dir = request
        .video_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("keyframes_cache")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&output_dir)?;
    let mut _cleanup_guard = CleanupOnDrop::new(output_dir.clone());

    emit_progress(request, 10.0, "提取关键帧");

    emit_progress(request, 30.0, "初始化分析器");

    // Render prompt through PromptManager before entering the VisualAnalyzer facade.
    let batch_size = request.vision_batch_size.unwrap_or(5);
    let max_concurrency = request.max_concurrency.unwrap_or(2);

    let mut vars = HashMap::new();
    let video_desc = request
        .custom_prompt
        .as_deref()
        .unwrap_or("根据视频关键帧生成帧分析报告");
    vars.insert("video_description", video_desc);
    vars.insert("language", "zh-CN");
    let analysis_prompt = prompt_manager
        .render_prompt("documentary", "frame_analysis", Some("v1.0"), &vars)
        .map_err(|e| PipelineError::Llm {
            source: crate::error::LLMError::Validation(e.to_string()),
        })?;

    let analysis_result = crate::visual::analyzer::analyze_video_frames(
        &request.video_path,
        &output_dir,
        provider,
        &analysis_prompt,
        batch_size,
        max_concurrency,
        Some(interval),
        None,
        None,
        None,
    )
    .await
    .map_err(PipelineError::from)?;

    emit_progress(request, 75.0, "逐帧分析完成");
    let keyframe_files = collect_keyframe_paths(&output_dir)?;

    // Save analysis artifact
    let batches = vec![analysis_result];
    let analysis_json_path = output_dir.join("analysis.json");
    let analysis_json = serde_json::to_string_pretty(&batches).map_err(|e| PipelineError::Llm {
        source: crate::error::LLMError::Validation(format!("分析结果序列化失败: {}", e)),
    })?;
    std::fs::write(&analysis_json_path, &analysis_json)?;

    // Success — cancel cleanup so files are retained for user inspection
    _cleanup_guard.cancel();

    Ok(FrameAnalysisResult {
        analysis_json_path,
        keyframe_files,
        batches,
    })
}

fn collect_keyframe_paths(output_dir: &std::path::Path) -> Result<Vec<PathBuf>, PipelineError> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(output_dir)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name.starts_with("keyframe_") && file_name.ends_with(".jpg") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// 从帧分析结果生成 Markdown 描述
pub fn convert_analysis_to_markdown(analysis: &FrameAnalysisResult) -> String {
    let mut md = String::new();

    for (i, batch) in analysis.batches.iter().enumerate() {
        md.push_str(&format!("## 片段 {}\n", i + 1));
        if let Some(ref summary) = batch.overall_activity_summary {
            md.push_str(&format!("- 片段描述：{}\n", summary));
        }
        md.push_str("- 详细描述：\n");
        for obs in &batch.observations {
            md.push_str(&format!(
                "  - {}: {}\n",
                obs.timestamp, obs.scene_description
            ));
        }
        md.push('\n');
    }

    md
}

/// 通过文本 LLM 生成解说文案
pub async fn generate_narration(
    analysis_markdown: &str,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    video_theme: Option<&str>,
) -> Result<Vec<ScriptClip>, PipelineError> {
    let mut vars = HashMap::new();
    let title = video_theme.unwrap_or("未命名视频");
    vars.insert("video_title", title);
    vars.insert("frame_analysis_json", analysis_markdown);
    vars.insert("language", "zh-CN");
    vars.insert("style", "正式");
    let prompt = prompt_manager
        .render_prompt("documentary", "narration_generation", Some("v2.0"), &vars)
        .map_err(|e| PipelineError::Llm {
            source: crate::error::LLMError::Validation(e.to_string()),
        })?;

    let response = provider
        .generate_text(
            &prompt,
            Some("你是一个专业的纪录片解说文案撰写专家。请用中文撰写生动的解说文案。"),
            Some(0.7),
            Some(8192),
            Some(LlmResponseFormat::Json),
        )
        .await
        .map_err(PipelineError::from)?;

    parse_script_clips(&response)
}

/// 主入口：生成纪录片脚本
///
/// 接受两个独立的 LLM provider：vision_provider 用于视觉帧分析，
/// text_provider 用于文本解说生成。两者可以指向同一个 provider，
/// 也可以分别配置以获得最佳效果。
pub async fn generate_documentary_script(
    request: ScriptGenRequest,
    vision_provider: &dyn LlmProvider,
    text_provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
) -> Result<Vec<ScriptClip>, PipelineError> {
    // Step 1: Analyze video frames using vision LLM
    let analysis = analyze_video(&request, vision_provider, prompt_manager).await?;
    emit_progress(&request, 80.0, "生成解说文案");

    // Step 2: Convert analysis to markdown
    let markdown = convert_analysis_to_markdown(&analysis);

    // Step 3: Generate narration via text LLM
    let clips = generate_narration(
        &markdown,
        text_provider,
        prompt_manager,
        request.video_theme.as_deref(),
    )
    .await?;

    emit_progress(&request, 100.0, "脚本生成完成");
    Ok(clips)
}

/// 解析 LLM 返回的脚本 JSON 为 ScriptClip 列表
fn parse_script_clips(response: &str) -> Result<Vec<ScriptClip>, PipelineError> {
    let cleaned = strip_and_repair_json(response);

    let parsed: serde_json::Value =
        serde_json::from_str(&cleaned).map_err(|e| PipelineError::Llm {
            source: crate::error::LLMError::Validation(format!("脚本 JSON 解析失败: {}", e)),
        })?;

    // Extract items array
    let items = parsed
        .get("items")
        .or_else(|| parsed.get("clips"))
        .ok_or_else(|| PipelineError::Llm {
            source: crate::error::LLMError::Validation("LLM 响应中未找到 items 数组".to_string()),
        })?;

    let arr = items.as_array().ok_or_else(|| PipelineError::Llm {
        source: crate::error::LLMError::Validation("items 不是数组".to_string()),
    })?;

    let mut clips = Vec::new();
    for (i, item) in arr.iter().enumerate() {
        let mut obj = item.clone();

        // Default OST to 2 (mixed) if missing
        if obj.get("OST").is_none() && obj.get("ost").is_none() {
            obj["OST"] = serde_json::Value::Number(2.into());
        }

        match serde_json::from_value::<ScriptClip>(obj) {
            Ok(clip) => clips.push(clip),
            Err(e) => {
                tracing::warn!("跳过无效脚本片段 #{}: {}", i + 1, e);
            }
        }
    }

    if clips.is_empty() {
        return Err(PipelineError::Llm {
            source: crate::error::LLMError::Validation("未解析到有效脚本片段".to_string()),
        });
    }

    Ok(clips)
}

/// JSON 修复：剥离代码块、修复常见格式问题
fn strip_and_repair_json(input: &str) -> String {
    let text = input.trim();

    // Strip code fences
    let text = crate::visual::types::strip_code_fence(text);

    // Try direct parse first
    if serde_json::from_str::<serde_json::Value>(text).is_ok() {
        return text.to_string();
    }

    // Try extracting JSON object from text
    if let Some(json) = extract_json_object(text) {
        if serde_json::from_str::<serde_json::Value>(&json).is_ok() {
            return json;
        }
    }

    // Try trailing comma removal
    let repaired = remove_trailing_commas(text);
    if serde_json::from_str::<serde_json::Value>(&repaired).is_ok() {
        return repaired;
    }

    // Try stripping doubled outer braces {{...}} → {...}
    if text.starts_with("{{") && text.ends_with("}}") && text.len() > 4 {
        let stripped = &text[1..text.len() - 1];
        if serde_json::from_str::<serde_json::Value>(stripped).is_ok() {
            return stripped.to_string();
        }
    }

    // Fix single quotes → double quotes (only when no double quotes present)
    if !text.contains('"') && text.contains('\'') {
        let repaired = text.replace('\'', "\"");
        if serde_json::from_str::<serde_json::Value>(&repaired).is_ok() {
            return repaired;
        }
    }

    text.to_string()
}

/// 移除 JSON 结构位置的尾随逗号，不影响字符串值内的 `,}` 和 `,]`
fn remove_trailing_commas(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escape_next = false;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if escape_next {
            escape_next = false;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && in_string {
            escape_next = true;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        if !in_string && c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // 尾随逗号——跳过逗号，保留后续空白字符
                i += 1;
                continue;
            }
        }
        result.push(c);
        i += 1;
    }
    result
}

/// 从文本中提取第一个完整的 JSON 对象
fn extract_json_object(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.iter().position(|&c| c == '{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;
    for i in start..chars.len() {
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
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(chars[start..=i].iter().collect());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn emit_progress(request: &ScriptGenRequest, pct: f32, msg: &str) {
    if let Some(ref cb) = request.progress {
        cb(pct, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::types::BatchAnalysisResult;

    #[test]
    fn test_convert_analysis_to_markdown() {
        let analysis = FrameAnalysisResult {
            analysis_json_path: PathBuf::from("analysis.json"),
            keyframe_files: vec![],
            batches: vec![
                BatchAnalysisResult {
                    observations: vec![crate::visual::types::FrameObservation {
                        frame_number: 0,
                        timestamp: "00:00:00.000".to_string(),
                        scene_description: "湖面".to_string(),
                        objects: vec!["水".to_string()],
                        actions: vec![],
                        on_screen_text: None,
                        visual_salience: None,
                    }],
                    overall_activity_summary: Some("开场画面".to_string()),
                    total_frames: 1,
                    analyzed_batches: 1,
                    errors: vec![],
                },
                BatchAnalysisResult {
                    observations: vec![crate::visual::types::FrameObservation {
                        frame_number: 1,
                        timestamp: "00:00:03.000".to_string(),
                        scene_description: "山峰".to_string(),
                        objects: vec!["山".to_string()],
                        actions: vec![],
                        on_screen_text: None,
                        visual_salience: None,
                    }],
                    overall_activity_summary: Some("远景".to_string()),
                    total_frames: 1,
                    analyzed_batches: 1,
                    errors: vec![],
                },
            ],
        };

        let md = convert_analysis_to_markdown(&analysis);
        assert!(md.contains("## 片段 1"), "markdown: {}", md);
        assert!(md.contains("## 片段 2"), "markdown: {}", md);
        assert!(md.contains("开场画面"), "markdown: {}", md);
        assert!(md.contains("湖面"), "markdown: {}", md);
    }

    #[test]
    fn test_json_repair_code_fence() {
        let input = "```json\n{\"items\": []}\n```";
        let result = strip_and_repair_json(input);
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    #[test]
    fn test_json_repair_trailing_comma() {
        let input = "{\"items\": [1, 2, 3,]}";
        let result = strip_and_repair_json(input);
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    #[test]
    fn test_json_repair_trailing_comma_in_string() {
        // 字符串内的 ",}" 不应被修改
        let input = r#"{"text": "a,}b", "items": [1, 2, 3,]}"#;
        let result = strip_and_repair_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["text"], "a,}b");
        assert_eq!(parsed["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_extract_json_object() {
        let text = "Some text before {\"key\": \"value\"} some text after";
        let result = extract_json_object(text);
        assert_eq!(result, Some("{\"key\": \"value\"}".to_string()));
    }

    #[test]
    fn test_extract_json_object_nested() {
        let text = "prefix {\"a\": {\"b\": 1}} suffix";
        let result = extract_json_object(text);
        assert_eq!(result, Some("{\"a\": {\"b\": 1}}".to_string()));
    }

    #[test]
    fn test_parse_script_clips_valid() {
        let json = r#"{"items": [
            {"_id": 1, "picture": "测试", "narration": "解说", "timestamp": "00:00-00:05", "OST": 2}
        ]}"#;
        let clips = parse_script_clips(json).unwrap();
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0]._id, 1);
    }

    #[test]
    fn test_parse_script_clips_code_fence() {
        let json = "```json\n{\"items\": [{\"_id\": 1, \"picture\": \"测试\", \"narration\": \"解说\", \"timestamp\": \"00:00-00:05\", \"OST\": 2}]}\n```";
        let clips = parse_script_clips(json).unwrap();
        assert_eq!(clips.len(), 1);
    }

    #[test]
    fn test_parse_script_clips_default_ost() {
        let json = r#"{"items": [
            {"_id": 1, "picture": "测试", "narration": "解说", "timestamp": "00:00-00:05"}
        ]}"#;
        let clips = parse_script_clips(json).unwrap();
        assert_eq!(clips[0].ost, crate::script::types::OstType::Mixed);
    }

    #[test]
    fn test_parse_script_clips_empty_array() {
        let json = r#"{"items": []}"#;
        let result = parse_script_clips(json);
        assert!(result.is_err());
    }
}
