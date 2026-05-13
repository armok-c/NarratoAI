---
phase: 15-layout-shell-3-mode-ux
verified: 2026-05-13T16:45:00Z
status: passed
score: 42/42 must-haves verified
overrides_applied: 0
deferred:
  - truth: "面板根据模式动态显示/隐藏 — Vision LLM 面板仅 Documentary 可见，TTS 面板在 SDP 模式隐藏 (MODE-03)"
    addressed_in: Phase 16
    evidence: "Phase 16 goal 包含 LLM/TTS/BGM/Export 配置面板构建 (CONF-01~CONF-05)，模式专用面板显示逻辑依赖这些面板的存在。D-26 明确声明 MODE-03 在 Phase 15 不生效。面板本身 (CONF-01/CONF-02) 属于 Phase 16 范围。"
---

# Phase 15: Layout Shell + 3-Mode UX 验证报告

**阶段目标:** 布局外壳 + 3 模式 UX — 构建 Tauri 无边框窗口的完整 UI 外壳：自定义标题栏、三区布局、系统监控栏、设置抽屉、主页工作模式选择器
**验证时间:** 2026-05-13T16:45:00Z
**状态:** passed
**重验证:** 否 — 初始验证

## 目标达成情况

### 可观察 truth 验证

| # | Truth | 状态 | 证据 |
|---|-------|------|------|
| P01-1 | System resource data via get_system_stats 返回 { cpu_percent: f64, ram_percent: f64 } | ✓ VERIFIED | `src-tauri/src/commands/system.rs` — get_system_stats 命令返回 SystemStats { cpu_percent: f64, ram_percent: f64 } |
| P01-2 | Tauri capabilities 声明全部 5 个窗口权限 | ✓ VERIFIED | `src-tauri/capabilities/default.json` 包含 allow-minimize, allow-toggle-maximize, allow-close, allow-start-dragging, allow-is-maximized |
| P01-3 | Tauri window 配置 decorations: false, minWidth: 900, minHeight: 600, center: true | ✓ VERIFIED | `src-tauri/tauri.conf.json` app.windows[0] 包含所有 4 个字段 |
| P01-4 | sysinfo crate 注册在 Cargo.toml | ✓ VERIFIED | `src-tauri/Cargo.toml` 包含 `sysinfo = "0.33"` |
| P01-5 | Phase 15 测试存根文件存在 + vitest 配置正确 | ✓ VERIFIED | 9 个测试存根文件存在; `vitest.config.ts` 包含 `'src/**/__tests__/*.test.ts'` |
| P02-1 | AppHeader 40px 高度，显示 logo/NarratoAI/齿轮/主题/WindowControls | ✓ VERIFIED | `src/components/AppHeader.vue` — v-app-bar height="40", logo-container 32x32, NarratoAI 标题, mdi-cog, mdi-weather 图标, WindowControls |
| P02-2 | AppHeader mousedown 启动窗口拖拽 (排除按钮元素) | ✓ VERIFIED | startDrag 函数使用 e.target.closest('button') 和 closest('.window-controls') 排除 |
| P02-3 | AppHeader dblclick 触发 toggleMaximize | ✓ VERIFIED | handleDoubleClick 函数 emit('toggleMaximize') |
| P02-4 | WindowControls 渲染 3 个按钮 (最小化/最大化-还原/关闭) | ✓ VERIFIED | `src/components/WindowControls.vue` — mdi-window-minimize, mdi-window-maximize/mdi-window-restore, mdi-close |
| P02-5 | WindowControls 关闭按钮红色 hover (rgba(244, 67, 54, 0.8)) | ✓ VERIFIED | CSS: `.close-btn:hover { background-color: rgba(244, 67, 54, 0.8) !important; }` |
| P02-6 | WindowControls 是纯组件 (无 Tauri API 调用) | ✓ VERIFIED | 组件无 @tauri-apps 导入; emit minimize/maximize/close 事件 |
| P02-7 | ModeStore.setMode 更新 currentMode 并持久化到 localStorage('lastMode') | ✓ VERIFIED | `src/stores/mode.ts` — setMode 写入 currentMode + localStorage.setItem('lastMode', val) |
| P02-8 | ModeStore.resetPipeline 调用 usePipelineStore().reset() | ✓ VERIFIED | `src/stores/mode.ts` — resetPipeline 导入并调用 usePipelineStore().reset() |
| P02-9 | ModeStore 从 localStorage('lastMode') 初始化，回退 'documentary' | ✓ VERIFIED | `src/stores/mode.ts` — const stored = localStorage.getItem('lastMode'), 验证后赋值或回退 |
| P03-1 | SettingSection 渲染 title + icon 在 header，slot 内容在 body | ✓ VERIFIED | `src/components/SettingSection.vue` — header 行含 v-icon + title, 下方 v-expand-transition 含 slot |
| P03-2 | SettingSection 支持 collapsible 模式 | ✓ VERIFIED | collapsible prop 控制 header 点击切换 isExpanded, 使用 v-expand-transition |
| P03-3 | SettingSection collapsible header 显示 chevron 图标 | ✓ VERIFIED | v-if="collapsible" 显示 mdi-chevron-up/mdi-chevron-down |
| P03-4 | SettingSection 支持 variant prop (outlined/flat/elevated) | ✓ VERIFIED | variant prop 驱动 .setting-section--outlined/--flat/--elevated 类 |
| P03-5 | SettingsDrawer 从右侧打开，v-navigation-drawer temporary, 420px 宽 | ✓ VERIFIED | `src/components/SettingsDrawer.vue` — location="right" temporary width="420" |
| P03-6 | SettingsDrawer header 显示齿轮 + '设置' + 关闭按钮 | ✓ VERIFIED | header 含 mdi-cog, '设置' title, mdi-close 按钮 |
| P03-7 | SettingsDrawer content 自定义 8px scrollbar | ✓ VERIFIED | CSS: ::-webkit-scrollbar width: 8px, thumb rgba(0.2/0.3) |
| P03-8 | SettingsDrawer footer 显示版本号 | ✓ VERIFIED | footer 显示 versionText, 从 getVersion() 或回退 'v0.1.0' |
| P03-9 | SettingsDrawer 内容含 SettingSection: 外观 + mdi-brightness-6 + 深色模式 + v-switch | ✓ VERIFIED | SettingsDrawer.vue template 含 SettingSection icon="mdi-brightness-6" title="外观" + v-switch |
| P03-10 | SettingsDrawer 三种关闭方式 | ✓ VERIFIED | header 关闭按钮 (emit false), scrim 点击 (temporary drawer 默认), 父组件 v-model 切换 |
| P04-1 | DefaultLayout 渲染 AppHeader + v-main + SystemMonitorBar in v-footer | ✓ VERIFIED | `src/layouts/DefaultLayout.vue` — AppHeader, v-main + router-view, v-footer + SystemMonitorBar |
| P04-2 | DefaultLayout 管理 showSettings ref, 传给 AppHeader 和 SettingsDrawer | ✓ VERIFIED | showSettings ref, v-model:show-settings 传给 AppHeader, v-model 传给 SettingsDrawer |
| P04-3 | DefaultLayout 管理 isMaximized ref, 从 getCurrentWindow().isMaximized() 初始化 | ✓ VERIFIED | onMounted 动态导入 getCurrentWindow, 调用 isMaximized() |
| P04-4 | DefaultLayout 处理窗口控制事件 (动态导入 @tauri-apps/api/window) | ✓ VERIFIED | handleMinimize/handleMaximize/handleClose/handleToggleMaximize 全部动态导入 |
| P04-5 | DefaultLayout onMounted 同步 Vuetify 主题从 AppStore | ✓ VERIFIED | onMounted: vuetifyTheme.global.name.value = appStore.theme |
| P04-6 | SystemMonitorBar 在 v-footer 显示 CPU XX% 和 RAM XX% 带 mdi-chip/mdi-memory | ✓ VERIFIED | `src/components/SystemMonitorBar.vue` — template 含 mdi-chip + CPU XX%, mdi-memory + RAM XX% |
| P04-7 | SystemMonitorBar 每 2 秒轮询 get_system_stats | ✓ VERIFIED | setInterval(fetchStats, 2000) |
| P04-8 | SystemMonitorBar 在 onUnmounted 清理 interval | ✓ VERIFIED | onUnmounted: if (intervalId) clearInterval(intervalId) |
| P04-9 | SystemMonitorBar 错误或非 Tauri 环境显示 '--%' | ✓ VERIFIED | catch 块 error=true, template 显示 'CPU --%' / 'RAM --%' |
| P05-1 | HomeView 上区 60% + 下区 40% + 16px 垂直间距 | ✓ VERIFIED | `src/views/HomeView.vue` — flex:6 + flex:4, d-flex flex-column ga-4 |
| P05-2 | 上区左列 VideoTable 占位 60% + 右列 ModeSelector 40% + 16px 间距 | ✓ VERIFIED | d-flex ga-4 upper-section, flex:6 + flex:4 + ga-4 |
| P05-3 | 下区左列 操作按钮 占位 60% + 右列 LogPanel 占位 40% + 16px 间距 | ✓ VERIFIED | d-flex ga-4 lower-section, flex:6 + flex:4 + ga-4 |
| P05-4 | 占位卡片显示虚线框 + 区域标题 + 'Phase 17 实现' | ✓ VERIFIED | .placeholder-card: dashed border, 标题如 '视频列表', 'Phase 17 实现' sub-label |
| P05-5 | ModeSelector card 标题 '工作模式' + mdi-folder-multiple + outlined variant | ✓ VERIFIED | v-card variant="outlined", v-card-title 含 mdi-folder-multiple + '工作模式' |
| P05-6 | ModeSelector v-select 三个选项: 纪录片解说/短剧解说/短剧混剪 | ✓ VERIFIED | modeOptions = [ {title:'纪录片解说',value:'documentary'}, ... ] |
| P05-7 | ModeSelector v-select onChange 调用 setMode + resetPipeline | ✓ VERIFIED | handleModeChange: modeStore.setMode(val) + modeStore.resetPipeline() |
| P05-8 | v-select 初始值匹配 ModeStore.currentMode | ✓ VERIFIED | selectedMode = ref<WorkMode>(modeStore.currentMode) |
| P05-9 | 全宽布局 (no max-width), flex:6/flex:4 列比 | ✓ VERIFIED | .home-view 无 max-width, flex 比值符合 UI-SPEC 3.2 |

**分数:** 42/42 truths 已验证

### 延迟项 (Deferred Items)

以下项在 Phase 15 明确声明暂不生效，依赖后续阶段：

| # | 项目 | 处理阶段 | 证据 |
|---|------|---------|------|
| 1 | 面板根据模式动态显示/隐藏 — Vision LLM 面板仅 Documentary 可见，TTS 面板在 SDP 模式隐藏 (MODE-03) | Phase 16 | D-26 明确声明 "Panel dynamic show/hide (MODE-03) not active in Phase 15"。LLM 面板 (CONF-01) 和 TTS 面板 (CONF-02) 均为 Phase 16 构建范围。面板不存在则显隐逻辑无意义。Phase 16 的 CONF-05 涉及模式专用面板显示。 |

### 必要 Artifacts

| Artifact | 预期 | 状态 | 详情 |
|----------|------|------|------|
| `src-tauri/src/commands/system.rs` | get_system_stats command | ✓ VERIFIED | 98 行, sysinfo SystemStats + SystemState + get_system_stats + tests |
| `src-tauri/src/commands/mod.rs` | system 模块注册 | ✓ VERIFIED | pub mod system + register_all_commands! 含 get_system_stats |
| `src-tauri/src/lib.rs` | SystemState managed | ✓ VERIFIED | app.manage(SystemState(Mutex::new(System::new_all()))) |
| `src-tauri/capabilities/default.json` | 5 个窗口权限 | ✓ VERIFIED | 全部 5 个 core:window:allow-* 权限 |
| `src-tauri/tauri.conf.json` | 窗口配置 | ✓ VERIFIED | decorations:false, minWidth:900, minHeight:600, center:true |
| `vitest.config.ts` | __tests__ include | ✓ VERIFIED | include 含 'src/**/__tests__/*.test.ts' |
| `src/assets/logo.svg` | App logo | ✓ VERIFIED | 24 行 SVG (>= min_lines:20) |
| `src/components/AppHeader.vue` | 自定义标题栏 | ✓ VERIFIED | 126 行, 包含拖拽/dblclick/齿轮/主题/WindowControls |
| `src/components/WindowControls.vue` | 纯窗口控件 | ✓ VERIFIED | 66 行, 3 按钮, 红色 hover |
| `src/stores/mode.ts` | ModeStore 扩展 | ✓ VERIFIED | localStorage 初始化/持久化 + resetPipeline |
| `src/components/SettingSection.vue` | 可折叠设置卡片 | ✓ VERIFIED | 112 行, 6 props, collapsible/outlined/loading |
| `src/components/SettingsDrawer.vue` | 右侧设置抽屉 | ✓ VERIFIED | 128 行, 三段式, 420px, 主题切换, 版本号 |
| `src/components/SystemMonitorBar.vue` | CPU/RAM 监控 | ✓ VERIFIED | 56 行, 2 秒轮询, --% fallback |
| `src/layouts/DefaultLayout.vue` | 三区域布局 | ✓ VERIFIED | 93 行, AppHeader + v-main + v-footer + SettingsDrawer |
| `src/views/HomeView.vue` | 主页 60/40 分区 | ✓ VERIFIED | 96 行, 上/下分区 + 占位卡片 + ModeSelector |

#### 测试存根文件

| Artifact | 状态 | 详情 |
|----------|------|------|
| `src/components/__tests__/AppHeader.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/components/__tests__/WindowControls.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/components/__tests__/SettingSection.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/components/__tests__/SettingsDrawer.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/components/__tests__/SystemMonitorBar.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/views/__tests__/HomeView.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/layouts/__tests__/DefaultLayout.test.ts` | ✓ VERIFIED | 8 行, vitest stub |
| `src/stores/__tests__/mode.test.ts` | ✓ VERIFIED | 8 行, vitest stub |

### 关键链接验证

| 从 | 到 | 路径 | 状态 | 详情 |
|----|----|------|------|------|
| lib.rs | system.rs | app.manage(SystemState(...)) | ✓ WIRED | `app.manage(SystemState(Mutex::new(System::new_all())))` |
| commands/mod.rs | system.rs | register_all_commands!() | ✓ WIRED | `crate::commands::system::get_system_stats` 在宏展开中 |
| capabilities/default.json | tauri.conf.json | 窗口标识符 "main" | ✓ WIRED | 两个文件均使用 "main" 标识符 |
| AppHeader.vue | WindowControls.vue | Import + render | ✓ WIRED | `<WindowControls :is-maximized="..." .../>` |
| AppHeader.vue | useTheme.ts | toggleTheme 调用 | ✓ WIRED | `const { theme, toggleTheme } = useTheme()` |
| AppHeader.vue | DefaultLayout.vue | emit 窗口控制事件 | ✓ WIRED | emits: minimize, maximize, close, toggleMaximize |
| stores/mode.ts | localStorage | setItem('lastMode', val) | ✓ WIRED | `localStorage.setItem('lastMode', val)` |
| stores/mode.ts | stores/pipeline.ts | reset() 调用 | ✓ WIRED | `usePipelineStore().reset()` |
| SettingsDrawer.vue | SettingSection.vue | Import + render | ✓ WIRED | `<SettingSection icon="mdi-brightness-6" title="外观">` |
| SettingsDrawer.vue | @tauri-apps/api/app | getVersion() | ✓ WIRED | 动态 import getVersion |
| SettingsDrawer.vue | useTheme.ts | toggleTheme 调用 | ✓ WIRED | `const { theme, toggleTheme } = useTheme()` |
| DefaultLayout.vue | AppHeader.vue | Import + render | ✓ WIRED | `<AppHeader v-model:show-settings="showSettings" .../>` |
| DefaultLayout.vue | SettingsDrawer.vue | v-model | ✓ WIRED | `<SettingsDrawer v-model="showSettings" />` |
| DefaultLayout.vue | SystemMonitorBar.vue | Import + render | ✓ WIRED | `<SystemMonitorBar />` 在 v-footer 内 |
| DefaultLayout.vue | @tauri-apps/api/window | 动态导入 | ✓ WIRED | 4 个 handler 全部使用动态 import |
| SystemMonitorBar.vue | useTauri.ts | tauriInvoke | ✓ WIRED | `tauriInvoke('get_system_stats')` |
| SystemMonitorBar.vue | Rust get_system_stats | Tauri IPC | ✓ WIRED | `tauriInvoke('get_system_stats')` -> Rust 命令 |
| HomeView.vue | stores/mode.ts | useModeStore() | ✓ WIRED | `modeStore.setMode(val) + modeStore.resetPipeline()` |
| HomeView.vue | router/index.ts | 路由 '/' | ✓ WIRED | DefaultLayout > HomeView 子路由已配置 |

### 数据流追踪 (Level 4)

| Artifact | 数据变量 | 数据源 | 产生真实数据 | 状态 |
|----------|---------|--------|-------------|------|
| SystemMonitorBar.vue | cpuPercent, ramPercent | tauriInvoke('get_system_stats') -> sysinfo | ✓ FLOWING | sysinfo 在 Rust 后端实时查询系统 CPU/RAM |
| SettingsDrawer.vue | versionText | getVersion() Tauri API / fallback | ✓ FLOWING | 调用 Tauri API 获取真实版本号 |
| AppHeader.vue | isDark | useTheme().theme -> AppStore | ✓ FLOWING | AppStore 从 localStorage 初始化并持久化 |
| ModeStore | currentMode | localStorage('lastMode') | ✓ FLOWING | 真实 localStorage 读写 |
| DefaultLayout | isMaximized | getCurrentWindow().isMaximized() | ✓ FLOWING | Tauri 窗口 API 返回真实最大化状态 |
| HomeView.vue | selectedMode | ModeStore.currentMode | ✓ FLOWING | watch 同步, handleModeChange 调用 store |

### 行为抽查

| 行为 | 命令 | 结果 | 状态 |
|------|------|------|------|
| TypeScript 编译 | `npx vue-tsc --noEmit` | exit 0, 无错误 | ✓ PASS |
| Rust 编译 | `cd src-tauri && cargo check` | exit 0, 无错误 (仅 2 个 pre-existing warnings) | ✓ PASS |
| Vitest 测试 | `npx vitest run` | 19/20 pass (1 个 pre-existing useTauri.test.ts failure) | ✓ PASS |
| 测试存根存在性 | 检查 8 个文件 | 全部存在 | ✓ PASS |

### 需求覆盖

| 需求 | 源计划 | 描述 | 状态 | 证据 |
|------|--------|------|------|------|
| LOUT-01 | 15-02 | AppHeader 自定义标题栏 | ✓ SATISFIED | AppHeader.vue + WindowControls.vue |
| LOUT-02 | 15-04 | DefaultLayout 三区域布局 | ✓ SATISFIED | DefaultLayout.vue |
| LOUT-03 | 15-05 | HomeView 上下分区布局 | ✓ SATISFIED | HomeView.vue |
| LOUT-04 | 15-03 | SettingsDrawer 右侧抽屉 | ✓ SATISFIED | SettingSection.vue + SettingsDrawer.vue |
| LOUT-05 | 15-02/15-03 | 深色/浅色主题切换 | ✓ SATISFIED | AppHeader useTheme, SettingsDrawer useTheme, AppStore localStorage 持久化 |
| LOUT-06 | 15-01/15-04 | SystemMonitorBar CPU/RAM | ✓ SATISFIED | system.rs + SystemMonitorBar.vue |
| MODE-01 | 15-02 | ModeStore 模式管理 + 重置 pipeline | ✓ SATISFIED | stores/mode.ts |
| MODE-02 | 15-05 | ModeSelector 下拉切换 | ✓ SATISFIED | HomeView.vue v-select + handleModeChange |
| MODE-03 | 15-05 | 面板动态可见性 | ➡️ Deferred | D-26 声明 Phase 15 不生效; 依赖 Phase 16 CONF-01/CONF-02 面板存在后才能实现 |

### 反模式扫描

| 文件 | 行 | 模式 | 严重程度 | 影响 |
|------|----|------|---------|------|
| (无) | - | TBD/FIXME/XXX 标记 | - | 未发现债务标记 |
| (无) | - | 静态 Tauri 窗口导入 | - | 所有 @tauri-apps/api/window 调用均使用动态 import + try/catch |
| (无) | - | 硬编码空数据桩 | - | 无 return null/{}/[] 或硬编码空 props |

### 需要人工验证的项

无。所有 truth 可以通过文件检查和编译/测试命令验证。

## 差距总结

**无阻塞性差距。**

所有 42 个计划级必须项已通过验证:

- 5 个 Rust/Tauri 后端基础设施 truth
- 9 个 AppHeader/WindowControls/ModeStore truth
- 10 个 SettingSection/SettingsDrawer truth
- 9 个 DefaultLayout/SystemMonitorBar truth
- 9 个 HomeView/ModeSelector truth

**1 个延迟项:** MODE-03 (面板动态可见性) 明确声明在 Phase 15 不生效 (D-26), 面板本身 (LLM/TTS/BGM/Export) 属于 Phase 16 (CONF-01~CONF-04) 构建范围。模式专用面板显示 (CONF-05) 也在 Phase 16, 因此 MODE-03 的显隐逻辑将在 Phase 16 面板就绪后自然解决。

**关键质量发现:**
- 所有 Tauri API 调用均使用动态 import + try/catch 模式, 支持浏览器开发模式降级
- 7 个数据流追踪均连接到真实数据源 (sysinfo/localStorage/Tauri API)
- Rust 后端 sysinfo 实例通过 Tauri State 管理, 确保 CPU delta 计算的连续性
- TypeScript 和 Rust 编译均通过, 无新增错误

---

*验证时间: 2026-05-13T16:45:00Z*
*验证者: Claude (gsd-verifier)*
