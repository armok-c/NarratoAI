# Phase 10: Tauri Command Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 10-tauri-command-layer
**Areas discussed:** 命令粒度与 API 设计, 进度事件设计

---

## 命令粒度与 API 设计

| Option | Description | Selected |
|--------|-------------|----------|
| Thin wrapper | 每个领域入口函数对应一条 Tauri 命令，约 15-20 条，1:1 映射现有 Rust API | ✓ |
| Facade 分组 | 按功能域合并为少数入口命令（5-8 条），内部解析 action 参数分发 | |
| 混合粒度 | 核心流水线独立命令，辅助功能合并（8-12 条） | |

**User's choice:** Thin wrapper — 直接映射现有 API，前端直接调用，无中间层。

---

### 配置加载

| Option | Description | Selected |
|--------|-------------|----------|
| 应用启动加载一次 | Tauri 启动时加载配置到 State，所有命令通过 tauri::State 读取 | ✓ |
| 每次命令调用加载 | 每条命令入口重新 load_config()，无需全局状态 | |
| 启动加载 + 手动刷新 | 启动加载一次，额外提供 reload_config 命令供前端主动触发 | |

**User's choice:** 应用启动加载一次 — 与现有 config 模块全局模式一致。

---

### 错误映射

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 CommandError 包装 | code + message 结构体，From 自动转换，中文信息保留 | ✓ |
| 直接为每个 Error 实现 Serialize | 在各领域错误枚举上加 serde derive | |
| 字符串简化 | 所有错误转为 String 返回 | |

**User's choice:** 统一 CommandError 包装 — 保留中文错误信息，code 用于前端区分。

---

### 命令注册清单

| Option | Description | Selected |
|--------|-------------|----------|
| 核心入口 + 脚本全操作 | 流水线 + 脚本 CRUD + 导出，约 15 条命令 | ✓ |
| 核心入口 + 脚本基础 | 流水线 + load/save 脚本，约 8 条 | |
| 最小命令集 | 仅流水线 + load/save + 导出，约 7 条 | |

**User's choice:** 核心入口 + 脚本全操作 — ~15 条命令覆盖完整脚本编辑流程。

---

### 注册模式

| Option | Description | Selected |
|--------|-------------|----------|
| 集中注册函数 | register_all_commands() 统管，与现有惯例对齐 | ✓ |
| main.rs 逐个注册 | Tauri builder 链中逐个注册 | |

**User's choice:** 集中注册 — 延续 register_all_* 惯例。

---

### 状态注入

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri State + 全局 Arc 混合 | Config 走 Tauri State，注册表保持全局 Arc | ✓ |
| 全部 Tauri State | 所有共享状态都作为 Tauri State | |
| 全部保持全局 Arc | Config 也不走 Tauri State | |

**User's choice:** 混合模式 — Config 唯一从前端感知的状态，其他保持全局模式最小改动。

---

### 流水线返回值

| Option | Description | Selected |
|--------|-------------|----------|
| 返回 OutputPath + 统计 | 结构化结果：output_path + 步骤耗时 + 中间产物 | ✓ |
| 仅返回 output_path | 最小化 API 面 | |
| 流式逐步返回 | 每步通过事件推送，命令返回 () | |

**User's choice:** OutputPath + 统计 — 与 PipelineState 最终状态对齐。

---

### 路径约定

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 UTF-8 PathBuf | Tauri 自动转换，不做额外处理 | ✓ |
| 全部标准化为 forward slash | normalize_path() 转 / 分隔符 | |

**User's choice:** 自然 PathBuf — 跨平台兼容。

---

### 参数类型

| Option | Description | Selected |
|--------|-------------|----------|
| 复用现有类型 + 新增 Command 专用 | 流水线用现有类型，新增 CommandResponse/CommandError | ✓ |
| 全部新建 Command 类型 | 每个命令专属 Request/Response，与内部类型解耦 | |
| 直接用内部类型 | 零额外代码 | |

**User's choice:** 复用现有类型 + 新增 Command 专用类型。

---

## 进度事件设计

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 ProgressPayload 结构体 | 通用结构含 pipeline_type，三个流水线共用 | ✓ |
| 每个流水线独立事件类型 | DocumentaryProgress/SdeProgress/SdpProgress 各有专属字段 | |
| JSON 字符串自由格式 | 每个流水线自由定义，无类型保障 | |

**User's choice:** 统一 ProgressPayload — 与现有 ProgressCallback 模式最接近。

---

### 事件发射机制

| Option | Description | Selected |
|--------|-------------|----------|
| 回调注入 + AppHandle emit | 命令持有 AppHandle，注入回调，emit 事件 | ✓ |
| Channel 模式 | tauri::ipc::Channel 推送进度，需要改造回调接口 | |
| 全局 EventEmitter | 单例模式，流水线内部直接调用 emit | |

**User's choice:** 回调注入 — 最小改动现有 step 函数签名。

---

### 事件命名

| Option | Description | Selected |
|--------|-------------|----------|
| 单一事件 + pipeline_type 字段 | "pipeline-progress"，payload 内含 pipeline_type | ✓ |
| 每个流水线独立事件名 | 三个独立事件名，前端分别监听 | |
| 单一事件 + 单流水线假设 | "task-progress"，不需要 task_id 和 pipeline_type | |

**User's choice:** 单一事件 + pipeline_type 区分 — 前端一个监听器。

---

### 错误事件

| Option | Description | Selected |
|--------|-------------|----------|
| 进度事件含状态字段 | status: running/success/error，错误时含 error 信息 | ✓ |
| 独立错误事件 | pipeline-error 单独事件，前端两个监听器 | |
| 仅命令返回值 | 错误仅通过 Result::Err 返回，不推送事件 | |

**User's choice:** 进度事件含状态字段 — 一个事件覆盖全部流程。

---

### 回调适配

| Option | Description | Selected |
|--------|-------------|----------|
| 扩展现有回调类型 | TauriProgressCallback: Fn(ProgressStep)，改现有签名 | ✓ |
| 命令层包装适配 | 保持现有签名，适配器闭包转换 | |
| 放弃回调，用进度查询 | 共享状态 + 前端轮询，无事件推送 | |

**User's choice:** 扩展现有回调类型 — ProgressStep 结构体覆盖 step_name/percent/message/status。

---

### 任务标识

| Option | Description | Selected |
|--------|-------------|----------|
| 启动时生成 UUID | 每个命令调用入口生成 UUID v4 | ✓ |
| 时间戳 | 人类可读但可能有碰撞 | |
| 不需要 task_id | 单用户桌面应用不需要区分 | |

**User's choice:** UUID task_id — 保持可扩展性。

---

## Claude's Discretion

- Tauri 2.0 项目脚手架生成方式
- 命令模块文件结构（`src-tauri/src/commands/` 目录组织）
- ProgressPayload 和 CommandResponse 的具体字段定义
- register_all_commands() 的具体实现位置
- Cargo.toml 依赖版本选择
- 是否需要 Cargo workspace 布局
- Tauri 文件对话框的权限配置

## Deferred Ideas

- 前端 Vue 3 + Vuetify 3 实现 — v2 里程碑
- 流水线取消/暂停功能 — 执行时评估
- 多语言错误信息 — 后续阶段
- 前端 UI 面板 — v2 前端里程碑
