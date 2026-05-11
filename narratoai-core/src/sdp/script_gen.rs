use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::llm::provider::LlmProvider;
use crate::prompt::manager::PromptManager;
use crate::sdp::error::SdpError;
use crate::subtitle::parser::parse_subtitle_file;

// ---------------------------------------------------------------------------
// Internal data models (for deserializing LLM JSON output)
// ---------------------------------------------------------------------------

/// 第一步 LLM 输出：字幕分析结果
#[derive(Deserialize)]
#[allow(dead_code)]
struct SubtitleAnalysis {
    summary: String,
    narrative_structure: Option<serde_json::Value>,
    plot_titles: Vec<String>,
    plot_connections: Option<Vec<String>>,
    analysis_details: Option<serde_json::Value>,
}

/// 第二步 LLM 输出顶层包装
#[derive(Deserialize)]
#[allow(dead_code)]
struct PlotPointsData {
    plot_points: Vec<PlotPoint>,
    editing_notes: Option<serde_json::Value>,
}

/// 单个情节点——对应一个 ScriptClip
#[derive(Deserialize)]
#[allow(dead_code)]
struct PlotPoint {
    sequence: i64,
    timestamp: String,
    title: String,
    narrative_function: Option<String>,
    picture: String,
    context_before: Option<String>,
    context_after: Option<String>,
    transition_note: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// 将 plot_titles Vec<String> 格式化为编号列表文本
fn format_plot_titles(titles: &[String]) -> String {
    let mut result = String::new();
    for (i, title) in titles.iter().enumerate() {
        result.push_str(&format!("{}. {}\n", i + 1, title));
    }
    result.trim().to_string()
}

/// 生成 narration 占位值，对齐 Python: `f"播放原生_{os.urandom(4).hex()}"`
fn generate_narration_placeholder() -> String {
    let uuid = uuid::Uuid::new_v4();
    let bytes: [u8; 4] = uuid.as_bytes()[..4].try_into().unwrap();
    format!("播放原生_{}", hex::encode(bytes))
}

// ---------------------------------------------------------------------------
// Core: merge_script
// ---------------------------------------------------------------------------

/// 将 plot_points 合并为 Vec<ScriptClip>
///
/// 对齐 Python `step5_merge_script.py`：
/// - 每个 plot_point → ScriptClip
/// - 全部 OST=1 (OriginalSound)
/// - narration 为占位值 "播放原生_xxxx"
/// - 保存到 task_dir/merged_subtitle.json
fn merge_script(
    plot_points: &[PlotPoint],
    task_dir: &Path,
) -> Result<crate::script::types::Script, SdpError> {
    if plot_points.is_empty() {
        return Err(SdpError::Validation {
            details: "plot_points 为空，无法生成脚本".into(),
        });
    }

    let mut clips = Vec::with_capacity(plot_points.len());
    for point in plot_points {
        let clip = crate::script::types::ScriptClip {
            _id: point.sequence,
            timestamp: point.timestamp.clone(),
            picture: point.picture.clone(),
            narration: generate_narration_placeholder(),
            ost: crate::script::types::OstType::OriginalSound,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };
        clips.push(clip);
    }

    // 校验
    crate::script::validate(&clips).map_err(|e| SdpError::Validation {
        details: format!("脚本校验失败: {}", e),
    })?;

    // 保存
    let path = task_dir.join("merged_subtitle.json");
    crate::script::save_script(&clips, &path).map_err(|e| match e {
        crate::script::error::ScriptError::Validation(errors) => SdpError::Validation {
            details: format!("脚本校验失败: {:?}", errors),
        },
        crate::script::error::ScriptError::Io(io) => SdpError::Io { source: io },
        crate::script::error::ScriptError::JsonParse(je) => SdpError::JsonRepair {
            details: format!("序列化失败: {}", je),
        },
        _ => SdpError::Io {
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        },
    })?;

    Ok(clips)
}

// ---------------------------------------------------------------------------
// Core: generate_sdp_script
// ---------------------------------------------------------------------------

/// SDP 脚本生成主入口
///
/// 编排两步 LLM 调用（D-10）：
/// 1. subtitle_analysis prompt → 分析叙事结构，提取 plot_titles + summary
/// 2. plot_extraction prompt → 基于第一步结果定位具体时间段
/// 然后 merge_script() 将 plot_points 合并为 Vec<ScriptClip>
///
/// # 参数
///
/// * `subtitle_path` — 字幕文件路径（SRT 或 ASS 格式）
/// * `provider` — LLM provider 引用
/// * `prompt_manager` — Prompt 管理器引用
/// * `task_dir` — 工作目录（中间产物保存于此）
/// * `temperature` — LLM temperature（默认 0.1）
/// * `custom_clips` — 建议片段数量（默认 5）
pub async fn generate_sdp_script(
    subtitle_path: &Path,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    task_dir: &Path,
    temperature: f64,
    custom_clips: u32,
) -> Result<crate::script::types::Script, SdpError> {
    // ---- 1. 解析字幕文件 ----
    let subtitle_path_owned = subtitle_path.to_path_buf();
    let (segments, normalized_text, _encoding) = tokio::task::spawn_blocking(move || {
        parse_subtitle_file(&subtitle_path_owned)
    })
    .await
    .map_err(|e| SdpError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
    })?
    .map_err(SdpError::from)?;

    // ---- 2. 提取纯对话文本（去除结构信息） ----
    let subtitle_plain = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<&str>>()
        .join("\n");

    // ---- 3. 校验字幕内容 ----
    if subtitle_plain.trim().is_empty() {
        return Err(SdpError::ParseSubtitle {
            details: "字幕内容为空，无法生成脚本".into(),
        });
    }

    // ---- 4. 保存字幕原文 ----
    let source_path = task_dir.join("subtitle_source.txt");
    tokio::fs::write(&source_path, &normalized_text).await?;

    // ---- 5. 第一步 LLM：subtitle_analysis ----
    let custom_clips_str = custom_clips.to_string();

    let mut vars_step1 = HashMap::new();
    vars_step1.insert("subtitle_content", subtitle_plain.as_str());
    vars_step1.insert("custom_clips", custom_clips_str.as_str());

    let analysis_prompt = prompt_manager
        .render_prompt(
            "short_drama_editing",
            "subtitle_analysis",
            Some("v2.0"),
            &vars_step1,
        )
        .map_err(|e| SdpError::LlmError {
            details: format!("渲染 subtitle_analysis prompt 失败: {}", e),
        })?;

    let analysis_raw = provider
        .generate_text(
            &analysis_prompt,
            Some(
                "你是一名资深短剧编剧和剪辑师，精通叙事结构和剧情节奏把控，\
                 擅长从字幕中提取能够形成完整故事线的关键情节。",
            ),
            Some(temperature as f32),
            Some(4000),
            None,
        )
        .await
        .map_err(|e| SdpError::LlmError {
            details: format!("subtitle_analysis LLM 调用失败: {}", e),
        })?;

    // ---- 6. 修复 + 解析 SubtitleAnalysis JSON ----
    let analysis_repaired = crate::sde::script_gen::repair_json(&analysis_raw);
    let analysis: SubtitleAnalysis = serde_json::from_str(&analysis_repaired).map_err(|e| {
        SdpError::JsonRepair {
            details: format!("解析 subtitle_analysis JSON 失败: {}", e),
        }
    })?;

    // ---- 7. 保存中间产物 ----
    let analysis_path = task_dir.join("subtitle_analysis.json");
    tokio::fs::write(&analysis_path, &analysis_repaired).await?;

    // ---- 8. 校验分析结果 ----
    if analysis.plot_titles.is_empty() {
        return Err(SdpError::LlmError {
            details: "第一步字幕分析未产生任何情节点（plot_titles 为空）".into(),
        });
    }
    if analysis.summary.trim().is_empty() {
        return Err(SdpError::LlmError {
            details: "第一步字幕分析未产生剧情梗概（summary 为空）".into(),
        });
    }

    // ---- 9. 构建 plot_titles_text ----
    let plot_titles_text = format_plot_titles(&analysis.plot_titles);

    // ---- 10. 第二步 LLM：plot_extraction ----
    let mut vars_step2 = HashMap::new();
    vars_step2.insert("subtitle_content", normalized_text.as_str());
    vars_step2.insert("plot_summary", analysis.summary.as_str());
    vars_step2.insert("plot_titles", plot_titles_text.as_str());

    let extraction_prompt = prompt_manager
        .render_prompt(
            "short_drama_editing",
            "plot_extraction",
            Some("v2.0"),
            &vars_step2,
        )
        .map_err(|e| SdpError::LlmError {
            details: format!("渲染 plot_extraction prompt 失败: {}", e),
        })?;

    let extraction_raw = provider
        .generate_text(
            &extraction_prompt,
            Some(
                "你是一名专业短剧剪辑师，精通视频叙事节奏和剧情连贯性，\
                 擅长选择能够流畅衔接的视频片段。",
            ),
            Some(temperature as f32),
            Some(4000),
            None,
        )
        .await
        .map_err(|e| SdpError::LlmError {
            details: format!("plot_extraction LLM 调用失败: {}", e),
        })?;

    // ---- 11. 修复 + 解析 PlotPointsData JSON ----
    let extraction_repaired = crate::sde::script_gen::repair_json(&extraction_raw);
    let extraction_path = task_dir.join("plot_extraction_raw.json");
    tokio::fs::write(&extraction_path, &extraction_repaired).await?;

    let points_data: PlotPointsData = serde_json::from_str(&extraction_repaired).map_err(|e| {
        SdpError::JsonRepair {
            details: format!("解析 plot_extraction JSON 失败: {}", e),
        }
    })?;

    // ---- 12. merge_script ----
    let script = merge_script(&points_data.plot_points, task_dir)?;

    Ok(script)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ========== generate_narration_placeholder ==========

    #[test]
    fn test_generate_narration_placeholder_prefix() {
        let placeholder = generate_narration_placeholder();
        assert!(
            placeholder.starts_with("播放原生_"),
            "应以 '播放原生_' 开头: {}",
            placeholder
        );
        // "播放原生_" (5 chars) + 8 hex chars = 13 chars total
        assert_eq!(
            placeholder.chars().count(),
            13,
            "长度应为 13 (5 + 8 hex): {}",
            placeholder
        );
    }

    #[test]
    fn test_generate_narration_placeholder_uniqueness() {
        let a = generate_narration_placeholder();
        let b = generate_narration_placeholder();
        assert_ne!(a, b, "连续调用应产生不同占位值");
    }

    // ========== format_plot_titles ==========

    #[test]
    fn test_format_plot_titles() {
        let titles = vec![
            "开场".to_string(),
            "冲突升级".to_string(),
            "结局反转".to_string(),
        ];
        let result = format_plot_titles(&titles);
        assert!(result.contains("1. 开场"), "应包含第一项: {}", result);
        assert!(result.contains("2. 冲突升级"), "应包含第二项: {}", result);
        assert!(result.contains("3. 结局反转"), "应包含第三项: {}", result);
    }

    #[test]
    fn test_format_plot_titles_empty() {
        let titles: Vec<String> = vec![];
        let result = format_plot_titles(&titles);
        assert!(result.is_empty(), "空输入应返回空字符串");
    }

    #[test]
    fn test_format_plot_titles_single() {
        let titles = vec!["唯一剧情".to_string()];
        let result = format_plot_titles(&titles);
        assert_eq!(result, "1. 唯一剧情");
    }

    // ========== merge_script ==========

    fn make_sample_point(seq: i64) -> PlotPoint {
        PlotPoint {
            sequence: seq,
            timestamp: format!(
                "00:00:{:02},000-00:00:{:02},000",
                seq * 5,
                seq * 5 + 5
            ),
            title: format!("情节 {}", seq),
            narrative_function: None,
            picture: format!("画面描述 {}", seq),
            context_before: None,
            context_after: None,
            transition_note: None,
        }
    }

    #[test]
    fn test_merge_script_valid() {
        let points = vec![make_sample_point(1), make_sample_point(2)];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = merge_script(&points, dir.path());
        assert!(
            result.is_ok(),
            "valid points should merge: {:?}",
            result.err()
        );
        let clips = result.unwrap();
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0]._id, 1);
        assert_eq!(clips[1]._id, 2);
    }

    #[test]
    fn test_merge_script_empty() {
        let points: Vec<PlotPoint> = vec![];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let result = merge_script(&points, dir.path());
        assert!(result.is_err(), "empty points should error");
        match result {
            Err(SdpError::Validation { .. }) => {}
            _ => panic!("Expected Validation error, got: {:?}", result),
        }
    }

    #[test]
    fn test_merge_script_output_ost1() {
        let points = vec![make_sample_point(1)];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = merge_script(&points, dir.path()).expect("merge should succeed");
        assert_eq!(
            clips[0].ost,
            crate::script::types::OstType::OriginalSound,
            "SDP clips should all be OST=1 (OriginalSound)"
        );
    }

    #[test]
    fn test_merge_script_saves_file() {
        let points = vec![make_sample_point(1)];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let _clips = merge_script(&points, dir.path()).expect("merge should succeed");
        let script_path = dir.path().join("merged_subtitle.json");
        assert!(
            script_path.exists(),
            "merged_subtitle.json should be saved"
        );
        let content = std::fs::read_to_string(&script_path).expect("should read saved file");
        assert!(
            content.contains("播放原生_"),
            "saved file should contain narration placeholder: {}",
            content
        );
    }

    #[test]
    fn test_merge_script_all_option_fields_none() {
        let points = vec![make_sample_point(1)];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = merge_script(&points, dir.path()).expect("merge should succeed");
        assert!(clips[0].duration.is_none());
        assert!(clips[0].source_time_range.is_none());
        assert!(clips[0].edited_time_range.is_none());
        assert!(clips[0].audio.is_none());
        assert!(clips[0].video.is_none());
        assert!(clips[0].subtitle.is_none());
    }

    #[test]
    fn test_merge_script_preserves_timestamp() {
        let points = vec![PlotPoint {
            sequence: 1,
            timestamp: "00:00:01,200-00:00:06,300".to_string(),
            title: "开场".to_string(),
            narrative_function: None,
            picture: "主角登场".to_string(),
            context_before: None,
            context_after: None,
            transition_note: None,
        }];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = merge_script(&points, dir.path()).expect("merge should succeed");
        assert_eq!(clips[0].timestamp, "00:00:01,200-00:00:06,300");
    }

    #[test]
    fn test_merge_script_preserves_picture() {
        let points = vec![PlotPoint {
            sequence: 1,
            timestamp: "00:00:01,000-00:00:05,000".to_string(),
            title: "title".to_string(),
            narrative_function: None,
            picture: "画面描述：主角站在门口".to_string(),
            context_before: None,
            context_after: None,
            transition_note: None,
        }];
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let clips = merge_script(&points, dir.path()).expect("merge should succeed");
        assert_eq!(clips[0].picture, "画面描述：主角站在门口");
    }
}
