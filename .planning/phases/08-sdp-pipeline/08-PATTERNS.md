# Phase 08: SDP Pipeline - Pattern Map

**Mapped:** 2026-05-05
**Files analyzed:** 14 (6 new, 5 modified, 3 extracted/removed)
**Analogs found:** 12 / 14

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/subtitle/mod.rs` | module (extracted) | file-I/O | `src/sde/subtitle.rs` | exact (same code, re-exported) |
| `src/sdp/mod.rs` | module (new) | n/a | `src/sde/mod.rs` | role-match |
| `src/sdp/types.rs` | types (new) | n/a | `src/sde/types.rs` | role-match (SDP-simplified) |
| `src/sdp/error.rs` | error (new) | n/a | `src/sde/error.rs` | role-match (SDP-simplified) |
| `src/sdp/script_gen.rs` | service (script gen) | request-response (LLM) | `src/sde/script_gen.rs` | role-match (different prompts) |
| `src/sdp/clip.rs` | service (clip wrapper) | file-I/O (FFmpeg) | `src/sde/clip.rs` | role-match (simplified, delegates) |
| `src/sdp/pipeline.rs` | service (pipeline) | file-I/O (FFmpeg) | `src/sde/pipeline.rs` + `src/documentary/pipeline.rs` | role-match (4-step SDP) |
| `src/lib.rs` | config (modified) | n/a | Current `src/lib.rs` | exact (add 2 pub mod lines) |
| `src/sde/mod.rs` | module (modified) | n/a | Current `src/sde/mod.rs` | exact (remove subtitle, add use) |
| `src/sde/subtitle.rs` | module (removed) | file-I/O | Self (extract-and-import) | exact (relocate to src/subtitle/) |
| `src/prompt/register.rs` | config (modified) | n/a | Current `src/prompt/register.rs` | exact (add 2 prompts, update 1) |
| `src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md` | template (new) | n/a | `src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md` | role-match (same category, different content) |
| `src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md` | template (modified) | n/a | Current template | exact (update parameters) |

## Pattern Assignments

---

### `src/subtitle/mod.rs` (module, file-I/O — extracted from `src/sde/subtitle.rs`)

**Analog:** `src/sde/subtitle.rs` (lines 1-731)

**Strategy:** Extract `parse_subtitle_file`, `detect_encoding`, `normalize_subtitle_text`, `SubtitleSegment` from `src/sde/subtitle.rs` into `src/subtitle/mod.rs`. Change error type from `SdeError` to a new error type (or use `ScriptError`/generic IO). Export `pub use` pattern.

**Imports pattern** (analog lines 1-9):
```rust
use std::fs;
use std::path::Path;

use encoding_rs;
use regex::Regex;

// WAS: use crate::sde::error::SdeError;
// WAS: use crate::sde::timestamp::parse_srt_timestamp;
// WAS: use crate::sde::types::SubtitleSegment;

// NEW: self-contained error or generic error for the public module
// NEW: keep parse_srt_timestamp inline or import from documentary
```

**Core public API** (analog lines 437-460):
```rust
/// 高级函数：读取字幕文件 → 检测编码 → 解码 → 标准化 → 解析段落
///
/// 返回 (段落列表, 标准化文本, 编码名) 三元组。
///
/// 注意：此函数是 CPU 密集型的，在 pipeline.rs 中应使用 spawn_blocking 包装。
pub fn parse_subtitle_file(
    path: &Path,
) -> Result<(Vec<SubtitleSegment>, String, String), SubtitleError> {
    let data = fs::read(path)?;
    let encoding = detect_encoding(&data)?;
    let decoded = decode_bytes(&data, &encoding)?;
    let normalized = normalize_subtitle_text(&decoded);

    let segments = if normalized.contains("-->") {
        parse_srt_blocks(&normalized)
    } else if normalized.to_ascii_lowercase().contains("[events]") {
        parse_ass_dialogues(&normalized)
    } else {
        Vec::new()
    };

    Ok((segments, normalized, encoding))
}
```

**SubtitleSegment type** (from `src/sde/types.rs` lines 105-111):
```rust
/// 字幕段落——解析后的 SRT 条目
#[derive(Debug, Clone)]
pub struct SubtitleSegment {
    pub index: usize,
    pub start_secs: f64,
    pub end_secs: f64,
    pub text: String,
}
```

**Error type needed** — new `SubtitleError` enum (simplified, in `src/subtitle/mod.rs`):
```rust
/// 字幕解析错误
#[derive(thiserror::Error, Debug)]
pub enum SubtitleError {
    #[error("编码检测失败: {0}")]
    EncodingDetection(String),

    #[error("解码失败: {0}")]
    Decode(String),

    #[error("IO 操作失败: {source}")]
    Io { #[source] source: std::io::Error },
}

impl From<std::io::Error> for SubtitleError {
    fn from(source: std::io::Error) -> Self {
        SubtitleError::Io { source }
    }
}
```

**Exports pattern:**
```rust
mod error;
// Re-export public API
pub use error::SubtitleError;  // (new error module, or inline)
pub use parse_subtitle_file;
pub use detect_encoding;
pub use normalize_subtitle_text;
pub use types::SubtitleSegment;  // define SubtitleSegment inline or in a types sub-mod
```

**Test pattern** (from analog lines 462-731): Copy all existing tests from `src/sde/subtitle.rs` lines 462-731, update error type assertions from `SdeError::ParseSubtitle` to `SubtitleError::*`.

---

### `src/sdp/mod.rs` (module, n/a — new)

**Analog:** `src/sde/mod.rs` (lines 1-12)

**Imports pattern (Wave 2 — Plan 03):**
```rust
pub mod error;
pub mod types;
pub mod script_gen;

pub use error::SdpError;
pub use types::{SdpPipelineState, SdpProgressStep, SdpRequest};
```

Note: `SdpPipelineState` is re-exported for test access, `SdpRequest` for external API consumers. Follow SDE's `pub use` convention exactly.

**Deferred to Wave 3 (Plan 04):** `pub mod clip; pub mod pipeline;` and `pub use pipeline::run_sdp;` are added when Plan 04 creates those files. Plan 03 mod.rs does NOT declare these modules yet — this avoids the forward-declaration issue where mod.rs declares modules that don't exist (B1 fix).

---

### `src/sdp/types.rs` (types, n/a — new)

**Analog:** `src/sde/types.rs` (lines 1-224)

**SdpRequest** — simplified from `SdeRequest`, no TTS/subtitle fields:
```rust
use std::path::PathBuf;

/// SDP 流水线请求参数（精简版，无 TTS/字幕相关字段）
#[derive(Debug, Clone)]
pub struct SdpRequest {
    pub subtitle_path: PathBuf,   // for generate_sdp_script
    pub video_path: PathBuf,      // for run_sdp
    pub script_path: PathBuf,     // pre-edited script (alternative to generate)
    pub output_dir: Option<PathBuf>,
    pub temperature: f64,         // LLM temperature (default 0.1)
    pub custom_clips: u32,        // Python default: 5
    pub original_volume: f64,     // default 0.7 (as SDE)
    pub bgm_volume: f64,          // default 0.3
    pub bgm_path: Option<PathBuf>,
    pub threads: u32,             // default 4
}
```

**Default + validate pattern** (from analog lines 30-85):
```rust
impl Default for SdpRequest {
    fn default() -> Self {
        Self {
            subtitle_path: PathBuf::new(),
            video_path: PathBuf::new(),
            script_path: PathBuf::new(),
            output_dir: None,
            temperature: 0.1,       // SDP-specific
            custom_clips: 5,
            original_volume: 0.7,
            bgm_volume: 0.3,
            bgm_path: None,
            threads: 4,
        }
    }
}

impl SdpRequest {
    /// 校验请求参数，返回错误描述或 Ok(())
    pub fn validate(&self) -> Result<(), String> {
        if self.subtitle_path.as_os_str().is_empty() && self.script_path.as_os_str().is_empty() {
            return Err("subtitle_path 或 script_path 必须提供一个".to_string());
        }
        if !(0.0..=10.0).contains(&self.original_volume) {
            return Err(format!("original_volume 超出有效范围 [0, 10]: {}", self.original_volume));
        }
        if !(0.0..=10.0).contains(&self.bgm_volume) {
            return Err(format!("bgm_volume 超出有效范围 [0, 10]: {}", self.bgm_volume));
        }
        if self.threads == 0 {
            return Err("threads 必须大于 0".to_string());
        }
        Ok(())
    }
}
```

**SdpProgressStep** — only 4 steps (analog `SdeProgressStep` lines 88-99):
```rust
/// SDP 进度步骤（4 步精简版）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdpProgressStep {
    Clip,        // 30%
    Concat,      // 60%
    Composite,   // 90%
    Done,        // 100%
}
```

**SdpPipelineState** — no TTS/subtitle fields (analog lines 114-162):
```rust
use std::collections::HashMap;
use std::path::PathBuf;

/// SDP 流水线中间状态——4 步所需字段，无 TTS/字幕相关
pub struct SdpPipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    pub script: crate::script::types::Script,
    pub video_clips: HashMap<i64, PathBuf>,
    pub combined_video_path: Option<PathBuf>,
    pub output_video_path: Option<PathBuf>,
    pub total_duration: f64,
    pub progress: Option<SdpProgressCallback>,
}

/// SDP 进度回调类型
pub type SdpProgressCallback = Box<dyn Fn(SdpProgressStep, f32, &str) + Send + Sync>;

impl SdpPipelineState {
    pub fn new(task_id: String, task_dir: PathBuf) -> Self {
        Self {
            task_id,
            task_dir,
            script: Vec::new(),
            video_clips: HashMap::new(),
            combined_video_path: None,
            output_video_path: None,
            total_duration: 0.0,
            progress: None,
        }
    }

    /// 触发进度回调（如果已注册）
    pub fn emit_progress(&self, step: SdpProgressStep, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}
```

**Test pattern** (from analog lines 164-224):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_request_validate_valid() {
        let req = SdpRequest {
            subtitle_path: PathBuf::from("sub.srt"),
            video_path: PathBuf::from("video.mp4"),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_sdp_request_default_values() {
        let req = SdpRequest::default();
        assert_eq!(req.temperature, 0.1);
        assert_eq!(req.custom_clips, 5);
        assert_eq!(req.original_volume, 0.7);
    }

    #[test]
    fn test_sdp_progress_step_order() {
        assert_eq!(SdpProgressStep::Clip as u8, 0);
        assert_eq!(SdpProgressStep::Done as u8, 3);
    }
}
```

---

### `src/sdp/error.rs` (error, n/a — new)

**Analog:** `src/sde/error.rs` (lines 1-85)

**SdpError enum** (simplified variants):
```rust
use crate::documentary::error::PipelineError;

/// SDP 流水线错误——覆盖 4 步流水线各环节 + IO
#[derive(thiserror::Error, Debug)]
pub enum SdpError {
    #[error("字幕解析失败: {details}")]
    ParseSubtitle { details: String },

    #[error("LLM 调用失败: {details}")]
    LlmCall { details: String },

    #[error("JSON 修复失败: {details}")]
    JsonRepair { details: String },

    #[error("视频处理失败: {0}")]
    VideoProcess(#[from] PipelineError),

    #[error("参数校验失败: {details}")]
    Validation { details: String },

    #[error("IO 操作失败: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for SdpError {
    fn from(source: std::io::Error) -> Self {
        SdpError::Io { source }
    }
}
```

**Test pattern** (from analog lines 37-85):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_error_chinese_messages() {
        let err = SdpError::ParseSubtitle { details: "时间戳格式无效".into() };
        assert!(err.to_string().contains("字幕解析失败"));

        let err = SdpError::LlmCall { details: "LLM 响应为空".into() };
        assert!(err.to_string().contains("LLM 调用失败"));

        let err = SdpError::JsonRepair { details: "JSON 解析错误".into() };
        assert!(err.to_string().contains("JSON 修复失败"));

        let err = SdpError::Validation { details: "video_path 不能为空".into() };
        assert!(err.to_string().contains("参数校验失败"));
    }
}
```

---

### `src/sdp/script_gen.rs` (service, request-response/LLM — new)

**Analog:** `src/sde/script_gen.rs` (lines 1-633)

**This is the most novel file** — it contains the full SDP script generation pipeline:
- `generate_sdp_script()`: orchestrates subtitle parsing + 2-step LLM + merge_script
- `step_subtitle_analysis()`: calls `short_drama_editing/subtitle_analysis` prompt
- `step_plot_extraction()`: calls `short_drama_editing/plot_extraction` prompt
- `merge_script()`: converts plot_points to `Vec<ScriptClip>` (all OST=1)
- Reuses `repair_json()` from `src/sde/script_gen.rs`

**Imports pattern** (from analog lines 1-10):
```rust
use std::collections::HashMap;
use std::path::Path;

use crate::llm::provider::LlmProvider;
use crate::prompt::manager::PromptManager;
use crate::sdp::error::SdpError;
use crate::script::types::{OstType, Script, ScriptClip};
use crate::subtitle::{parse_subtitle_file, SubtitleSegment};
```

**LLM call pattern** — step 1 (from analog lines 22-63):
```rust
async fn step_subtitle_analysis(
    subtitle_text: &str,
    custom_clips: u32,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<String, SdpError> {
    let mut vars = HashMap::new();
    vars.insert("subtitle_content", subtitle_text);
    vars.insert("custom_clips", &custom_clips.to_string());

    let prompt = prompt_manager
        .render_prompt("short_drama_editing", "subtitle_analysis", Some("v2.0"), &vars)
        .map_err(|e| SdpError::LlmCall {
            details: format!("渲染 prompt 失败: {}", e),
        })?;

    let result = provider
        .generate_text(
            &prompt,
            Some("你是一位专业的短剧剧情分析助手，严格按 JSON 格式输出。"),
            Some(temperature as f32),
            Some(4000),
            None,
        )
        .await
        .map_err(|e| SdpError::LlmCall {
            details: format!("LLM 调用失败: {}", e),
        })?;

    Ok(result)
}
```

**LLM call pattern** — step 2 (analog lines 70-115):
```rust
async fn step_plot_extraction(
    subtitle_text: &str,
    plot_summary: &str,
    plot_titles: &str,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<String, SdpError> {
    let mut vars = HashMap::new();
    vars.insert("subtitle_content", subtitle_text);
    vars.insert("plot_summary", plot_summary);
    vars.insert("plot_titles", plot_titles);

    let prompt = prompt_manager
        .render_prompt("short_drama_editing", "plot_extraction", Some("v2.0"), &vars)
        .map_err(|e| SdpError::LlmCall {
            details: format!("渲染 prompt 失败: {}", e),
        })?;

    let result = provider
        .generate_text(
            &prompt,
            Some("你是一位专业的短剧片段选取助手，严格按 JSON 格式输出。"),
            Some(temperature as f32),
            Some(4000),
            None,
        )
        .await
        .map_err(|e| SdpError::LlmCall {
            details: format!("LLM 调用失败: {}", e),
        })?;

    Ok(result)
}
```

**JSON repair reuse** — use `crate::sde::script_gen::repair_json` directly:
```rust
use crate::sde::script_gen::repair_json;
// repair_json 6-step strategy already implemented and tested in Phase 7
```

**merge_script()** — custom SDP logic (no analog in SDE, based on Python `step5_merge_script.py`):
```rust
/// 将 plot_points 合并为 Vec<ScriptClip>，所有片段 OST=1
pub fn merge_script(
    plot_points: &[PlotPoint],
) -> Result<Script, SdpError> {
    let mut clips = Vec::new();
    for (i, point) in plot_points.iter().enumerate() {
        let clip = ScriptClip {
            _id: (i + 1) as i64,
            timestamp: point.timestamp.clone(),
            picture: point.picture.clone(),
            narration: format!("播放原生_{:04x}", i + 1),
            ost: OstType::OriginalSound,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };
        clips.push(clip);
    }

    if clips.is_empty() {
        return Err(SdpError::Validation {
            details: "合并后的脚本为空".into(),
        });
    }

    // 校验脚本
    crate::script::validate(&clips).map_err(|e| SdpError::Validation {
        details: format!("脚本校验失败: {}", e),
    })?;

    Ok(clips)
}
```

**PlotPoint type** — simple data model for LLM parsing:
```rust
#[derive(Debug, Clone, serde::Deserialize)]
struct PlotPoint {
    timestamp: String,
    picture: String,
}
```

**generate_sdp_script()** main orchestrator:
```rust
/// 生成 SDP 混剪脚本——两步 LLM 分析 + 合并
///
/// 1. 解析字幕文件（使用公共 src/subtitle/ 模块）
/// 2. 调用 subtitle_analysis prompt（分析剧情结构）
/// 3. 调用 plot_extraction prompt（定位具体时间段）
/// 4. merge_script() → Vec<ScriptClip>（全部 OST=1）
pub async fn generate_sdp_script(
    subtitle_path: &Path,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
    custom_clips: u32,
) -> Result<Script, SdpError> {
    // Step 1: 字幕解析
    let (segments, text, _encoding) = tokio::task::spawn_blocking(move || {
        parse_subtitle_file(subtitle_path)
    })
    .await
    .map_err(|e| SdpError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e),
    })??;

    // Step 2: 剧情分析
    let analysis_result = step_subtitle_analysis(
        &text, custom_clips, provider, prompt_manager, temperature
    ).await?;
    let repaired = repair_json(&analysis_result);
    let analysis_value: serde_json::Value = serde_json::from_str(&repaired)
        .map_err(|e| SdpError::JsonRepair {
            details: format!("剧情分析 JSON 解析失败: {}", e),
        })?;
    let summary = analysis_value.get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let plot_titles = analysis_value.get("plot_titles")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Step 3: 情节点提取
    let extraction_result = step_plot_extraction(
        &text, summary, plot_titles, provider, prompt_manager, temperature
    ).await?;
    let plot_data = repair_json(&extraction_result);
    let plot_points: Vec<PlotPoint> = serde_json::from_str(&plot_data)
        .map_err(|e| SdpError::JsonRepair {
            details: format!("情节点 JSON 解析失败: {}", e),
        })?;

    // Step 4: 合并脚本
    let script = merge_script(&plot_points)?;

    Ok(script)
}
```

**Test pattern** (from analog lines 381-633):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_script_creates_ost1_only() {
        let points = vec![
            PlotPoint { timestamp: "00:00:01,000-00:00:05,000".into(), picture: "场景1".into() },
            PlotPoint { timestamp: "00:00:06,000-00:00:10,000".into(), picture: "场景2".into() },
        ];
        let script = merge_script(&points).expect("merge should succeed");
        assert_eq!(script.len(), 2);
        assert_eq!(script[0].ost, OstType::OriginalSound);
        assert_eq!(script[1].ost, OstType::OriginalSound);
        assert!(script[0].narration.starts_with("播放原生_"));
    }

    #[test]
    fn test_merge_script_empty_points() {
        let points = vec![];
        let result = merge_script(&points);
        assert!(result.is_err());
    }
}
```

---

### `src/sdp/clip.rs` (service, file-I/O — new)

**Analog:** `src/sde/clip.rs` (lines 1-200)

**Simplified clip wrapper** — delegates to `sde_step_clip()` (all OST=1, same logic):
```rust
use std::path::Path;

use crate::sde::clip::sde_step_clip;  // reuse SDE's OST=1 clip logic
use crate::sdp::error::SdpError;
use crate::sdp::types::SdpPipelineState;

/// SDP 视频裁剪——委托给 SDE 的 sde_step_clip()（全部 OST=1）
///
/// SDP 全部 OST=1，与 SDE 的 OST=1 裁剪逻辑完全一致。
/// sde_step_clip() 内部完成时间戳校正 → clip_all_videos()。
pub(crate) async fn sdp_step_clip(
    state: &mut SdpPipelineState,
    video_path: &Path,
) -> Result<(), SdpError> {
    // 委托给 sde_step_clip——需要构造临时 SdePipelineState 或
    // 让 sde_step_clip 支持通用 script + subtitle_segments 参数。
    // 注意：sde_step_clip 接受 &mut SdePipelineState，SDP 需要适配。
    // 方案A：将 sde_step_clip 拆为内部函数接受 (script, subtitle_segments, task_dir)
    // 方案B：SDP 直接调用 clip_all_videos 并自行时间戳校正
    // 方案C：sde_step_clip 提取公共函数 sde_correct_timestamps + sde_call_clip_all_videos
    //
    // 详见 RESEARCH.md Pitfall 1: sde_step_clip() 可见性讨论
    
    // For now, call clip_all_videos directly (same as sde_step_clip does):
    let video_clips = crate::documentary::clip::clip_all_videos(
        video_path,
        &state.script,
        &HashMap::new(),  // no TTS results (all OST=1)
        &state.task_dir,
    )
    .await
    .map_err(|e| SdpError::VideoProcess(e))?;

    state.video_clips = video_clips;
    Ok(())
}
```

Note: The precise analog is `src/sde/clip.rs` `sde_step_clip()` lines 15-45. The SDP version is simplified because no TTS results exist and no `find_precise_range` timestamp correction is needed at the clip level (the script from LLM already has corrected timestamps during generation, or correction happens inline).

---

### `src/sdp/pipeline.rs` (service, file-I/O FFmpeg pipeline — new)

**Analog:** `src/sde/pipeline.rs` (lines 1-882) + `src/documentary/pipeline.rs` (lines 1-120)

**Imports pattern** (from SDE pipeline lines 1-20):
```rust
use std::path::{Path, PathBuf};

use crate::config::types::{AppConfig, ProxySection};
use crate::ffmpeg::command::run_ffmpeg;
use crate::sdp::clip::sdp_step_clip;
use crate::sdp::error::SdpError;
use crate::sdp::types::{SdpPipelineState, SdpProgressStep, SdpRequest};
use crate::script::types::OstType;
```

**run_sdp() main orchestrator** (4 steps, from RESEARCH.md example, analog SDE lines 34-587):
```rust
pub async fn run_sdp(
    request: SdpRequest,
    config: &AppConfig,
    proxy: Option<&ProxySection>,
) -> Result<PathBuf, SdpError> {
    // 0. 参数校验
    request.validate().map_err(|e| SdpError::Validation { details: e })?;

    // 1. 创建任务目录
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request.output_dir.clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    std::fs::create_dir_all(&task_dir)?;

    // 2. 初始化
    let mut state = SdpPipelineState::new(task_id, task_dir);

    // 3. 加载脚本
    state.script = crate::script::load_script(&request.script_path)?;

    // 4. G4 Guard: 所有片段必须 OST=1
    if state.script.iter().any(|c| c.ost != OstType::OriginalSound) {
        return Err(SdpError::Validation {
            details: "SDP 仅支持 OST=1（原声）片段".into(),
        });
    }

    // ============================================================
    //  Step 1: Clip  [0% → 30%]
    // ============================================================
    state.emit_progress(SdpProgressStep::Clip, 0.0, "正在裁剪视频...");
    sdp_step_clip(&mut state, &request.video_path).await?;
    state.emit_progress(SdpProgressStep::Clip, 30.0, "视频裁剪完成");

    // ============================================================
    //  Step 2: Concat  [30% → 60%]
    // ============================================================
    state.emit_progress(SdpProgressStep::Concat, 30.0, "正在拼接视频...");
    sdp_step_concat(&mut state).await?;
    state.emit_progress(SdpProgressStep::Concat, 60.0, "视频拼接完成");

    // ============================================================
    //  Step 3: Composite + BGM  [60% → 100%]
    // ============================================================
    state.emit_progress(SdpProgressStep::Composite, 60.0, "正在最终合成...");
    sdp_step_composite(&mut state, &request).await?;
    state.emit_progress(SdpProgressStep::Composite, 100.0, "最终合成完成");

    // 返回结果
    state.output_video_path.ok_or_else(|| SdpError::Validation {
        details: "流水线完成但未生成输出视频".into(),
    })
}
```

**step_concat** (from SDE pipeline lines 286-367, simplified):
```rust
pub(crate) async fn sdp_step_concat(
    state: &mut SdpPipelineState,
) -> Result<(), SdpError> {
    let concat_list_path = state.task_dir.join("concat_list.txt");
    let mut concat_content = String::new();
    for clip in &state.script {
        if let Some(ref video_path) = clip.video {
            let path_str = video_path
                .to_string_lossy()
                .replace('\\', "/")
                .replace("'", "'\\''");
            concat_content.push_str(&format!("file '{}'\n", path_str));
        } else {
            return Err(SdpError::VideoProcess(
                crate::documentary::error::PipelineError::Concat {
                    details: format!("片段 {} 缺少视频文件", clip._id),
                }
            ));
        }
    }
    std::fs::write(&concat_list_path, &concat_content)?;

    let combined_path = state.task_dir.join("merger_sdp.mp4");

    run_ffmpeg({
        let concat_path_str = concat_list_path.to_string_lossy().to_string();
        let combined_path_str = combined_path.to_string_lossy().to_string();
        move || {
            let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
            cmd.arg("-f").arg("concat")
                .arg("-safe").arg("0")
                .arg("-i").arg(&concat_path_str)
                .arg("-c").arg("copy")
                .arg("-y").output(&combined_path_str);
            // ... spawn + error handling as SDE lines 321-365
            Ok(())
        }
    })
    .await
    .map_err(|e: crate::error::FFmpegError| {
        SdpError::VideoProcess(crate::documentary::error::PipelineError::FFmpeg { source: e })
    })?;

    state.combined_video_path = Some(combined_path);
    Ok(())
}
```

**step_composite** — simplified, no TTS/no subtitle burning (from SDE pipeline lines 370-581, adapted):
```rust
pub(crate) async fn sdp_step_composite(
    state: &mut SdpPipelineState,
    request: &SdpRequest,
) -> Result<(), SdpError> {
    let merged_video = state.combined_video_path.as_ref()
        .ok_or_else(|| SdpError::VideoProcess(
            crate::documentary::error::PipelineError::Composite {
                details: "缺少拼接视频".into(),
            }
        ))?;

    let final_path = state.task_dir.join("combined_sdp.mp4");

    // Simplified filter_complex — only original audio + optional BGM
    // No TTS audio, no subtitle burning (D-06)
    let has_bgm = request.bgm_path.is_some();
    let output_str = final_path.to_string_lossy().to_string();
    let video_str = merged_video.to_string_lossy().to_string();

    run_ffmpeg({
        let bgm_path = request.bgm_path.clone();
        let orig_vol = request.original_volume;
        let bgm_vol = request.bgm_volume;
        let total_duration = state.total_duration;

        move || {
            let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
            cmd.arg("-i").arg(&video_str);

            let mut filter_complex_parts = Vec::new();
            let mut amix_count = 0usize;
            let mut external_inputs = 0usize;

            // Original audio with volume control
            filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", orig_vol));
            amix_count += 1;

            // Optional BGM
            if let Some(ref bgm) = bgm_path {
                cmd.arg("-i").arg(bgm.to_string_lossy().to_string());
                let fade_start = (total_duration - 3.0).max(0.0);
                filter_complex_parts.push(format!(
                    "[{}:a]aloop=loop=-1:size=2e+09,atrim=0:{:.3},asetpts=PTS-STARTPTS,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
                    1, total_duration, bgm_vol, fade_start
                ));
                external_inputs += 1;
                amix_count += 1;
            }

            // Mix: only [orig] + optional [bgm], no [tts]
            let mix_labels = if has_bgm { "[orig][bgm]" } else { "[orig]" };
            filter_complex_parts.push(format!(
                "{}amix=inputs={}:duration=longest[aout]",
                mix_labels, amix_count
            ));

            // Video: pass-through (no subtitle filter needed)
            cmd.arg("-filter_complex")
                .arg(&filter_complex_parts.join(";"))
                .arg("-map").arg("0:v")
                .arg("-map").arg("[aout]")
                .codec_video("libx264")
                .arg("-preset").arg("medium")
                .arg("-pix_fmt").arg("yuv420p")
                .codec_audio("aac")
                .arg("-y").output(&output_str);

            // spawn + error handling as SDE lines 535-578
            Ok(())
        }
    })
    .await
    .map_err(|e: crate::error::FFmpegError| {
        SdpError::VideoProcess(crate::documentary::error::PipelineError::FFmpeg { source: e })
    })?;

    state.output_video_path = Some(final_path);
    Ok(())
}
```

**FFmpeg spawn pattern** (from SDE pipeline lines 309-365, `run_ffmpeg` closure):
```rust
run_ffmpeg(move || {
    let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
    // ... build command ...
    let mut child = cmd.spawn()
        .map_err(|e| crate::error::FFmpegError::SpawnFailed(e.to_string()))?;
    let iter = match child.iter() {
        Ok(iter) => iter,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(crate::error::FFmpegError::SpawnFailed(e.to_string()));
        }
    };
    let mut had_errors = false;
    for event in iter {
        if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
            tracing::error!("SDP composite error: {}", e);
            had_errors = true;
        }
    }
    let status = child.wait()
        .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;
    if had_errors {
        return Err(crate::error::FFmpegError::ExecutionError(
            "Composite reported FFmpeg errors".into(),
        ));
    }
    if !status.success() {
        return Err(crate::error::FFmpegError::ExecutionError(format!(
            "Composite exited with code {:?}", status.code()
        )));
    }
    Ok(())
}).await
```

**Test pattern** — OST validation test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::{OstType, ScriptClip};
    use std::path::PathBuf;

    #[test]
    fn test_ost_validation_all_original() {
        let script = vec![
            ScriptClip {
                _id: 1, timestamp: "00:00:00-00:00:05".into(),
                picture: "a".into(), narration: "播放原生".into(),
                ost: OstType::OriginalSound, duration: None,
                source_time_range: None, edited_time_range: None,
                audio: None, video: None, subtitle: None,
            },
        ];
        assert!(script.iter().all(|c| c.ost == OstType::OriginalSound));
    }

    #[test]
    fn test_ost_validation_mixed_rejected() {
        let script = vec![
            ScriptClip { ost: OstType::OriginalSound, ..make_clip(1, "00:00:00-00:00:05") },
            ScriptClip { ost: OstType::NarrationOnly, ..make_clip(2, "00:00:05-00:00:10") },
        ];
        assert!(script.iter().any(|c| c.ost != OstType::OriginalSound));
    }

    fn make_clip(id: i64, ts: &str) -> ScriptClip {
        ScriptClip {
            _id: id, timestamp: ts.to_string(), picture: "p".into(),
            narration: "n".into(), ost: OstType::OriginalSound,
            duration: None, source_time_range: None, edited_time_range: None,
            audio: None, video: None, subtitle: None,
        }
    }
}
```

---

### `src/lib.rs` (config, n/a — modified)

**Modification pattern** — add 2 lines after existing `pub mod sde;`:
```rust
pub mod sde;
pub mod sdp;        // NEW: SDP pipeline module
pub mod subtitle;   // NEW: public subtitle parsing module (extracted from sde)
```

---

### `src/sde/mod.rs` (module, n/a — modified)

**Modification pattern** — remove `pub mod subtitle;` line, add `pub use` or `use crate::subtitle;`:
```rust
// BEFORE:
pub mod subtitle;

// AFTER:
// Subtitle module extracted to crate::subtitle — use that instead
```

---

### `src/prompt/register.rs` (config, n/a — modified)

**Modification pattern** — add `register_sdp_prompts` function + call from `register_all_prompts` (see analog lines 24-32, 66-80):

In `register_all_prompts`, add:
```rust
pub fn register_all_prompts(registry: &mut PromptRegistry) -> Result<(), RegistrationErrors> {
    let mut errors = Vec::new();
    register_documentary_prompts(registry, &mut errors);
    register_short_drama_editing_prompts(registry, &mut errors); // MODIFIED: now registers 2 prompts
    register_short_drama_narration_prompts(registry, &mut errors);
    // register_sdp_prompts not needed — SDP shares short_drama_editing category

    if errors.is_empty() { Ok(()) }
    else { Err(RegistrationErrors(errors)) }
}
```

Update `register_short_drama_editing_prompts` to register 2 prompts (analog lines 66-80):
```rust
fn register_short_drama_editing_prompts(registry: &mut PromptRegistry, errors: &mut Vec<PromptError>) {
    // 1. subtitle_analysis v2.0 (NEW for SDP step 1)
    let analysis = Prompt {
        metadata: PromptMetadata {
            name: "subtitle_analysis".into(),
            category: "short_drama_editing".into(),
            version: "v2.0".into(),
            model_type: ModelType::Text,
            output_format: OutputFormat::Json,
            tags: vec!["short-drama-editing".into(), "subtitle-analysis".into(), "text".into()],
            parameters: vec![
                ParameterDef { name: "subtitle_content".into(), required: true, default: None, description: "字幕内容".into() },
                ParameterDef { name: "custom_clips".into(), required: false, default: Some("5".into()), description: "期望的片段数量".into() },
            ],
        },
        content: include_str!("templates/short_drama_editing/subtitle_analysis_v2.0.md").into(),
    };
    if let Err(e) = registry.register(analysis, true) { errors.push(e); }

    // 2. plot_extraction v2.0 (MODIFIED: added plot_summary, plot_titles params)
    let extraction = Prompt {
        metadata: PromptMetadata {
            name: "plot_extraction".into(),
            category: "short_drama_editing".into(),
            version: "v2.0".into(),
            model_type: ModelType::Text,
            output_format: OutputFormat::Json,
            tags: vec!["short-drama-editing".into(), "plot-extraction".into(), "text".into()],
            parameters: vec![
                ParameterDef { name: "subtitle_content".into(), required: true, default: None, description: "字幕内容".into() },
                ParameterDef { name: "plot_summary".into(), required: true, default: None, description: "剧情摘要（来自 subtitle_analysis）".into() },
                ParameterDef { name: "plot_titles".into(), required: true, default: None, description: "情节标题列表（来自 subtitle_analysis）".into() },
            ],
        },
        content: include_str!("templates/short_drama_editing/plot_extraction_v2.0.md").into(),
    };
    if let Err(e) = registry.register(extraction, true) { errors.push(e); }
}
```

Update test to expect 2 prompts in `short_drama_editing` category:
```rust
assert_eq!(registry.list_prompts("short_drama_editing").len(), 2);
```

---

### `src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md` (template, n/a — new)

**Pattern:** Follow existing plot_extraction template format (from existing `plot_extraction_v2.0.md`):
```
你是一个专业的短剧剧情分析助手。分析以下字幕内容，提取完整的剧情结构和关键情节点。

字幕内容:
${subtitle_content}

期望片段数量: ${custom_clips}

## 分析任务
...

## 输出格式
请严格按以下 JSON 格式输出:
{
  "summary": "...剧情摘要...",
  "plot_titles": "...关键情节点标题列表...",
  "narrative_structure": "...开端→发展→高潮→结局...",
  "plot_connections": "...情节间逻辑关联..."
}
```

---

### `src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md` (template, n/a — modified)

**Modification:** Update existing template to accept `plot_summary` and `plot_titles` parameters (replacing old `language` param), and add SDP-specific extraction instructions:

```
你是一个专业的短剧片段选取助手。基于以下分析结果，从字幕中精确定位需要提取的视频片段。

字幕内容:
${subtitle_content}

剧情摘要:
${plot_summary}

情节标题:
${plot_titles}

## 提取要求
1. 每个片段必须对应一个完整的情节单元
2. 时间戳必须精确到毫秒，与字幕时间码对齐
3. 每个片段时长建议 5-15 秒
4. 确保片段连续性能让观众理解故事线

## 输出格式
请严格按以下 JSON 格式输出:
{
  "plot_points": [
    {
      "timestamp": "HH:MM:SS,mmm-HH:MM:SS,mmm",
      "picture": "画面描述"
    }
  ]
}
```

---

## Shared Patterns

### Auth/Guard Pattern
Not applicable — SDP is a data-processing pipeline with no authentication.

### Progress Callback Pattern
**Source:** `src/sde/types.rs` lines 102, 157-162
**Apply to:** `src/sdp/types.rs` (SdpProgressCallback type + emit_progress method)
```rust
pub type SdpProgressCallback = Box<dyn Fn(SdpProgressStep, f32, &str) + Send + Sync>;

pub fn emit_progress(&self, step: SdpProgressStep, pct: f32, msg: &str) {
    if let Some(ref cb) = self.progress {
        cb(step, pct, msg);
    }
}
```

### Error Handling (thiserror pattern)
**Source:** `src/sde/error.rs` lines 4-35
**Apply to:** `src/sdp/error.rs`, `src/subtitle/error.rs` (both use thiserror + From<io::Error>)

### Spawn Blocking for CPU-intensive work
**Source:** `src/sde/pipeline.rs` lines 70-76
**Apply to:** `src/sdp/script_gen.rs` (spawn_blocking for parse_subtitle_file), `src/sdp/pipeline.rs` (run_ffmpeg for FFmpeg operations)
```rust
let (segments, text, encoding) = tokio::task::spawn_blocking(move || {
    parse_subtitle_file(&subtitle_path)
})
.await
.map_err(|e| SdpError::Io {
    source: std::io::Error::new(std::io::ErrorKind::Other, e),
})??;
```

### FFmpeg run_ffmpeg pattern
**Source:** `src/ffmpeg/command.rs` lines 16-24
**Apply to:** `src/sdp/pipeline.rs` (step_concat, step_composite)
```rust
use crate::ffmpeg::command::run_ffmpeg;

run_ffmpeg(move || {
    let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
    // ... build command ...
    // spawn + iter + error handling ...
    Ok(())
}).await
```

### JSON Repair reuse
**Source:** `src/sde/script_gen.rs` lines 127-221
**Apply to:** `src/sdp/script_gen.rs`
```rust
// Direct reuse of Phase 7's tested 6-step repair_json
use crate::sde::script_gen::repair_json;
```

### Script Validation
**Source:** `src/script/mod.rs` lines 66-129
**Apply to:** `src/sdp/script_gen.rs` (merge_script output validation), `src/sdp/pipeline.rs` (input script validation)
```rust
crate::script::validate(&clips).map_err(|e| SdpError::Validation {
    details: format!("脚本校验失败: {}", e),
})?;
```

### OST Validation Guard
**Source:** `src/sde/pipeline.rs` lines 596-660 (detect_and_abort_overlaps, warn_ost_ratio)
**Apply to:** `src/sdp/pipeline.rs` — simpler OST check:
```rust
if state.script.iter().any(|c| c.ost != OstType::OriginalSound) {
    return Err(SdpError::Validation {
        details: "SDP 仅支持 OST=1（原声）片段".into(),
    });
}
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md` | template | n/a | New prompt for SDP step 1; no existing `subtitle_analysis` template in Rust. Python source at `app/services/prompts/short_drama_editing/subtitle_analysis.py` should be translated. |

No close Rust analog exists for this prompt template because the SDE pipeline uses `short_drama_narration/plot_analysis` (different category name, different approach). The template content should be translated from Python `app/services/prompts/short_drama_editing/subtitle_analysis.py`.

---

## Metadata

**Analog search scope:**
- `src/sde/` (mod, types, error, clip, pipeline, subtitle, script_gen)
- `src/documentary/` (pipeline, types)
- `src/script/` (mod, types)
- `src/ffmpeg/` (command)
- `src/prompt/` (register, manager, templates)
- `src/lib.rs`

**Files scanned:** 15 source files + 2 planning docs + 1 Python source reference
**Pattern extraction date:** 2026-05-05
