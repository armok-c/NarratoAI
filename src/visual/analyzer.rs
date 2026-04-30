//! 视觉分析编排器
//!
//! 编排完整的视频帧分析流水线：帧提取 → 图片 base64 编码 → LLM 视觉分析 → JSON 解析。
//! 支持批量发送、错误收集和在线护栏（D-13/D-14/D-16/D-19/D-23）。
//!
//! 核心函数：
//! - `analyze_video_frames()` — 主入口，完整编排流水线
//! - `parse_and_retry()` — JSON 反序列化含 markdown 剥离和重试
//! - `collect_frame_paths()` — 收集帧文件路径匹配 `keyframe_*.jpg`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{error, info, warn};

use crate::ffmpeg::command::ProgressCallback;
use crate::llm::provider::LlmProvider;
use crate::llm::types::LlmResponseFormat;
use crate::visual::error::VisualError;
use crate::visual::frame_extractor::extract_frames;
use crate::visual::types::{BatchAnalysisResult, FrameObservation};

// ---------------------------------------------------------------------------
// 公共 API
// ---------------------------------------------------------------------------

/// 视觉分析主入口（D-13）
///
/// 完整编排帧提取 → LLM 视觉分析 → JSON 解析流水线：
///
/// 1. 调用 `extract_frames()` 提取关键帧
/// 2. 收集帧文件路径（匹配 `keyframe_*.jpg`）
/// 3. 渲染带有 JSON schema 约束的 prompt
/// 4. 通过 `LlmProvider::analyze_images()` 批量发送给视觉 LLM
/// 5. 使用 `parse_and_retry()` 反序列化每批次 JSON 响应，收集错误
/// 6. 返回 `BatchAnalysisResult`（含观察列表、错误统计）
///
/// # 在线护栏
///
/// - **Step 3:** 0 帧检查 — 未提取到帧时返回 `VisualError::FrameExtraction`
/// - **Step 8:** 空结果屏障 — 所有批次全失败时返回 `VisualError::BatchPartial`
pub async fn analyze_video_frames(
    video_path: &Path,
    output_dir: &Path,
    llm_provider: &dyn LlmProvider,
    prompt_template: &str,
    _context: &HashMap<&str, &str>,
    batch_size: usize,
    max_concurrency: usize,
    progress: Option<ProgressCallback>,
) -> Result<BatchAnalysisResult, VisualError> {
    // Step 1 — 进度汇报
    if let Some(ref cb) = progress {
        cb(Some(0.0), "开始帧提取");
    }

    // Step 2 — 帧提取（调用方指定 output_dir, D-23）
    let frame_count = extract_frames(
        video_path,
        output_dir,
        3.0, // interval_seconds: 默认 3 秒
        None, // quality: 默认使用 extract_frames 内置默认值
        None, // progress: 帧提取进度已包含在 extract_frames 内部
        None, // cancel
    )
    .await
    .map_err(|e| {
        error!(error = %e, "帧提取失败");
        VisualError::FrameExtraction(format!("帧提取失败: {}", e))
    })?;

    // Step 3 — 空检查（在线护栏：帧数量边界检查，AI-SPEC Section 6）
    if frame_count == 0 {
        return Err(VisualError::FrameExtraction(
            "未提取到任何帧".into(),
        ));
    }

    // Step 4 — 收集帧文件路径
    let frame_paths = collect_frame_paths(output_dir)?;

    // Step 5 — 进度汇报
    if let Some(ref cb) = progress {
        cb(
            Some(0.3),
            &format!("提取到 {} 帧，开始分析", frame_count),
        );
    }

    // Step 6 — 渲染 prompt + 通过 LlmProvider 批量发送（D-19 混合模式）
    let rendered_prompt = format!(
        "{}\n\nIMPORTANT: Respond with valid JSON matching this schema: \
         {{frame_observations: [{{frame_number: u64, timestamp: string, \
         scene_description: string, objects: [string], actions: [string], \
         on_screen_text: string|null, visual_salience: f64|null}}], \
         overall_activity_summary: string}}",
        prompt_template
    );

    let raw_results = llm_provider
        .analyze_images(
            &frame_paths,
            &rendered_prompt,
            Some(
                "你是一个专业的影视解说视觉分析助手。\
                 分析视频帧并严格按照指定 JSON schema 返回结构化结果。",
            ),
            Some(batch_size),
            Some(max_concurrency),
            Some(LlmResponseFormat::Json),
            Some(0.1),  // temperature: 低温度减少幻觉
            Some(4096), // max_tokens
        )
        .await
        .map_err(|e| VisualError::Analysis(format!("LLM 分析调用失败: {}", e)))?;

    // Step 7 — 反序列化并收集错误（D-14: 收集错误继续执行）
    if let Some(ref cb) = progress {
        cb(Some(0.8), "解析分析结果");
    }

    let mut observations = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (idx, text) in raw_results.iter().enumerate() {
        match parse_and_retry(text, 2) {
            Ok(obs) => observations.push(obs),
            Err(e) => {
                let err_msg = format!("批次 {} 解析失败: {}", idx, e);
                warn!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    // Step 8 — 空结果屏障（在线护栏，AI-SPEC Section 6）
    if observations.is_empty() && !errors.is_empty() {
        return Err(VisualError::BatchPartial {
            analyzed_count: 0,
            total_count: raw_results.len(),
            errors,
        });
    }

    // Step 9 — 返回结果
    if let Some(ref cb) = progress {
        cb(Some(1.0), "视觉分析完成");
    }

    info!(
        frame_count,
        batches = raw_results.len(),
        observations_count = observations.len(),
        errors_count = errors.len(),
        "visual.analysis.completed"
    );

    Ok(BatchAnalysisResult {
        observations,
        overall_activity_summary: None,
        total_frames: frame_count as usize,
        analyzed_batches: raw_results.len(),
        errors,
    })
}

// ---------------------------------------------------------------------------
// JSON 解析与重试
// ---------------------------------------------------------------------------

/// JSON 反序列化失败时的重试策略（对齐 AI-SPEC 4b 节）
///
/// 每次迭代先防御性清理 markdown 代码块包裹（```json / ```），
/// 然后尝试 `serde_json::from_str::<FrameObservation>()`。
/// 最多重试 `max_attempts` 次，全部失败后返回 `VisualError::Analysis`。
pub fn parse_and_retry(
    json_text: &str,
    max_attempts: usize,
) -> Result<FrameObservation, VisualError> {
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        // Step 1: 防御性清理 markdown 代码块包裹
        let cleaned = json_text
            .trim()
            .strip_prefix("```json")
            .or_else(|| json_text.trim().strip_prefix("```"))
            .map(|s| s.trim_end_matches("```").trim())
            .unwrap_or(json_text.trim());

        // Step 2: 尝试反序列化
        match serde_json::from_str::<FrameObservation>(cleaned) {
            Ok(obs) => return Ok(obs),
            Err(e) => {
                warn!(
                    attempt,
                    error = %e,
                    raw_preview = &json_text[..json_text.len().min(200)],
                    "JSON 反序列化失败",
                );
                last_error = Some(e);
                if attempt >= max_attempts {
                    break;
                }
            }
        }
    }

    Err(VisualError::Analysis(format!(
        "JSON 解析失败 ({} 次尝试): {}",
        max_attempts,
        last_error.as_ref().unwrap()
    )))
}

// ---------------------------------------------------------------------------
// 帧文件收集
// ---------------------------------------------------------------------------

/// 收集指定目录下匹配 `keyframe_*.jpg` 模式的帧文件（D-24）
///
/// 按文件名字典序排序（帧序号自然排序），
/// 如果没有匹配文件则返回 `VisualError::FrameExtraction`。
fn collect_frame_paths(output_dir: &Path) -> Result<Vec<PathBuf>, VisualError> {
    let mut paths: Vec<PathBuf> = Vec::new();

    let mut dir_reader = std::fs::read_dir(output_dir).map_err(|e| {
        VisualError::FrameExtraction(format!("读取帧目录失败: {}", e))
    })?;

    while let Some(entry) = dir_reader.next() {
        let entry = entry.map_err(|e| {
            VisualError::FrameExtraction(format!("读取目录项失败: {}", e))
        })?;

        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name.starts_with("keyframe_") && file_name.ends_with(".jpg") {
            paths.push(path);
        }
    }

    if paths.is_empty() {
        return Err(VisualError::FrameExtraction(
            "没有匹配 keyframe_*.jpg 的文件".into(),
        ));
    }

    // Sort by frame number (lexical sort on zero-padded prefix)
    paths.sort();
    Ok(paths)
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: 有效 JSON 字符串应正确解析
    #[test]
    fn test_parse_and_retry_valid_json() {
        let json = r#"{
            "frame_number": 0,
            "timestamp": "00:00:00.000",
            "scene_description": "测试场景",
            "objects": ["对象1"],
            "actions": ["动作1"]
        }"#;
        let result = parse_and_retry(json, 2);
        assert!(result.is_ok(), "有效 JSON 应解析成功");
        let obs = result.unwrap();
        assert_eq!(obs.frame_number, 0);
        assert_eq!(obs.scene_description, "测试场景");
    }

    /// Test: ```json ... ``` 包裹的 JSON 应被剥离后解析
    #[test]
    fn test_parse_and_retry_code_block_wrapped() {
        let raw = "```json\n{\"frame_number\": 1, \"timestamp\": \"00:00:03.000\", \"scene_description\": \"室内\", \"objects\": [\"桌子\"], \"actions\": [\"摆放\"]}\n```";
        let result = parse_and_retry(raw, 2);
        assert!(
            result.is_ok(),
            "代码块包裹的 JSON 应被剥离后解析成功"
        );
        let obs = result.unwrap();
        assert_eq!(obs.frame_number, 1);
        assert_eq!(obs.scene_description, "室内");
    }

    /// Test: ``` ... ```（无 json 标记）也应剥离
    #[test]
    fn test_parse_and_retry_backtick_wrapped() {
        let raw = "```\n{\"frame_number\": 2, \"timestamp\": \"00:00:06.000\", \"scene_description\": \"室外\", \"objects\": [\"树\"], \"actions\": [\"摇摆\"]}\n```";
        let result = parse_and_retry(raw, 2);
        assert!(
            result.is_ok(),
            "无 json 标记的代码块包裹也应被剥离后解析成功"
        );
        let obs = result.unwrap();
        assert_eq!(obs.frame_number, 2);
        assert_eq!(obs.scene_description, "室外");
    }

    /// Test: 纯文本响应应返回 VisualError::Analysis
    #[test]
    fn test_parse_and_retry_invalid_json() {
        let raw = "这是一段纯文本响应，不是 JSON 格式";
        let result = parse_and_retry(raw, 2);
        assert!(
            result.is_err(),
            "纯文本响应应返回错误"
        );
        match result {
            Err(VisualError::Analysis(msg)) => {
                assert!(msg.contains("JSON 解析失败"), "错误消息应包含 JSON 解析失败");
                assert!(msg.contains("2 次尝试"), "错误消息应包含 2 次尝试");
            }
            _ => panic!("应返回 VisualError::Analysis"),
        }
    }

    /// Test: 验证重试次数不超过 max_attempts
    #[test]
    fn test_parse_and_retry_max_attempts() {
        let raw = "{not valid json at all}";
        let result = parse_and_retry(raw, 3);
        assert!(result.is_err(), "无效 JSON 应返回错误");
        if let Err(VisualError::Analysis(msg)) = result {
            assert!(msg.contains("3 次尝试"), "错误消息应包含 3 次尝试");
        } else {
            panic!("应返回 VisualError::Analysis");
        }
    }

    /// Test: 创建临时目录写入模拟帧文件，验证收集正确
    #[test]
    fn test_collect_frame_paths() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_collect");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        // Create mock keyframe files
        let files = [
            "keyframe_000000_000000000.jpg",
            "keyframe_000001_000003000.jpg",
            "keyframe_000002_000006000.jpg",
        ];
        for f in &files {
            std::fs::write(temp_dir.join(f), b"mock frame data").expect("应能写入测试文件");
        }

        let result = collect_frame_paths(&temp_dir);
        assert!(result.is_ok(), "应成功收集帧文件");
        let paths = result.unwrap();
        assert_eq!(paths.len(), 3, "应收集到 3 个帧文件");

        // Verify sorting order
        assert!(
            paths[0].file_name().unwrap().to_str().unwrap().contains("000000"),
            "应按帧序号排序"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Test: 空目录返回错误
    #[test]
    fn test_collect_frame_paths_empty() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_collect_empty");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        let result = collect_frame_paths(&temp_dir);
        assert!(
            result.is_err(),
            "空目录应返回错误"
        );
        if let Err(VisualError::FrameExtraction(msg)) = result {
            assert!(
                msg.contains("没有匹配 keyframe_*.jpg 的文件"),
                "错误消息应说明没有匹配文件: {}",
                msg
            );
        } else {
            panic!("应返回 VisualError::FrameExtraction");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Test: 非 keyframe 前缀的文件被过滤
    #[test]
    fn test_collect_frame_paths_filters_non_matching() {
        let temp_dir = std::env::temp_dir().join("narratoai_test_filter");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("应能创建临时目录");

        // Create keyframe and non-keyframe files
        std::fs::write(
            temp_dir.join("keyframe_000000_000000000.jpg"),
            b"valid frame",
        )
        .expect("应能写入测试文件");
        std::fs::write(temp_dir.join("fastframe_000000.jpg"), b"fast path frame")
            .expect("应能写入测试文件");
        std::fs::write(temp_dir.join("notes.txt"), b"not a frame")
            .expect("应能写入测试文件");

        let result = collect_frame_paths(&temp_dir);
        assert!(result.is_ok(), "应成功收集帧文件");
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1, "应只收集到 1 个 keyframe 文件");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
