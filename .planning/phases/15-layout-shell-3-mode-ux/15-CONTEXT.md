# Phase 15: Layout Shell + 3-Mode UX - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

完整布局外壳和模式切换体验 — AppHeader 自定义标题栏（Tauri 原生窗口控制）、DefaultLayout 三区域布局（AppHeader + v-main + SettingsDrawer）、HomeView 上下双列分区、SettingsDrawer 420px 右侧抽屉、深色/浅色主题切换、SystemMonitorBar 实时系统监控、三模式动态面板切换。

**覆盖需求:** LOUT-01, LOUT-02, LOUT-03, LOUT-04, LOUT-05, LOUT-06, MODE-01, MODE-02, MODE-03

**依赖:** Phase 14（前端脚手架 — stores/composables/router/plugins 已就绪）
</domain>

<decisions>
## Implementation Decisions

### AppHeader 自定义标题栏
- **D-01:** `tauri.conf.json` 设置 `decorations: false` 隐藏原生标题栏，使用 40px 高的自定义 AppHeader 实现窗口拖拽和按钮控制。
- **D-02:** 关闭按钮 = `getCurrentWindow().close()` 直接退出进程，不隐藏到系统托盘。
- **D-03:** Logo 复用 SmartEdit `logo.svg`（`E:/GitLib/SmartEdit/src/assets/logo.svg`），复制到 NarratoAI `src/assets/`。
- **D-04:** 窗口控制按钮排列：最小化 | 最大化 | 关闭，放在 AppHeader 最右侧。创建 WindowControls.vue 组件（参考 SmartEdit 模式，不复制代码）。
- **D-05:** AppHeader 元素排列（左→右）：Logo | "NarratoAI" 标题 | v-spacer | 设置齿轮按钮 | 主题切换按钮 | WindowControls。
- **D-06:** 窗口拖拽：整个 AppHeader 区域 mousedown 触发 `getCurrentWindow().startDragging()`，排除 button 和 window-controls 子元素。
- **D-07:** AppHeader 双击 = `getCurrentWindow().toggleMaximize()`。
- **D-08:** AppHeader 下方 `border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.12)`。
- **D-09:** 设置齿轮按钮点击 = 切换 SettingsDrawer 显示/隐藏。
- **D-10:** 主题切换按钮点击 = 调用 `useTheme().toggleTheme()`（Phase 14 composable 已实现）。

### DefaultLayout 三区域布局
- **D-11:** DefaultLayout 结构：AppHeader（固定顶部）→ v-main（router-view）→ SystemMonitorBar（v-footer 固定底部）。SettingsDrawer 为 v-navigation-drawer temporary 右侧覆盖，独立于三区域流。
- **D-12:** DefaultLayout 管理 SettingsDrawer 开关状态（`showSettings: ref<boolean>`），通过 props 传递给 AppHeader 和 SettingsDrawer。
- **D-13:** DefaultLayout 管理窗口最大化状态（`isMaximized: ref<boolean>`），传给 AppHeader → WindowControls 更新图标。
- **D-14:** DefaultLayout 处理窗口控制事件（minimize/maximize/close），使用 `@tauri-apps/api/window` 动态导入。

### HomeView 上下分区
- **D-15:** 上下分区固定比例 60/40（不可拖拽）。上区占 60%，下区占 40%。
- **D-16:** 上区双列：左侧 VideoTable 占 60%，右侧设置卡片区占 40%。列间 16px gap。
- **D-17:** 下区双列：左侧操作按钮行占 60%，右侧 LogPanel 占 40%。列间 16px gap。与上区左右对齐。
- **D-18:** 全宽布局（无最大宽度），内容充满 v-main。Tauri 窗口设置最小宽度（如 900px），保证双列布局始终可用，不做响应式断点。
- **D-19:** 上下区间用 16px gap 分隔，双列间用 16px gap 分隔。无 v-divider 物理分割线。
- **D-20:** VideoTable、操作按钮行、LogPanel 区域使用 labeled placeholder（灰色虚线框 + 区域标题 + 「Phase 17 实现」标注）。随 Phase 17 替换为真实组件。
- **D-21:** 上区右列仅包含 ModeSelector 设置卡片（详见 ModeSelector 决策），不放其他配置项。

### ModeSelector 模式切换
- **D-22:** 控件：Vuetify v-select 下拉选择框，纯中文标签（纪录片解说 | 短剧解说 | 短剧混剪）。
- **D-23:** 包装：v-card 卡片，标题「工作模式」+ mdi-folder-multiple 图标，outlined 变体。
- **D-24:** 模式切换行为：更新 `ModeStore.currentMode` + 调用 `ModeStore.resetPipeline()`（清空 pipeline 状态和进度）。不弹出确认对话框，不阻止。
- **D-25:** 默认模式：`documentary`。用户选择通过 localStorage(`lastMode`) 持久化，下次启动恢复。ModeStore 初始化时读取。
- **D-26:** 面板动态显隐规则（MODE-03）在当前阶段不生效 — LLM/TTS/BGM/Export 面板属于 Phase 16 SettingsDrawer 内容。Phase 15 仅建立模式切换基础。

### SettingsDrawer 设置抽屉
- **D-27:** 实现：`v-navigation-drawer` location="right" temporary width="420"。
- **D-28:** 内部结构（三段式）：
  - Header：齿轮 mdi-cog 图标 + "设置" 标题 + 关闭按钮（mdi-close）
  - Content（可滚动）：外观设置 — `SettingSection` 组件包裹的主题切换（mdi 动态图标 + "深色模式" 标签 + 状态描述 + 右侧 v-switch）
  - Footer：版本号 `vX.X.X`（通过 `@tauri-apps/api/app` 的 `getVersion()` 获取，非 Tauri 环境回退默认值）。无保存按钮（Phase 16 添加）。
- **D-29:** 创建 `SettingSection.vue` 可复用组件（collapsible card + icon + title + default slot），供主题切换区域和 Phase 16 配置面板复用。
- **D-30:** 自定义滚动条样式：6px 宽，圆角，半透明（`rgba(var(--v-theme-on-surface), 0.2)`），hover 变深。
- **D-31:** 关闭方式：Header 关闭按钮 / 点击 scrim（v-navigation-drawer temporary 默认行为）/ AppHeader 齿轮按钮 toggle。

### SystemMonitorBar 系统监控
- **D-32:** 位置：DefaultLayout v-footer 中，应用所有页面共享，始终可见。
- **D-33:** 数据来源：Rust 后端新增 `get_system_stats` Tauri 命令，返回 `{ cpu_percent: f64, ram_percent: f64 }`。后端使用 `sysinfo` crate 读取系统指标。
- **D-34:** 显示格式：mdi-chip CPU 图标 + "CPU XX%" | mdi-memory RAM 图标 + "RAM XX%"。字体小号，灰色。v-footer 紧凑单行布局。
- **D-35:** 更新策略：前端 `setInterval` 每 2 秒调用一次 `get_system_stats`，组件 `onUnmounted` 时清理 interval。

### Claude's Discretion
- `tauri.conf.json` 的 `minWidth` 具体像素值和窗口默认大小
- AppHeader 和 WindowControls 按钮的具体 Vuetify icon 名称和 hover 样式
- v-footer 的高度和颜色 token
- HomeView 占位卡片的精确样式（虚线框颜色、文字大小）
- SystemMonitorBar `get_system_stats` 命令的具体签名和错误处理
- SettingSection.vue 的 collapsible 实现细节和过渡动画
- `isMaximized` 状态的初始化方式（`getCurrentWindow().isMaximized()` 调用时机）
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SmartEdit 参考项目（布局和交互模式来源）
- `E:/GitLib/SmartEdit/src/components/AppHeader.vue` — 自定义标题栏：Logo + 标题 + 设置齿轮 + 主题切换 + WindowControls，窗口拖拽和双击最大化
- `E:/GitLib/SmartEdit/src/components/SettingsDrawer.vue` — 420px 右侧抽屉：三段式结构（Header/Content/Footer），SettingSection 组件用法
- `E:/GitLib/SmartEdit/src/components/WindowControls.vue` — 窗口控制按钮组件（最小化/最大化/关闭），Tauri window API 调用
- `E:/GitLib/SmartEdit/src/components/SettingSection.vue` — 可复用设置分组卡片组件（icon + title + collapsible + slot）
- `E:/GitLib/SmartEdit/src/layouts/DefaultLayout.vue` — 三区域布局：AppHeader + v-main（router-view）+ SettingsDrawer，窗口事件处理
- `E:/GitLib/SmartEdit/src/assets/logo.svg` — **直接复用为 NarratoAI logo**（复制到 `src/assets/logo.svg`）

### NarratoAI Phase 14 已产出前端代码
- `src/main.ts` — 应用入口：createApp + Pinia + Router + Vuetify 初始化
- `src/App.vue` — 根组件（`<v-app><router-view /></v-app>`）
- `src/router/index.ts` — Hash 路由（createWebHashHistory），`/` → DefaultLayout → HomeView，404 catch-all
- `src/layouts/DefaultLayout.vue` — **当前简易版布局（v-app-bar + v-main），本 Phase 需重写**为三区域 + AppHeader + SettingsDrawer
- `src/views/HomeView.vue` — **当前欢迎占位页，本 Phase 需重写**为上下双列分区布局
- `src/views/NotFoundView.vue` — 404 页面（保持不变）
- `src/plugins/vuetify.ts` — Vuetify 4 插件：双主题色定义、zhHans 本地化、@mdi/font 图标
- `src/composables/useTheme.ts` — 主题切换桥接（AppStore ↔ Vuetify theme.global.name）
- `src/stores/app.ts` — AppStore：theme 状态 + toggle/set + localStorage 持久化
- `src/stores/mode.ts` — ModeStore：WorkMode 类型 + currentMode + setMode + resetPipeline（stub）
- `src/stores/llm.ts` — LLM Store 骨架（Phase 16 使用）
- `src/stores/tts.ts` — TTS Store 骨架（Phase 16 使用）
- `src/stores/bgm.ts` — BGM Store 骨架（Phase 16 使用）
- `src/stores/video.ts` — Video Store 骨架（Phase 17 使用）
- `src/stores/pipeline.ts` — Pipeline Store 骨架（Phase 17 使用）
- `src/stores/script.ts` — Script Store 骨架（Phase 17 使用）
- `src/stores/export.ts` — Export Store 骨架（Phase 16 使用）
- `.planning/phases/14-foundation-frontend-scaffold/14-CONTEXT.md` — Phase 14 全部决策（目录结构、Store 粒度、API 封装、类型同步）
- `.planning/phases/14-foundation-frontend-scaffold/14-UI-SPEC.md` — 设计体系（spacing 8px grid、typography、颜色、copywriting）

### Tauri 2.0 窗口 API
- `src-tauri/tauri.conf.json` — Tauri 2.0 配置（需新增 `decorations: false`、`minWidth`、前端 dev/build 命令）
- `@tauri-apps/api/window` — `getCurrentWindow().minimize()/toggleMaximize()/close()/startDragging()/isMaximized()`
- `@tauri-apps/api/app` — `getVersion()` 用于 SettingsDrawer 版本号

### Rust 后端（新增 Tauri 命令）
- `src-tauri/src/commands/` — 现有 15 个 Tauri 命令，需新增 `get_system_stats`
- `src-tauri/Cargo.toml` — 需新增 `sysinfo` crate 依赖

### 项目规划文档
- `.planning/ROADMAP.md` — Phase 15 定义、成功标准、依赖 Phase 14
- `.planning/REQUIREMENTS.md` — LOUT-01~06、MODE-01~03 详细要求
- `.planning/PROJECT.md` — 项目约束（桌面单用户、文件存储、简体中文 UI、SmartEdit 视觉参考）
- `.planning/STATE.md` — 项目状态
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets（Phase 14 已产出）
- `src/composables/useTheme.ts` — 完整的主题切换逻辑，AppHeader + SettingsDrawer 直接调用 `toggleTheme()` / `theme`
- `src/stores/app.ts` — localStorage 持久化主题偏好，已实现
- `src/stores/mode.ts` — `WorkMode` 类型和 `currentMode`/`setMode()`，需扩展 localStorage 持久化和 `resetPipeline()` 实现
- `src/plugins/vuetify.ts` — light/dark 双主题完整配色，全局可用
- `src/composables/useTauri.ts` — tauriInvoke 封装，SystemMonitorBar 调用 `get_system_stats` 用

### SmartEdit 参考模式（不复制代码，重写但对齐交互）
- **AppHeader.vue**: 40px v-app-bar + logo.png + 标题 + 齿轮按钮 + 主题按钮 + WindowControls。关键：mousedown 拖拽（排除 button 区域）、dblclick toggleMaximize
- **WindowControls.vue**: 三个 v-btn icon（mdi-window-minimize / mdi-window-maximize-or-mdi-window-restore / mdi-close），通过 emit 通知父组件
- **SettingsDrawer.vue**: v-navigation-drawer temporary right 420px + Header（齿轮+设置+关闭）+ Content（SettingSection 组件）+ Footer（保存按钮+版本号）
- **SettingSection.vue**: 可折叠卡片，props: title + icon + collapsible + defaultExpanded + variant，slot: default

### Integration Points
- **DefaultLayout.vue** → 重写现文件，替换 v-app-bar placeholder 为 AppHeader，新增 SettingsDrawer + SystemMonitorBar
- **HomeView.vue** → 重写现文件，替换欢迎占位为上下双列分区布局 + 占位卡片
- **ModeStore** → 扩展 localStorage 持久化和 `resetPipeline()` 实现
- **AppStore** → 无需修改（theme 已完整实现）
- **Router** → 无需修改（Hash 路由 + DefaultLayout 嵌套结构不变）
- **Rust 后端** → 新增 `src-tauri/src/commands/system.rs`（get_system_stats 命令）+ `Cargo.toml` 加 `sysinfo`

### 新建文件清单
| 文件 | 用途 |
|------|------|
| `src/components/AppHeader.vue` | 自定义标题栏组件 |
| `src/components/WindowControls.vue` | 窗口控制按钮组件 |
| `src/components/SettingsDrawer.vue` | 右侧设置抽屉组件 |
| `src/components/SettingSection.vue` | 可复用设置分组卡片组件 |
| `src/components/SystemMonitorBar.vue` | 系统监控栏组件 |
| `src/assets/logo.svg` | 从 SmartEdit 复制 |
</code_context>

<specifics>
## Specific Ideas

- AppHeader 应与 SmartEdit AppHeader 视觉高度一致（40px 紧凑设计、border-bottom 分割线、窗口控制按钮排列），但组件代码完全独立编写
- WindowControls 三个按钮使用 Vuetify v-btn icon，图标 mdi-window-minimize / mdi-window-maximize（或 mdi-window-restore 根据 isMaximized 状态切换）/ mdi-close
- 设置齿轮按钮复用 AppHeader 已有的齿轮元素，点击切换 SettingsDrawer
- 主题按钮图标根据当前主题动态切换：`isDark ? 'mdi-weather-sunny' : 'mdi-weather-night'`
- HomeView 占位卡片标注明确告知开发者哪个组件在哪个 Phase 实现，方便后续替换
- SystemMonitorBar 的 `get_system_stats` Tauri 命令应在 Rust 侧使用 `sysinfo` crate 的 `System::refresh_all()` 获取 CPU/RAM 数据，避免连续创建 System 实例（复用）
- SettingSection 组件设计应支持 Phase 16 的 collapsible 用法（LLM/TTS/BGM/Export 各用一个 SettingSection）
- Phase 15 不引入新的 npm 依赖 — `@tauri-apps/api` 已安装，`sysinfo` 是 Rust 侧 crate
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

### Reviewed Todos (not folded)
- **Replace integration test stubs using assert!(true)** — 已折叠进 Phase 14 TDET-03，已完成。不适用于 Phase 15。
- **Fix Tauri State read locks held across long pipelines** — Phase 18 TDET-02，不在本 Phase 范围。

</deferred>

---

*Phase: 15-layout-shell-3-mode-ux*
*Context gathered: 2026-05-13*
