# Phase 10: Tauri Command Layer - Research

**Researched:** 2026-05-07
**Domain:** Tauri 2.0 Desktop Application Backend (Commands, Events, File Dialogs)
**Confidence:** HIGH

## Summary

Phase 10 wraps the existing `narratoai-core` Rust library crate (`src/lib.rs`) into a Tauri 2.0 desktop application backend. This involves creating a `src-tauri/` binary crate that consumes the library, registers ~15 Tauri commands (1:1 mapping to existing Rust APIs), implements real-time progress event emission via `AppHandle.emit()`, and integrates the `tauri-plugin-dialog` for file selection dialogs.

The key architectural challenge is bridging three pipeline-specific progress types (Documentary's 6-step, SDE's 9-step, SDP's 3-step) into a unified `ProgressPayload` struct for Tauri events. The secondary challenge is wrapping seven distinct domain error types (`PipelineError`, `SdeError`, `SdpError`, `ConfigError`, `ScriptError`, `SdpError`, `JianYingError`) into a serializable `CommandError` with `code: String` + `message: String`.

The project references `smartEdit` (`E:\GitLib\smartEdit`) as a proven Tauri 2.0 + Rust architecture, though code will not be reused. smartEdit's structure (`src-tauri/src/lib.rs` with `run()` + `src-tauri/src/main.rs` calling it, commands in `src-tauri/src/commands/`, capabilities in `src-tauri/capabilities/`) serves as the recommended project structure template.

**Primary recommendation:** Cargo workspace layout with `narratoai-core` as the library member and a new `src-tauri` binary member. Tauri 2.0 v2.11.1, `tauri-plugin-dialog` v2.7.1, `tauri-build` v2.5.6.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Thin wrapper 模式——每个领域入口函数对应一条 Tauri 命令，约 15 条命令，1:1 映射现有 Rust API。不做 Facade 分组。
- **D-02:** 配置启动加载——应用启动时加载 config.toml 到 Tauri State（`tauri::State<AppConfig>`），命令通过 `tauri::State` 读取。不需要显式配置命令。
- **D-03:** 统一 CommandError 包装——定义 `CommandError`（`code: String` + `message: String`），实现 `serde::Serialize`。为所有领域错误实现 `From<T>` 自动转换。`message` 保留中文错误信息（与 Phase 1 D-18 一致），`code` 用于前端程序化区分。
- **D-04:** 命令清单——流水线（`run_documentary`、`run_sde`、`run_sdp`、`generate_documentary_script`、`generate_sdp_script`）、脚本全操作（`load_script`、`save_script`、`validate_script`、`get_script_info`、`update_narration`、`set_ost`、`update_timestamp`）、导出（`export_jianying_draft`）。约 15 条命令。
- **D-05:** 集中注册函数——创建 `register_all_commands()` 统管命令注册，与现有 `register_all_providers()`/`register_all_prompts()` 模式对齐。在 Tauri builder 的 `setup` 阶段调用。
- **D-06:** Tauri State + 全局 Arc 混合——Config 通过 Tauri State 注入（`app.manage(config)`），LLM/Prompt/TTS 注册表保持现有全局 Arc/RwLock 模式，命令内部直接访问。最小改动现有代码。
- **D-07:** 返回 OutputPath + 统计——流水线命令 await 完成后返回 `CommandResponse`（`output_video_path: PathBuf` + `step_durations: Vec<StepDuration>` + `intermediate_paths: HashMap<String, PathBuf>`）。前端可预览视频和查看统计。与现有 PipelineState 最终状态对齐。
- **D-08:** 统一 UTF-8 PathBuf——Tauri 2.0 自动将 JS 路径转为 Rust `PathBuf`，不做额外标准化。跨平台兼容。
- **D-09:** 复用现有类型 + 新增 CommandResponse——流水线命令复用现有 `DocumentaryRequest`/`SdeRequest`/`SdpRequest`/`ScriptClip` 等类型。新增 `CommandResponse` 作为统一返回包装，`CommandError` 作为统一错误类型。
- **D-10:** 统一 ProgressPayload 结构体——所有流水线共用：`pipeline_type`（"documentary"/"sde"/"sdp"）、`task_id`（UUID）、`step_name`、`percent`（0-100）、`message`（中文描述）、`step_index`、`total_steps`、`status`（"running"/"success"/"error"）、`error_code`、`error_message`。
- **D-11:** 回调注入 + AppHandle emit——流水线命令入口持有 `AppHandle`，注入到 `ProgressCallback`。每次回调触发时 `app_handle.emit("pipeline-progress", payload)` 推送事件。最小改动现有 step 函数签名。
- **D-12:** 单一事件 + pipeline_type 区分——事件名统一为 `"pipeline-progress"`，前端一个监听器，按 `pipeline_type` 字段路由到不同 UI 面板。
- **D-13:** 进度事件含状态字段——`status` 字段覆盖 running/success/error。错误时 payload 含 `error_code` + `error_message`，不单独发送错误事件。
- **D-14:** 扩展现有回调类型——定义 `TauriProgressCallback`：`Fn(ProgressStep)`，其中 `ProgressStep` 含 step_name、percent、message、status。命令入口用闭包捕获 AppHandle，每次回调 emit 事件。现有流水线进度调用点改为新签名。
- **D-15:** UUID task_id——每个命令调用入口生成 UUID v4 作为 `task_id`，嵌入 ProgressPayload。前端可追踪特定任务，保持可扩展性。

### Claude's Discretion
- Tauri 2.0 项目脚手架生成方式（`cargo tauri init` vs 手动搭建）
- 命令模块文件结构（`src-tauri/src/commands/` 目录组织）
- ProgressPayload 的具体字段类型和序列化属性
- CommandResponse 的具体字段定义
- register_all_commands() 的具体实现位置（`src-tauri/src/main.rs` setup 阶段或独立文件）
- Cargo.toml 依赖版本选择（tauri, tauri-build, serde）
- 是否需要 Cargo workspace 布局（lib crate + tauri binary）
- Tauri 文件对话框的权限配置（`tauri.conf.json` 中的 fs scope）

### Deferred Ideas (OUT OF SCOPE)
- 前端的 Vue 3 + Vuetify 3 实现 — v2 里程碑
- 流水线取消/暂停功能
- 多语言错误信息支持
- 前端 UI 面板（纪录片/SDE/SDP 工作面板、脚本编辑器、进度展示）
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TAURI-01 | Tauri 2.0 命令注册——暴露所有 Rust 业务函数给前端 | ~15 thin wrapper commands per D-04; `register_all_commands!()` macro pattern; `generate_handler!` registration; workspaces isolate Tauri deps |
| TAURI-02 | 进度事件推送——通过 Tauri events 实时推送任务进度 | D-10/11/12/13/14/15 define ProgressPayload + AppHandle.emit pattern; Tauri 2.0 `Emitter` trait on `AppHandle`; note Channel API as modern alternative with caveat D-11 locked |
| TAURI-03 | 文件对话框集成——视频/字幕文件选择 | `tauri-plugin-dialog` v2.7.1 with `open()` for file selection; capabilities config for `dialog:default`; paths auto-converted to PathBuf per D-08 |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Command registration | Tauri binary (src-tauri) | Library (narratoai-core) | Tauri commands must be registered in the `tauri::Builder` invoke_handler; business logic lives in the library |
| Config management | Tauri startup (setup) | Library (ConfigManager) | Config loaded at app start, stored as `tauri::State<AppConfig>` for command access |
| Pipeline execution | Library (narratoai-core) | Tauri command wrapper | Commands are thin wrappers that call existing `run_documentary()`, `run_sde()`, `run_sdp()` |
| Progress events | Tauri command (AppHandle) | Library (callback injection) | Commands hold `AppHandle`, inject Tauri-aware callback into existing pipeline |
| File dialog | Tauri frontend JS | Tauri plugin (tauri-plugin-dialog) | Open/save dialogs are called from JS via `@tauri-apps/plugin-dialog`, paths passed to Rust commands |
| Error serialization | Tauri command layer | Library (From impls) | `CommandError` in src-tauri, `From<T>` impls for each domain error type |
| Script management | Library (script module) | Tauri command wrapper | Commands call existing `load_script()`, `save_script()`, `validate()` |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri` | 2.11.1 | Desktop app framework (IPC, window, events) | Current stable; all Tauri 2.0 APIs stable; tested by smartEdit project |
| `tauri-build` | 2.5.6 | Build script for Tauri app | Required build dependency for Tauri 2.0 [VERIFIED: crates.io] |
| `tauri-plugin-dialog` | 2.7.1 | Native file open/save dialogs | Tauri's official dialog plugin; supports filters, platform-native UI [VERIFIED: crates.io] |
| `tauri-plugin-fs` | 2.5.0 | File system access with capability scoping | Required when dialog returns file paths; manages runtime scope [VERIFIED: crates.io] |
| `serde` | 1.0.228 | Serialization for Tauri IPC | Already in project; required for command params and return types |
| `serde_json` | 1.0.140 | JSON serialization for events | Already in project; needed for event payload serialization |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `uuid` | 1.23.1 | Generate task_id UUIDs | Already in project; used by D-15 for ProgressPayload task_id |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `app.emit()` events | `tauri::ipc::Channel` | Channel is Tauri's **recommended** approach for progress streaming (type-safe, point-to-point, auto-cleanup). Events use pub/sub broadcast with JSON string payloads. Decision D-11 locks events; Channels noted here for future consideration. |
| Monolithic `src-tauri` crate | Cargo workspace | Workspace allows `narratoai-core` to remain a pure library (testable, reusable). Also separates Tauri-specific deps. |
| Single command file | Per-domain command files | smartEdit uses per-domain files (commands/health.rs, commands/tts.rs, etc.); aligns with D-04's ~15 commands. |

**Installation (Cargo.toml additions for src-tauri):**
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
narratoai-core = { path = ".." }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Version verification:**
```
tauri = "2.11.1"   [VERIFIED: cargo search, 2026-05-07]
tauri-plugin-dialog = "2.7.1" [VERIFIED: cargo search]
tauri-build = "2.5.6" [VERIFIED: lib.rs/crates/tauri-build]
tauri-plugin-fs = "2.5.0" [VERIFIED: crates.io]
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2.0 App (src-tauri)              │
│                                                           │
│  ┌─────────────────┐    ┌──────────────────────────────┐ │
│  │   main.rs        │    │   lib.rs                     │ │
│  │   fn main() {    │    │   pub fn run() {             │ │
│  │     app_lib::run │───▸│     tauri::Builder::default()│ │
│  │   }              │    │       .plugin(dialog::init())│ │
│  └─────────────────┘    │       .plugin(fs::init())     │ │
│                         │       .manage(config)         │ │
│  ┌─────────────────┐    │       .setup(|app| { ... })   │ │
│  │  commands/       │    │       .invoke_handler(       │ │
│  │  mod.rs          │    │         generate_handler![   │ │
│  │  pipeline.rs     │───▸│           ...                │ │
│  │  script.rs       │    │         ])                    │ │
│  │  export.rs       │    │       .run(generate_context!())│
│  │  config.rs       │    └──────────────────────────────┘ │
│  └────────┬────────┘                                      │
│           │                                                │
└───────────┼────────────────────────────────────────────────┘
            │ invoke (IPC)
            ▼
┌─────────────────────────────────────────────────────────┐
│           Frontend (Vue 3 + Vuetify 3 - v2 milestone)    │
│                                                           │
│  await invoke('run_documentary', {...})                   │
│  await listen('pipeline-progress', handler)               │
│  const file = await open({ filters: [...] })              │
│                                                           │
└─────────────────────────────────────────────────────────┘
            │
            │ emit events during pipeline execution
            ▼
┌─────────────────────────────────────────────────────────┐
│              narratoai-core Library Crate                 │
│                                                           │
│  ┌────────────┐ ┌────────┐ ┌─────────┐ ┌──────────────┐ │
│  │ documentary │ │  sde   │ │   sdp   │ │   script     │ │
│  │ pipeline.rs│ │pipeline│ │pipeline │ │   mod.rs      │ │
│  └────────────┘ └────────┘ └─────────┘ └──────────────┘ │
│         │            │          │              │          │
│         ▼            ▼          ▼              ▼          │
│  ┌──────────────────────────────────────────────────────┐│
│  │  Callback: ProgressCallback (Fn(ProgressStep, f32,  ││
│  │  &str)) → Tauri command wraps with AppHandle emit    ││
│  └──────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### Data Flow (Pipeline Execution)

```
1. JS invoke('run_documentary', { video_path, script_path, ... })
   │
2. Tauri IPC deserializes args into DocumentaryRequest (serde)
   │
3. Command function:
   a. Generates UUID task_id (D-15)
   b. Reads AppConfig from tauri::State<AppConfig>
   c. Reads LLM registry from global Arc<RwLock<...>>
   d. Creates TauriProgressCallback closure capturing AppHandle
   e. Injects callback into pipeline
   f. Awaits run_documentary(...)
   │
4. Pipeline runs (6 steps):
   ┌─────────────────────────────────────────────────┐
   │ Each step → callback(step, pct, msg)             │
   │   → closure calls app_handle.emit(              │
   │       "pipeline-progress", ProgressPayload {...} │
   │     )                                            │
   └─────────────────────────────────────────────────┘
   │
5. Pipeline returns Result<PathBuf, PipelineError>
   │
6. Command maps to CommandResponse (success) or CommandError (error)
   │
7. JS receives response
```

### Cargo Workspace Layout (Recommended)

```
narratoai/
├── Cargo.toml              # [workspace] manifest
│   members = ["narratoai-core", "src-tauri"]
│
├── narratoai-core/         # Existing library crate
│   ├── Cargo.toml          # (current ./Cargo.toml)
│   └── src/
│       ├── lib.rs
│       ├── documentary/...
│       ├── sde/...
│       ├── sdp/...
│       └── ...
│
├── src-tauri/              # NEW: Tauri binary crate
│   ├── Cargo.toml          # depends on narratoai-core
│   ├── build.rs            # tauri_build::build()
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/              # App icons (generated)
│   └── src/
│       ├── main.rs         # fn main() → calls lib::run()
│       ├── lib.rs          # pub fn run() → Tauri builder
│       ├── error.rs        # CommandError + From impls
│       ├── commands/
│       │   ├── mod.rs      # register_all_commands() + re-exports
│       │   ├── pipeline.rs # run_documentary, run_sde, run_sdp commands
│       │   ├── script.rs   # script management commands
│       │   ├── export.rs   # jianying export command
│       │   └── config.rs   # get_config command (D-02)
│       └── models/
│           ├── mod.rs
│           └── progress.rs # ProgressPayload, CommandResponse, StepDuration
```

### Alternative Layout (Single Crate, No Workspace)

If the workspace overhead is undesirable, Tauri 2.0 supports having both `[lib]` and `[[bin]]` in the same `Cargo.toml`:

```
narratoai/
├── Cargo.toml              # [lib] + [[bin]] + [dependencies] tauri
│   [lib] name = "narratoai_core"
│   [[bin]] name = "narratoai" path = "src-tauri/src/main.rs"
├── src/                    # Existing library code
├── src-tauri/              # Tauri-specific files
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── error.rs
│       └── commands/
```

**Recommendation:** Use the workspace layout. It keeps the existing library crate clean (no Tauri dependency leakage) and follows standard Tauri ecosystem practice. [VERIFIED: smartEdit project structure, Tauri GitHub discussions]

### Pattern 1: register_all_commands() (D-05)
**What:** Central command registration aligned with existing `register_all_providers()`/`register_all_prompts()` patterns.
**When to use:** In `src-tauri/src/commands/mod.rs`, export a macro or function that returns the handler list.
**Example:**
```rust
// src-tauri/src/commands/mod.rs
mod pipeline;
mod script;
mod export;
mod config;

pub use pipeline::*;
pub use script::*;
pub use export::*;
pub use config::*;

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

Then in `lib.rs`:
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(/* AppConfig */)
        .invoke_handler(register_all_commands!())
        .run(tauri::generate_context!())
        .expect("应用启动失败");
}
```

[Pattern source: smartEdit `src-tauri/src/lib.rs` - uses inline `generate_handler!` with ~60 commands]
[VERIFIED: Tauri 2.0 docs - `generate_handler!` macro]

### Pattern 2: TauriProgressCallback (D-14)
**What:** Extends the existing `ProgressCallback` type to capture `AppHandle` and emit events. Three pipeline types have their own `ProgressStep` enums — the command layer unifies them via `ProgressPayload`.
**When to use:** In each pipeline command function, create a closure that converts the pipeline-specific step enum into a unified `ProgressPayload` and emits it.
**Example:**
```rust
// src-tauri/src/models/progress.rs
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

/// 供 documentary 流水线使用的默认百分比映射
pub fn documentary_percent_ranges() -> Vec<(u32, &'static str, f32)> {
    vec![
        (0, "load_script", 0.0),      // Step 1: 0%
        (1, "tts", 20.0),             // Step 2: 20%
        (2, "clip", 60.0),            // Step 3: 60%
        (3, "merge_audio", 70.0),     // Step 4: 70%
        (4, "concat", 80.0),          // Step 5: 80%
        (5, "composite", 100.0),      // Step 6: 100%
    ]
}
```

[Source: CONTEXT.md D-10 through D-15; existing `ProgressCallback` in `src/documentary/types.rs`]

### Pattern 3: CommandError + From<T> (D-03)
**What:** Unified error type with `code: String` + `message: String` that all domain errors convert into.
**When to use:** As the return type `Result<T, CommandError>` for all Tauri commands.
**Example:**
```rust
// src-tauri/src/error.rs
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

// From implementations for each domain error type
impl From<narratoai_core::documentary::error::PipelineError> for CommandError {
    fn from(err: narratoai_core::documentary::error::PipelineError) -> Self {
        CommandError {
            code: "PIPELINE_ERROR".into(),
            message: err.to_string(),  // Chinese message preserved (D-18)
        }
    }
}

impl From<narratoai_core::sde::error::SdeError> for CommandError { /* ... */ }
impl From<narratoai_core::sdp::error::SdpError> for CommandError { /* ... */ }
impl From<narratoai_core::error::ConfigError> for CommandError { /* ... */ }
impl From<narratoai_core::script::error::ScriptError> for CommandError { /* ... */ }

// Tauri requires errors to implement std::error::Error for Into<tauri::ipc::InvokeError>
impl std::error::Error for CommandError {}
```

**Important:** Tauri 2.0 requires command errors to implement `std::error::Error`. The `tauri::command` macro automatically converts `Result<T, E: std::error::Error>` into serialized error responses. [VERIFIED: Tauri 2.0 docs - Command Error Handling]

### Pattern 4: Config Injection via Tauri State (D-02, D-06)
**What:** Load config.toml in `setup()` callback, store via `app.manage()`, access in commands via `tauri::State<AppConfig>`.
**When to use:** In the `setup` closure, load `ConfigManager`, extract `AppConfig`, and manage it.
**Example:**
```rust
// src-tauri/src/lib.rs — setup phase
.setup(|app| {
    let config_path = app.path().resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.toml");

    // Fallback: try current dir
    let config_path = if config_path.exists() {
        config_path
    } else {
        PathBuf::from("config.toml")
    };

    let config_manager = narratoai_core::config::ConfigManager::load(&config_path)
        .map_err(|e| format!("配置加载失败: {}", e))?;

    // Clone AppConfig for Tauri State (does NOT include secrets if skip_serializing)
    let app_config = config_manager.get();
    app.manage(app_config);

    // Also store the Arc<RwLock<AppConfig>> for hot-reload use if needed
    app.manage(config_manager.config());

    Ok(())
})
```

**Note:** `AppConfig` uses `#[serde(skip_serializing)]` on API key fields (verified in `src/config/types.rs` lines 42, 49, 62-64, 100, 109, 120, 135, 174-176). These fields will be empty when `Clone`d via `get()`. If commands need API keys, they should read from the `Arc<RwLock<AppConfig>>` instead, or the `skip_serializing` attributes should be removed. **This is a potential data gap to verify.**

### Anti-Patterns to Avoid
- **Wrapping Arc/RwLock in Tauri State again:** Tauri's `State` already provides shared ownership. If the library already uses `Arc<RwLock<AppConfig>>`, do not wrap it in another `Arc` — use `State<Arc<RwLock<AppConfig>>>` directly. [VERIFIED: Tauri 2.0 State Management docs]
- **Blocking the async runtime in commands:** Pipeline functions use `tokio::task::spawn_blocking` for FFmpeg calls. Tauri commands are async — ensure long-running CPU work doesn't block the main runtime. Existing code already handles this correctly [VERIFIED: `src/ffmpeg/command.rs` uses `spawn_blocking`].
- **Duplicate serialization:** Tauri 2.0 auto-serializes return values via serde. Do not manually JSON-serialize in the command function — return the struct directly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| File dialogs | Custom file picker UI | `tauri-plugin-dialog` (open/save/confirm) | Platform-native dialogs, auto path scoping, filter support |
| File path scoping | Custom permission checking | `tauri-plugin-fs` + `tauri::FsExt` runtime API | Tauri's capabilities system handles paths at the Rust level; dynamic scope allows runtime path grants |
| IPC serialization | Manual JSON encode/decode | `#[tauri::command]` + serde derive | Tauri 2.0 auto-serializes args and return values via serde; manual encoding adds boilerplate with no benefit |
| Event pub/sub | Custom WebSocket/channel | `AppHandle::emit()` / `app_handle.listen()` | Built-in Tauri event system handles webview lifecycle and cleanup |

**Key insight:** Tauri 2.0 provides all the IPC infrastructure needed. The command functions should be "thin" — minimal glue code wrapping existing library calls. All business logic remains in `narratoai-core`.

## Common Pitfalls

### Pitfall 1: ProgressStep Enum Mismatch
**What goes wrong:** Each pipeline defines its own `ProgressStep` enum with different variants. The Tauri event needs a unified payload, but the callback pattern currently uses `Fn(ProgressStep, f32, &str)` — type-specific per pipeline.
**Why it happens:** Documentary has 6 steps (`ProgressStep` in `documentary/types.rs`), SDE has 9 (`SdeProgressStep` in `sde/types.rs`), SDP has 3 (`SdpProgressStep` in `sdp/types.rs`). They cannot be used polymorphically.
**How to avoid:** The command-layer closure should match on each step type and construct a unified `ProgressPayload`. Alternatively, use `Box<dyn Fn(&str, f32, &str)>` as the pipeline callback (string-based step names), avoid type coupling. D-14 already specifies this pattern — the command closure is responsible for conversion.
**Warning signs:** Compile errors about mismatched callback types, or trying to pass `SdeProgressStep` where `ProgressStep` is expected.

### Pitfall 2: Missing `std::error::Error` impl on CommandError
**What goes wrong:** Tauri 2.0 commands returning `Result<T, E>` require `E: std::error::Error`. Without it, the command fails to compile or errors serialize incorrectly.
**Why it happens:** `CommandError` is a custom struct with `Serialize` but missing the `Error` trait impl.
**How to avoid:** Derive or implement `std::error::Error` for `CommandError`. The minimal impl is `impl std::error::Error for CommandError {}` (no methods needed since Display and Debug handle the rest).
**Warning signs:** Tauri command compilation errors mentioning `InvokeError` or `Into<tauri::ipc::InvokeError>`.

### Pitfall 3: `#[tauri::command]` Argument Type Constraints
**What goes wrong:** Tauri commands can deserialize arguments from JS, but only types implementing `serde::Deserialize`. Complex types like `PathBuf` work (Tauri 2.0 auto-converts JS strings to PathBuf), but type mismatches cause silent failures.
**Why it happens:** Some existing request types may not derive `Deserialize`. The `DocumentaryRequest`, `SdeRequest`, `SdpRequest` currently derive `Debug, Clone` only — need `Deserialize` added.
**How to avoid:** Add `#[derive(Deserialize)]` to all command argument types. Verify by testing `serde_json::from_str::<T>(json_string)`.
**Warning signs:** Tauri returning "invalid args" error from JS `invoke()` calls.

### Pitfall 4: Capability Permissions Blocking File Dialog
**What goes wrong:** The file dialog opens, selects a file, but the path is not accessible for reading because `fs` permissions are not configured.
**Why it happens:** Tauri 2.0's capabilities system requires explicit permission grants. `dialog:allow-open` alone does not grant file read access.
**How to avoid:** Configure `dialog:allow-open` with appropriate path scopes AND `fs:allow-read` (or use `FsExt::allow_directory()` at runtime to grant access to the selected path). [VERIFIED: Tauri 2.0 docs + Stack Overflow]
**Warning signs:** Dialog returns a path, but `std::fs::read_to_string(path)` returns PermissionDenied.

### Pitfall 5: Tauri Event System Flooding (Known Bug #13234)
**What goes wrong:** Rapid successive `app.emit()` calls freeze the app when `tracing` feature is enabled in Tauri.
**Why it happens:** A Tauri 2.0 bug where event emission with `tracing` causes a deadlock under high-frequency emits.
**How to avoid:** If pipeline steps emit many events rapidly, either: (a) disable the `tracing` feature in Tauri, (b) remove the `devtools` plugin, or (c) throttle emits in the callback closure (e.g., only emit on percentage boundaries). For this project (one pipeline at a time, ~6-9 step emits per run), flooding is unlikely to be an issue. [CITED: tauri-apps/tauri#13234]
**Warning signs:** App becomes unresponsive during pipeline progress.

### Pitfall 6: Library Visibility — pub(crate) Functions
**What goes wrong:** Pipeline functions like `step_tts`, `step_clip` are `pub(crate)` and not accessible from the `src-tauri` binary crate.
**Why it happens:** The Tauri binary crate is a separate crate in the workspace — `pub(crate)` means "visible within the same crate," not the workspace.
**How to avoid:** The main entry points (`run_documentary`, `run_sde`, `run_sdp`) are already `pub` — commands call these, not the individual steps. For script functions, `load_script` and `save_script` are already `pub`. Verify all 15 command target functions have `pub` visibility. [VERIFIED: `src/documentary/pipeline.rs` line 471 `pub async fn run_documentary`]
**Warning signs:** Compile error "function `xxx` is private" in `src-tauri`.

## Code Examples

### Pipeline Command Wrapper (Thin Pattern, D-01)

```rust
// src-tauri/src/commands/pipeline.rs
use narratoai_core::documentary::types::DocumentaryRequest;
use narratoai_core::documentary::pipeline;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::models::progress::ProgressPayload;
use crate::error::CommandError;

#[tauri::command]
pub async fn run_documentary(
    app: AppHandle,
    request: DocumentaryRequest,
    config: tauri::State<'_, narratoai_core::config::types::AppConfig>,
    // LLM registry accessed via global Arc — same pattern as existing code
) -> Result<crate::models::progress::CommandResponse, CommandError> {
    let task_id = Uuid::new_v4().to_string();
    let config_ref: &narratoai_core::config::types::AppConfig = &config;

    // Build Tauri-aware callback
    let app_handle = app.clone();
    let progress_pipeline_type = "documentary".to_string();
    let progress_task_id = task_id.clone();
    let progress_callback: narratoai_core::documentary::types::ProgressCallback =
        Box::new(move |_step: narratoai_core::documentary::types::ProgressStep,
                       pct: f32,
                       msg: &str| {
            let payload = ProgressPayload {
                pipeline_type: progress_pipeline_type.clone(),
                task_id: progress_task_id.clone(),
                step_name: format!("{:?}", _step),
                percent: pct,
                message: msg.to_string(),
                step_index: _step as u32,
                total_steps: 6,
                status: "running".to_string(),
                error_code: None,
                error_message: None,
            };
            let _ = app_handle.emit("pipeline-progress", payload);
        });

    // Execute pipeline — note: run_documentary doesn't accept callback in current signature
    // This requires modifying run_documentary to accept ProgressCallback
    let output_path = pipeline::run_documentary(request, config_ref, None).await?;

    Ok(crate::models::progress::CommandResponse {
        output_video_path: output_path,
        step_durations: vec![],
        intermediate_paths: std::collections::HashMap::new(),
    })
}
```

**IMPORTANT UPDATE NEEDED:** The current `run_documentary()` signature (line 471-475) does NOT accept a `ProgressCallback` parameter. D-14 explicitly states: "扩展现有回调类型...命令入口用闭包捕获 AppHandle，每次回调 emit 事件。现有流水线进度调用点改为新签名。" This means `run_documentary()`, `run_sde()`, `run_sdp()` must be modified to accept a `ProgressCallback` parameter. This is a required code change in Phase 10.

[Source: Tauri 2.0 docs - Calling Rust from Frontend; smartEdit pattern — `src-tauri/src/lib.rs` uses `invoke_handler(generate_handler![...])`]

### Unified CommandResponse (D-07, D-09)

```rust
// src-tauri/src/models/progress.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 命令统一返回包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub output_video_path: PathBuf,
    pub step_durations: Vec<StepDuration>,
    pub intermediate_paths: HashMap<String, PathBuf>,
}

/// 每步耗时
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDuration {
    pub step_name: String,
    pub duration_secs: f64,
}
```

### File Dialog Usage (TAURI-03)

```javascript
// Frontend JS (v2 milestone, but documents the API contract)
import { open } from '@tauri-apps/plugin-dialog';

// Select video file
const videoPath = await open({
  multiple: false,
  filters: [{
    name: '视频文件',
    extensions: ['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm']
  }]
});

// Select subtitle file
const subtitlePath = await open({
  multiple: false,
  filters: [{
    name: '字幕文件',
    extensions: ['srt', 'ass', 'ssa', 'vtt']
  }]
});

// Pass to Rust command
const result = await invoke('run_documentary', {
  request: {
    videoPath: videoPath,
    scriptPath: scriptPath,
    // ... other fields
  }
});
```

[Source: Tauri 2.0 docs - Dialog Plugin]

### Capabilities Configuration (TAURI-03)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for NarratoAI",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "core:event:default",
    "core:event:allow-emit",
    "core:event:allow-listen"
  ]
}
```

**Note:** If using `tauri-plugin-fs` for file operations, add `"fs:default"` or specific read permissions. Since Phase 10 commands receive paths as strings from the dialog and pass them to existing Rust functions that use `std::fs`, the actual file I/O happens in the library crate, not through Tauri's plugin. The `fs` plugin may not be needed unless the frontend needs direct file access.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri 1.x `allowlist` | Tauri 2.x `capabilities/` (.json files) | Tauri 2.0 GA (Sep 2024) | Permission config moved from `tauri.conf.json` into separate capability files; per-window/platform scoping |
| `app.emit()` events for progress | `Channel<T>` for streaming | Tauri 2.0 Channel API | Channel is type-safe, point-to-point, auto-cleanup. Events are for app-wide broadcasts only. Decision D-11 locks events; plan should note Channel as upgrade path |
| `tauri::command` returning `Result<T, String>` | Return `Result<T, E: std::error::Error>` | Tauri 2.0 | Custom error types must implement `std::error::Error` + `Serialize` |
| Single binary crate | `[lib]` + `[[bin]]` layout | Tauri 2.0 mobile support | lib crate needed for mobile; `crate-type = ["staticlib", "cdylib", "rlib"]` required |
| `tauri::Window` | `WebviewWindow` | Tauri 2.0 API rename | Window API methods changed; use `app.get_webview_window("label")` |

**Deprecated/outdated:**
- Tauri 1.x `allowlist` — replaced by capabilities system
- Tauri 1.x `tauri::Window` — replaced by `WebviewWindow`
- Tauri 1.x `tauri.conf.json` `[tauri]` root key — replaced by `[app]`
- `tauri::command` returning `Result<T, String>` — works but loses structured error data

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `DocumentaryRequest`, `SdeRequest`, `SdpRequest` need `#[derive(Deserialize)]` added | Code Examples | Low — serde already in dependencies, trivial change |
| A2 | API keys in `AppConfig` have `#[serde(skip_serializing)]` and will be empty when cloned via `get()` | Pattern 4: Config Injection | MEDIUM — if commands need API keys, they won't be in `State<AppConfig>`; must use `Arc<RwLock<AppConfig>>` state instead |
| A3 | Tauri 2.0 `Channel<T>` is not used due to locked decision D-11 | Standard Stack | MEDIUM — if events prove insufficient, re-planning needed |
| A4 | The `run_documentary()`/`run_sde()`/`run_sdp()` functions need signature changes to accept `ProgressCallback` | Pitfalls | HIGH — without callback injection, progress events cannot be emitted |
| A5 | Tauri `capabilities/default.json` needs `core:event:allow-emit` and `core:event:allow-listen` | Code Examples | LOW — Tauri 2.0 defaults may include these; verify at build time |
| A6 | `tauri-state` is not needed; use `app.manage()` directly with `AppConfig` | Pattern 4 | LOW — `app.manage()` stores state; commands access via `State<AppConfig>` |

## Open Questions

1. **API Key Access in Commands (from A2)**
   - What we know: `AppConfig` fields with `skip_serializing` (vision_openai_api_key, text_openai_api_key, pexels_api_keys, pixabay_api_keys, azure.speech_key, tencent.secret_id/key, soulvoice.api_key, tts_qwen.api_key, doubaotts.ak/sk/token) are empty in the `Clone`d `AppConfig` returned by `get()`.
   - What's unclear: Do the pipeline commands need these keys? Yes — LLM and TTS pipelines need API keys.
   - Recommendation: Either (a) remove `skip_serializing` from API key fields, or (b) store the `Arc<RwLock<AppConfig>>` in Tauri State instead of the cloned `AppConfig`, and commands read from the RwLock.

2. **Embedded vs External config.toml path**
   - What we know: Development uses `config.toml` in the project root. Production needs it bundled or side-by-side with the binary.
   - What's unclear: Tauri's `resource_dir()` may not point to the project root in dev mode. How does the setup callback find the config file?
   - Recommendation: Use multiple fallback paths: (1) resource_dir()/config.toml, (2) std::env::current_dir()/config.toml, (3) PathBuf::from("config.toml"). Document in the plan.

3. **Callback Signature Modification Impact**
   - What we know: D-14 requires modifying pipeline main functions to accept `ProgressCallback`. Currently `run_documentary()` does NOT accept it.
   - What's unclear: How many existing call sites need updating? The library tests call `run_documentary()` directly — they'll need the callback parameter too.
   - Recommendation: Add `Option<ProgressCallback>` parameter (backward compatible with `None` = no progress) rather than changing the signature to require it. All pipeline functions already support this pattern via `state.progress`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | None — standard Rust test runner |
| Quick run command | `cargo check -p src-tauri` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TAURI-01 | Command registration compiles without error | compile | `cargo check -p src-tauri` | ❌ Wave 0 |
| TAURI-01 | Each command deserializes args correctly | unit | `cargo test -p src-tauri --test test_command_deserialize` | ❌ Wave 0 |
| TAURI-02 | ProgressPayload serializes to valid JSON | unit | `cargo test -p src-tauri --test test_progress_payload_serialize` | ❌ Wave 0 |
| TAURI-03 | File dialog path flows through to command | integration | Manual (requires UI) | ❌ Manual |
| TAURI-03 | Capability permissions allow dialog + read | config | `cargo build` (checks tauri.conf.json validity) | ❌ Wave 0 |
| D-03 | CommandError From impls for all domain errors | unit | `cargo test -p src-tauri --test test_command_error_from` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check -p src-tauri`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/tests/test_command_deserialize.rs` — tests that Tauri command argument types deserialize correctly
- [ ] `src-tauri/tests/test_progress_payload.rs` — tests ProgressPayload JSON serialization
- [ ] `src-tauri/tests/test_command_error.rs` — tests CommandError From impls for all domain errors
- [ ] `src-tauri/Cargo.toml` — Tauri dependencies need to be added
- [ ] `src-tauri/build.rs` — `tauri_build::build()` invocation
- [ ] `src-tauri/tauri.conf.json` — minimal Tauri configuration
- [ ] `src-tauri/capabilities/default.json` — permission configuration

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | serde `#[derive(Deserialize)]` + existing `validate()` methods on request types |
| V8 Data Protection | partial | `#[serde(skip_serializing)]` on API keys in AppConfig; verify this pattern is preserved in Tauri State |
| V12 File Upload | no | Desktop app, not web; file paths passed via dialog |
| V14 Configuration | yes | config.toml loaded from disk, not from user input; path traversal mitigated by dialog plugin |

### Known Threat Patterns for Tauri 2.0

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Command argument injection | Tampering | serde deserialization naturally limits types; request types have validate() |
| Path traversal via file dialog | Elevation of Privilege | Tauri dialog plugin auto-scopes paths; attacker cannot modify dialog selection |
| IPC message flooding | Denial of Service | Single user, single pipeline — not a realistic threat in desktop context |
| API key leakage in event payload | Information Disclosure | `#[serde(skip_serializing)]` on sensitive fields; event payloads never include config |

**Note:** This phase has minimal new security surface beyond what the existing library code already handles. The main risk is accidentally serializing API keys in Tauri State — verify `skip_serializing` is present on all key fields.

## Sources

### Primary (HIGH confidence)
- [Tauri 2.0 Calling Rust from Frontend](https://tauri.app/develop/calling-rust/) — Command definition, State, events, Channels
- [Tauri 2.0 State Management](https://v2.tauri.app/develop/state-management/) — `app.manage()`, `State<T>`, `Arc` avoidance
- [Tauri 2.0 Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — `AppHandle::emit()`, events
- [Tauri 2.0 Configuration Reference](https://v2.tauri.app/ko/reference/config/) — `tauri.conf.json` schema
- [Tauri Dialog Plugin](https://v2.tauri.app/plugin/dialog/) — `tauri-plugin-dialog` API
- [Tauri Capabilities System](https://v2.tauri.org.cn/security/capabilities/) — Permission scoping
- [Tauri 2.0 Channels API](https://v2.tauri.app/develop/calling-frontend/#channels) — Channel vs Event comparison
- [crates.io: tauri](https://crates.io/crates/tauri) — v2.11.1 verified
- [crates.io: tauri-plugin-dialog](https://lib.rs/crates/tauri-plugin-dialog) — v2.7.1 verified
- [smartEdit project structure](E:\GitLib\smartEdit\src-tauri) — Reference architecture (verified on local machine)

### Secondary (MEDIUM confidence)
- [Tauri 2.0 Cargo.toml and Entry Files](https://jishuzhan.net/article/1906014592613036033) — Lib + binary layout pattern
- [Cargo workspace with Tauri discussion](https://github.com/orgs/tauri-apps/discussions/12630) — Workspace setup caveats
- [Tauri + Rust Async Patterns](https://dev.to/hiyoyok/rust-async-patterns-in-tauri-keeping-the-ui-responsive-while-rust-does-heavy-work-e9d) — Progress reporting patterns
- [Tauri Event Freeze Bug #13234](https://github.com/tauri-apps/tauri/issues/13234) — Rapid emit warning

### Tertiary (LOW confidence)
- Stack Overflow dialog + fs permission issue ([79707145](https://stackoverflow.com/questions/79707145)) — Unverified but consistent with docs
- Tauri 2.0 blog post about Channels vs events — Only dev.to sources found

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Tauri 2.0 versions verified on crates.io; smartEdit validates architecture
- Architecture: HIGH — Patterns derived from Tauri official docs + verified local reference (smartEdit)
- Pitfalls: MEDIUM — Most verified; event flooding bug from GitHub issue, capability issues from docs

**Research date:** 2026-05-07
**Valid until:** 2026-06-07 (Tauri 2.x is stable; versions above are advisory, pattern guidance is durable)
