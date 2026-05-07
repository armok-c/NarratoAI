---
phase: 10-tauri-command-layer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - Cargo.toml
  - narratoai-core/Cargo.toml
  - src-tauri/Cargo.toml
  - src-tauri/src/main.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/error.rs
  - src-tauri/src/models/mod.rs
  - src-tauri/src/models/progress.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/pipeline.rs
  - src-tauri/src/commands/script.rs
  - src-tauri/src/commands/export.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/jianying/builder.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sdp/pipeline.rs
findings:
  critical: 0
  warning: 7
  info: 5
  total: 12
status: issues_found
---

# Phase 10: Tauri Command Layer - Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Reviewed 20 files implementing the Tauri 2.0 command layer wrapping the NarratoAI core rust library. The architecture follows the thin-wrapper pattern: command modules in `src-tauri/src/commands/` bridge Tauri invoke calls to `narratoai-core` pipeline functions, progress is emitted via Tauri events, and domain errors are uniformly converted to `CommandError`.

No BLOCKER issues were found. The code is structurally sound and follows the design decisions documented in the phase context. However, 7 WARNING-level issues were identified, primarily around proxy support being silently disabled, progress events not being wired for script generation commands, inconsistent synchronous I/O in the SDP pipeline, and a macro export scoping issue. Additionally, 5 INFO-level quality items were noted.

## Warnings

### WR-01: 代理支持在所有 Tauri 流水线命令中被硬编码为 None

**Files:**
- `src-tauri/src/commands/pipeline.rs:59`
- `src-tauri/src/commands/pipeline.rs:122`
- `src-tauri/src/commands/pipeline.rs:176`

**Issue:** 三个流水线 Tauri 命令（`run_documentary`, `run_sde`, `run_sdp`）在调用核心流水线函数时，均将 `proxy` 参数硬编码为 `None`：

```rust
// run_documentary (line 59)
let output_path = narratoai_core::documentary::pipeline::run_documentary(
    request,
    &config,
    None,        // proxy — 硬编码为 None
    Some(progress_callback),
).await?;

// run_sde (line 122) 同样将 proxy 写死为 None
// run_sdp (line 176) 同样
```

`config` 已通过 Tauri State 注入，可以从 `config.proxy` 提取 `ProxySection`。注释提到"Tauri 启动时读取 config 中的 proxy 配置"，但实际代码并未实现。

**Fix:** 从 `config` 中提取 proxy 配置并传递给流水线：

```rust
let proxy = if config.proxy.enabled {
    Some(&config.proxy)
} else {
    None
};

let output_path = narratoai_core::documentary::pipeline::run_documentary(
    request,
    &config,
    proxy,
    Some(progress_callback),
).await?;
```

### WR-02: 脚本生成命令的进度事件被静默丢弃

**File:** `src-tauri/src/commands/pipeline.rs:192-213`

**Issue:** `generate_documentary_script` 和 `generate_sdp_script` 命令没有将进度回调挂接到 Tauri 事件系统。核心层的 `script_gen.rs` 通过 `ScriptGenRequest.progress` 上报进度（帧分析 10%→75%、文案生成 80%→100%），但由于 `progress` 字段标记为 `#[serde(skip)]` 且命令层未注入回调，所有进度事件被静默丢弃。用户在前端只能看到"等待响应"的状态，无法感知后台正在执行的帧分析和 LLM 调用进度。

```rust
// ScriptGenRequest 定义
#[derive(Serialize, Deserialize)]
pub struct ScriptGenRequest {
    pub video_path: PathBuf,
    // ...
    #[serde(skip)]
    pub progress: Option<Box<dyn Fn(f32, &str) + Send + Sync>>,  // 反序列化后始终为 None
}

// 命令层没有创建闭包装入 progress
pub async fn generate_documentary_script(
    _app: AppHandle,          // AppHandle 已注入但未使用
    request: ScriptGenRequest, // progress 始终为 None
    config: tauri::State<'_, AppConfig>,
    registry: tauri::State<'_, ...>,
) -> Result<Script, CommandError> {
    // 未创建 progress_callback
    // 核心层的 emit_progress 调用全部被静默丢弃
}
```

**Fix:** 在命令入口创建闭包，通过 `AppHandle.emit` 转发进度，并设置到 `ScriptGenRequest.progress`。由于 `ScriptGenRequest.progress` 需要 `'static` 生命周期，可以使用 `Arc<Mutex<...>>`：

```rust
pub async fn generate_documentary_script(
    app: AppHandle,
    request: ScriptGenRequest,
    config: tauri::State<'_, AppConfig>,
    registry: tauri::State<'_, ...>,
) -> Result<Script, CommandError> {
    let app_handle = app.clone();
    let progress_cb: Box<dyn Fn(f32, &str) + Send + Sync> = Box::new(move |pct, msg| {
        let payload = ProgressPayload {
            pipeline_type: "documentary".to_string(),
            // 使用 task_id
            step_name: "script_gen".to_string(),
            percent: pct,
            message: msg.to_string(),
            // ...
        };
        let _ = app_handle.emit("pipeline-progress", payload);
    });
    let mut request = request;
    request.progress = Some(progress_cb);
    // ...
}
```

### WR-03: SDP 流水线在异步上下文中使用同步文件 I/O

**File:** `narratoai-core/src/sdp/pipeline.rs:34`
**File:** `narratoai-core/src/sdp/pipeline.rs:102`

**Issue:** SDP 流水线在 async 函数中使用同步 I/O，与纪录片和 SDE 流水线的做法不一致：

| 操作 | 纪录片 | SDE | SDP |
|------|--------|-----|-----|
| `create_dir_all` | `tokio::fs` | `tokio::fs` | `std::fs` (line 34) |
| `write(concat_list)` | `tokio::fs` | `tokio::fs` | `std::fs` (line 102) |

在 `async fn run_sdp()` 中调用 `std::fs::create_dir_all` 和 `std::fs::write` 会阻塞 tokio 运行时线程。虽然单用户桌面场景下影响有限，但与其他两个流水线的行为不一致，且违背了异步编程的最佳实践。

```rust
// sdp/pipeline.rs:29-34 — 应使用 tokio::fs
let task_dir = request
    .output_dir
    .clone()
    .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
std::fs::create_dir_all(&task_dir)?;  // XXX: 同步阻塞

// sdp/pipeline.rs:101-102
std::fs::write(&concat_list_path, &concat_content)?;  // XXX: 同步阻塞
```

**Fix:** 改用 `tokio::fs`：

```rust
tokio::fs::create_dir_all(&task_dir).await?;
// ...
tokio::fs::write(&concat_list_path, &concat_content).await?;
```

### WR-04: SDP Concat 缺少路径换行符校验

**File:** `narratoai-core/src/sdp/pipeline.rs:88-100`

**Issue:** 纪录片和 SDE 的 concat 步骤在将路径写入 FFmpeg concat demuxer 文件前，会校验路径中是否含有换行符（`\n` / `\r`），防止 FFmpeg 解析出错。SDP 的 concat 步骤省略了此校验。

```rust
// documentary/pipeline.rs:190-194 — 有校验
if path_str.contains('\n') || path_str.contains('\r') {
    return Err(PipelineError::Concat {
        details: format!("视频路径包含非法字符: {}", clip._id),
    });
}
let escaped = path_str.replace("'", "'\\''");

// sdp/pipeline.rs:89-98 — 缺少换行符校验
let path_str = video_path
    .to_string_lossy()
    .replace('\\', "/")
    .replace("'", "'\\''");
concat_content.push_str(&format!("file '{}'\n", path_str));
```

虽然路径中包含换行符在实际中极为罕见，但与纪录片/SDE 流水线的防护级别不一致。若路径意外包含换行符，concat demuxer 文件将被破坏，导致 FFmpeg 静默失败。

**Fix:** 添加与纪录片流水线相同的换行符校验：

```rust
let path_str = video_path
    .to_string_lossy()
    .replace('\\', "/");
if path_str.contains('\n') || path_str.contains('\r') {
    return Err(SdpError::Validation {
        details: format!("视频路径包含非法字符: {}", clip._id),
    });
}
let escaped = path_str.replace("'", "'\\''");
```

### WR-05: register_all_commands 宏使用 #[macro_export] 超出所需作用域

**File:** `src-tauri/src/commands/mod.rs:14`

**Issue:** `register_all_commands!` 宏被标记为 `#[macro_export]`，这意味着它不仅对当前 crate 可见，还会被导出到引用 `narratoai_app` crate 的所有外部 crate。该宏内部硬编码了 `crate::commands::pipeline::*` 等路径，这些路径只对 `narratoai_app` crate 自身有意义。外部 crate 无法正确使用此宏（因为路径解析会错误地指向外部 crate 的 `crate::commands`），但宏仍然被错误地暴露在了公共 API 中。

```rust
#[macro_export]  // 不必要的 pub 暴露
macro_rules! register_all_commands {
    () => {
        tauri::generate_handler![
            crate::commands::pipeline::run_documentary,  // 外部 crate 调用时路径错误
            // ...
        ]
    };
}
```

由于 `register_all_commands!()` 仅在 `src-tauri/src/lib.rs` 的 `run()` 函数中调用一次，不需要将宏暴露为公共 API。

**Fix:** 移除 `#[macro_export]`，该宏仅在 crate 内部可用：

```rust
// 不添加 #[macro_export]，仅在 crate 内部可见
macro_rules! register_all_commands {
    () => {
        tauri::generate_handler![
            crate::commands::pipeline::run_documentary,
            // ...
        ]
    };
}
```

### WR-06: CommandResponse 的 step_durations 和 intermediate_paths 始终为空

**Files:**
- `src-tauri/src/commands/pipeline.rs:63-67`
- `src-tauri/src/commands/pipeline.rs:129-133`
- `src-tauri/src/commands/pipeline.rs:179-183`

**Issue:** 三个流水线命令都返回 `CommandResponse`，但 `step_durations` 始终为 `vec![]`，`intermediate_paths` 始终为 `HashMap::new()`。核心流水线 API（`run_documentary`, `run_sde`, `run_sdp`）只返回 `PathBuf`，不返回中间产物路径或每步耗时统计。前端收到空的统计信息，无法展示处理详情。

根据 D-07 设计决策，返回的 `CommandResponse` 预期包含完整统计。当前实现与设计目标存在差距。

```rust
Ok(CommandResponse {
    output_video_path: output_path,
    step_durations: vec![],        // 始终为空
    intermediate_paths: HashMap::new(),  // 始终为空
})
```

**Fix（短期方案）:** 修改核心流水线函数，使其返回包含中间状态的结构体而非 `PathBuf`，或提取 `PipelineState` 中的信息传递给 `CommandResponse`。

**Fix（长期方案）:** 重构核心流水线函数签名，使它们返回一个包含所有必要信息的结果类型，例如：

```rust
pub struct PipelineResult {
    pub output_path: PathBuf,
    pub intermediate_paths: HashMap<String, PathBuf>,
    pub total_duration: f64,
}
```

### WR-07: error.rs 中 use 语句散布在文件各位置

**File:** `src-tauri/src/error.rs:26,36,46,56,66,76,86,96,107`

**Issue:** 所有的 `use` 导入语句散布在文件中部（各个 `impl From` 块之前），而非按照 Rust 惯例放在文件顶部。这虽然语法正确（Rust 允许 `use` 出现在模块层级任何位置），但降低了代码可读性，且容易导致读者遗漏某些导入。

```rust
impl fmt::Display for CommandError { ... }
impl std::error::Error for CommandError {}

// ---- From<T> implementations ----

use narratoai_core::error::ConfigError;  // 散布在文件中间
impl From<ConfigError> for CommandError { ... }

use narratoai_core::documentary::error::PipelineError;
impl From<PipelineError> for CommandError { ... }
// ...
```

**Fix:** 将所有 `use` 语句集中到文件顶部：

```rust
use std::fmt;
use serde::Serialize;
use narratoai_core::error::{ConfigError, LLMError, TTSError};
use narratoai_core::documentary::error::PipelineError;
use narratoai_core::sde::error::SdeError;
use narratoai_core::sdp::error::SdpError;
use narratoai_core::script::error::ScriptError;
use narratoai_core::jianying::error::JianYingError;

// Then all impl From blocks follow
```

## Info

### IN-01: `run_sdp` 文档注释步骤计数与实际代码不一致

**File:** `narratoai-core/src/sdp/pipeline.rs:10-11`

文档注释写有"4 步顺序编排"，但实际只有 3 个处理步骤（clip, concat, composite），且 `total_steps` 也是 3。

### IN-02: `find_config` 的 Path 2 和 Path 3 逻辑冗余

**File:** `src-tauri/src/lib.rs:92-104`

Path 2（`current_dir()/config.toml`）和 Path 3（`./config.toml`）在运行时表现一致——都解析到当前工作目录。Path 3 可以删除，或者合并为一个检查点。

### IN-03: `ScriptGenRequest` 缺少 `validate()` 方法

**File:** `narratoai-core/src/documentary/script_gen.rs:12-24`

`ScriptGenRequest` 没有 `validate()` 校验方法，而 `DocumentaryRequest`、`SdeRequest`、`SdpRequest` 都有。如果前端传递了空 `video_path`，错误要到 FFmpeg 调用时才暴露，而非在参数校验阶段立即给出友好提示。

### IN-04: SDP pipeline 中 `_config` 参数未使用

**File:** `narratoai-core/src/sdp/pipeline.rs:20`

`run_sdp` 接收 `_config: &AppConfig` 但完全未使用。虽然用 `_` 前缀抑制了编译器警告，但与其他流水线保持一致的签名只是权宜之计。如果 SDP 确实不需要配置，应将参数从签名中移除。

### IN-05: `register_all_commands!()` 宏注册了 13 条命令，文档预期约 15 条

**File:** `src-tauri/src/commands/mod.rs:17-31`

命令清单中注册了 13 条命令，但 D-04 文档预期"约 15 条命令"。缺少的命令可能包括文件对话框相关命令或其他辅助命令。这可能是合理的（前端将在 v2 里程碑实现），但文档与实现之间的差距应被追踪。

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
