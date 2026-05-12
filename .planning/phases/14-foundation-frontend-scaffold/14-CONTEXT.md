# Phase 14: Foundation — Frontend Scaffold - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

前端项目基础架构就绪 — 构建工具链（Vite 6 + Vue 3.5 + TypeScript）、路由（vue-router 5 Hash 模式）、状态管理（9 个 Pinia 3 Store）、API 层（tauriInvoke 封装）、类型同步（ts-rs 自动导出）。集成测试桩替换（TDET-03），如桩已清理则跳过。

**覆盖需求:** FNDT-01, FNDT-02, FNDT-03, FNDT-04, FNDT-05, FNDT-06, TDET-03

**依赖:** Phase 13（v1.0 gap closure）
</domain>

<decisions>
## Implementation Decisions

### 前端目录结构
- **D-01:** 按文件类型分组 — `src/components/`、`src/stores/`、`src/views/`、`src/types/`、`src/composables/`、`src/api/`、`src/plugins/`、`src/styles/`、`src/layouts/`、`src/router/`、`src/utils/`。不按业务域（documentary/sde/sdp）拆分目录。
- **D-02:** 路径别名 `@/` 映射到 `src/`，与 SmartEdit 和 Vite 生态约定一致。
- **D-03:** 入口文件参考 SmartEdit 模式：`main.ts`（createApp + use Pinia/Router/Vuetify）→ `App.vue` → `layouts/DefaultLayout.vue` → `router-view`。

### Pinia Store 粒度
- **D-04:** 严格 9 Store，按需求定义各司其职：
  - `app` — 主题切换（dark/light）、全局 UI 状态、localStorage 持久化
  - `mode` — 当前工作模式（documentary/sde/sdp），模式切换时重置 pipeline 状态
  - `video` — 视频列表数据、上传状态、视频元信息
  - `pipeline` — 流水线进度事件（ProgressPayload）、运行状态（idle/running/error）、步骤追踪
  - `script` — 脚本数据加载/保存/编辑、JSON 校验
  - `llm` — 双模型配置（Vision + Text），Provider/Model/API Key/Base URL 四字段
  - `tts` — 7 引擎选择 + 各引擎专属参数、Edge-TTS voice 列表
  - `bgm` — BGM 文件夹路径、随机/指定模式切换
- **D-05:** 每个 Store 使用 Composition API 风格（`defineStore('name', () => { ... })`），与 SmartEdit 一致。
- **D-06:** Phase 14 创建全部 9 个 Store 骨架文件（含类型定义和初始状态），后续 Phase 逐步实现完整逻辑。

### tauriInvoke API 封装
- **D-07:** 直接移植 SmartEdit `src/composables/useTauri.ts` 的 tauriInvoke 封装：
  - BigInt → Number 安全序列化（检查安全整数范围）
  - 嵌套对象/数组递归标准化
  - 敏感字段过滤（apiKey/password/token/secret/api_key/accessToken）
  - 调试日志（仅在 DEV 模式或 VITE_DEBUG=true 时输出）
  - 统一错误处理和抛出
- **D-08:** 新增 `src/api/` 目录，按领域拆分命令调用模块（参考 SmartEdit `src/api/commands.ts`、`llmCommands.ts` 等）。

### ts-rs 类型同步
- **D-09:** 在 Rust 结构体上添加 `#[derive(TS)]` + `#[ts(export)]`，通过 `ts-rs` crate 自动导出 `.ts` 类型文件到 `src-tauri/gen/types/`（或前端 `src/types/generated/`）。
- **D-10:** 导出范围：所有 Tauri 命令的请求/响应类型（DocumentaryRequest、SdeRequest、SdpRequest、CommandResponse、ProgressPayload、ScriptClip 等）以及配置类型（AppConfig 各段）。
- **D-11:** Rust 侧需在 `Cargo.toml` 添加 `ts-rs` 依赖，并在 build 流程中加入 `cargo test` 触发生成（或独立脚本）。

### 集成测试桩替换（TDET-03）
- **D-12:** 当前代码库中 `assert!(true)` 桩已不存在（已预先清理）。Planner 需验证确认：搜索 `assert!(true)` 覆盖 `narratoai-core/tests/` 和项目根 `tests/`，若无匹配则此子任务标记为已完成。
- **D-13:** 若发现残留桩：替换为真实断言，无法脱离外部依赖的测试标记 `#[ignore = "需要 FFmpeg/网络"]`。

### 技术栈版本
- **D-14:** Vite 6（最新稳定版），不等待 Vite 8。TypeScript ~5.7、Vue 3.5、Vuetify 3.5+、Pinia 2.1+（当前 Pinia 3 尚未发布，以实际最新稳定版为准）、vue-router 4.x（对应规范中 "vue-router 5" 概念）。
- **D-15:** 依赖版本优先参考 SmartEdit `package.json`（已生产验证），升级到各包当前最新稳定版。

### Claude's Discretion
- Vite 配置具体参数（端口、manualChunks 策略、Tauri host 配置）
- tsconfig 严格度级别和路径别名配置
- ESLint/Prettier 规则集选择
- Vuetify 主题配置（颜色、字体、zhHans 本地化设置细节）
- 各 Store 的具体接口方法和返回类型定义
- `src/api/` 下命令模块的拆分粒度（按领域：pipelineApi、scriptApi、configApi 等）
- Rust 侧 ts-rs 导出的具体文件组织和 `#[ts(export_to = "...")]` 路径
- Vite dev server 的 Tauri 集成方式（`@tauri-apps/cli` 的 `beforeDevCommand` / `beforeBuildCommand`）
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SmartEdit 参考项目（布局和模式来源）
- `E:/GitLib/SmartEdit/src/main.ts` — 应用入口：createApp + Pinia + Router + Vuetify 初始化模式
- `E:/GitLib/SmartEdit/src/App.vue` — 根组件结构
- `E:/GitLib/SmartEdit/src/layouts/DefaultLayout.vue` — 三区域布局：AppHeader + v-main(router-view) + SettingsDrawer
- `E:/GitLib/SmartEdit/src/router/index.ts` — Hash 路由配置（createWebHashHistory），单页架构
- `E:/GitLib/SmartEdit/src/composables/useTauri.ts` — tauriInvoke 封装（BigInt 序列化、敏感字段过滤、调试日志）— **直接移植**
- `E:/GitLib/SmartEdit/src/stores/app.ts` — Pinia Composition API Store 模式参考
- `E:/GitLib/SmartEdit/src/plugins/` — Vuetify 插件注册
- `E:/GitLib/SmartEdit/vite.config.ts` — Vite 配置：vue 插件、路径别名、manualChunks 分包策略
- `E:/GitLib/SmartEdit/tsconfig.json` — TypeScript 严格模式配置
- `E:/GitLib/SmartEdit/package.json` — 依赖版本参考（已验证的生产组合）

### Rust 后端 — Tauri 命令层（前端对接目标）
- `src-tauri/src/commands/` — 15 个 Tauri IPC 命令（run_documentary、run_sde、run_sdp、load_script、save_script 等）
- `src-tauri/src/models/progress.rs` — ProgressPayload 结构体（pipeline_type、task_id、step_name、percent、message、status）
- `src-tauri/src/error.rs` — CommandError 定义（code + message，支持 serde）
- `.planning/phases/10-tauri-command-layer/10-CONTEXT.md` — Phase 10 决策（命令粒度、进度事件、Tauri State 架构）

### Rust 后端 — 核心类型（ts-rs 导出目标）
- `narratoai-core/src/documentary/types.rs` — DocumentaryRequest、PipelineState
- `narratoai-core/src/sde/types.rs` — SdeRequest
- `narratoai-core/src/sdp/types.rs` — SdpRequest
- `narratoai-core/src/script/types.rs` — ScriptClip、OstType
- `narratoai-core/src/config/types.rs` — AppConfig 全配置段（LLM、TTS、Audio、BGM、Export 等）
- `narratoai-core/src/llm/` — LLM 配置类型（Provider/Model/API Key 等）
- `narratoai-core/src/tts/mod.rs` — TTS 引擎枚举和配置

### Tauri 配置
- `src-tauri/tauri.conf.json` — 现有 Tauri 2.0 配置（需新增前端 dev/build 命令）
- `src-tauri/Cargo.toml` — Rust 端依赖管理（需新增 ts-rs）

### 项目规划文档
- `.planning/ROADMAP.md` — Phase 14 定义、成功标准、依赖关系
- `.planning/REQUIREMENTS.md` — FNDT-01~06、TDET-03 详细要求
- `.planning/PROJECT.md` — 项目约束（桌面单用户、文件存储、简体中文 UI）
- `.planning/STATE.md` — 项目状态（Phase 14 ready to plan）
- `.planning/phases/13-gap-closure/13-CONTEXT.md` — 前一 Phase 决策（Phase 14 紧接其后）
- `.planning/todos/pending/2026-05-12-replace-integration-test-stubs-assert-true.md` — 集成测试桩替换 Todo（TDET-03 来源）

### 集成测试（TDET-03 验证目标）
- `narratoai-core/tests/` — 14 个集成测试文件，3,219 行（当前无 assert!(true) 桩，需 planner 验证）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets（SmartEdit 可移植资源）
- `useTauri.ts` — 完整的 tauriInvoke 封装，BigInt 序列化 + 敏感字段过滤 + 调试日志，**直接移植到 `src/composables/useTauri.ts`**
- `src/stores/app.ts` — Pinia Store 模板（Composition API + localStorage 持久化 + 主题切换）
- `src/plugins/vuetify.ts` — Vuetify 3 插件注册，**直接复用**（调整主题色和本地化）
- `src/router/index.ts` — Hash 路由骨架，**直接复用**（调整路由表）
- `src/main.ts` — 应用入口骨架，**直接复用**（调整 Store 注册）
- `vite.config.ts` — Vite 构建配置（manualChunks 策略已验证），**直接复用**（升级到 Vite 6）
- `tsconfig.json` — TypeScript 严格模式配置，**直接复用**

### Established Patterns
- **Pinia Composition API 模式**（SmartEdit）：`defineStore('name', () => { const state = ref(...); function action() {...}; return { state, action } })` — 所有 9 个 Store 统一使用此模式
- **Tauri Invoke 封装**（SmartEdit）：所有 IPC 调用统一走 `tauriInvoke<T>(command, args)` 而非直接 `invoke()`
- **Vuetify 组件模式**（SmartEdit）：`v-layout` → `v-main` → `v-app-bar` / `v-navigation-drawer` 布局模式
- **Hash 路由**（SmartEdit）：`createWebHashHistory` 兼容 Tauri file:// 协议
- **ts-rs 导出**：在 Rust 结构体上 `#[derive(TS)]` + `#[ts(export)]`，构建脚本触发生成

### Integration Points
- **前端 ↔ 后端 Tauri 命令**：前端 `src/api/` 调用 `tauriInvoke('command_name', args)` → Tauri IPC → Rust `src-tauri/src/commands/`
- **类型同步**：Rust 结构体 `#[derive(TS)]` → `ts-rs` 生成 `.ts` 文件 → 前端 `src/types/generated/` 导入
- **Vite ↔ Tauri**：`vite.config.ts` 的 `server.host: '127.0.0.1'` + `server.watch.ignored: ['**/src-tauri/**']` — Tauri 2.0 标准集成
- **前端项目位置**：与 `src-tauri/` 并列项目根目录，不放在 `src-tauri/` 内部（Tauri 2.0 约定）

### 前端目录结构（按 D-01 类型分组）
```
NarratoAI/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── main.ts                  # 应用入口
│   ├── App.vue                  # 根组件
│   ├── api/                     # Tauri IPC 命令调用模块
│   │   ├── pipelineApi.ts       # 流水线命令（run_documentary/run_sde/run_sdp）
│   │   ├── scriptApi.ts         # 脚本命令（load/save/validate/update）
│   │   ├── configApi.ts         # 配置命令（如果有）
│   │   └── exportApi.ts         # 导出命令（jianying）
│   ├── components/              # 可复用 UI 组件
│   │   ├── AppHeader.vue        # (Phase 15 实现)
│   │   ├── SettingsDrawer.vue   # (Phase 15 实现)
│   │   ├── VideoTable.vue       # (Phase 17 实现)
│   │   ├── LogPanel.vue         # (Phase 17 实现)
│   │   └── SystemMonitorBar.vue # (Phase 15 实现)
│   ├── composables/             # 组合式函数
│   │   ├── useTauri.ts          # tauriInvoke 封装（移植自 SmartEdit）
│   │   └── useTheme.ts          # 主题切换逻辑
│   ├── layouts/
│   │   └── DefaultLayout.vue    # 主布局（AppHeader + v-main + SettingsDrawer）
│   ├── plugins/
│   │   └── vuetify.ts           # Vuetify 3 插件注册
│   ├── router/
│   │   └── index.ts             # Hash 路由配置
│   ├── stores/                  # 9 个 Pinia Store
│   │   ├── app.ts
│   │   ├── mode.ts
│   │   ├── video.ts
│   │   ├── pipeline.ts
│   │   ├── script.ts
│   │   ├── llm.ts
│   │   ├── tts.ts
│   │   ├── bgm.ts
│   │   └── index.ts
│   ├── types/
│   │   ├── generated/           # ts-rs 自动生成
│   │   └── index.ts             # 手写补充类型
│   ├── utils/
│   │   └── logger.ts            # 日志工具（loglevel）
│   ├── styles/
│   │   └── main.css             # 全局样式
│   └── views/
│       ├── HomeView.vue         # 主页视图
│       └── NotFoundView.vue     # 404 回退
└── src-tauri/                   # Tauri 后端（已存在）
```

</code_context>

<specifics>
## Specific Ideas

- 高度模仿 SmartEdit 布局和操作逻辑是 v2.0 里程碑的核心目标，前端脚手架应最大化复用 SmartEdit 已验证的配置和模式（vite.config、tsconfig、tauriInvoke、Pinia store 写法）
- SmartEdit 图标使用 `@mdi/font`，NarratoAI 直接沿用即可 — 无需额外图标库
- `tauriInvoke` 的 BigInt 序列化逻辑源自 SmartEdit 的生产实践（Tauri 2.0 IPC 不支持 BigInt），直接移植避免重新踩坑
- Vite dev server 固定 `127.0.0.1:5173` 端口与 Tauri 2.0 的 `beforeDevCommand` 配合 — SmartEdit 已验证此模式
- Vuetify 3 的 manualChunks 分包策略显著改善首屏加载 — SmartEdit `vite.config.ts` 已调优（vuetify/vue-vendor/tauri 三个包）
- `config.toml` 靠 Rust 后端 `ConfigManager` 加载，前端通过 Tauri State 读取，不需要在前端复制配置逻辑
- 9 个 Store 在 Phase 14 只需骨架（类型 + 初始状态 + 基础 actions），后续 Phase 按需充实；pinia 的 `defineStore` 允许分阶段实现
- TDET-03（集成测试桩）当前代码中未找到 `assert!(true)` — planner 应在执行前验证，若确实不存在则标记为「已完成，无需操作」
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

### Folded Todos
- **Replace integration test stubs using assert!(true)** — 折叠进 Phase 14 TDET-03。问题：v1.0 审计发现集成测试中存在 `assert!(true)` 桩（无意义断言）。当前代码库中桩已不存在（已预先清理），planner 需验证并标记完成或修复残留。

### Reviewed Todos (not folded)
- **Fix Tauri State read locks held across long pipelines** — 匹配但未折叠。属于 Phase 18 TDET-02，不在本 Phase 范围。
</deferred>

---

*Phase: 14-foundation-frontend-scaffold*
*Context gathered: 2026-05-12*
