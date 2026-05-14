# Roadmap: NarratoAI

## Milestones

- ✅ **v1.0 Rust Rewrite** — Phases 1-13 (shipped 2026-05-12) — [Archive](milestones/v1.0-ROADMAP.md)
- 📋 **v2.0 Tauri + Vue 3 Frontend** — Phases 14-18 (planned)

## Phases

<details>
<summary>✅ v1.0 Rust Rewrite (Phases 1-13) — SHIPPED 2026-05-12</summary>

- [x] Phase 1: Foundation (3/3 plans)
- [x] Phase 2: LLM Service Layer (3/3 plans)
- [x] Phase 3: TTS Core + Edge-TTS (3/3 plans)
- [x] Phase 4: Prompt System + Visual Analyzer (4/4 plans)
- [x] Phase 5: Script Management (2/2 plans)
- [x] Phase 6: Documentary Pipeline (4/4 plans)
- [x] Phase 7: SDE Pipeline (4/4 plans)
- [x] Phase 8: SDP Pipeline (5/5 plans)
- [x] Phase 9: JianYing Export (4/4 plans)
- [x] Phase 10: Tauri Command Layer (2/2 plans)
- [x] Phase 11: Extended Features (5/5 plans)
- [x] Phase 12: Additional TTS Engines (3/3 plans)
- [x] Phase 13: Gap Closure (5/5 plans)

</details>

### 📋 v2.0 Tauri + Vue 3 Frontend (Planned)

**Milestone Goal:** 高度模仿 SmartEdit 前端布局和操作逻辑，完整适配 NarratoAI Rust 后端，清理 v1.0 全部技术债务，FFmpeg 打包内置。

- [x] **Phase 14: Foundation — Frontend Scaffold** - Vite 8 + Vue 3.5 + Vuetify 4 脚手架，路由/状态管理/API层，集成测试桩替换（2026-05-13）
- [ ] **Phase 15: Layout Shell + 3-Mode UX** - AppHeader/DefaultLayout/HomeView/SettingsDrawer，三模式切换与动态面板
- [x] **Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt** - LLM/TTS/BGM/Export 配置面板，FFmpeg 打包，Edge-TTS 代理隧道 (completed 2026-05-14)
- [ ] **Phase 17: Pipeline Controls + Progress + Script Preview** - VideoTable/操作按钮/进度事件流/脚本预览/日志面板
- [ ] **Phase 18: Remaining Tech Debt + Polish** - Tauri State读锁优化，YouTube/Pexels命令暴露，ConfigManager热加载

## Phase Details

### Phase 14: Foundation — Frontend Scaffold
**Goal**: 前端项目脚手架和基础架构就绪 — 构建工具链、路由、状态管理、API层、类型同步全部就位，集成测试桩替换为真实测试
**Depends on**: Phase 13 (v1.0 gap closure)
**Requirements**: FNDT-01, FNDT-02, FNDT-03, FNDT-04, FNDT-05, FNDT-06, TDET-03
**Success Criteria** (what must be TRUE):
  1. Vite 8 dev server 启动无错误，Vuetify 3 组件正常渲染，HMR 工作
  2. Vue Router 5 Hash 路由正确导航到 HomeView，未知路由回退到 404 视图
  3. 9 个 Pinia 3 stores 全部正确实例化（app/mode/video/pipeline/script/llm/tts/bgm），返回初始状态
  4. `tauriInvoke()` 封装层正确处理 BigInt 序列化、敏感字段过滤和调试日志 — 通过单元测试验证
  5. 集成测试中所有 `assert!(true)` 桩替换为真实断言
  6. TypeScript 编译通过无类型错误（`npm run typecheck` passes），ts-rs 生成类型与 Rust 后端一致
**Plans**: 5 plans
**UI hint**: yes
Plans:
- [x] 14-01-PLAN.md — 项目脚手架 + Vuetify 插件 + 路由 + 布局/视图 + Tauri 集成（FNDT-01, FNDT-02, FNDT-03）
- [x] 14-02-PLAN.md — tauriInvoke 封装 + 9 个 Pinia Store 骨架 + API 命令模块（FNDT-04, FNDT-05）
- [x] 14-03-PLAN.md — ts-rs 类型导出 + TDET-03 集成测试桩验证（FNDT-06, TDET-03）
- [x] 14-04-PLAN.md — Gap closure: useTauri 单元测试 + ts-rs 类型文件生成（FNDT-05, FNDT-06）
- [x] 14-05-PLAN.md — Gap closure: 集成测试桩替换（TDET-03）

### Phase 15: Layout Shell + 3-Mode UX
**Goal**: 完整布局外壳和模式切换体验 — AppHeader 自定义标题栏、DefaultLayout 三区域布局、HomeView 上下分区、SettingsDrawer 抽屉、深色/浅色主题、系统监控、三模式动态面板切换
**Depends on**: Phase 14
**Requirements**: LOUT-01, LOUT-02, LOUT-03, LOUT-04, LOUT-05, LOUT-06, MODE-01, MODE-02, MODE-03
**Success Criteria** (what must be TRUE):
  1. AppHeader 显示 Logo + "NarratoAI" 标题 + 设置齿轮按钮 + 主题切换图标 + 窗口控制按钮（最小化/最大化/关闭）
  2. DefaultLayout 三区域布局正确渲染 — AppHeader 固定在顶部，v-main 内容区居中，SettingsDrawer 从右侧弹出临时抽屉（420px 宽度）
  3. HomeView 上下分区布局 — 上部：VideoTable | 设置卡片双列，下部：操作按钮行 + LogPanel 双列
  4. 用户通过 ModeSelector 下拉可在 Documentary/SDE/SDP 之间切换 — 面板根据模式动态显示/隐藏：Vision LLM 面板仅 Documentary 可见，TTS 面板在 SDP 模式隐藏
  5. 用户可一键切换深色/浅色主题 — 偏好通过 localStorage 持久化，刷新页面后保持
  6. SystemMonitorBar 在底部实时显示 CPU 和 RAM 使用率百分比
**Plans**: 5 plans
**UI hint**: yes
Plans:
- [ ] 15-01-PLAN.md — Rust sysinfo backend + Tauri config (permissions, decorations) + Wave 0 test stubs（LOUT-06）
- [ ] 15-02-PLAN.md — AppHeader + WindowControls + logo + ModeStore extension（LOUT-01, MODE-01）
- [ ] 15-03-PLAN.md — SettingSection + SettingsDrawer（LOUT-04）
- [ ] 15-04-PLAN.md — SystemMonitorBar + DefaultLayout rewrite（LOUT-02, LOUT-05, LOUT-06）
- [ ] 15-05-PLAN.md — HomeView 60/40 split + ModeSelector（LOUT-03, MODE-02, MODE-03）

### Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt
**Goal**: 所有设置面板可用，支持双模型 LLM 配置、7 引擎 TTS 选择、BGM/Export 设置、模式专用参数。FFmpeg 内置打包，Edge-TTS 代理隧道和 VisualAnalyzer 门面补齐
**Depends on**: Phase 15
**Requirements**: CONF-01, CONF-02, CONF-03, CONF-04, CONF-05, CONF-06, MODE-03, TDET-01, TDET-06, TDET-07
**Success Criteria** (what must be TRUE):
  1. 用户在 LLM/Mode 面板中可配置双模型（Vision + Text）— Provider/Model/API Key/Base URL 四个字段完整，输入值通过 Pinia store 持久化
  2. 用户在 TTS 面板中可选择 7 个引擎之一，配置各引擎专属参数；Edge-TTS 可加载可用音色列表供选择
  3. 用户在 BGM 面板中可选择 BGM 文件夹路径，切换随机/指定模式
  4. 用户在 Export 面板中可选择输出目录和导出格式选项
  5. 模式专用面板根据当前选中模式显示不同参数 — Documentary：帧间隔+视觉批处理，SDE：剧名+温度，SDP：自定义片段数
  6. FFmpeg 通过 Tauri sidecar 调用成功（externalBin 配置正确，shell:allow-execute 权限已添加），流水线可正常输出视频
  7. Edge-TTS 在代理网络环境下可正常生成音频（HTTP CONNECT 隧道建立，平台代理自动检测工作）
  8. Local Draft 模式保留 SettingsDrawer 中未保存的编辑，仅显式保存或应用启动时持久化到后端
**Plans**: 5 plans
**UI hint**: yes
Plans:
- [x] 16-01-PLAN.md — FFmpeg sidecar + 5 Tauri config 命令 + shell 插件集成（Wave 0）
- [x] 16-02-PLAN.md — Edge-TTS 代理隧道补齐 + VisualAnalyzer 门面审查（Wave 0）
- [x] 16-03-PLAN.md — Pinia stores 扩展 + useConfig composable + ts-rs 类型同步（Wave 1）
- [x] 16-04-PLAN.md — SettingsDrawer 手风琴 + 7 个配置面板组件 + 面板显隐规则（Wave 2）
- [x] 16-05-PLAN.md — Local Draft 暂存 + 全局保存/验证/snackbar + 模式切换保护 + vitest 测试（Wave 3）

### Phase 17: Pipeline Controls + Progress + Script Preview
**Goal**: 完整流水线操作体验 — 视频列表数据表、操作按钮行（Upload/Start/Stop/Clear）、模式感知 Start 分发、实时进度事件流、脚本预览弹窗、滚动日志查看器
**Depends on**: Phase 16
**Requirements**: PIPE-01, PIPE-02, PIPE-03, PIPE-04, PIPE-05, PIPE-06
**Success Criteria** (what must be TRUE):
  1. VideoTable 以数据表形式显示已上传视频 — 列包含名称/主题/字幕/状态/操作，状态徽章随流水线推进实时更新
  2. 操作按钮行包含 Upload（上传视频）/ Start（启动流水线）/ Stop（停止运行）/ Clear（清空列表）/ 输出目录 — 各按钮根据流水线状态正确启用/禁用
  3. Start 按钮根据当前 ModeStore 模式分发到对应 Tauri 命令 — Documentary 调 run_documentary，SDE 调 run_sde，SDP 调 run_sdp
  4. 进度条实时显示当前步骤名称和步骤号/总步数（Documentary=6，SDE=9，SDP=3），与后端 ProgressPayload 事件同步
  5. 用户通过 ScriptPreviewDialog 预览生成的解说脚本 — 弹窗内容根据当前模式渲染不同格式
  6. LogPanel 实时显示流水线日志，支持自动滚动到底部、导出日志文件、一键清空
**Plans**: TBD
**UI hint**: yes

### Phase 18: Remaining Tech Debt + Polish
**Goal**: 剩余技术债务清理完成 — Tauri State 跨 await 读锁消除、YouTube 下载和 Pexels 素材搜索命令暴露及前端集成、ConfigManager 热加载通知前端
**Depends on**: Phase 17
**Requirements**: TDET-02, TDET-04, TDET-05
**Success Criteria** (what must be TRUE):
  1. 后端流水线处理中不再跨 `.await` 点持有 Tauri State 读锁 — 通过代码审查和压力测试验证，写操作不再被阻塞
  2. 用户可在应用内输入 YouTube URL 下载视频，或在 Pexels 面板搜索素材并导入 VideoTable
  3. ConfigManager 检测到 `config.toml` 文件变更后自动通知前端热加载新设置 — 无需重启应用，设置面板即时反映
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 14 -> 15 -> 16 -> 17 -> 18

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 3/3 | Complete | 2026-04-28 |
| 2. LLM Service Layer | v1.0 | 3/3 | Complete | 2026-04-29 |
| 3. TTS Core + Edge-TTS | v1.0 | 3/3 | Complete | 2026-04-28 |
| 4. Prompt System + Visual Analyzer | v1.0 | 4/4 | Complete | - |
| 5. Script Management | v1.0 | 2/2 | Complete | 2026-04-29 |
| 6. Documentary Pipeline | v1.0 | 4/4 | Complete | 2026-05-03 |
| 7. SDE Pipeline | v1.0 | 4/4 | Complete | 2026-05-05 |
| 8. SDP Pipeline | v1.0 | 5/5 | Complete | 2026-05-06 |
| 9. JianYing Export | v1.0 | 4/4 | Complete | 2026-04-29 |
| 10. Tauri Command Layer | v1.0 | 2/2 | Complete | 2026-05-07 |
| 11. Extended Features | v1.0 | 5/5 | Complete | 2026-04-30 |
| 12. Additional TTS Engines | v1.0 | 3/3 | Complete | 2026-04-30 |
| 13. Gap Closure | v1.0 | 5/5 | Complete | 2026-05-12 |
| 14. Foundation — Frontend Scaffold | v2.0 | 5/5 | Gap closure complete | - |
| 15. Layout Shell + 3-Mode UX | v2.0 | 0/5 | Planned | - |
| 16. Configuration Panels + FFmpeg Sidecar + Core Tech Debt | v2.0 | 5/5 | Complete   | 2026-05-14 |
| 17. Pipeline Controls + Progress + Script Preview | v2.0 | 0/0 | Not started | - |
| 18. Remaining Tech Debt + Polish | v2.0 | 0/0 | Not started | - |
