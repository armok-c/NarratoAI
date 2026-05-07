# Phase 10: Tauri Command Layer - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

将所有 Rust 业务功能通过 Tauri 2.0 命令暴露给前端 JS 端，支持实时进度事件推送和文件对话框。覆盖需求：TAURI-01（命令注册）、TAURI-02（进度事件推送）、TAURI-03（文件对话框集成）。依赖 Phase 6/7/8（三个流水线全部完成）。

当前项目为纯 Rust 库 crate（`src/lib.rs`），Phase 10 需要将其包装为 Tauri 2.0 桌面应用后端。

</domain>

<decisions>
## Implementation Decisions

### 命令粒度与 API 设计
- **D-01:** Thin wrapper 模式——每个领域入口函数对应一条 Tauri 命令，约 15 条命令，1:1 映射现有 Rust API。不做 Facade 分组。
- **D-02:** 配置启动加载——应用启动时加载 config.toml 到 Tauri State（`tauri::State<AppConfig>`），命令通过 `tauri::State` 读取。不需要显式配置命令。
- **D-03:** 统一 CommandError 包装——定义 `CommandError`（`code: String` + `message: String`），实现 `serde::Serialize`。为所有领域错误（ScriptError、PipelineError、SdeError、SdpError 等）实现 `From<T>` 自动转换。`message` 保留中文错误信息（与 Phase 1 D-18 一致），`code` 用于前端程序化区分。
- **D-04:** 命令清单——流水线（`run_documentary`、`run_sde`、`run_sdp`、`generate_documentary_script`、`generate_sdp_script`）、脚本全操作（`load_script`、`save_script`、`validate_script`、`get_script_info`、`update_narration`、`set_ost`、`update_timestamp`）、导出（`export_jianying_draft`）。约 15 条命令。
- **D-05:** 集中注册函数——创建 `register_all_commands()` 统管命令注册，与现有 `register_all_providers()`/`register_all_prompts()` 模式对齐。在 Tauri builder 的 `setup` 阶段调用。
- **D-06:** Tauri State + 全局 Arc 混合——Config 通过 Tauri State 注入（`app.manage(config)`），LLM/Prompt/TTS 注册表保持现有全局 Arc/RwLock 模式，命令内部直接访问。最小改动现有代码。
- **D-07:** 返回 OutputPath + 统计——流水线命令 await 完成后返回 `CommandResponse`（`output_video_path: PathBuf` + `step_durations: Vec<StepDuration>` + `intermediate_paths: HashMap<String, PathBuf>`）。前端可预览视频和查看统计。与现有 PipelineState 最终状态对齐。
- **D-08:** 统一 UTF-8 PathBuf——Tauri 2.0 自动将 JS 路径转为 Rust `PathBuf`，不做额外标准化。跨平台兼容。
- **D-09:** 复用现有类型 + 新增 CommandResponse——流水线命令复用现有 `DocumentaryRequest`/`SdeRequest`/`SdpRequest`/`ScriptClip` 等类型。新增 `CommandResponse` 作为统一返回包装，`CommandError` 作为统一错误类型。

### 进度事件设计
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

</decisions>

<specifics>
## Specific Ideas

- 桌面应用单用户场景，同一时间只运行一个流水线——不需要并发任务管理。但 task_id 机制保留扩展性
- 现有代码中 ProgressCallback 模式已经在三个流水线中统一使用，改动可以一次性完成
- 参考 smartEdit 的技术选型（Tauri 2.0 + Rust），但不复用其代码
- 文件对话框（TAURI-03）在本次讨论中未深入展开——planner 和 researcher 应自主确定最佳实现方式

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Rust 代码库——待暴露的领域模块
- `src/lib.rs` — 库入口，当前导出所有 pub mod，需新增 Tauri 命令模块引用
- `src/documentary/pipeline.rs` — `run_documentary()` + PipelineState + 6 步 step 函数 + ProgressCallback
- `src/documentary/types.rs` — DocumentaryRequest 参数结构体
- `src/sde/pipeline.rs` — `run_sde()` + 9 步流水线 + guardrails
- `src/sde/types.rs` — SdeRequest 参数结构体
- `src/sdp/pipeline.rs` — `run_sdp()` + 4 步流水线
- `src/sdp/types.rs` — SdpRequest 参数结构体
- `src/sdp/script_gen.rs` — `generate_sdp_script()` 脚本生成预处理
- `src/script/mod.rs` — load_script / save_script / validate_script / 编辑 API
- `src/script/types.rs` — ScriptClip + OstType 数据模型
- `src/jianying/builder.rs` — 剪映草稿导出 builder
- `src/config/types.rs` — AppConfig 配置段定义
- `src/error.rs` — 领域错误枚举汇总

### 流水线上下文决策（前序阶段）
- `.planning/phases/01-foundation/01-CONTEXT.md` — D-15 ProgressCallback、D-16/D-17 thiserror 错误枚举、D-18 中文错误信息
- `.planning/phases/06-documentary-pipeline/06-CONTEXT.md` — D-01 Step-by-Step 函数链、D-02 PipelineState、D-03 DocumentaryRequest、D-05 不做跨模式抽象
- `.planning/phases/08-sdp-pipeline/08-CONTEXT.md` — 三个流水线完成状态和 pub(crate) 接口暴露方式

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库 + Tauri 命令、文件存储、桌面单用户）
- `.planning/REQUIREMENTS.md` — Phase 10 需求：TAURI-01, TAURI-02, TAURI-03
- `.planning/ROADMAP.md` — Phase 10 定义、依赖（Phase 6/7/8）、成功标准

### 外部参考
- Tauri 2.0 官方文档 — Command 注册、Event 系统、State 管理、File Dialog 插件
- smartEdit 项目（`E:\GitLib\smartEdit`）— Tauri 2.0 + Rust 技术栈验证，可参考项目结构但不复用代码

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/documentary/pipeline.rs` — `run_documentary()` + 6 个 pub(crate) step 函数，Tauri 命令直接调用（通过 new pub 包装或调整可见性）
- `src/sde/pipeline.rs` — `run_sde()` + 9 步流水线，同上
- `src/sdp/pipeline.rs` — `run_sdp()` + 4 步流水线，同上
- `src/script/mod.rs` — load_script / save_script / validate_script / update_narration / set_ost / update_timestamp，许多已有 pub 可见性
- `src/jianying/builder.rs` — export_draft 等导出函数
- 现有 ProgressCallback 类型 — 待扩展为 ProgressStep 版本
- 现有 domain error 枚举 — 为 CommandError 实现 From<T>

### Established Patterns
- **register_all_* 模式（Phase 2/3/4）**：`register_all_commands()` 延续此约定
- **领域错误枚举（Phase 1 D-16/D-17）**：thiserror 派生，中文 Display
- **ProgressCallback 回调（Phase 1 D-15）**：流水线每步完成后上报进度
- **Step-by-Step 函数链（Phase 6 D-01）**：每个 step 独立函数，主编排顺序调用
- **pub(crate) 跨模块复用（Phase 7/8）**：需调整为 pub 或新增 pub 包装函数
- **Arc 全局状态（Phase 2/3）**：LLM/Prompt/TTS 注册表用 Arc<RwLock<T>>

### Integration Points
- `src/lib.rs` — 新增 `pub mod tauri_commands;` 或直接在 src-tauri 中引用
- 流水线模块可见性 — 部分函数为 `pub(crate)`，需调为 `pub` 或新建 Tauri 命令包装层
- 现有 ProgressCallback — 改为 ProgressStep 结构体回调
- Cargo.toml — 新增 tauri 相关依赖，可能拆分为 workspace
- `config.toml` — Tauri 启动时加载到 State
- 前端项目位置 — Phase 10 仅建后端命令层，前端在后续里程碑实现

</code_context>

<deferred>
## Deferred Ideas

- 前端的 Vue 3 + Vuetify 3 实现 — v2 里程碑（REQUIREMENTS.md v2 Requirements）
- 流水线取消/暂停功能 — 当前讨论未涉及，可作为 Phase 10 执行时由 planner 评估复杂度后添加
- 多语言错误信息支持 — 当前只支持中文，国际化留待后续
- 前端 UI 面板（纪录片/SDE/SDP 工作面板、脚本编辑器、进度展示）— v2 前端里程碑

</deferred>

---

*Phase: 10-tauri-command-layer*
*Context gathered: 2026-05-07*
