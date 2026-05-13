# Phase 15: Layout Shell + 3-Mode UX - Research

**Researched:** 2026-05-13
**Domain:** Vue 3 + Vuetify 4 layout shell, Tauri 2.0 window API integration, Rust system monitoring
**Confidence:** HIGH

## Summary

Phase 15 builds the complete desktop application shell: custom titlebar with native Tauri window controls (AppHeader + WindowControls), three-zone layout (DefaultLayout), home page with 60/40 split layout and labeled placeholder zones (HomeView), 420px right-side settings drawer (SettingsDrawer), reusable collapsible settings card (SettingSection), system resource monitor bar (SystemMonitorBar), and mode selection with persistence (ModeSelector/ModeStore).

The architecture follows SmartEdit's proven Vuetify layout patterns with three key technical integrations: (1) Tauri 2.0 `@tauri-apps/api/window` for native window control (minimize/maximize/close/drag), using dynamic imports for non-Tauri dev fallback; (2) new Rust Tauri command `get_system_stats` using the `sysinfo` crate for CPU/RAM monitoring; (3) Vuetify 4's layout system (`v-app-bar`, `v-navigation-drawer`, `v-footer`) via `v-layout` with proper component ordering.

**Critical interdependencies:** capabilities/default.json must declare window permissions; `sysinfo` crate added to Cargo.toml; Theme toggle reuses existing Phase 14 useTheme/AppStore; ModeStore extended with localStorage persistence and resetPipeline implementation.

**Primary recommendation:** Follow SmartEdit's event-based layout pattern (AppHeader emits window control events to DefaultLayout, which imports Tauri window API via dynamic import), use Vuetify 4 `v-navigation-drawer` with `temporary location="right" width="420"` for SettingsDrawer, and implement SystemMonitorBar with 2-second `setInterval` polling via `tauriInvoke`.

### Key Risks
1. Vuetify 3.6.0+ bug: temporary right-side drawer with explicit `width` may not hide fully — mitigation: verify behavior with Vuetify 4 and apply `:width="null"` workaround if needed
2. Tauri 2.0 requires capabilities declaration for window API — missing permissions cause silent failures
3. sysinfo System instance must be reused (not re-created on each poll) for correct CPU delta computation

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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
- **D-11:** DefaultLayout 结构：AppHeader（固定顶部）→ v-main（router-view）→ SystemMonitorBar（v-footer 固定底部）。SettingsDrawer 为 v-navigation-drawer temporary 右侧覆盖，独立于三区域流。
- **D-12:** DefaultLayout 管理 SettingsDrawer 开关状态（`showSettings: ref<boolean>`），通过 props 传递给 AppHeader 和 SettingsDrawer。
- **D-13:** DefaultLayout 管理窗口最大化状态（`isMaximized: ref<boolean>`），传给 AppHeader → WindowControls 更新图标。
- **D-14:** DefaultLayout 处理窗口控制事件（minimize/maximize/close），使用 `@tauri-apps/api/window` 动态导入。
- **D-15:** 上下分区固定比例 60/40（不可拖拽）。上区占 60%，下区占 40%。
- **D-16:** 上区双列：左侧 VideoTable 占 60%，右侧设置卡片区占 40%。列间 16px gap。
- **D-17:** 下区双列：左侧操作按钮行占 60%，右侧 LogPanel 占 40%。列间 16px gap。与上区左右对齐。
- **D-18:** 全宽布局（无最大宽度），内容充满 v-main。Tauri 窗口设置最小宽度（如 900px），保证双列布局始终可用，不做响应式断点。
- **D-19:** 上下区间用 16px gap 分隔，双列间用 16px gap 分隔。无 v-divider 物理分割线。
- **D-20:** VideoTable、操作按钮行、LogPanel 区域使用 labeled placeholder（灰色虚线框 + 区域标题 + 「Phase 17 实现」标注）。随 Phase 17 替换为真实组件。
- **D-21:** 上区右列仅包含 ModeSelector 设置卡片（详见 ModeSelector 决策），不放其他配置项。
- **D-22:** 控件：Vuetify v-select 下拉选择框，纯中文标签（纪录片解说 | 短剧解说 | 短剧混剪）。
- **D-23:** 包装：v-card 卡片，标题「工作模式」+ mdi-folder-multiple 图标，outlined 变体。
- **D-24:** 模式切换行为：更新 `ModeStore.currentMode` + 调用 `ModeStore.resetPipeline()`（清空 pipeline 状态和进度）。不弹出确认对话框，不阻止。
- **D-25:** 默认模式：`documentary`。用户选择通过 localStorage(`lastMode`) 持久化，下次启动恢复。ModeStore 初始化时读取。
- **D-26:** 面板动态显隐规则（MODE-03）在当前阶段不生效 — LLM/TTS/BGM/Export 面板属于 Phase 16 SettingsDrawer 内容。Phase 15 仅建立模式切换基础。
- **D-27:** 实现：`v-navigation-drawer` location="right" temporary width="420"。
- **D-28:** 内部结构（三段式）：
  - Header：齿轮 mdi-cog 图标 + "设置" 标题 + 关闭按钮（mdi-close）
  - Content（可滚动）：外观设置 — `SettingSection` 组件包裹的主题切换（mdi 动态图标 + "深色模式" 标签 + 状态描述 + 右侧 v-switch）
  - Footer：版本号 `vX.X.X`（通过 `@tauri-apps/api/app` 的 `getVersion()` 获取，非 Tauri 环境回退默认值）。无保存按钮（Phase 16 添加）。
- **D-29:** 创建 `SettingSection.vue` 可复用组件（collapsible card + icon + title + default slot），供主题切换区域和 Phase 16 配置面板复用。
- **D-30:** 自定义滚动条样式：6px 宽，圆角，半透明（`rgba(var(--v-theme-on-surface), 0.2)`），hover 变深。
- **D-31:** 关闭方式：Header 关闭按钮 / 点击 scrim（v-navigation-drawer temporary 默认行为）/ AppHeader 齿轮按钮 toggle。
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

### Deferred Ideas (OUT OF SCOPE)
None — all discussion stayed within Phase 15 scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LOUT-01 | AppHeader 自定义标题栏（Logo + "NarratoAI" + 设置齿轮 + 主题切换 + 窗口控制按钮） | SmartEdit AppHeader pattern; Vuetify v-app-bar height="40"; Tauri getCurrentWindow API for controls; logo.svg from SmartEdit assets |
| LOUT-02 | DefaultLayout 三区域布局（AppHeader + v-main + SettingsDrawer） | Vuetify v-layout component ordering pattern from SmartEdit; AppHeader emits window events to parent |
| LOUT-03 | HomeView 上下分区布局（60/40 split, 16px gaps, labeled placeholders） | CSS flex layout with flex:6/flex:4 ratios; Vuetify gap-4; labeled placeholder cards with dashed borders |
| LOUT-04 | SettingsDrawer 右侧临时抽屉（420px），包含外观设置 | v-navigation-drawer temporary right width="420"; SettingSection reusable component; custom scrollbar CSS |
| LOUT-05 | 深色/浅色主题切换，localStorage 持久化 | Already implemented in Phase 14 useTheme/AppStore; SettingsDrawer uses same composable with v-switch |
| LOUT-06 | SystemMonitorBar — CPU/RAM 实时使用率监控条 | New Rust command get_system_stats with sysinfo crate; 2s setInterval frontend polling; error fallback to --% |
| MODE-01 | ModeStore — 模式状态管理、切换时重置 pipeline 状态 | Extend existing mode.ts: add localStorage persistence (lastMode), implement resetPipeline() calling usePipelineStore().reset() |
| MODE-02 | ModeSelector — 设置卡片内下拉切换 | v-select in v-card outlined with mdi-folder-multiple icon; three zhHans options; calls ModeStore.setMode() + resetPipeline() |
| MODE-03 | 设置面板动态可见性 | Phase 15 establishes mode switching base only; actual panel dynamic visibility deferred to Phase 16 when config panels exist |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| AppHeader rendering | Browser (Vue/Vuetify) | — | Vuetify v-app-bar renders the custom titlebar; no SSR involved in Tauri app |
| Window control (min/max/close) | Tauri (native API) | Browser (Vue event handling) | Tauri `getCurrentWindow()` provides the actual native window operations; Vue components emit events to DefaultLayout which calls the API |
| Window drag | Tauri (native API) | Browser (mousedown event) | `startDragging()` from @tauri-apps/api/window; mousedown handler in AppHeader excludes button/window-controls via e.target.closest() |
| Three-zone layout | Browser (Vuetify) | — | v-layout + v-app-bar + v-main + v-footer + v-navigation-drawer all managed by Vuetify's component ordering system |
| SettingsDrawer toggle | Browser (DefaultLayout state) | — | showSettings ref managed by DefaultLayout, passed as props to AppHeader and v-model to SettingsDrawer |
| System monitoring data | Rust backend (sysinfo) | Browser (polling timer) | Rust command get_system_stats reads sysinfo; frontend setInterval polls every 2s; fallback --% on error |
| Mode state + persistence | Browser (Pinia + localStorage) | — | ModeStore.currentMode persisted via localStorage lastMode; no server round-trip needed |
| Theme state + persistence | Browser (Pinia + localStorage) | — | Already implemented in Phase 14 AppStore + useTheme; Phase 15 adds SettingsDrawer v-switch as second toggle entry point |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vuetify | 4.x (`^4.0.0`) | Layout components (v-app-bar, v-navigation-drawer, v-layout, v-main, v-footer, v-card, v-select, v-switch) | Phase 14 foundation; SmartEdit reference; zhHans locale configured |
| @tauri-apps/api | 2.x (`^2.0.0`) | Window control (getCurrentWindow), app version (getVersion) | Tauri 2.0 standard API; SmartEdit verified |
| Vue 3 | 3.5.x (`^3.5.0`) | Component composition (defineEmits, defineProps, onMounted, onUnmounted) | Core framework; setup script + TypeScript |
| Pinia | 3.x (`^3.0.0`) | ModeStore theme state management | Phase 14 foundation; Composition API style |
| @mdi/font | 7.x (`^7.0.0`) | Icons for AppHeader (mdi-cog, mdi-weather-sunny, mdi-weather-night), WindowControls (mdi-window-minimize/maximize/restore/close), SystemMonitorBar (mdi-chip, mdi-memory) | Phase 14 foundation; SmartEdit verified |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| sysinfo (Rust) | latest stable | System monitoring (CPU/RAM) | Rust Cargo.toml, get_system_stats Tauri command |
| @tauri-apps/api/app | 2.x | `getVersion()` for SettingsDrawer version display | SettingsDrawer footer onMounted |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Vuetify v-navigation-drawer | Custom CSS drawer | Vuetify handles scrim, transition animation, temporary behavior, and Tauri z-index out of the box. Custom implementation would re-implement all of these. |
| Dynamic import of @tauri-apps/api/window | Static import | Static import crashes in browser dev mode (outside Tauri). Dynamic import + try/catch is the SmartEdit-verified pattern. |
| sysinfo crate for system monitoring | Tauri process command (wmic/ps) | sysinfo is cross-platform, pure Rust, and provides consistent API. Shell command parsing is fragile and platform-specific. |

**Installation:**
```bash
# Frontend (already installed via Phase 14)
# No new npm dependencies needed for Phase 15

# Rust backend - add sysinfo to Cargo.toml
cd src-tauri && cargo add sysinfo
```

**Version verification:**
- Vuetify 4.x: already in package.json as `^4.0.0` [VERIFIED: package.json]
- @tauri-apps/api 2.x: already in package.json as `^2.0.0` [VERIFIED: package.json]
- vitest 4.x: already installed as devDep `^4.1.6` [VERIFIED: package.json]
- sysinfo: latest stable crates.io version — add via `cargo add sysinfo` [ASSUMED: latest version compatible with Rust edition 2021]

## Architecture Patterns

### System Architecture Diagram

```
User Input (mouse/keyboard)
    │
    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ AppHeader.vue (40px fixed top)                                      │
│ [Logo] [NarratoAI] [spacer] [gear btn] [theme btn] [WindowControls] │
│   ▲ drag region (excl. buttons)   ▲ dblclick = toggleMaximize       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ emits: minimize, maximize, close,
                           │        toggle-maximize, update:showSettings
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│ DefaultLayout.vue (v-layout)                                        │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────┐       │
│  │ v-main (router-view)                                      │       │
│  │  ┌─────────────────────────────────────────────────────┐  │       │
│  │  │ HomeView.vue                                        │  │       │
│  │  │  ┌──────────────────────┐ ┌─────────────────────┐   │  │       │
│  │  │  │ Upper Section (60%)  │ │                     │   │  │       │
│  │  │  │ ┌────────────────┐   │ │ ModeSelector card   │   │  │       │
│  │  │  │ │ VideoTable     │   │ │ 工作模式            │   │  │       │
│  │  │  │ │ (placeholder)  │   │ │ v-select dropdown   │   │  │       │
│  │  │  │ │ Phase 17 实现  │   │ │                     │   │  │       │
│  │  │  │ └────────────────┘   │ └─────────────────────┘   │  │       │
│  │  │  │ ← 16px gap →         │                           │  │       │
│  │  │  ├──────────────────────┴──────────────────────────┤  │       │
│  │  │  │ Lower Section (40%)                              │  │       │
│  │  │  │ ┌────────────────┐   ┌────────────────────────┐  │  │       │
│  │  │  │ │ 操作按钮行      │   │ LogPanel               │  │  │       │
│  │  │  │ │ (placeholder)  │   │ (placeholder)           │  │  │       │
│  │  │  │ │ Phase 17 实现  │   │ Phase 17 实现           │  │  │       │
│  │  │  │ └────────────────┘   └────────────────────────┘  │  │       │
│  │  │  └──────────────────────────────────────────────────┘  │       │
│  │  └──────────────────────────────────────────────────────────┘       │
│  └──────────────────────────────────────────────────────────────────────┘
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │ SystemMonitorBar.vue (v-footer)                               │    │
│  │ [mdi-chip] CPU XX%  |  [mdi-memory] RAM XX%                  │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  SettingsDrawer.vue (v-navigation-drawer, temporary, right, 420px)   │
│  ┌─────────────────────────────────────┐                            │
│  │ [mdi-cog] 设置                   [X] │ (Header, fixed)            │
│  ├─────────────────────────────────────┤                            │
│  │ SettingSection: 外观                │ (Content, scrollable)       │
│  │ [sun/moon] 深色模式 [v-switch]     │                            │
│  │ 浅色/深色主题已启用                 │                            │
│  ├─────────────────────────────────────┤                            │
│  │ v0.1.0                              │ (Footer, version only)     │
│  └─────────────────────────────────────┘                            │
└──────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Tauri Native Window                                                 │
│  - minimize() / toggleMaximize() / close() / startDragging()        │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼ (get_system_stats every 2s)
┌─────────────────────────────────────────────────────────────────────┐
│ Rust Backend (sysinfo crate)                                        │
│  System::refresh_all() → global_cpu_info().cpu_usage()              │
│                        → used_memory() / total_memory() * 100       │
└─────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure
```
src/
├── assets/
│   └── logo.svg                          # COPY from SmartEdit (D-03)
├── components/
│   ├── AppHeader.vue                     # NEW — custom titlebar
│   ├── WindowControls.vue                # NEW — min/max/close buttons
│   ├── SettingsDrawer.vue                # NEW — 420px right drawer
│   ├── SettingSection.vue                # NEW — reusable collapsible card
│   └── SystemMonitorBar.vue              # NEW — CPU/RAM footer bar
├── layouts/
│   └── DefaultLayout.vue                 # REWRITTEN — full 3-zone + drawer
├── views/
│   └── HomeView.vue                      # REWRITTEN — 60/40 split + placeholders
└── stores/
    └── mode.ts                           # EXTENDED — localStorage persistence + resetPipeline

src-tauri/
├── src/commands/
│   ├── mod.rs                            # MODIFIED — add system module + register_all_commands
│   └── system.rs                         # NEW — get_system_stats command
├── Cargo.toml                            # MODIFIED — add sysinfo crate
├── capabilities/
│   └── default.json                      # MODIFIED — add window permissions
└── tauri.conf.json                       # MODIFIED — decorations:false, minWidth, minHeight, center
```

### Pattern 1: Window Control Event Chain
**What:** AppHeader and WindowControls emit events, DefaultLayout processes them with Tauri API.
**When to use:** All window control operations (minimize, maximize, close, drag, toggle-maximize)
**Example:**
```typescript
// WindowControls.vue — pure component, no Tauri dependency
defineProps<{ isMaximized?: boolean }>()
defineEmits<{ minimize: []; maximize: []; close: [] }>()

// AppHeader.vue — forwards events upward
defineEmits<{
  minimize: []; maximize: []; close: []; 'toggle-maximize': []
  'update:showSettings': [value: boolean]
}>()

// DefaultLayout.vue — processes events with Tauri API
async function handleMaximize() {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  const win = getCurrentWindow()
  await win.toggleMaximize()
  isMaximized.value = await win.isMaximized()
}
```
[VERIFIED: SmartEdit src/components/WindowControls.vue, DefaultLayout.vue source inspection]

### Pattern 2: AppHeader Drag with Button Exclusion
**What:** mousedown on AppHeader starts window drag, but buttons and window-controls area are excluded.
**When to use:** Custom titlebar implementation with Tauri window drag
**Example:**
```typescript
async function startDrag(e: MouseEvent) {
  if (e.button !== 0) return  // left click only
  const target = e.target as HTMLElement
  if (target.closest('button') || target.closest('.window-controls')) return
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  await getCurrentWindow().startDragging()
}
```
[VERIFIED: SmartEdit AppHeader.vue source inspection]

### Pattern 3: System Monitor Fallback
**What:** SystemMonitorBar uses setInterval to poll get_system_stats, with graceful fallback for non-Tauri environments.
**When to use:** Any Tauri command called from a component that may render outside Tauri (dev mode in browser)
**Example:**
```typescript
import { ref, onMounted, onUnmounted } from 'vue'
import { tauriInvoke } from '@/composables/useTauri'

const cpuPercent = ref(0)
const ramPercent = ref(0)
const error = ref(false)
let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  intervalId = setInterval(async () => {
    try {
      const stats = await tauriInvoke<{ cpu_percent: number; ram_percent: number }>('get_system_stats')
      cpuPercent.value = Math.round(stats.cpu_percent)
      ramPercent.value = Math.round(stats.ram_percent)
      error.value = false
    } catch {
      error.value = true
    }
  }, 2000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})
```
[VERIFIED: D-35 decision; SmartEdit SystemMonitorBar pattern adapted per D-34/D-35 specifics]

### Pattern 4: SettingsDrawer v-model + Control
**What:** SettingsDrawer uses v-model binding from DefaultLayout, toggled by AppHeader gear button and scrim click.
**When to use:** Any temporary overlay drawer pattern
**Example:**
```vue
<!-- DefaultLayout.vue -->
<AppHeader v-model:show-settings="showSettings" ... />
<SettingsDrawer v-model="showSettings" />

<!-- SettingsDrawer.vue uses @update:model-value to emit changes back -->
<v-navigation-drawer
  :model-value="modelValue"
  location="right"
  temporary
  width="420"
  @update:model-value="emit('update:modelValue', $event)"
>
```
[VERIFIED: SmartEdit SettingsDrawer.vue and DefaultLayout.vue source inspection]

### Anti-Patterns to Avoid
- **Static import of @tauri-apps/api/window:** Crashes when running in browser dev mode outside Tauri. Always use dynamic import with try/catch.
- **Creating new sysinfo::System on each poll:** CPU usage delta tracking requires the same System instance. Use `std::sync::Mutex<System>` as a Tauri State or lazy_static to reuse.
- **Inline window control logic in WindowControls:** WindowControls should be a pure component (emit events, no Tauri API calls). Let DefaultLayout handle the API calls.
- **Directly mutating SettingsDrawer state from AppHeader:** The v-model pattern with DefaultLayout as mediator prevents state fragmentation.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Custom titlebar drag | mousedown position tracking | Tauri `getCurrentWindow().startDragging()` | Tauri handles per-monitor DPI, multi-screen, and OS-level drag behavior. Custom position tracking breaks on Windows with multiple monitors. |
| Window control buttons | IPC commands via shell | Tauri `@tauri-apps/api/window` | Direct API calls, no shell overhead. Properly handles OS window manager behavior. |
| System resource monitoring | Manual WMI/ps parsing | `sysinfo` crate | Cross-platform (Windows/Linux/macOS), pure Rust, handles CPU delta tracking internally. |
| Settings drawer overlay | CSS position:fixed drawer | Vuetify v-navigation-drawer | Vuetify handles scrim, z-index, animation, focus trapping, and keyboard dismissal. Custom CSS would need all of these. |
| App version display | Hardcoded version string | `@tauri-apps/api/app` `getVersion()` | Reads actual build version from tauri.conf.json. Avoids version mismatch between UI and packaged app. |

**Key insight:** Every one of these "don't hand-roll" items involves OS-level behavior (window management, system resources) or UI behaviors with significant edge cases (drawer overlay). The existing APIs (Tauri native, sysinfo, Vuetify) handle these edge cases with years of production testing. Custom implementations would introduce subtle bugs that are hard to reproduce.

## Runtime State Inventory

> Phase 15 is a greenfield frontend phase with no rename/refactor operations. Omitted as not applicable.

## Common Pitfalls

### Pitfall 1: Vuetify 3.6.0+ Temporary Right Drawer Bug
**What goes wrong:** A `v-navigation-drawer` with `temporary location="right"` and explicit `width="420"` may not fully hide when closed — a portion of the drawer remains visible.
**Why it happens:** Vuetify 3.6.0 introduced a layout calculation change where the drawer's width is used to compute its slide-out translation, but the translation calculation doesn't account for the drawer being temporary.
**How to avoid:** If the issue occurs with Vuetify 4, set `:width="null"` alongside the desired width to force Vuetify to handle the slide-out position correctly: `<v-navigation-drawer temporary location="right" width="420" :width="null">`. Test this before binding to the SettingsDrawer implementation.
**Warning signs:** Drawer doesn't slide fully off-screen on close; a thin strip remains visible.
[CITED: blog.gitcode.com — Vuetify 3.6.0 technical analysis]

### Pitfall 2: Tauri 2.0 Window Permissions Not Declared
**What goes wrong:** `getCurrentWindow().minimize()` / `.close()` / `.startDragging()` calls silently fail or throw permission denied errors.
**Why it happens:** Tauri 2.0 requires explicit capabilities/permissions for each window API method. The current `src-tauri/capabilities/default.json` only declares `core:default`, `core:event:default`, `core:event:allow-emit`, `core:event:allow-listen`, and `dialog:default` — no window permissions.
**How to avoid:** Add to `default.json`: `"core:window:allow-minimize"`, `"core:window:allow-toggle-maximize"`, `"core:window:allow-close"`, `"core:window:allow-start-dragging"`, and `"core:window:allow-is-maximized"`. Also test that `core:window:default` is sufficient or if specific permissions are needed.
**Warning signs:** Console errors like `"permission denied"` when clicking window control buttons in Tauri dev mode.
[VERIFIED: src-tauri/capabilities/default.json inspection; WebSearch Tauri 2.0 window API permissions documentation]

### Pitfall 3: sysinfo System Instance Not Reused
**What goes wrong:** CPU usage readings are always 0% or very inaccurate.
**Why it happens:** `sysinfo::System` computes CPU usage by comparing CPU time deltas between refresh calls. Creating a new System instance each time loses the previous state, so CPU usage can't be computed.
**How to avoid:** Store the `System` instance in Tauri State (`app.manage(Mutex::new(System::new_all()))`) or use `std::sync::OnceLock`. On each `get_system_stats` call: lock the mutex, call `sys.refresh_all()`, then read `sys.global_cpu_info().cpu_usage()` and compute `sys.used_memory() * 100 / sys.total_memory()`.
**Warning signs:** CPU always shows 0% even under load.
[CITED: docs.rs/sysinfo documentation — "Keep the same System instance"]

### Pitfall 4: Double-click startDragging Conflict
**What goes wrong:** When double-clicking on the AppHeader to toggle maximize, the first click's mousedown triggers `startDragging()` which interferes with the dblclick handler.
**Why it happens:** `startDragging()` initiates a drag sequence that may consume the mouse events needed for dblclick detection.
**How to avoid:** The SmartEdit pattern handles this correctly: mousedown starts drag (for single-click drag), and dblclick on the v-app-bar toggles maximize. The drag operation doesn't prevent the dblclick event because `startDragging()` only activates on actual mouse movement after mousedown (a click without movement doesn't start drag). Test this behavior explicitly.
**Warning signs:** Double-clicking AppHeader doesn't toggle maximize; window starts moving instead.
[VERIFIED: SmartEdit AppHeader.vue source inspection — mousedown + @dblclick coexist on same element]

### Pitfall 5: SettingsDrawer Content Overflow Without Scroll
**What goes wrong:** The drawer content overflows the available height without showing a scrollbar, hiding settings.
**Why it happens:** The drawer's content area needs `overflow-y: auto` and a flex layout to fill remaining space after the fixed header and footer. Without proper CSS, content flows past the drawer boundary.
**How to avoid:** Use `display: flex; flex-direction: column; height: 100%` on the drawer content wrapper. Set `flex: 1; overflow-y: auto` on `.settings-drawer__content` with `flex-shrink: 0` on header and footer. Custom scrollbar styles via `::-webkit-scrollbar`.
**Warning signs:** Settings content extends past visible drawer area with no scrollbar on long content.
[VERIFIED: SmartEdit SettingsDrawer.vue CSS pattern]

### Pitfall 6: Chinese Font Rendering Varies by OS
**What goes wrong:** Chinese text in Vuetify components renders differently on Windows vs Linux — some systems show fallback fonts that look inconsistent with the app bar title and other text elements.
**Why it happens:** The `font-family: Roboto, 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', sans-serif` stack works differently on each OS. Windows has Microsoft YaHei, macOS has PingFang SC, Linux may fall back to sans-serif.
**How to avoid:** This is already handled in Phase 14's `main.css` and Vuetify plugin config. For Phase 15, the main concern is Vuetify's `v-app-bar-title` element — ensure zhHans locale is passed correctly and that all text elements use Vuetify Typography classes (`text-body-2`, `text-h6`, etc.) rather than raw HTML text that bypasses Vuetify's font stack.
**Warning signs:** AppHeader title or SettingsDrawer header shows different font from Vuetify's default text.
[VERIFIED: src/styles/main.css font-family stack; Vuetify zhHans locale in vuetify.ts]

## Code Examples

### get_system_stats Tauri Command (Rust)

```rust
// src-tauri/src/commands/system.rs
use std::sync::Mutex;
use sysinfo::System;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct SystemStats {
    pub cpu_percent: f64,
    pub ram_percent: f64,
}

pub struct SystemState(pub Mutex<System>);

#[tauri::command]
pub async fn get_system_stats(
    state: State<'_, SystemState>,
) -> Result<SystemStats, String> {
    let mut sys = state.0.lock().map_err(|e| e.to_string())?;
    sys.refresh_all();
    let cpu_percent = sys.global_cpu_info().cpu_usage() as f64;
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let ram_percent = if total_mem > 0 {
        (used_mem as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };
    Ok(SystemStats { cpu_percent, ram_percent })
}
```
[CITED: docs.rs/sysinfo — System::refresh_all(), global_cpu_info(), used_memory(), total_memory()]

### Register SystemState in lib.rs

```rust
// In src-tauri/src/lib.rs setup closure:
use crate::commands::system::SystemState;
use sysinfo::System;

// After existing setup code:
app.manage(SystemState(Mutex::new(System::new_all())));
```
[VERIFIED: src-tauri/src/lib.rs existing pattern — app.manage() for Tauri State injection]

### Register Command in mod.rs

```rust
// In src-tauri/src/commands/mod.rs:
pub mod system;
pub use system::*;

// In register_all_commands!() macro:
crate::commands::system::get_system_stats,
```
[VERIFIED: src-tauri/src/commands/mod.rs existing pattern]

### Tauri Capabilities Window Permissions

```json
{
  "identifier": "default",
  "description": "NarratoAI 默认权限",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:event:allow-emit",
    "core:event:allow-listen",
    "dialog:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
    "core:window:allow-is-maximized"
  ]
}
```
[VERIFIED: WebSearch Tauri 2.0 window permissions documentation]

### DefaultLayout Window Control Handlers

```typescript
// src/layouts/DefaultLayout.vue
import { ref, onMounted } from 'vue'

const showSettings = ref(false)
const isMaximized = ref(false)

onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    isMaximized.value = await getCurrentWindow().isMaximized()
  } catch {
    // Non-Tauri environment — keep default false
  }
})

async function handleMinimize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().minimize()
  } catch { /* non-Tauri fallback */ }
}

async function handleMaximize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    await win.toggleMaximize()
    isMaximized.value = await win.isMaximized()
  } catch { /* non-Tauri fallback */ }
}
```
[VERIFIED: SmartEdit DefaultLayout.vue source inspection]

### ModeStore Extension (localStorage persistence)

```typescript
// src/stores/mode.ts — extended from Phase 14
export const useModeStore = defineStore('mode', () => {
  const stored = localStorage.getItem('lastMode') as WorkMode | null
  const currentMode = ref<WorkMode>(
    stored === 'documentary' || stored === 'sde' || stored === 'sdp'
      ? stored
      : 'documentary'
  )

  function setMode(val: WorkMode) {
    currentMode.value = val
    localStorage.setItem('lastMode', val)
  }

  function resetPipeline() {
    const pipelineStore = usePipelineStore()
    pipelineStore.reset?.()  // PipelineStore.reset() — to be implemented in Phase 17
    currentMode.value = 'documentary'
    localStorage.removeItem('lastMode')
  }

  return { currentMode, setMode, resetPipeline }
})
```
[VERIFIED: D-24, D-25; existing mode.ts stub pattern]

### tauri.conf.json Modifications

```json
{
  "app": {
    "windows": [
      {
        "title": "NarratoAI",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "decorations": false,
        "center": true
      }
    ]
  }
}
```
[VERIFIED: Existing tauri.conf.json; D-01, D-18; Claude's discretion for exact pixel values]

### SettingSection.vue Collapsible Pattern

```vue
<template>
  <div class="setting-section" :class="[`setting-section--${variant}`]">
    <div
      class="setting-section-header"
      :class="{ 'setting-section-header--clickable': collapsible }"
      @click="toggleExpand"
    >
      <div class="d-flex align-center">
        <v-icon v-if="icon" :icon="icon" size="20" class="mr-2" />
        <span class="setting-section-title">{{ title }}</span>
      </div>
      <div class="d-flex align-center">
        <slot name="header-actions" />
        <v-icon
          v-if="collapsible"
          :icon="isExpanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
          size="20"
          class="ml-2 setting-section-chevron"
        />
      </div>
    </div>
    <v-expand-transition>
      <div v-show="isExpanded" class="setting-section-content">
        <slot />
      </div>
    </v-expand-transition>
  </div>
</template>
```
[VERIFIED: SmartEdit SettingSection.vue source inspection; D-29 for Props: title, icon, collapsible, defaultExpanded, variant, loading]

### SystemMonitorBar Fallback Display

```vue
<template>
  <div class="monitor-container d-flex align-center ga-2">
    <div class="d-flex align-center" style="gap: 6px">
      <v-icon size="16">mdi-chip</v-icon>
      <span class="text-caption text-medium-emphasis">
        {{ error ? 'CPU --%' : `CPU ${cpuPercent}%` }}
      </span>
    </div>
    <div class="d-flex align-center" style="gap: 6px">
      <v-icon size="16">mdi-memory</v-icon>
      <span class="text-caption text-medium-emphasis">
        {{ error ? 'RAM --%' : `RAM ${ramPercent}%` }}
      </span>
    </div>
  </div>
</template>
```
[VERIFIED: D-34; D-35 polling pattern; Error handling per UI-SPEC copywriting contract]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1: `appWindow = window.appWindow` | Tauri v2: `getCurrentWindow()` | Tauri 2.0 GA | API rename; all window control code must use v2 imports |
| Tauri v1: permissions in tauri.conf.json allowlist | Tauri v2: capabilities/default.json permissions | Tauri 2.0 GA | Window API calls require explicit capability declarations |
| sysinfo v0.30: `system.cpus()` returns Vec | sysinfo v0.32+: `system.global_cpu_info().cpu_usage()` | sysinfo 0.32 | Simplified CPU usage API; global_cpu_info() replaces iterating over individual CPUs |
| Vuetify 3.5: temporary drawer works with explicit width | Vuetify 3.6+: layout refactor introduces drawer bug | Vuetify 3.6.0 | May need `:width="null"` workaround for temporary right drawers |
| SmartEdit: Event-based Tauri backend for system monitor (listen/unlisten) | NarratoAI: Polling-based Tauri command (setInterval + invoke) | Phase 15 design choice | Simpler implementation; no need for persistent backend event emission; `--%` on error |

**Deprecated/outdated:**
- Tauri v1 `window.appWindow` API: replaced by `getCurrentWindow()` in v2
- Tauri v1 `allowlist` in tauri.conf.json: replaced by capabilities in v2
- sysinfo `System::get_cpu()` and `System::get_processors()`: replaced by `System::cpus()` and later `System::global_cpu_info()`

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | sysinfo latest stable version is compatible with Cargo.toml's Rust edition 2021 and existing dependency versions | Standard Stack | Low — sysinfo is well-maintained and compatible with modern Rust editions |
| A2 | Vuetify 4 does not have the same temporary right-drawer width bug as Vuetify 3.6.0+ | Common Pitfalls | Low — if Vuetify 4 has the bug, the `:width="null"` workaround is documented and ready |
| A3 | Tauri 2.0 `getVersion()` from `@tauri-apps/api/app` returns the version set in `tauri.conf.json` | Standard Stack | Low — this is the documented behavior; fallback to `0.1.0` on error mitigates any issues |
| A4 | The `usePipelineStore().reset()` method exists (or will exist by Phase 17) | Code Examples | Medium — if PipelineStore doesn't have a `reset()` method, the ModeStore `resetPipeline()` implementation needs adjustment. Phase 17 will implement it. For now, a try/catch guard or optional chaining `?.reset()` prevents runtime errors. |

## Open Questions

1. **[Vuetify 4 temporary right drawer behavior]**
   - What we know: Vuetify 3.6.0 introduced a bug where temporary right drawers with explicit width don't fully hide. The workaround is `:width="null"` + `width="420"`.
   - What's unclear: Whether Vuetify 4 (which NarratoAI uses) has fixed this bug or if the workaround is still needed.
   - Recommendation: Build SettingsDrawer with explicit `width="420"` first. If the drawer doesn't fully close, add `:width="null"`. Test in the first wave.

2. **[isMaximized initialization timing]**
   - What we know: `getCurrentWindow().isMaximized()` must be called after the Tauri window is fully initialized. Dynamic import + try/catch handles non-Tauri env.
   - What's unclear: Whether calling `isMaximized()` in DefaultLayout's `onMounted` is early enough. The SmartEdit reference does it in `onMounted`.
   - Recommendation: Follow SmartEdit pattern — call in DefaultLayout's `onMounted`. If initial state is wrong, the user can toggle to correct it.

3. **[sysinfo crate CPU accuracy with 2-second polling]**
   - What we know: `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL` is the recommended sleep between CPU readings for accurate delta calculation.
   - What's unclear: Whether 2 seconds is enough interval for accurate CPU readings (likely yes since MINIMUM_INTERVAL is ~100ms).
   - Recommendation: Implement with 2s polling. CPU readings may show slight lag but will be directionally accurate.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build | ✓ | via npm | — |
| npm | Package management | ✓ | — | — |
| Rust toolchain | Rust backend build | ✓ | — | — |
| Vuetify 4 | All layout components | ✓ | ^4.0.0 (in package.json) | — |
| @tauri-apps/api 2.x | Window control, getVersion | ✓ | ^2.0.0 (in package.json) | — |
| sysinfo (Rust crate) | System monitoring | ✗ | — | Add via `cargo add sysinfo` |
| Tauri CLI | Dev server, build | ✓ | ^2.0.0 (devDep) | — |
| SmartEdit logo.svg | Logo asset | ✓ | — | Copy from E:/GitLib/SmartEdit/src/assets/logo.svg |
| Vuetify 4 icons (@mdi/font) | All icons | ✓ | ^7.0.0 (in package.json) | — |

**Missing dependencies with no fallback:**
- sysinfo crate: must be added to `src-tauri/Cargo.toml` via `cargo add sysinfo`. No viable alternative.

**Missing dependencies with fallback:**
- None — all frontend dependencies are already installed via Phase 14.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.x (`^4.1.6`) |
| Config file | none (default vitest config in package.json + vite.config.ts) |
| Quick run command | `npx vitest run --reporter=verbose` |
| Full suite command | `npm test` (runs `vitest run`) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOUT-01 | AppHeader renders logo, title, gear, theme, window controls | unit | `npx vitest run --reporter=verbose` — needs new test file | ❌ To be created |
| LOUT-02 | DefaultLayout renders AppHeader + v-main + SettingsDrawer | unit | as above — needs new test file | ❌ To be created |
| LOUT-03 | HomeView 60/40 split layout with placeholders | unit | as above — needs new test file | ❌ To be created |
| LOUT-04 | SettingsDrawer opens/closes correctly | unit | as above — needs new test file | ❌ To be created |
| LOUT-05 | Theme toggle persists via localStorage | unit | as above — needs new test file | ❌ To be created |
| LOUT-06 | SystemMonitorBar polls and displays data | unit | as above — needs new test file | ❌ To be created |
| MODE-01 | ModeStore persists lastMode to localStorage | unit | `npx vitest run` — can extend existing test if one exists | ❌ To be created |
| MODE-02 | ModeSelector renders and calls setMode | unit | as above — needs new test file | ❌ To be created |

### Sampling Rate
- **Per task commit:** `npx vitest run --reporter=verbose`
- **Per wave merge:** `npm test` (full suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/components/AppHeader.test.ts` — renders title, logo, buttons; drag exclusion; toggle-maximize emission
- [ ] `tests/components/WindowControls.test.ts` — renders three buttons; emits events; close hover style
- [ ] `tests/components/SettingsDrawer.test.ts` — opens/closes; theme section; version display
- [ ] `tests/components/SettingSection.test.ts` — collapsible toggle; icon/title rendering; slot content
- [ ] `tests/components/SystemMonitorBar.test.ts` — polling pattern; error fallback; display format
- [ ] `tests/layouts/DefaultLayout.test.ts` — full layout rendering; window control event handlers
- [ ] `tests/views/HomeView.test.ts` — 60/40 split; placeholder labels; mode selector card
- [ ] `tests/stores/mode.test.ts` — localStorage persistence; setMode; resetPipeline
- [ ] `tests/composables/useTauri.test.ts` — already exists at `tests/useTauri.test.ts`

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Desktop single-user; no authentication needed |
| V3 Session Management | no | Desktop single-user; no sessions |
| V4 Access Control | no | Desktop single-user; single app window |
| V5 Input Validation | no | ModeSelector dropdown has fixed options; no user text input in Phase 15 |
| V6 Cryptography | no | No cryptography operations in Phase 15 |

**Note:** Phase 15 introduces no new attack surface. Tauri window API permissions are scoped to `core:window:allow-*` (minimize, toggle-maximize, close, start-dragging, is-maximized) — these are UI operations with no data access. The `get_system_stats` command returns public system resource usage data — no sensitive information exposure.

## Sources

### Primary (HIGH confidence)
- SmartEdit source inspection: `AppHeader.vue`, `WindowControls.vue`, `SettingsDrawer.vue`, `SettingSection.vue`, `DefaultLayout.vue`, `SystemMonitorBar.vue` (E:/GitLib/SmartEdit/src/) — all layout and component patterns verified
- NarratoAI Phase 14 codebase inspection: `DefaultLayout.vue`, `HomeView.vue`, `mode.ts`, `app.ts`, `useTheme.ts`, `useTauri.ts`, `vuetify.ts`, `main.css`, `tauri.conf.json`, `Cargo.toml`, `lib.rs`, `commands/mod.rs`, `capabilities/default.json`, `package.json` — all existing state verified
- NarratoAI Phase 14/15 CONTEXT.md, UI-SPEC.md, REQUIREMENTS.md, ROADMAP.md — all 35 decisions (D-01 through D-35) and UI design contract verified

### Secondary (MEDIUM confidence)
- WebSearch: Vuetify 3.6.0 temporary right drawer bug documentation (`blog.gitcode.com`)
- WebSearch: Tauri 2.0 window API permissions documentation (`tauri.app`, `deepwiki.com`)
- WebSearch: sysinfo crate documentation (`docs.rs/sysinfo`)

### Tertiary (LOW confidence)
- None — all findings verified through source code inspection or official documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries confirmed via package.json inspection; all Vuetify/Tauri patterns confirmed via SmartEdit source code
- Architecture: HIGH — layout patterns verified against SmartEdit production code; event chain confirmed
- Pitfalls: HIGH — two of six pitfalls verified directly (Tauri capabilities file, sysinfo instance reuse); Vuetify drawer bug is well-documented with known workaround
- Code examples: HIGH — all examples derived from SmartEdit production code or official Tauri/sysinfo documentation

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days — stable dependencies; no fast-moving packages)
