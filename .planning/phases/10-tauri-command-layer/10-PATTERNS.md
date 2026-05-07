# Phase 10: Tauri Command Layer - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 21 (14 new, 7 modified)
**Analogs found:** 16 / 16 with at least role-match
**No analog found:** 5 (Tauri-specific files with no existing Rust counterpart)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src-tauri/Cargo.toml` | config | new | no Rust analog (Tauri-specific) | none |
| `src-tauri/build.rs` | build-script | new | no Rust analog (Tauri-specific) | none |
| `src-tauri/tauri.conf.json` | config | new | no Rust analog (Tauri-specific) | none |
| `src-tauri/capabilities/default.json` | config | new | no Rust analog (Tauri-specific) | none |
| `src-tauri/src/main.rs` | binary | request-response | no Rust analog (Tauri-specific) | none |
| `src-tauri/src/lib.rs` | setup | request-response | no Rust analog (Tauri-specific) | none |
| `src-tauri/src/error.rs` | model/utility | CRUD | `src/error.rs` | role-match |
| `src-tauri/src/models/mod.rs` | module | re-export | `src/jianying/mod.rs` | exact |
| `src-tauri/src/models/progress.rs` | model | request-response | `src/documentary/types.rs` | role-match |
| `src-tauri/src/commands/mod.rs` | macro | config | `src/llm/register.rs` | exact-pattern |
| `src-tauri/src/commands/pipeline.rs` | controller | request-response + event-driven | `src/documentary/pipeline.rs` | exact |
| `src-tauri/src/commands/script.rs` | controller | CRUD | `src/script/mod.rs` + `src/script/edit.rs` | exact |
| `src-tauri/src/commands/export.rs` | controller | request-response | `src/jianying/builder.rs` | exact |
| `src-tauri/src/commands/config.rs` | controller | request-response | `src/config/types.rs` | role-match |
| `Cargo.toml` (root, modify) | config | workspace | existing `Cargo.toml` | exact |
| `src/documentary/types.rs` (modify) | model | extend | existing `src/documentary/types.rs` | exact |
| `src/sde/types.rs` (modify) | model | extend | existing `src/sde/types.rs` | exact |
| `src/sdp/types.rs` (modify) | model | extend | existing `src/sdp/types.rs` | exact |
| `src/documentary/pipeline.rs` (modify) | service | extend | existing `src/documentary/pipeline.rs` | exact |
| `src/sde/pipeline.rs` (modify) | service | extend | existing `src/sde/pipeline.rs` | exact |
| `src/sdp/pipeline.rs` (modify) | service | extend | existing `src/sdp/pipeline.rs` | exact |

---

## Pattern Assignments

### `src-tauri/src/error.rs` (model/utility, error conversion)

**Analog:** `src/error.rs` (thiserror + From<T> pattern)

**Existing thiserror pattern** (src/error.rs lines 1-5):
```rust
use thiserror::Error;

/// 配置领域错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件读取失败: {0}")]
    IoError(String),
    // ...
}
```

**Existing From<T> pattern** (src/error.rs lines 42-46):
```rust
impl From<notify::Error> for ConfigError {
    fn from(err: notify::Error) -> Self {
        ConfigError::WatchError(err.to_string())
    }
}
```

**New CommandError pattern** (derived from D-03 + RESEARCH.md lines 366-401):
```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

// From impl per domain error type — 遵循 src/error.rs 的 From 转换模式
impl From<narratoai_core::documentary::error::PipelineError> for CommandError {
    fn from(err: narratoai_core::documentary::error::PipelineError) -> Self {
        CommandError {
            code: "PIPELINE_ERROR".into(),
            message: err.to_string(),  // Chinese message preserved (D-18)
        }
    }
}
```

**Key difference from analog:** `CommandError` uses `struct` + manual `Serialize` instead of `enum` + thiserror, because Tauri requires `Serialize` for IPC serialization.

**Domain errors requiring From< T > impls:**
| Domain Error | Error Code | Source File | Variants |
|---|---|---|---|
| `PipelineError` | `PIPELINE_ERROR` | `src/documentary/error.rs` | ScriptLoad, TtsGeneration, VideoClip, AudioMerge, Concat, Composite, Io, Timestamp, SrtGeneration, Validation, FrameExtraction, FFmpeg, Llm |
| `SdeError` | `SDE_ERROR` | `src/sde/error.rs` | ParseSubtitle, PlotAnalysis, ScriptGeneration, JsonRepair, VideoProcess, Validation, Io |
| `SdpError` | `SDP_ERROR` | `src/sdp/error.rs` | ParseSubtitle, LlmError, JsonRepair, VideoProcess, Validation, Io |
| `ConfigError` | `CONFIG_ERROR` | `src/error.rs` | IoError, ParseError, ValidationError, NotFound, WatchError |
| `ScriptError` | `SCRIPT_ERROR` | `src/script/error.rs` | JsonParse, Validation, Io, IndexOutOfBounds, InvalidTimestamp |
| `JianYingError` | `JIANYING_ERROR` | `src/jianying/error.rs` | Io, JsonSerialize, Validation, MissingField, ProbeError |
| `TTSError` | `TTS_ERROR` | `src/error.rs` | UnknownEngine, ConnectionFailed, AuthenticationFailed, SynthesisFailed, RetryExhausted |
| `LLMError` | `LLM_ERROR` | `src/error.rs` | General, ProviderNotFound, Configuration, APICall, Validation, ModelNotSupported, RateLimit, Authentication, ContentFilter |

---

### `src-tauri/src/models/mod.rs` (module, re-export)

**Analog:** `src/jianying/mod.rs` (lines 1-11)
```rust
pub mod builder;
pub mod error;
pub mod material;
pub mod segment;
pub mod template;
pub mod time;
pub mod track;
pub mod types;
```

**Pattern for new models/mod.rs:**
```rust
pub mod progress;
pub use progress::*;
```

---

### `src-tauri/src/models/progress.rs` (model, request-response)

**Analog:** `src/documentary/types.rs` (lines 86-98) — ProgressStep enum + ProgressCallback type

**Existing ProgressCallback pattern** (src/documentary/types.rs lines 96-98):
```rust
/// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(ProgressStep, f32, &str) + Send + Sync>;
```

**Existing PipelineState with emit_progress** (src/documentary/pipeline.rs lines 12-33):
```rust
pub struct PipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    // ...
    pub progress: Option<ProgressCallback>,
}

impl PipelineState {
    fn emit_progress(&self, step: ProgressStep, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}
```

**New ProgressPayload pattern** (derived from D-10 + RESEARCH.md lines 327-357):
```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub pipeline_type: String,      // "documentary" | "sde" | "sdp"
    pub task_id: String,            // UUID v4
    pub step_name: String,
    pub percent: f32,
    pub message: String,
    pub step_index: u32,
    pub total_steps: u32,
    pub status: String,             // "running" | "success" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandResponse {
    pub output_video_path: std::path::PathBuf,
    pub step_durations: Vec<StepDuration>,
    pub intermediate_paths: std::collections::HashMap<String, std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepDuration {
    pub step_name: String,
    pub duration_secs: f64,
}
```

**New TauriProgressCallback type** (derived from D-14 + RESEARCH.md):
```rust
/// Tauri 进度回调——命令闭包捕获 AppHandle，在回调中 emit 事件
pub type TauriProgressCallback = Box<dyn Fn(&str, f32, &str) + Send + Sync>;
```

**Key insight:** Instead of pipeline-specific progress types (Documentary's 6-step, SDE's 9-step, SDP's 3-step), the command layer uses a unified `ProgressPayload` with string-based `step_name` and `percent`. The command closure is responsible for converting pipeline-specific step enums to this unified format.

---

### `src-tauri/src/commands/mod.rs` (macro/registration, config)

**Analog:** `src/llm/register.rs` — `register_all_providers()` pattern (lines 45-116)

**Existing register_all_providers pattern** (src/llm/register.rs lines 45-116):
```rust
/// 注册所有活跃 provider（D-07）
pub fn register_all_providers(config: &AppConfig, registry: &mut Registry) -> Result<(), RegistrationErrors> {
    let mut errors: Vec<LLMError> = Vec::new();
    // ... sequential registration with error collection
    if errors.is_empty() { Ok(()) }
    else { Err(RegistrationErrors(errors)) }
}
```

**Analog:** `src/prompt/register.rs` — `register_all_prompts()` pattern (lines 24-32)

**Existing register_all_prompts pattern** (src/prompt/register.rs lines 24-32):
```rust
pub fn register_all_prompts(registry: &mut PromptRegistry) -> Result<(), RegistrationErrors> {
    let mut errors = Vec::new();
    register_documentary_prompts(registry, &mut errors);
    register_short_drama_editing_prompts(registry, &mut errors);
    register_short_drama_narration_prompts(registry, &mut errors);
    if errors.is_empty() { Ok(()) }
    else { Err(RegistrationErrors(errors)) }
}
```

**New register_all_commands!() macro pattern** (derived from D-05 + RESEARCH.md lines 284-305):
```rust
/// 集中注册所有 Tauri 命令，对齐 register_all_providers()/register_all_prompts() 模式
#[macro_export]
macro_rules! register_all_commands {
    () => {
        tauri::generate_handler![
            crate::commands::run_documentary,
            crate::commands::run_sde,
            crate::commands::run_sdp,
            crate::commands::generate_documentary_script,
            crate::commands::generate_sdp_script,
            crate::commands::load_script,
            crate::commands::save_script,
            crate::commands::validate_script,
            crate::commands::get_script_info,
            crate::commands::update_narration,
            crate::commands::set_ost,
            crate::commands::update_timestamp,
            crate::commands::export_jianying_draft,
        ]
    };
}
```

**Key pattern difference:** Uses `#[macro_export] macro_rules!` instead of a regular function because `tauri::generate_handler![]` must be expanded at the call site.

---

### `src-tauri/src/commands/pipeline.rs` (controller, request-response + event-driven)

**Analog:** `src/documentary/pipeline.rs` — `run_documentary()` (lines 471-523)

**Existing run_documentary signature** (src/documentary/pipeline.rs lines 471-475):
```rust
pub async fn run_documentary(
    request: DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
) -> Result<PathBuf, PipelineError> {
```

**Existing run_sde signature** (src/sde/pipeline.rs lines 34-40):
```rust
pub async fn run_sde(
    request: SdeRequest,
    config: &AppConfig,
    proxy: Option<&ProxySection>,
    registry: &Registry,
    prompt_manager: &PromptManager,
) -> Result<PathBuf, SdeError> {
```

**Existing run_sdp signature** (src/sdp/pipeline.rs lines 18-21):
```rust
pub async fn run_sdp(
    request: SdpRequest,
    _config: &crate::config::types::AppConfig,
) -> Result<PathBuf, SdpError> {
```

**New Tauri command wrapper pattern** (derived from RESEARCH.md lines 496-547 + D-01/D-11):
```rust
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[tauri::command]
pub async fn run_documentary(
    app: AppHandle,
    request: narratoai_core::documentary::types::DocumentaryRequest,
    config: tauri::State<'_, narratoai_core::config::types::AppConfig>,
) -> Result<crate::models::progress::CommandResponse, crate::error::CommandError> {
    let task_id = Uuid::new_v4().to_string();
    let progress_task_id = task_id.clone();

    // D-11: Build Tauri-aware callback closure
    let app_handle = app.clone();
    let progress_callback = Box::new(move |_step: &str, pct: f32, msg: &str| {
        let payload = crate::models::progress::ProgressPayload {
            pipeline_type: "documentary".to_string(),
            task_id: progress_task_id.clone(),
            step_name: _step.to_string(),
            percent: pct,
            message: msg.to_string(),
            step_index: 0,
            total_steps: 6,
            status: "running".to_string(),
            error_code: None,
            error_message: None,
        };
        let _ = app_handle.emit("pipeline-progress", payload);
    });

    // Execute pipeline — modified to accept ProgressCallback
    let output_path = narratoai_core::documentary::pipeline::run_documentary(
        request, &config, None, Some(progress_callback),
    ).await?;

    Ok(crate::models::progress::CommandResponse {
        output_video_path: output_path,
        step_durations: vec![],
        intermediate_paths: std::collections::HashMap::new(),
    })
}
```

**Key difference from analog:** The Tauri command is async, takes `AppHandle` as first arg, accesses config via `tauri::State<T>`, and uses `tauri::command` attribute macro.

---

### `src-tauri/src/commands/script.rs` (controller, CRUD)

**Analog:** `src/script/mod.rs` — load_script, save_script, validate (lines 131-154)

**Existing script functions** (src/script/mod.rs lines 132-154):
```rust
pub fn load_script(path: &Path) -> Result<Script, ScriptError> {
    let content = std::fs::read_to_string(path).map_err(ScriptError::Io)?;
    let script: Script = serde_json::from_str(&content).map_err(ScriptError::JsonParse)?;
    validate(&script)?;
    Ok(script)
}

pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    validate(script)?;
    let json = serde_json::to_string_pretty(script).map_err(ScriptError::JsonParse)?;
    // ... atomic write with temp file
}
```

**Analog:** `src/script/edit.rs` — update_narration, set_ost, update_timestamp (lines 6-43)

**Existing edit functions** (src/script/edit.rs lines 6-43):
```rust
pub fn update_narration(script: &Script, index: usize, text: &str) -> Result<Script, ScriptError> {
    if index >= script.len() { return Err(ScriptError::IndexOutOfBounds); }
    // validation...
    let mut new_script = script.clone();
    new_script[index].narration = text.to_string();
    Ok(new_script)
}

pub fn set_ost(script: &Script, index: usize, ost: OstType) -> Result<Script, ScriptError> {
    if index >= script.len() { return Err(ScriptError::IndexOutOfBounds); }
    let mut new_script = script.clone();
    new_script[index].ost = ost;
    Ok(new_script)
}

pub fn update_timestamp(script: &Script, index: usize, ts: &str) -> Result<Script, ScriptError> {
    if index >= script.len() { return Err(ScriptError::IndexOutOfBounds); }
    if !validate_timestamp(ts) { return Err(ScriptError::InvalidTimestamp(ts.to_string())); }
    let mut new_script = script.clone();
    new_script[index].timestamp = ts.to_string();
    Ok(new_script)
}
```

**Also:** `src/script/types.rs` — get_script_info (ScriptClip fields, lines 44-65)

**New command pattern:**
```rust
#[tauri::command]
pub async fn load_script(
    path: std::path::PathBuf,
) -> Result<narratoai_core::script::types::Script, crate::error::CommandError> {
    let script = narratoai_core::script::load_script(&path)?;
    Ok(script)
}

#[tauri::command]
pub async fn save_script(
    script: narratoai_core::script::types::Script,
    path: std::path::PathBuf,
) -> Result<(), crate::error::CommandError> {
    narratoai_core::script::save_script(&script, &path)?;
    Ok(())
}

#[tauri::command]
pub async fn validate_script(
    script: narratoai_core::script::types::Script,
) -> Result<(), crate::error::CommandError> {
    narratoai_core::script::validate(&script)?;
    Ok(())
}

// get_script_info: 返回剪辑片段的信息 (scripts.len(), etc.)
#[tauri::command]
pub async fn get_script_info(
    script: narratoai_core::script::types::Script,
) -> Result<ScriptInfo, crate::error::CommandError> {
    Ok(ScriptInfo {
        clip_count: script.len(),
        // ... other metadata
    })
}
```

**Key pattern:** All `#[tauri::command]` functions return `Result<T, CommandError>`. The `?` operator auto-converts via `From<T>` impls.

---

### `src-tauri/src/commands/export.rs` (controller, request-response)

**Analog:** `src/jianying/builder.rs` — `export_draft()` (line 254-277)

**Existing export_draft function** (src/jianying/builder.rs lines 254-277):
```rust
/// 导出脚本为剪映草稿
pub fn export_draft(req: &ExportRequest) -> Result<PathBuf, JianYingError> {
    validate_export_request(req)?;
    let draft_name = req.draft_name_or_default();
    let (folder, mut script_file) =
        DraftFolder::create_draft(&req.draft_path, &draft_name, req.width, req.height)?;
    let result = build_timeline(req, &mut script_file);
    match result {
        Ok(()) => match script_file.save(&folder) {
            Ok(draft_path) => Ok(draft_path),
            Err(e) => { let _ = std::fs::remove_dir_all(folder.draft_dir()); Err(e) }
        },
        Err(e) => { let _ = std::fs::remove_dir_all(folder.draft_dir()); Err(e) }
    }
}
```

**New command pattern:**
```rust
#[tauri::command]
pub async fn export_jianying_draft(
    script: narratoai_core::script::types::Script,
    video_origin_path: std::path::PathBuf,
    draft_path: std::path::PathBuf,
    draft_name: String,
    width: u32,
    height: u32,
) -> Result<std::path::PathBuf, crate::error::CommandError> {
    let request = narratoai_core::jianying::builder::ExportRequest {
        script,
        video_origin_path,
        draft_path,
        draft_name,
        width,
        height,
    };
    let result = narratoai_core::jianying::builder::export_draft(&request)?;
    Ok(result)
}
```

---

### `src-tauri/src/commands/config.rs` (controller, request-response)

**Analog:** `src/config/types.rs` — AppConfig struct (lines 4-29)

**Existing AppConfig pattern** (src/config/types.rs lines 4-29):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSection,
    // ...
}
```

**Key D-02 note:** Config is loaded at startup into Tauri State, not via a command. However, a `get_config` command may be needed for the frontend to read non-sensitive config values:
```rust
#[tauri::command]
pub async fn get_config(
    config: tauri::State<'_, narratoai_core::config::types::AppConfig>,
) -> Result<narratoai_core::config::types::AppConfig, crate::error::CommandError> {
    Ok(config.inner().clone())
}
```

**Warning:** API keys have `#[serde(skip_serializing)]` (src/config/types.rs lines 41, 49, 62-64, 100, 109, 120, 135, 174-176). When `AppConfig` is serialized via serde, these fields will be empty. Commands needing API keys must read from the `Arc<RwLock<AppConfig>>` global, not from `State<AppConfig>`.

---

### `src-tauri/src/lib.rs` (setup, application lifecycle) — **No Analog**

This file has no existing analog in the Rust library crate. It is a new Tauri-specific file.

**Expected pattern** (derived from RESEARCH.md lines 309-318):
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Load config.toml from multiple fallback paths
            let config = load_config()?;
            app.manage(config);
            Ok(())
        })
        .invoke_handler(register_all_commands!())
        .run(tauri::generate_context!())
        .expect("应用启动失败");
}
```

---

### `src-tauri/src/main.rs` (binary, entry point) — **No Analog**

This is the standard Tauri binary entry point:

**Expected pattern:**
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    narratoai_app::run();
}
```

---

### Cargo.toml (root, modify) — workspace manifest

**Existing Cargo.toml** (lines 1-43):
```toml
[package]
name = "narratoai-core"
version = "0.1.0"
edition = "2021"
# ... dependencies ...
```

**New workspace pattern:**
```toml
[workspace]
members = ["narratoai-core", "src-tauri"]
resolver = "2"

[package]
name = "narratoai-core"
# ... rest unchanged ...
```

---

## Modified Files

### `src/documentary/types.rs` — Add Deserialize + Modify ProgressCallback

**Current type** (lines 6-8):
```rust
#[derive(Debug, Clone)]
pub struct DocumentaryRequest {
```

**New:** Add `Deserialize` to derive and modify `ProgressCallback`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]  // Added Deserialize
pub struct DocumentaryRequest {
    // fields unchanged
}

// Changed from pipeline-specific to generic string-based callback
pub type ProgressCallback = Box<dyn Fn(&str, f32, &str) + Send + Sync>;
```

### `src/sde/types.rs` — Add Deserialize + Modify Callback

**Same pattern:** Add `Deserialize` to `SdeRequest`, change `SdeProgressCallback` type.

### `src/sdp/types.rs` — Add Deserialize + Modify Callback

**Same pattern:** Add `Deserialize` to `SdpRequest`, change `SdpProgressCallback` type.

### `src/documentary/pipeline.rs` — Accept ProgressCallback

**Current signature** (lines 471-475):
```rust
pub async fn run_documentary(
    request: DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
) -> Result<PathBuf, PipelineError> {
```

**New signature:**
```rust
pub async fn run_documentary(
    request: DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
    progress: Option<ProgressCallback>,  // NEW: D-14 callback injection
) -> Result<PathBuf, PipelineError> {
```

The `progress: None` default at internal call sites ensures backward compatibility.

### `src/sde/pipeline.rs` — Accept ProgressCallback

**Current signature** (lines 34-40):
```rust
pub async fn run_sde(
    request: SdeRequest,
    config: &AppConfig,
    proxy: Option<&ProxySection>,
    registry: &Registry,
    prompt_manager: &PromptManager,
) -> Result<PathBuf, SdeError> {
```

**New signature:** Add `progress: Option<ProgressCallback>` parameter.

### `src/sdp/pipeline.rs` — Accept ProgressCallback

**Current signature** (lines 18-21):
```rust
pub async fn run_sdp(
    request: SdpRequest,
    _config: &crate::config::types::AppConfig,
) -> Result<PathBuf, SdpError> {
```

**New signature:** Add `progress: Option<ProgressCallback>` parameter.

---

## Shared Patterns

### `#[tauri::command]` Attribute Macro
**Source:** Tauri 2.0 convention
**Apply to:** All command files (`pipeline.rs`, `script.rs`, `export.rs`, `config.rs`)
```rust
#[tauri::command]
pub async fn command_name(
    app: AppHandle,  // only for pipeline commands that emit events
    // other params as needed
) -> Result<ResponseType, CommandError> {
    // ...
}
```
**Rule:** All commands must:
1. Be annotated with `#[tauri::command]`
2. Return `Result<T, CommandError>` 
3. Use serde-serializable types for parameters and return values

### CommandError + `std::error::Error` + Serialize
**Source:** Tauri 2.0 requirements + RESEARCH.md Pitfall 2
**Apply to:** All commands returning `Result`
```rust
#[derive(Debug, Clone, Serialize)]
pub struct CommandError { /* ... */ }
impl std::fmt::Display for CommandError { /* ... */ }
impl std::error::Error for CommandError {}
```

### Config Access via `tauri::State<T>`
**Source:** D-02 + D-06
**Apply to:** All commands needing `AppConfig`
```rust
config: tauri::State<'_, narratoai_core::config::types::AppConfig>,
```

### Global Arc/RwLock Access (for LLM/Prompt/TTS registries)
**Source:** D-06 + existing patterns
**Apply to:** Pipeline commands that need LLM provider or prompt manager
```rust
let registry = narratoai_core::llm::registry::Registry::global();  // or similar
```

### Progress Event Emission via `AppHandle`
**Source:** D-10/D-11/D-12
**Apply to:** Pipeline commands (`run_documentary`, `run_sde`, `run_sdp`)
```rust
let app_handle = app.clone();
let callback = Box::new(move |step: &str, pct: f32, msg: &str| {
    let payload = ProgressPayload { /* ... */ };
    let _ = app_handle.emit("pipeline-progress", payload);
});
```

### Multi-fallback Config Loading
**Source:** RESEARCH.md Open Questions (Q2)
**Apply to:** `src-tauri/lib.rs` setup
```rust
// Try (1) resource_dir, (2) current_dir, (3) plain path
let config_path = app.path().resource_dir()
    .unwrap_or_else(|_| PathBuf::from("."))
    .join("config.toml");
let config_path = if config_path.exists() {
    config_path
} else {
    // fallback
    PathBuf::from("config.toml")
};
```

---

## No Analog Found

These files have no close match in the existing codebase because they are Tauri-specific (new to Phase 10). Planner should use RESEARCH.md patterns and Tauri 2.0 official documentation:

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `src-tauri/Cargo.toml` | config | static | Tauri binary crate deps, no existing analog |
| `src-tauri/build.rs` | build-script | static | Tauri build script, no existing analog |
| `src-tauri/tauri.conf.json` | config | static | Tauri app configuration, no existing analog |
| `src-tauri/capabilities/default.json` | config | static | Tauri permission system, no existing analog |
| `src-tauri/src/main.rs` | binary | startup | Tauri binary entry point, no existing analog |
| `src-tauri/src/lib.rs` | setup | startup | Tauri builder setup, no existing analog |

**Reference project:** smartEdit (`E:\GitLib\smartEdit\src-tauri`) provides a proven Tauri 2.0 project structure template for all no-analog files. Do NOT reuse code, but do reference for file structure and convention.

---

## Metadata

**Analog search scope:**
- `E:\GitLib\NarratoAI\src\**\*.rs` (all Rust library source)
- `E:\GitLib\NarratoAI\Cargo.toml` (workspace/dependencies)

**Files scanned:** 65 Rust source files, 1 Cargo.toml
**Pattern extraction date:** 2026-05-07
