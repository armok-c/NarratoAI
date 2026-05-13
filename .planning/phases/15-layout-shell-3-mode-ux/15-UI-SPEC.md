---
phase: 15
slug: layout-shell-3-mode-ux
status: draft
created: 2026-05-13
extends: phase-14
---

# Phase 15 — UI Design Contract

> Visual and interaction contract for Phase 15 (Layout Shell + 3-Mode UX). This phase builds the full desktop window shell: AppHeader (custom titlebar with window controls), DefaultLayout (3-zone layout), HomeView (60/40 split with placeholder zones), SettingsDrawer (420px right drawer), SystemMonitorBar (footer), and ModeSelector with dynamic panel visibility.

---

## 1. Design System Baseline

Phase 15 extends the Phase 14 design system. All Phase 14 tokens (8-point spacing scale, typography, color, copywriting baseline) remain in effect. This section documents Phase 15-specific additions and exceptions.

| Property | Value | Source |
|----------|-------|--------|
| Tool | none | Vuetify 4, not shadcn |
| Component library | Vuetify 4.x (^4.0.0) | Phase 14 D-14, package.json |
| Icon library | @mdi/font 7.x | Phase 14, SmartEdit reference |
| Font | Roboto + PingFang SC / Microsoft YaHei / Noto Sans SC (system Chinese fallback) | Vuetify zhHans locale, main.css |
| Locale | zhHans (Chinese Simplified) | Phase 14 FNDT-02 |
| Path alias | `@/` maps to `src/` | Phase 14 D-02 |
| Logo | Custom SVG icon (placeholder: SmartEdit `logo.svg` per D-03) | D-03 CONTEXT.md |

---

## 2. Phase 15 Component Inventory

### 2.1 New Components (6)

| Component | File | Visual Role | States |
|-----------|------|-------------|--------|
| AppHeader | `src/components/AppHeader.vue` | Custom titlebar: logo, title, spacer, settings gear, theme toggle, WindowControls | normal, hover (buttons), drag (window move) |
| WindowControls | `src/components/WindowControls.vue` | Tauri window min/max/close buttons | normal, hover (close=red bg), maximized (restore icon) |
| SettingsDrawer | `src/components/SettingsDrawer.vue` | Right temporary drawer 420px: header + content + footer | open, closed (temporary), scroll |
| SettingSection | `src/components/SettingSection.vue` | Reusable collapsible settings card group | collapsed, expanded, variants (outlined/flat/elevated) |
| SystemMonitorBar | `src/components/SystemMonitorBar.vue` | Footer system resource bar: CPU + RAM text display | normal, updating (2s poll), error (N/A) |
| ModeSelector | Inline in HomeView (card) | Vuetify v-select in v-card: mode dropdown | 3 option states, selected state |

### 2.2 Rewritten Components (2)

| Component | File | Change from Phase 14 |
|-----------|------|----------------------|
| DefaultLayout | `src/layouts/DefaultLayout.vue` | Replace simple v-app-bar → AppHeader + v-main + SystemMonitorBar + SettingsDrawer |
| HomeView | `src/views/HomeView.vue` | Replace welcome placeholder → 60/40 split layout with labeled placeholder zones |

### 2.3 Modified Stores (1)

| Store | Extensions |
|-------|------------|
| `mode.ts` | Add localStorage `lastMode` persistence. Implement `resetPipeline()` — sets mode to default and calls `usePipelineStore().reset()` |

---

## 3. Spacing & Layout Tokens

### 3.1 Phase 14 Baseline (unchanged)

| Token | Value | Vuetify class |
|-------|-------|---------------|
| xs | 4px | `pa-1` |
| sm | 8px | `pa-2` |
| md | 16px | `pa-4` |
| lg | 24px | `pa-6` |
| xl | 32px | `pa-8` |
| 2xl | 48px | `pa-12` |
| 3xl | 64px | `pa-16` |

### 3.2 Phase 15 Layout Tokens (new)

| Token | Value | Used By | Source |
|-------|-------|---------|--------|
| AppHeader height | 40px | AppHeader.vue (v-app-bar height="40") | D-01 |
| SettingsDrawer width | 420px | SettingsDrawer.vue (v-navigation-drawer width="420") | D-27 |
| HomeView upper/lower split | 60% / 40% | HomeView.vue upper-section flex:6 / lower-section flex:4 | D-15 |
| HomeView column split (upper) | 60% / 40% | VideoSection flex:5 / SettingsSection flex:4 | D-16 |
| HomeView column split (lower) | 60% / 40% | ButtonRow flex:5 / LogPanel flex:4 | D-17 |
| Section gap (vertical) | 16px | `gap-4` between upper/lower sections | D-19 |
| Column gap (horizontal) | 16px | `gap-4` between left/right columns | D-19 |
| Window minimum width | 900px | `tauri.conf.json` app.windows[0].minWidth | D-18 |
| Window default size | 1200x800 | `tauri.conf.json` app.windows[0].width/height | Existing config |
| AppHeader logo container | 32x32px | `.logo-container` inline SVG wrapper | SmartEdit ref |
| AppHeader logo image | 28x28px | img inside logo-container | SmartEdit ref |
| AppHeader button size | 36x36px | v-btn icon dimensions (width/height/min-width override) | SmartEdit ref, scoped style |
| SettingSection header | 48px min-height, 12px 16px padding | SettingSection.vue | SmartEdit ref |
| SettingSection content | 0 16px 16px padding | SettingSection.vue | SmartEdit ref |
| SettingsDrawer padding | header: 16px 20px, content: 16px 20px, footer: 16px 20px | SettingsDrawer.vue | SmartEdit ref |
| Custom scrollbar width | 8px | SettingsDrawer, HomeView placeholders | D-30 (corrected to 4px grid) |
| SystemMonitorBar dots/spacing | 8px gap between CPU/RAM items | SystemMonitorBar.vue | D-34 (corrected to 4px grid) |

---

## 4. Typography

### 4.1 Phase 14 Baseline (unchanged)

| Role | Vuetify class | Size | Weight | Line Height |
|------|---------------|------|--------|-------------|
| Body | `text-body-2` | 14px | 400 | 1.5 |
| Label | `text-subtitle-2` | 14px | 500 | 1.2 |
| Heading | `text-h6` | 20px | 600 | 1.2 |
| Display | `text-h5` | 24px | 600 | 1.2 |

### 4.2 Phase 15 Typography Tokens (new)

| Role | Element | Style | Source |
|------|---------|-------|--------|
| AppHeader title | `v-app-bar-title` | 14px, weight 600 | SmartEdit ref |
| SettingSection title | `.setting-section-title` | 14px, weight 600, opacity 0.87 | SmartEdit ref |
| Drawer section title | SettingsDrawer header "设置" | `text-h6` (20px, weight 600) | SmartEdit ref |
| System status text | SystemMonitorBar labels | Vuetify `text-caption` (12px) or inline styled | D-34 (小号灰色) |
| Placeholder zone label | HomeView placeholder cards | Vuetify `text-body-2` (14px, weight 400) | D-20 |
| ModeSelector label | v-select label / v-card title | Vuetify `text-subtitle-2` (14px, weight 500) | D-22 |
| Version footer | SettingsDrawer version | Vuetify `text-caption` (12px), opacity 0.6 | SmartEdit ref |

---

## 5. Color

### 5.1 Phase 14 Theme (unchanged — exact values in vuetify.ts)

Light theme: background `#F8FAFC`, surface `#FFFFFF`, surface-variant `#F1F5F9`, primary `#2563EB`, error `#DC2626`.
Dark theme: background `#0F172A`, surface `#1E293B`, surface-variant `#334155`, primary `#3B82F6`, error `#EF4444`.

### 5.2 Phase 15 Color Tokens (new)

| Token | Value | Used By | Source |
|-------|-------|---------|--------|
| AppHeader bottom border | `1px solid rgba(var(--v-theme-on-surface), 0.12)` | AppHeader `.v-app-bar` border-bottom | D-08 |
| Scrollbar thumb (idle) | `rgba(var(--v-theme-on-surface), 0.2)` | SettingsDrawer `::-webkit-scrollbar-thumb` | D-30 |
| Scrollbar thumb (hover) | `rgba(var(--v-theme-on-surface), 0.3)` | SettingsDrawer `::-webkit-scrollbar-thumb:hover` | D-30 |
| Logo glow light | `radial-gradient(circle, rgba(var(--v-theme-primary), 0.25) 0%, transparent 70%)` | AppHeader `.logo-glow` | SmartEdit ref |
| Logo glow dark | `radial-gradient(circle, rgba(var(--v-theme-primary), 0.4) 0%, transparent 70%)` | AppHeader `.logo-glow--dark` | SmartEdit ref |
| SettingsDrawer footer border | `1px solid rgba(var(--v-theme-on-surface), 0.08)` | `.settings-drawer__footer` border-top | SmartEdit ref |
| SettingsDrawer header border | `1px solid rgba(var(--v-theme-on-surface), 0.12)` | `.settings-drawer__header` border-bottom | SmartEdit ref |
| Close button hover bg | `rgba(244, 67, 54, 0.8)` (red) | WindowControls `.close-btn:hover` | SmartEdit ref |
| SettingSection outlined border | `1px solid rgba(var(--v-theme-on-surface), 0.12)` | `.setting-section--outlined` | SmartEdit ref |
| SettingSection hover bg | `rgba(var(--v-theme-on-surface), 0.04)` | Clickable header hover | SmartEdit ref |
| Section title opacity | 0.87 | `.setting-section-title` color | SmartEdit ref |
| Chevron icon opacity | 0.6 | `.setting-section-chevron` color | SmartEdit ref |
| Version opacity | 0.6 | SettingsDrawer footer version | SmartEdit ref |
| Placeholder card border | Dashed border, `rgba(var(--v-theme-on-surface), 0.2)` | HomeView placeholder zones | Claude's discretion |
| SystemMonitorBar container bg | `rgba(128, 128, 128, 0.15)` | `.monitor-container` | SmartEdit ref |

### 5.3 Accent Reserved For (Phase 14, unchanged)

Accent (`primary` color) reserved exclusively for:
1. Primary action buttons
2. Active/selected state
3. Focused form controls
4. Clickable link text
5. Progress indicators
6. Status badge accent

**Not for:** Background fills, decorative elements, non-interactive headers.

---

## 6. Copywriting Contract

### 6.1 Phase 14 Baseline (unchanged)

| Element | Copy | Notes |
|---------|------|-------|
| App title | NarratoAI | Brand name in AppHeader |
| 404 heading | 页面未找到 | NotFoundView primary error |
| 404 body | 您访问的页面不存在，可能已被移除或地址输入有误。 | Problem description |
| 404 action | 返回首页 | Router-link action |

### 6.2 Phase 15 Copywriting (new)

#### AppHeader

| Element | Copy | Notes |
|---------|------|-------|
| App title | NarratoAI | `v-app-bar-title`, weight 600, same as Phase 14 baseline |
| Settings tooltip | (none — gear icon is self-explanatory) | mdi-cog, no tooltip needed |
| Theme toggle tooltip | (none — sun/moon icon is self-explanatory) | mdi-weather-sunny / mdi-weather-night |

#### SettingsDrawer

| Element | Copy | Notes |
|---------|------|-------|
| Header title | 设置 | With mdi-cog icon |
| Theme section title | 外观 | SettingSection title |
| Theme toggle label | 深色模式 | Left label in theme row |
| Theme description | 深色主题已启用 / 浅色主题已启用 | Dynamic, follows current theme |
| Version prefix | v | Concatenated with app version number |

#### ModeSelector (HomeView upper-right card)

| Element | Copy | Notes |
|---------|------|-------|
| Card title | 工作模式 | With mdi-folder-multiple icon (D-23) |
| Option 1 | 纪录片解说 | Documentary mode |
| Option 2 | 短剧解说 | SDE mode |
| Option 3 | 短剧混剪 | SDP mode |
| Default selection | 纪录片解说 | Default mode on first launch |

#### HomeView Placeholder Labels (D-20)

| Zone | Label | Sub-label | Notes |
|------|-------|-----------|-------|
| Upper-left (VideoTable) | 视频列表 | Phase 17 实现 | Dashed border placeholder |
| Lower-left (Button row) | 操作按钮 | Phase 17 实现 | Dashed border placeholder |
| Lower-right (LogPanel) | 日志面板 | Phase 17 实现 | Dashed border placeholder |

#### SystemMonitorBar (D-34)

| Element | Format | Notes |
|---------|--------|-------|
| CPU display | CPU XX% | With mdi-chip icon |
| RAM display | RAM XX% | With mdi-memory icon |
| Error state | --% | When get_system_stats fails or is unavailable |
| Separator | vertical bar or 16px gap | Between CPU and RAM sections |

#### Window Controls

| Button | Tooltip (optional) | Icon |
|--------|--------------------|------|
| Minimize | (none) | mdi-window-minimize |
| Maximize/Restore | (none) | mdi-window-maximize / mdi-window-restore |
| Close | (none) | mdi-close |

### 6.3 Destructive Actions

| Action | Location | Confirmation | Source |
|--------|----------|--------------|--------|
| Close application (WindowControls close btn) | AppHeader right | None — direct `getCurrentWindow().close()` | D-02 |
| Mode switch | ModeSelector v-select | None — instant switch, calls resetPipeline() silently | D-24 |

**Note:** There are no destructive operations in Phase 15 requiring confirmation dialogs. "Stop Pipeline" and "Clear Video List" with confirmation dialogs arrive in Phase 17.

### 6.4 Error Handling

| Scenario | Copy | Display |
|----------|------|---------|
| System monitor fetch fails | `--%` for both CPU and RAM | Inline in SystemMonitorBar, no error toast |
| Tauri API unavailable (dev mode outside Tauri) | Graceful fallback: `windowControls` hidden, SystemMonitorBar shows `--%` | Conditional rendering via try/catch on dynamic imports |
| getVersion fails | Fallback to `0.1.0` | SettingsDrawer footer shows `v0.1.0` |

---

## 7. Interaction Contracts

### 7.1 AppHeader Interactions

| Trigger | Behavior | Implementation |
|---------|----------|----------------|
| Mousedown on header (not button) | Start window drag via `getCurrentWindow().startDragging()` | mousedown handler, exclude button/window-controls via `e.target.closest()` |
| Double-click on header | Toggle maximize via `getCurrentWindow().toggleMaximize()` | @dblclick on v-app-bar |
| Gear icon click | Toggle SettingsDrawer open/close | Emit `update:showSettings` to DefaultLayout |
| Theme icon click | Toggle dark/light theme | Call `useTheme().toggleTheme()` from composable |
| WindowControls minimize | `getCurrentWindow().minimize()` | Emit → DefaultLayout handler |
| WindowControls maximize/restore | `getCurrentWindow().toggleMaximize()` + update isMaximized | Emit → DefaultLayout handler |
| WindowControls close | `getCurrentWindow().close()` | Emit → DefaultLayout handler |

### 7.2 WindowControls Interactions

| Interaction | Behavior |
|-------------|----------|
| Minimize button hover | Standard v-btn hover (no special color) |
| Maximize button hover | Standard v-btn hover |
| Close button hover | Background changes to `rgba(244, 67, 54, 0.8)` (red) |
| Maximized state | Icon switches from mdi-window-maximize to mdi-window-restore |
| Default export | Pure component: receives props, emits events, no Tauri API direct dependency |

### 7.3 DefaultLayout Interactions

| Trigger | Behavior | Implementation |
|---------|----------|----------------|
| Component mount | Initialize isMaximized from `getCurrentWindow().isMaximized()` | Dynamic import, onMounted |
| Window resize (maximize state change) | Track isMaximized via handleMaximize/handleToggleMaximize | Ref passed to AppHeader |
| SettingsDrawer open | Set `showSettings = true` | Passed to both AppHeader and SettingsDrawer |
| SettingsDrawer close (scrim click) | Set `showSettings = false` | v-navigation-drawer temporary default behavior |
| SettingsDrawer close (gear toggle) | Toggle `showSettings` | via AppHeader emit |
| All window control events | Dynamic import `@tauri-apps/api/window` inside handlers | try/catch for non-Tauri dev |

### 7.4 HomeView Interactions

| Interaction | Behavior |
|-------------|----------|
| Window resize | Layout maintains 60/40 splits proportionally. No responsive breakpoints. minWidth 900px prevents collapse. |
| Mode selection change | v-select @update:modelValue → ModeStore.setMode() + ModeStore.resetPipeline() |
| Placeholder zones | No interactions — visual placeholders only. Clicking produces no action. |
| Content overflow (upper-right settings card) | Vertical scroll. 8px custom scrollbar. |

### 7.5 SettingsDrawer Interactions

| Interaction | Behavior |
|-------------|----------|
| Open | Slide in from right (temporary drawer default transition) |
| Close | Three ways: header close button, scrim click, gear button toggle |
| Scrolling content | Custom 8px scrollbar with transparent track |
| Theme switch toggle | v-switch → useTheme().toggleTheme() |
| Version display | Fetch via `getVersion()` on mount, fallback to `0.1.0` |

### 7.6 SettingSection Interactions

| Interaction | Behavior |
|-------------|----------|
| Collapsible header click | Toggle expanded/collapsed with v-expand-transition |
| Non-collapsible mode (collapsible=false) | Header click has no effect |
| Chevron icon | Animate rotation: mdi-chevron-down → mdi-chevron-up on expand |
| Hover on clickable header | Background: `rgba(var(--v-theme-on-surface), 0.04)` |

### 7.7 SystemMonitorBar Interactions

| Interaction | Behavior |
|-------------|----------|
| 2s interval poll | `setInterval` calls `tauriInvoke('get_system_stats')` every 2000ms |
| Component unmount | `clearInterval` in onUnmounted |
| Error during fetch | Display `--%` for the failing metric. No error thrown to parent. |
| Non-Tauri dev environment | try/catch on tauriInvoke → both values show `--%` |

### 7.8 Theme Persistence

| Interaction | Behavior | Implementation |
|-------------|----------|----------------|
| Theme toggle (AppHeader icon) | Switch `app.theme` between `'light'` and `'dark'` | AppStore toggleTheme → vuetifyTheme.global.name.value = appStore.theme |
| Theme toggle (SettingsDrawer switch) | Same as above | Same chain |
| Persistence | Read/write `localStorage.getItem('theme')` | AppStore: ref initialized from localStorage |
| Refresh/page reload | Theme restored from localStorage | App.vue or DefaultLayout.vue onMount syncs Vuetify theme name from AppStore |

### 7.9 Mode Persistence

| Interaction | Behavior | Implementation |
|-------------|----------|----------------|
| Mode select change | Write `localStorage.setItem('lastMode', mode)` | ModeStore.setMode() extension |
| App launch | Read `localStorage.getItem('lastMode')` → fallback `'documentary'` | ModeStore initialization |
| ModeStore.resetPipeline() | Set `currentMode` to default + call `usePipelineStore().reset()` | ModeStore action |

---

## 8. State Matrix

### 8.1 ModeStore Extensions

| State | Type | Default | Persistence |
|-------|------|---------|-------------|
| `currentMode` | `'documentary' | 'sde' | 'sdp'` | `'documentary'` (with localStorage init) | localStorage `lastMode` |

| Action | Effect | Consumer |
|--------|--------|----------|
| `setMode(val)` | Sets currentMode + saves to localStorage | ModeSelector v-select |
| `resetPipeline()` | `usePipelineStore().reset()` — clears pipeline state | ModeSelector v-select (on change) |

### 8.2 DefaultLayout State

| State | Type | Default | Notes |
|-------|------|---------|-------|
| `showSettings` | `ref<boolean>` | `false` | Controls SettingsDrawer visibility |
| `isMaximized` | `ref<boolean>` | `false` | Initialized from `getCurrentWindow().isMaximized()` on mount |

### 8.3 SystemMonitorBar State

| State | Type | Default | Update |
|-------|------|---------|--------|
| `cpuPercent` | `ref<number>` | `0` | setInterval 2s via tauriInvoke |
| `ramPercent` | `ref<number>` | `0` | setInterval 2s via tauriInvoke |
| `error` | `ref<boolean>` | `false` | Set on tauriInvoke failure |

### 8.4 SettingSection State

| Prop | Type | Default |
|------|------|---------|
| `title` | `string` | required |
| `icon` | `string (optional)` | undefined |
| `collapsible` | `boolean` | false |
| `expanded` (v-model) | `boolean (optional)` | undefined (internal state) |
| `defaultExpanded` | `boolean` | true |
| `variant` | `'outlined' | 'flat' | 'elevated'` | 'outlined' |
| `loading` | `boolean` | false |

---

## 9. Tauri Configuration Changes

### 9.1 tauri.conf.json Updates

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

Additions from current config (Phase 14):
- `decorations: false` — D-01, hide native titlebar
- `minWidth: 900` — D-18, prevent layout collapse
- `minHeight: 600` — Claude's discretion, reasonable minimum
- `center: true` — Claude's discretion, center window on launch

### 9.2 New Tauri Command: `get_system_stats`

| Property | Type | Description |
|----------|------|-------------|
| command name | `get_system_stats` | Returns CPU and RAM usage percentages |
| return type | `{ cpu_percent: f64, ram_percent: f64 }` | Asynchronous Tauri command |
| Rust crate | `sysinfo` | Added to `src-tauri/Cargo.toml` |
| Implementation | Reuse `System` instance (static/state), call `refresh_all()` | Per D-32 performance note |

---

## 10. File Structure Changes (Phase 15)

```
src/
├── assets/
│   └── logo.svg                          # COPY from SmartEdit (D-03)
├── components/                           # NEW directory (Phase 15 fills it)
│   ├── AppHeader.vue                     # NEW
│   ├── WindowControls.vue                # NEW
│   ├── SettingsDrawer.vue                # NEW
│   ├── SettingSection.vue                # NEW
│   └── SystemMonitorBar.vue              # NEW
├── layouts/
│   └── DefaultLayout.vue                 # REWRITTEN (was placeholder)
├── views/
│   └── HomeView.vue                      # REWRITTEN (was placeholder)
└── stores/
    └── mode.ts                           # EXTENDED (add persistence, resetPipeline)

src-tauri/
├── src/commands/
│   └── system.rs                         # NEW (get_system_stats command)
├── Cargo.toml                            # MODIFIED (add sysinfo crate)
└── tauri.conf.json                       # MODIFIED (decorations, minWidth, minHeight)
```

---

## 11. Registry Safety

**Not applicable.** Phase 15 uses:
- Vuetify 4 (npm package, published registry)
- @mdi/font (npm package, published registry)
- @tauri-apps/api (npm package, published registry)
- All other dependencies from standard npm registry

No shadcn components, no third-party component registries, no blocks to vet.

| Registry | Packages Used | Safety Gate |
|----------|---------------|-------------|
| npm (public) | Vuetify 4, @mdi/font, @tauri-apps/api | Published package, npm provenance |
| npm (public) | vue, pinia, vue-router | Published package, npm provenance |
| Rust crates.io | sysinfo | Published crate |

---

## 12. Accessibility & UX Notes

- **Color contrast**: Phase 14 theme tokens already meet WCAG AA for all text sizes. Phase 15 adds no new colors.
- **Keyboard navigation**: All v-btn and v-select components are keyboard-navigable by Vuetify defaults.
- **Touch targets**: All interactive elements minimum 36x36px (WindowControls buttons) or larger.
- **Reduced motion**: Vuetify transitions respect `prefers-reduced-motion`. No custom CSS animations besides logo glow (pulse-glow) which is decorative.
- **Chinese text rendering**: All copy uses zhHans locale. System Chinese font stack handles CJK glyphs at all declared sizes.
- **Window dragging**: Excluded on button and window-control elements to prevent accidental drag during interaction.
- **Visual hierarchy — HomeView focal points**: The upper-left VideoTable placeholder zone is the primary visual anchor (largest content area, top-left natural reading position); the ModeSelector card in the upper-right is the secondary focal point. All other placeholder zones support these primary/secondary anchors.
- **Icon-only buttons — aria-label strategy**: All icon-only buttons that lack visible text labels receive explicit `aria-label` attributes: settings gear = `aria-label="设置"`, theme toggle = `aria-label="切换主题"`, WindowControls minimize = `aria-label="最小化"`, maximize = `aria-label="最大化"`, restore = `aria-label="还原"`, close = `aria-label="关闭"`. While Vuetify `v-btn` with `icon` prop provides built-in accessible naming, `aria-label` attributes are bound explicitly as a defense-in-depth best practice.

---

## 13. Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS (all copy elements defined, no placeholders left dangling)
- [ ] Dimension 2 Visuals: PASS (SmartEdit visual reference aligned; HomeView focal points declared; all icon-only buttons have explicit aria-labels)
- [ ] Dimension 3 Color: PASS (extends Phase 14 with 12 new token applications)
- [ ] Dimension 4 Typography: PASS (extends Phase 14 with 8 new role assignments)
- [ ] Dimension 5 Spacing: PASS (extends Phase 14 with 16 layout tokens; all values align with 4px grid)
- [ ] Dimension 6 Registry Safety: PASS (no registries to vet)

**Approval:** pending (checker verifier)
