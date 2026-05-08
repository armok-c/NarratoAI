use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::documentary::error::PipelineError;
use crate::llm::provider::LlmProvider;
use crate::llm::types::LlmResponseFormat;
use crate::script::types::ScriptClip;
use crate::visual::types::BatchAnalysisResult;

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
) -> Result<FrameAnalysisResult, PipelineError> {
    let interval = request.frame_interval.unwrap_or(3.0);

    // Create output directory for keyframes
    // On failure, clean up automatically via guard; on success, retain for user inspection
    let output_dir = request.video_path.parent()
        .unwrap_or(std::path::Path::new("."))
        .join("keyframes_cache")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&output_dir)?;
    let mut _cleanup_guard = CleanupOnDrop::new(output_dir.clone());

    emit_progress(request, 10.0, "提取关键帧");

    // Extract frames (returns (count, sorted path list))
    let (frame_count, keyframe_files) = crate::visual::frame_extractor::extract_frames(
        &request.video_path,
        &output_dir,
        interval,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| PipelineError::FrameExtraction {
        details: e.to_string(),
    })?;

    if frame_count == 0 {
        return Err(PipelineError::FrameExtraction {
            details: "未提取到任何关键帧".to_string(),
        });
    }

    emit_progress(request, 30.0, "初始化分析器");

    // extract_frames already returned the keyframe paths -- no re-scan needed

    // Analyze frames using vision LLM
    let batch_size = request.vision_batch_size.unwrap_or(5);
    let max_concurrency = request.max_concurrency.unwrap_or(2);

    let analysis_prompt = request.custom_prompt.as_deref().unwrap_or(
        "分析这些视频关键帧，描述每帧的场景、物体、动作和文字内容。以 JSON 格式返回分析结果。"
    );

    let batch_results = provider
        .analyze_images(
            &keyframe_files,
            analysis_prompt,
            None,
            Some(batch_size),
            Some(max_concurrency),
            Some(LlmResponseFormat::Json),
            Some(0.3),
            Some(4096),
            None,  // cancel: Option<CancellationToken>
        )
        .await
        .map_err(PipelineError::from)?;

    // Parse batch results
    let mut batches = Vec::new();
    for (i, result_text) in batch_results.iter().enumerate() {
        let parsed = parse_batch_response(result_text);
        batches.push(parsed);

        let pct = 40.0 + (i as f32 / batch_results.len().max(1) as f32) * 25.0;
        emit_progress(request, pct, &format!("分析批次 ({}/{})", i + 1, batch_results.len()));
    }

    let total_errors: usize = batches.iter().map(|b| b.errors.len()).sum();
    if total_errors == batches.len() && !batches.is_empty() {
        return Err(PipelineError::Llm {
            source: crate::error::LLMError::Validation(
                format!("所有批次分析均解析失败 ({} 个批次)", batches.len()),
            ),
        });
    }

    emit_progress(request, 75.0, "逐帧分析完成");

    // Save analysis artifact
    let analysis_json_path = output_dir.join("analysis.json");
    let analysis_json = serde_json::to_string_pretty(&batches)
        .map_err(|e| PipelineError::Llm {
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
            md.push_str(&format!("  - {}: {}\n", obs.timestamp, obs.scene_description));
        }
        md.push('\n');
    }

    md
}

/// 通过文本 LLM 生成解说文案
pub async fn generate_narration(
    analysis_markdown: &str,
    provider: &dyn LlmProvider,
    video_theme: Option<&str>,
) -> Result<Vec<ScriptClip>, PipelineError> {
    let mut prompt = format!(
        "根据以下视频帧分析结果，生成纪录片解说脚本。\n\n\
         每个片段应包含以下字段：\n\
         - picture: 画面描述\n\
         - narration: 解说文案\n\
         - timestamp: 时间范围，格式 HH:MM:SS-HH:MM:SS\n\
         - OST: 2 (混合模式)\n\n\
         请以 JSON 格式返回，包含一个 items 数组。\n\n\
         视频帧分析：\n{}",
        analysis_markdown
    );

    if let Some(theme) = video_theme {
        prompt.push_str(&format!("\n\n视频主题：{}", theme));
    }

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
pub async fn generate_documentary_script(
    request: ScriptGenRequest,
    provider: &dyn LlmProvider,
) -> Result<Vec<ScriptClip>, PipelineError> {
    // Step 1: Analyze video frames
    let analysis = analyze_video(&request, provider).await?;
    emit_progress(&request, 80.0, "生成解说文案");

    // Step 2: Convert analysis to markdown
    let markdown = convert_analysis_to_markdown(&analysis);

    // Step 3: Generate narration via text LLM
    let clips = generate_narration(
        &markdown,
        provider,
        request.video_theme.as_deref(),
    )
    .await?;

    emit_progress(&request, 100.0, "脚本生成完成");
    Ok(clips)
}

/// 解析批次响应为 BatchAnalysisResult
fn parse_batch_response(text: &str) -> BatchAnalysisResult {
    let cleaned = crate::visual::types::strip_code_fence(text);

    match serde_json::from_str::<serde_json::Value>(cleaned) {
        Ok(val) => {
            // Try to extract frame_observations array
            if let Some(observations) = val.get("frame_observations") {
                if let Ok(obs) = serde_json::from_value::<Vec<crate::visual::types::FrameObservation>>(observations.clone()) {
                    let summary = val.get("overall_activity_summary")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());
                    return BatchAnalysisResult {
                        observations: obs,
                        overall_activity_summary: summary,
                        total_frames: 0,
                        analyzed_batches: 1,
                        errors: vec![],
                    };
                }
            }

            // Fallback: return empty with the raw text as error
            BatchAnalysisResult {
                observations: vec![],
                overall_activity_summary: None,
                total_frames: 0,
                analyzed_batches: 1,
                errors: vec![format!("无法解析批次响应: {}", cleaned.chars().take(200).collect::<String>())],
            }
        }
        Err(e) => BatchAnalysisResult {
            observations: vec![],
            overall_activity_summary: None,
            total_frames: 0,
            analyzed_batches: 1,
            errors: vec![format!("JSON 解析失败: {}", e)],
        },
    }
}

/// 解析 LLM 返回的脚本 JSON 为 ScriptClip 列表
fn parse_script_clips(response: &str) -> Result<Vec<ScriptClip>, PipelineError> {
    let cleaned = strip_and_repair_json(response);

    let parsed: serde_json::Value = serde_json::from_str(&cleaned).map_err(|e| {
        PipelineError::Llm {
            source: crate::error::LLMError::Validation(format!("脚本 JSON 解析失败: {}", e)),
        }
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
    let repaired = text.replace(",}", "}").replace(",]", "]");
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

/// 收集 keyframe_*.jpg 文件
fn collect_keyframe_paths(dir: &std::path::Path) -> Result<Vec<PathBuf>, PipelineError> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry.map_err(|e| PipelineError::FrameExtraction {
            details: format!("读取关键帧目录失败: {}", e),
        })?;
        if entry.path().extension()
            .is_some_and(|ext| ext == "jpg" || ext == "jpeg" || ext == "png")
            && entry.file_name().to_str()
                .is_some_and(|name| name.starts_with("keyframe_"))
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_analysis_to_markdown() {
        let analysis = FrameAnalysisResult {
            analysis_json_path: PathBuf::from("analysis.json"),
            keyframe_files: vec![],
            batches: vec![
                BatchAnalysisResult {
                    observations: vec![
                        crate::visual::types::FrameObservation {
                            frame_number: 0,
                            timestamp: "00:00:00.000".to_string(),
                            scene_description: "湖面".to_string(),
                            objects: vec!["水".to_string()],
                            actions: vec![],
                            on_screen_text: None,
                            visual_salience: None,
                        },
                    ],
                    overall_activity_summary: Some("开场画面".to_string()),
                    total_frames: 1,
                    analyzed_batches: 1,
                    errors: vec![],
                },
                BatchAnalysisResult {
                    observations: vec![
                        crate::visual::types::FrameObservation {
                            frame_number: 1,
                            timestamp: "00:00:03.000".to_string(),
                            scene_description: "山峰".to_string(),
                            objects: vec!["山".to_string()],
                            actions: vec![],
                            on_screen_text: None,
                            visual_salience: None,
                        },
                    ],
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
