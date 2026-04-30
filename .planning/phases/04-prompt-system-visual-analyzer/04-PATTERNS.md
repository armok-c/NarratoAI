# Phase 4: Prompt System + Visual Analyzer - Pattern Map

**Mapped:** 2026-04-30
**Files analyzed:** 22 (20 new + 2 modified)
**Analogs found:** 17 / 22

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/prompt/mod.rs` | config | static | `src/script/mod.rs` | exact |
| `src/prompt/types.rs` | model | static | `src/script/types.rs` | exact |
| `src/prompt/error.rs` | model | static | `src/script/error.rs` | exact |
| `src/prompt/registry.rs` | model | CRUD | `src/llm/registry.rs` | exact |
| `src/prompt/template.rs` | utility | transform | (none — new pattern) | — |
| `src/prompt/manager.rs` | service | CRUD | (none — new pattern) | — |
| `src/prompt/validators.rs` | utility | transform | `src/script/mod.rs:validate()` | role-match |
| `src/prompt/registry/mod.rs` | service | static | `src/llm/register.rs` | exact |
| `src/prompt/registry/documentary.rs` | service | static | `src/llm/register.rs` | role-match |
| `src/prompt/registry/short_drama_editing.rs` | service | static | `src/llm/register.rs` | role-match |
| `src/prompt/registry/short_drama_narration.rs` | service | static | `src/llm/register.rs` | role-match |
| `src/prompt/templates/documentary/frame_analysis_v1.0.md` | config | static | (none — new asset embedding) | — |
| `src/prompt/templates/documentary/narration_generation_v2.0.md` | config | static | (none — new asset embedding) | — |
| `src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md` | config | static | (none — new asset embedding) | — |
| `src/prompt/templates/short_drama_narration/script_generation_v1.0.md` | config | static | (none — new asset embedding) | — |
| `src/visual/mod.rs` | config | static | `src/ffmpeg/mod.rs` | exact |
| `src/visual/error.rs` | model | static | `src/script/error.rs` | exact |
| `src/visual/types.rs` | model | static | `src/script/types.rs` | exact |
| `src/visual/frame_extractor.rs` | service | file-I/O | `src/ffmpeg/command.rs` | exact |
| `src/visual/analyzer.rs` | service | request-response | `src/llm/openai_compatible.rs` | role-match |
| `src/lib.rs` (modify) | config | static | `src/lib.rs` (existing) | exact |
| `Cargo.toml` (modify) | config | static | `Cargo.toml` (existing) | exact |

---

## Pattern Assignments

### `src/prompt/mod.rs` (config, static)

**Analog:** `src/script/mod.rs` (lines 1-3)

**Imports/module pattern:**
```rust
// Source: src/script/mod.rs lines 1-3
pub mod edit;
pub mod error;
pub mod types;
```

**Prompt module should follow same pattern:**
```rust
// Expected pattern for src/prompt/mod.rs
pub mod types;
pub mod error;
pub mod registry;
pub mod template;
pub mod manager;
pub mod validators;

pub mod registry;  // re-export sub-module
```

---

### `src/prompt/types.rs` (model, static)

**Analog:** `src/script/types.rs`

**Imports pattern** (lines 1-3):
```rust
use serde::{Deserialize, Serialize};
```

**Type definition pattern with serde derives:**
```rust
// Source: src/llm/types.rs lines 1-10
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmResponseFormat {
    Text,
    Json,
}
```

**Prompt types should follow same pattern:**
```rust
// Expected pattern — ModelType/OutputFormat enums + Prompt struct with serde
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "text")]
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    pub category: String,
    pub version: String,
    pub model_type: ModelType,
    pub output_format: OutputFormat,
    pub tags: Vec<String>,
    pub parameters: Vec<ParameterDef>,
}
```

---

### `src/prompt/error.rs` (model, static)

**Analog:** `src/script/error.rs` (lines 1-20)

**Imports + error enum pattern:**
```rust
// Source: src/script/error.rs lines 1-20
use crate::script::types::ValidationError;

/// 脚本领域错误
#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("JSON 解析失败: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("脚本校验失败")]
    Validation(Vec<ValidationError>),

    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("索引越界")]
    IndexOutOfBounds,

    #[error("时间戳格式无效: {0}")]
    InvalidTimestamp(String),
}
```

**PromptError should follow same pattern (D-21/22):**
```rust
// Expected pattern — PromptError variants per D-22
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("模板渲染失败: 缺少必需参数 {0}")]
    TemplateRender(String),
    #[error("未找到模板: {category}/{name} v{version:?}")]
    NotFound { category: String, name: String, version: Option<String> },
    #[error("模板注册失败: {0}")]
    Registration(String),
    #[error("模板校验失败: {0}")]
    Validation(String),
    #[error("版本错误: {0}")]
    Version(String),
}
```

---

### `src/prompt/registry.rs` (model, CRUD)

**Analog:** `src/llm/registry.rs` (entire file, 49 lines)

**Imports + Registry pattern:**
```rust
// Source: src/llm/registry.rs lines 1-48
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::LLMError;
use crate::llm::provider::LlmProvider;

/// Provider 注册中心
///
/// 运行时 trait object 注册模式：HashMap 持有 Arc<dyn LlmProvider>。
pub struct Registry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(name.to_lowercase(), provider);
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>, LLMError> {
        self.providers
            .get(&name.to_lowercase())
            .cloned()
            .ok_or_else(|| LLMError::ProviderNotFound(name.to_string()))
    }

    pub fn list_providers(&self) -> Vec<String> {
        let mut names: Vec<String> = self.providers.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for Registry {
    fn default() -> Self { Self::new() }
}
```

**PromptRegistry should follow same pattern but with 3-level HashMap (D-18):**
```rust
// Expected pattern — PromptRegistry with Arc<RwLock<>> outer wrapper
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::prompt::types::Prompt;
use crate::prompt::error::PromptError;

pub type SharedPromptRegistry = Arc<RwLock<PromptRegistry>>;

pub struct PromptRegistry {
    // 3-level index: {category: {name: {version: Prompt}}}
    prompts: HashMap<String, HashMap<String, HashMap<String, Prompt>>>,
    default_versions: HashMap<String, HashMap<String, String>>,
}

impl PromptRegistry {
    pub fn new() -> Self { /* ... */ }
    pub fn register(&mut self, prompt: Prompt, is_default: bool) -> Result<(), PromptError> { /* ... */ }
    pub fn get(&self, category: &str, name: &str, version: Option<&str>) -> Result<&Prompt, PromptError> { /* ... */ }
    pub fn search(&self, query: &str) -> Vec<&Prompt> { /* ... */ }
}
```

---

### `src/prompt/template.rs` (utility, transform)

**Analog:** None (new pattern). Use RESEARCH.md two-pass regex pattern.

**Core rendering pattern (from RESEARCH.md lines 227-268):**
```rust
// Two-pass approach: validate first, then replace
// (Regex::replace_all closure cannot return Result, so pre-validate)
pub fn render(template: &str, vars: &HashMap<&str, &str>) -> Result<String, PromptError> {
    let var_re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").map_err(|e| {
        PromptError::TemplateRender(format!("正则编译失败: {}", e))
    })?;

    // Pass 1: validate all variables exist
    let mut missing = Vec::new();
    for cap in var_re.captures_iter(template) {
        let name = cap.get(1).or_else(|| cap.get(2))
            .map(|m| m.as_str()).unwrap_or("");
        if !vars.contains_key(name) && !missing.contains(&name.to_string()) {
            missing.push(name.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}", missing.join(", ")
        )));
    }

    // Pass 2: replace (all variables guaranteed to exist)
    let result = var_re.replace_all(template, |caps: &regex::Captures| {
        let name = caps.get(1).or_else(|| caps.get(2))
            .map(|m| m.as_str()).unwrap_or("");
        vars.get(name).copied().unwrap_or("")
    });

    // Pass 3: apply filters (${variable|filter_name})
    let filter_re = Regex::new(r"\$\{(\w+)\|(\w+)\}").unwrap();
    let filters = builtin_filters();
    let result = filter_re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if let (Some(filter_fn), Some(value)) = (filters.get(filter_name), vars.get(var_name)) {
            return filter_fn(value);
        }
        caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
    });

    Ok(result.to_string())
}
```

**Filter function registration pattern (from RESEARCH.md lines 452-483):**
```rust
type FilterFn = fn(&str) -> String;

fn builtin_filters() -> HashMap<&'static str, FilterFn> {
    let mut m = HashMap::new();
    m.insert("upper", |s| s.to_uppercase());
    m.insert("lower", |s| s.to_lowercase());
    m.insert("title", |s| { /* capitalize each word */ });
    m.insert("strip", |s| s.trim());
    m.insert("truncate", |s| {
        if s.len() <= 100 { s.to_string() }
        else { format!("{}...", &s[..97]) }
    });
    m.insert("json", |s| {
        serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
    });
    m
}
```

---

### `src/prompt/manager.rs` (service, CRUD)

**Analog:** None at this granularity. Combines registry + template + validation.

**Exposed API pattern (from RESEARCH.md Recommendation):**
- `get_prompt(category, name, version?) -> Result<Prompt>` — delegates to registry
- `render_prompt(category, name, version?, vars) -> Result<String>` — get + render
- `register_prompt(prompt, is_default) -> Result<()>` — delegates to registry
- `search_prompts(query) -> Vec<Prompt>` — delegates to registry
- `validate_output(output, format) -> Result<()>` — delegates to validators

---

### `src/prompt/validators.rs` (utility, transform)

**Analog:** `src/script/mod.rs:validate()` (lines 75-129)

**Validation pattern — collect all errors then return:**
```rust
// Source: src/script/mod.rs lines 75-83, 124-128
pub fn validate(script: &Script) -> Result<(), ScriptError> {
    let mut errors = Vec::new();
    // ... per-field checks ...
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ScriptError::Validation(errors))
    }
}
```

**Expected pattern for validate_output():**
```rust
pub fn validate_output(output: &str, format: OutputFormat) -> Result<(), PromptError> {
    match format {
        OutputFormat::Json => {
            // Try serde_json::from_str, return PromptError::Validation on failure
        }
        OutputFormat::NarrationScript => {
            // Check required sections exist
        }
        OutputFormat::PlotAnalysis => {
            // Check plot analysis structure
        }
    }
}
```

---

### `src/prompt/registry/mod.rs` (service, static/init)

**Analog:** `src/llm/register.rs` (entire file, 158 lines)

**register_all_*() init pattern:**
```rust
// Source: src/llm/register.rs lines 28-45, 111-116
/// 注册所有活跃 provider
///
/// 在库初始化时显式调用。
pub fn register_all_providers(config: &AppConfig, registry: &mut Registry) -> Result<(), RegistrationErrors> {
    // ... config-dependent registration ...

    if errors.is_empty() {
        Ok(())
    } else {
        Err(RegistrationErrors(errors))
    }
}
```

**register_all_prompts() should follow same pattern (D-17):**
```rust
// Expected pattern — register_all_prompts() using include_str!() for template content
use std::sync::RwLock;
use crate::prompt::types::{Prompt, PromptMetadata, ModelType, OutputFormat};
use crate::prompt::error::PromptError;
use crate::prompt::registry::{PromptRegistry, SharedPromptRegistry};

pub fn register_all_prompts(registry: &PromptRegistry) -> Result<(), Vec<PromptError>> {
    let mut errors = Vec::new();
    // Each category module registers its prompts
    register_documentary_prompts(registry, &mut errors);
    register_short_drama_editing_prompts(registry, &mut errors);
    register_short_drama_narration_prompts(registry, &mut errors);

    if errors.is_empty() { Ok(()) }
    else { Err(errors) }
}
```

**Per-category registration function pattern (D-02/D-03):**
```rust
// Expected pattern for src/prompt/registry/documentary.rs
use crate::prompt::types::*;

pub fn register_documentary_prompts(registry: &PromptRegistry, errors: &mut Vec<PromptError>) {
    match registry.register(
        Prompt {
            metadata: PromptMetadata {
                name: "frame_analysis".into(),
                category: "documentary".into(),
                version: "v1.0".into(),
                model_type: ModelType::Vision,
                output_format: OutputFormat::Json,
                tags: vec!["frame_analysis".into(), "vision".into()],
                parameters: vec![
                    ParameterDef {
                        name: "video_description".into(),
                        required: true,
                        description: "视频内容描述".into(),
                    },
                    ParameterDef {
                        name: "language".into(),
                        required: false,
                        default: Some("zh-CN".into()),
                        description: "输出语言".into(),
                    },
                ],
            },
            // Compile-time embed: include_str!("../templates/documentary/frame_analysis_v1.0.md")
            content: include_str!("../templates/documentary/frame_analysis_v1.0.md").to_string(),
        },
        true,
    ) {
        Ok(_) => {},
        Err(e) => errors.push(e),
    }
    // Register narration_generation_v2.0 similarly...
}
```

---

### `src/prompt/templates/*.md` (config, static)

**No analog in existing Rust code.** These are new `.md` template files that are compiled into the binary via `include_str!()`.

**Pattern:** Each `.md` file contains Jinja-style `${variable}` placeholders. No special front-matter — metadata is in the Rust registration function.

```
# 帧分析模板

请分析以下视频帧内容。

视频主题: ${video_description}
输出语言: ${language|default:zh-CN}

${frame_data}
```

---

### `src/visual/mod.rs` (config, static)

**Analog:** `src/ffmpeg/mod.rs` (line 1-3)

```rust
// Source: src/ffmpeg/mod.rs lines 1-3
pub mod command;
pub mod probe;
pub mod hwaccel;
```

```rust
// Expected pattern for src/visual/mod.rs
pub mod error;
pub mod types;
pub mod frame_extractor;
pub mod analyzer;
```

---

### `src/visual/error.rs` (model, static)

**Analog:** `src/script/error.rs` (see PromptError pattern above — same thiserror pattern)

**VisualError pattern (D-21/22):**
```rust
// Expected pattern — per D-22 variants
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisualError {
    #[error("帧提取失败: {0}")]
    FrameExtraction(String),
    #[error("视觉分析失败: {0}")]
    Analysis(String),
    #[error("部分批次失败: 已分析 {analyzed_count}/{total_count} 批次")]
    BatchPartial { analyzed_count: usize, total_count: usize, errors: Vec<String> },
}
```

---

### `src/visual/types.rs` (model, static)

**Analog:** `src/script/types.rs` — same serde struct pattern

**FrameObservation + BatchAnalysisResult pattern (D-15, from RESEARCH.md lines 652-673):**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrameObservation {
    pub frame_number: u64,
    pub timestamp: String,
    pub scene_description: String,
    pub objects: Vec<String>,
    pub actions: Vec<String>,
    pub on_screen_text: Option<String>,
    pub visual_salience: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAnalysisResult {
    pub observations: Vec<FrameObservation>,
    pub overall_activity_summary: Option<String>,
    pub total_frames: usize,
    pub analyzed_batches: usize,
    pub errors: Vec<String>,
}
```

---

### `src/visual/frame_extractor.rs` (service, file-I/O)

**Analog:** `src/ffmpeg/command.rs` — exact same spawn_blocking + ffmpeg-sidecar pattern

**Imports + ProgressCallback + spawn_blocking pattern:**
```rust
// Source: src/ffmpeg/command.rs lines 1-14
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;
use tokio_util::sync::CancellationToken;

use crate::error::FFmpegError;

/// FFmpeg 进度回调类型
/// 参数: Option<f64> 表示 0.0 到 1.0 的进度分数, &str 表示步骤描述
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send + Sync>;
```

**Fast path frame extraction pattern (D-12, from RESEARCH.md lines 534-578):**
```rust
// Expected pattern — fast path: single ffmpeg fps filter command
pub async fn extract_frames_fast_path(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: u32,
) -> Result<usize, VisualError> {
    let video = video_path.to_path_buf();
    let output = output_dir.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<usize, VisualError> {
        let mut cmd = FfmpegCommand::new();
        cmd.hide_banner()
            .loglevel("error")
            .input(video.to_str().ok_or_else(|| {
                VisualError::FrameExtraction("视频路径无效".into())
            })?)
            .video_filter(format!("fps=1/{}", interval_seconds))
            .codec_video("mjpeg")
            .args(["-q:v", &quality.to_string()])
            .start_number(0)
            .overwrite()
            .output(output.join("fastframe_%06d.jpg").to_str().unwrap());

        let mut child = cmd.spawn()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg启动失败: {}", e)))?;
        let iter = child.iter()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg迭代失败: {}", e)))?;

        for event in iter {
            if let FfmpegEvent::Error(e) = event {
                tracing::warn!("FFmpeg fast path error: {}", e);
            }
        }

        let _ = child.wait();
        // Rename: fastframe_XXXXXX.jpg -> keyframe_XXXXXX_HHMMSSmmm.jpg
        rename_fast_path_frames(&output, interval_seconds)
    }).await.map_err(|e| VisualError::FrameExtraction(format!("任务崩溃: {}", e)))?
}
```

**Frame extraction main entry with fallback:**
```rust
// Expected pattern — matches D-10/D-11/D-12
pub async fn extract_frames(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: Option<u32>,
    progress: Option<ProgressCallback>,
) -> Result<usize, VisualError> {
    let quality = quality.unwrap_or(5);  // default q:v=5
    
    // Try fast path first
    match extract_frames_fast_path(video_path, output_dir, interval_seconds, quality).await {
        Ok(count) if count > 0 => return Ok(count),
        _ => {
            // Clean up fast-path artifacts
            // Fallback to per-frame extraction with 4-level fallback:
            // 1) HW accel JPEG → 2) Software JPEG → 3) PNG → 4) BMP/MJPEG
            extract_frames_fallback(video_path, output_dir, interval_seconds, quality, progress).await
        }
    }
}
```

**Cancellation token pattern (from `src/ffmpeg/command.rs` lines 60-97):**
```rust
let cancel = CancellationToken::new();

let handle = tokio::task::spawn_blocking({
    let cancel = cancel.clone();
    move || {
        // ... ffmpeg setup ...
        for event in iter {
            if cancel.is_cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                return Err(...);
            }
            // process event
        }
    }
});

tokio::select! {
    result = handle => { /* handle result */ }
    _ = tokio::time::sleep(Duration::from_secs(600)) => {
        cancel.cancel();
        Err(VisualError::FrameExtraction("超时".into()))
    }
}
```

---

### `src/visual/analyzer.rs` (service, request-response + file-I/O)

**Analog:** `src/llm/openai_compatible.rs:analyze_images()` (lines 412-553)

**LlmProvider::analyze_images() calling pattern:**
```rust
// Source: src/llm/openai_compatible.rs lines 412-553
// Key pattern: spawn_blocking for image preprocessing, then Semaphore for LLM calls
async fn analyze_images(
    &self,
    images: &[PathBuf],
    prompt: &str,
    system_prompt: Option<&str>,
    batch_size: Option<usize>,
    max_concurrency: Option<usize>,
    response_format: Option<LlmResponseFormat>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<Vec<String>, LLMError> {
    // Step 1: Preprocess all images via spawn_blocking
    let handles: Vec<JoinHandle<Result<String, LLMError>>> = images.iter()
        .map(|p| {
            let p = p.clone();
            tokio::task::spawn_blocking(move || image_to_base64_data_url(&p))
        })
        .collect();
    let data_urls: Vec<String> = futures::future::join_all(handles).await
        .into_iter().collect::<Result<Vec<_>, _>>()
        .map_err(|join_err| LLMError::General(...))?;

    // Step 2: Chunk into batches
    let chunks: Vec<Vec<String>> = data_urls.chunks(batch_size).map(|c| c.to_vec()).collect();

    // Step 3: Semaphore-controlled concurrent LLM calls
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut handles = Vec::with_capacity(total_batches);
    for (batch_idx, batch) in chunks.into_iter().enumerate() {
        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire_owned().await.map_err(|_| ...)?;
            // build_vision_messages + client.chat().create(request)
            Ok::<_, LLMError>((batch_idx, text))
        }));
    }

    // Step 4: Collect results, sorting by batch index
    let results = futures::future::join_all(handles).await;
    // ...
    sorted_results.sort_by_key(|(idx, _)| *idx);
    Ok(sorted_results.into_iter().map(|(_, text)| text).collect())
}
```

**Visual analyzer entry point pattern (D-13/D-14/D-19, from RESEARCH.md lines 326-374):**
```rust
pub async fn analyze_video_frames(
    video_path: &Path,
    llm_provider: &dyn LlmProvider,
    prompt_template: &str,
    context: &HashMap<&str, &str>,
    batch_size: usize,
    max_concurrency: usize,
    progress: Option<ProgressCallback>,
) -> Result<BatchAnalysisResult, VisualError> {
    // Step 1: Extract keyframes
    let frame_dir = caller_specified_output_dir;
    let frame_count = extract_frames(video_path, &frame_dir, 3.0, None, progress.clone())
        .await.map_err(VisualError::FrameExtraction)?;

    // Step 2: Collect frame paths (D-24 naming pattern)
    let frame_paths = collect_frame_paths(&frame_dir)?;

    // Step 3: Hardcoded JSON format constraint (D-19 hybrid mode)
    let system_prompt = Some("你是一个专业的影视解说视觉分析助手...");
    let rendered_prompt = render_template(prompt_template, context)?;
    let full_prompt = format!(
        "{}\n\nIMPORTANT: Respond with valid JSON matching this schema: {{...}}",
        rendered_prompt
    );

    // Step 4: Batch send via Phase 2 LlmProvider
    let results = llm_provider.analyze_images(
        &frame_paths, &full_prompt, system_prompt.as_deref(),
        Some(batch_size), Some(max_concurrency),
        Some(LlmResponseFormat::Json),
        Some(0.1), Some(4096),
    ).await.map_err(|e| VisualError::Analysis(e.to_string()))?;

    // Step 5: Parse results, collecting errors (D-14)
    let mut observations = Vec::new();
    let mut errors = Vec::new();
    for (idx, text) in results.iter().enumerate() {
        match parse_and_retry(text, 2) {
            Ok(obs) => observations.push(obs),
            Err(e) => errors.push(format!("批次 {}: {}", idx, e)),
        }
    }

    if observations.is_empty() && !errors.is_empty() {
        return Err(VisualError::BatchPartial { analyzed_count: 0, total_count: results.len(), errors });
    }
    Ok(BatchAnalysisResult { observations, total_frames: frame_count, analyzed_batches: results.len(), errors })
}
```

**JSON parse-and-retry pattern (anti-pitfall 3, from RESEARCH.md lines 411-424):**
```rust
fn parse_and_retry(text: &str, max_attempts: usize) -> Result<FrameObservation, VisualError> {
    for attempt in 1..=max_attempts {
        let cleaned = text.trim()
            .strip_prefix("```json").or_else(|| text.trim().strip_prefix("```"))
            .map(|s| s.trim_end_matches("```").trim())
            .unwrap_or(text.trim());
        match serde_json::from_str::<FrameObservation>(cleaned) {
            Ok(obs) => return Ok(obs),
            Err(e) => { /* log and retry */ }
        }
    }
    Err(VisualError::Analysis("JSON parse failed after retries".into()))
}
```

---

### `src/lib.rs` (config, static) — MODIFY

**Current content (lines 1-6):**
```rust
// Source: src/lib.rs lines 1-6
pub mod config;
pub mod error;
pub mod tts;
pub mod llm;
pub mod ffmpeg;
pub mod script;
```

**Add `pub mod prompt;` and `pub mod visual;` after existing modules.**

---

### `Cargo.toml` (config, static) — MODIFY

**Add `regex = "1.11"` to [dependencies] (lines 10-31).**

---

## Shared Patterns

### Domain Error Enum with thiserror + Chinese Messages
**Source:** `src/error.rs` (entire file), `src/script/error.rs` (entire file)
**Apply to:** `src/prompt/error.rs`, `src/visual/error.rs`

All error enums in this project follow this pattern:
```rust
#[derive(thiserror::Error, Debug)]
pub enum FooError {
    #[error("中文错误类型描述: {0}")]
    VariantName(String),
}
```

### Registry with HashMap and Explicit Registration
**Source:** `src/llm/registry.rs` (lines 1-48), `src/llm/register.rs` (lines 28-116)
**Apply to:** `src/prompt/registry.rs`, `src/prompt/registry/mod.rs`

The two-phase pattern:
1. A `Registry` struct wrapping `HashMap<String, T>` with `register()` / `get()` / `list()` methods
2. A `register_all_*()` function that creates instances and registers them, collecting errors

### spawn_blocking for Blocking Operations
**Source:** `src/ffmpeg/command.rs` (lines 15-23, 62-135)
**Apply to:** `src/visual/frame_extractor.rs`

All FFmpeg/blocking I/O operations must be wrapped in `tokio::task::spawn_blocking()` to avoid blocking the async runtime.

### ProgressCallback Type
**Source:** `src/ffmpeg/command.rs` (line 13)
**Apply to:** `src/visual/frame_extractor.rs`, `src/visual/analyzer.rs`

```rust
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send + Sync>;
```

### serde Struct with `deny_unknown_fields` for Security
**Source:** `src/script/types.rs` (all structs), `src/config/types.rs` (all structs)
**Apply to:** `src/visual/types.rs`, `src/prompt/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foo { /* ... */ }
```

### Module Structure with pub mod + Inline Tests
**Source:** `src/ffmpeg/mod.rs`, `src/llm/mod.rs`, `src/script/mod.rs`
**Apply to:** `src/prompt/mod.rs`, `src/visual/mod.rs`

Each module follows:
```rust
pub mod submodule;
// (no re-exports or shared functions unless necessary)
// Each submodule has #[cfg(test)] mod tests { ... }
```

### Semaphore for Concurrency Control
**Source:** `src/llm/openai_compatible.rs` (lines 455-469)
**Apply to:** `src/visual/analyzer.rs`

```rust
let semaphore = Arc::new(Semaphore::new(max_concurrency));
// Acquire permit BEFORE the async block work
let _permit = sem_clone.acquire_owned().await.map_err(|_| ...)?;
```

### Image Preprocessing Reuse
**Source:** `src/llm/image_utils.rs:image_to_base64_data_url()` (entire file, 88 lines)
**Apply to:** `src/visual/analyzer.rs` — call the existing function, do NOT reimplement

The visual analyzer should NOT implement image preprocessing. Use `src/llm/image_utils.rs::image_to_base64_data_url()` directly, which is already used by `LlmProvider::analyze_images()`.

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/prompt/template.rs` | utility | transform | First regex-based template engine; no existing analog |
| `src/prompt/manager.rs` | service | CRUD | First manager/facade layer; no existing service with this granularity |
| `src/prompt/templates/*.md` | config | static | First compile-time embedded assets via include_str!() |
| `src/visual/analyzer.rs` | service | request-response | First visual analysis orchestration combining frames + LLM |

**Template files (`.md`)**: These are passive data files. They need no analog — their content is embedded at compile time and their schema is defined by the `${variable}` placeholders referenced in their corresponding Rust registration function. Each template file should be a copy/port of the corresponding Python template from `app/services/prompts/`.

---

## Metadata

**Analog search scope:**
- `src/ffmpeg/` — `mod.rs`, `command.rs`, `hwaccel.rs`, `probe.rs`
- `src/llm/` — `mod.rs`, `provider.rs`, `registry.rs`, `register.rs`, `types.rs`, `image_utils.rs`, `openai_compatible.rs`
- `src/script/` — `mod.rs`, `types.rs`, `error.rs`, `edit.rs`
- `src/config/` — `mod.rs`, `types.rs`, `defaults.rs`, `watcher.rs`
- `src/error.rs`, `src/lib.rs`, `Cargo.toml`

**Files scanned:** 22 existing Rust source files
**Pattern extraction date:** 2026-04-30
