---
phase: 14
slug: foundation-frontend-scaffold
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-12
---

# Phase 14 — UI Design Contract

> Visual and interaction contract for Phase 14 (Foundation -- Frontend Scaffold). This phase is a **toolchain skeleton** with minimal visual surface: Vite 6 + Vue 3.5 + TypeScript + Vuetify 3.5+ scaffolding, 9 Pinia stores (skeleton), vue-router Hash routing, tauriInvoke() wrapper, ts-rs type export, and integration test stub replacement. The actual visual UI components (AppHeader, SettingsDrawer, HomeView layout, VideoTable, LogPanel) are in Phases 15-17.

---

## Design System

| Property | Value | Source |
|----------|-------|--------|
| Tool | none | Vuetify 3, not shadcn |
| Component library | Vuetify 3.5+ | D-14 (CONTEXT.md), SmartEdit reference |
| Icon library | @mdi/font 7.4+ | D-14, SmartEdit package.json |
| Font | Roboto + fallback (via Vuetify zhHans locale) | Vuetify 3 default, SmartEdit reference |
| Locale | zhHans (Chinese Simplified) | FNDT-02, SmartEdit vuetify.ts |

**Path alias**: `@/` maps to `src/` -- D-02 (CONTEXT.md), SmartEdit tsconfig.json / vite.config.ts.

**Entry chain**: `main.ts` -> `App.vue` -> `layouts/DefaultLayout.vue` -> `router-view` -- D-03 (CONTEXT.md), matches SmartEdit pattern.

---

## Spacing Scale

Vuetify 3 Material Design 8px grid. Vuetify spacing utility classes (`pa-N`, `ma-N`, `gap-N`) use 4px increments natively. Declared scale:

| Token | Value | Vuetify class | Usage |
|-------|-------|---------------|-------|
| xs | 4px | `pa-1` / `ma-1` | Icon gaps, inline padding, dense form elements |
| sm | 8px | `pa-2` / `ma-2` | Compact element spacing, list item padding |
| md | 16px | `pa-4` / `ma-4` | Default element spacing, card padding |
| lg | 24px | `pa-6` / `ma-6` | Section padding, container margins |
| xl | 32px | `pa-8` / `ma-8` | Layout gaps between major sections |
| 2xl | 48px | `pa-12` / `ma-12` | Major section breaks, page padding |
| 3xl | 64px | `pa-16` / `ma-16` | Page-level spacing, outermost gutters |

**Exceptions**: None for Phase 14. This phase produces only:
- `v-app-bar` (Vuetify default 64px height)
- `v-main` content area
- `v-navigation-drawer` (Vuetify default 300px width, if used in DefaultLayout)

Phase 15 will define layout-specific exception tokens (e.g., SettingsDrawer width 420px).

---

## Typography

Using Vuetify 3 Material Design 3 type scale defaults. The zhHans locale loads system Chinese font family as fallback alongside Roboto.

| Role | Vuetify class | Size | Weight | Line Height | Usage |
|------|---------------|------|--------|-------------|-------|
| Body | `text-body-2` | 14px | 400 (regular) | 1.5 | Default body text, form labels, table content |
| Label | `text-subtitle-2` | 14px | 500 (medium) | 1.2 | Section labels, list item titles, button labels |
| Heading | `text-h6` | 20px | 600 (semibold) | 1.2 | Section headings, card titles, dialog titles |
| Display | `text-h5` | 24px | 600 (semibold) | 1.2 | Page title, hero text, major section headers |

**Notes:**
- Phase 14 uses Vuetify text classes directly (`class="text-body-2"`, `class="text-h5"`) rather than custom CSS tokens.
- Phase 15 may add custom typography tokens if Vuetify defaults do not match the SmartEdit layout mock.
- Chinese text renders correctly at all declared sizes with default system font stack (PingFang SC / Microsoft YaHei / Noto Sans SC fallback).
- Font weight 500 (medium) is used for labels to maintain readability in Chinese UI.

---

## Color

Colors derived from SmartEdit reference project (`src/plugins/vuetify.ts`), which is the stated visual reference for v2.0.

### Light Theme

| Role | Hex | Vuetify token | Usage |
|------|-----|---------------|-------|
| Dominant (60%) | `#F8FAFC` | `background` | Page background, app background |
| Secondary (30%) | `#FFFFFF` | `surface` | Cards, dialogs, drawers, elevated surfaces |
| Secondary variant | `#F1F5F9` | `surface-variant` | Sidebar, nav drawer, list hover background |
| Accent (10%) | `#2563EB` | `primary` | Interactive elements (see accent reserved-for list) |
| Destructive | `#DC2626` | `error` | Destructive actions only |
| On-dominant | `#334155` | `on-background` | Text on background surfaces |
| On-secondary | `#1E293B` | `on-surface` | Text on surface surfaces |
| Informational | `#0284C7` | `info` | Status info, informational banners |
| Success | `#16A34A` | `success` | Success states, completion indicators |
| Warning | `#D97706` | `warning` | Warning states, caution indicators |

### Dark Theme

| Role | Hex | Vuetify token | Usage |
|------|-----|---------------|-------|
| Dominant (60%) | `#0F172A` | `background` | Page background |
| Secondary (30%) | `#1E293B` | `surface` | Cards, dialogs, drawers |
| Secondary variant | `#334155` | `surface-variant` | Sidebar, nav drawer |
| Accent (10%) | `#3B82F6` | `primary` | Interactive elements (see accent reserved-for list) |
| Destructive | `#EF4444` | `error` | Destructive actions only |
| On-dominant | `#E2E8F0` | `on-background` | Text on background surfaces |
| On-secondary | `#F1F5F9` | `on-surface` | Text on surface surfaces |
| Informational | `#0EA5E9` | `info` | Status info |
| Success | `#22C55E` | `success` | Success states |
| Warning | `#F59E0B` | `warning` | Warning states |

### Accent Reserved For

Accent (primary) color is reserved **exclusively** for:

1. **Primary action buttons** -- `v-btn` with `color="primary"` variant (e.g., "Start Pipeline", "Save Settings")
2. **Active/selected state** -- `v-tab--active`, `v-list-item--active`, selected chip outline
3. **Focused form controls** -- `v-field` focus ring, `v-switch` active track
4. **Clickable link text** -- `v-breadcrumbs`, inline router-links, hyperlink-style text
5. **Progress indicators** -- `v-progress-linear` track, `v-progress-circular` stroke
6. **Status badge accent** -- Online/active status dots

**Not for:** Background fills (use `surface-variant`), decorative elements (use `on-surface` muted), non-interactive headers (use `on-background`).

---

## Copywriting Contract

### App Identity

| Element | Copy | Notes |
|---------|------|-------|
| App title | NarratoAI | Brand name, appears in AppHeader (Phase 15) and window title |
| App subtitle | AI 视频解说与自动化剪辑工具 | Informational subtitle for placeholder |

### HomeView Placeholder (Phase 14)

The HomeView in Phase 14 is a **placeholder view** showing welcome content. Phase 15 replaces this with the full layout.

| Element | Copy | Notes |
|---------|------|-------|
| Page heading | 欢迎使用 NarratoAI | Centered welcome heading |
| Page body | AI 驱动的视频解说与自动化剪辑工具。选择工作模式并配置参数后，即可开始制作。 | Brief description of app purpose |
| Secondary text | 请从右上角设置菜单开始配置。 | Guidance for next step |
| Mode hint | 支持纪录片解说、短剧解说和短剧混剪三种模式。 | Mode overview for new users |

### NotFoundView (404)

| Element | Copy | Notes |
|---------|------|-------|
| Error code | 404 | Large display numeral |
| Heading | 页面未找到 | Primary error message |
| Body | 您访问的页面不存在，可能已被移除或地址输入有误。 | Problem description |
| Action link | 返回首页 | `router-link` pointing to `/` |
| Action copy | 返回首页 | Clickable navigation hint |

### Destructive Actions

**None in Phase 14.** This phase has no destructive operations (no delete, no reset, no stop). Phase 17 introduces "Stop Pipeline" and "Clear Video List" with confirmation dialogs.

### No-Data / Empty States

Not applicable for Phase 14. The HomeView placeholder always shows welcome content (not data-bound). Phase 15+ defines empty states for VideoTable, LogPanel, etc.

### Error Handling

| Scenario | Copy | Display |
|----------|------|---------|
| 404 route | 页面未找到。请检查地址或返回首页。 | NotFoundView full page |
| Tauri invoke error | Application error: {message} | Thrown from `tauriInvoke()`, caught by caller (Phase 15+ defines UI error display) |

---

## Registry Safety

**Not applicable.** Phase 14 uses Vuetify 3.5+ (npm package), not shadcn. No third-party component registries.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| npm (Vuetify 3.5+) | Full library | npm provenance, published package |
| npm (@mdi/font) | Icon font | npm provenance, published package |
| npm (pinia, vue-router, etc.) | State, routing | npm provenance, published package |

---

## Interaction Contracts for Phase 14

Since Phase 14 is a scaffolding phase, interaction contracts are minimal:

### Router Navigation

| Route | View | Guard | Behavior |
|-------|------|-------|----------|
| `/` | `HomeView.vue` | None | Render welcome placeholder content |
| `/:pathMatch(.*)*` | `NotFoundView.vue` | Vue Router catch-all | Render 404 page with "返回首页" link |
| Hash mode | All routes | `createWebHashHistory()` | Compatible with Tauri `file://` protocol |

### Theme Persistence

| Interaction | Behavior | Implementation |
|-------------|----------|----------------|
| Theme toggle | Switch `app.theme` between `'light'` and `'dark'` | AppStore toggleTheme action (Phase 14 skeleton) |
| Persistence | Read from `localStorage.getItem('theme')` on init | AppStore initial state (Phase 14 skeleton) |
| Write | `localStorage.setItem('theme', theme)` on toggle | AppStore toggleTheme action (Phase 14 skeleton) |

Phase 14 stores must support these interactions; the actual toggle UI button arrives in Phase 15 (AppHeader).

### Pinia Store Interface Contracts

Each store skeleton in Phase 14 must export the following minimal contract:

| Store | State | Basic Actions | Consumed By |
|-------|-------|---------------|-------------|
| `app` | `theme: 'light' | 'dark'` | `toggleTheme()`, `setTheme(val)` | AppHeader (Ph15), SettingsDrawer (Ph15) |
| `mode` | `current: 'documentary' | 'sde' | 'sdp'` | `setMode(val)`, `resetPipeline()` | ModeSelector (Ph15), Start button (Ph17) |
| `video` | `videos: VideoMeta[]` | `addVideo(v)`, `removeVideo(id)`, `clear()` | VideoTable (Ph17) |
| `pipeline` | `status: 'idle' | 'running' | 'error'`, `progress: ProgressPayload | null` | `start()`, `stop()`, `reset()`, `updateProgress(p)` | VideoTable (Ph17), LogPanel (Ph17) |
| `script` | `content: string | null` | `load(id)`, `save()`, `validate()`, `update(text)` | ScriptPreviewDialog (Ph17) |
| `llm` | `visionConfig`, `textConfig` (provider/model/apiKey/baseUrl) | `updateVisionConfig(c)`, `updateTextConfig(c)` | LLM Panel (Ph16) |
| `tts` | `engine: string`, `engineConfigs: Record<string, any>` | `setEngine(e)`, `updateConfig(e, c)` | TTS Panel (Ph16) |
| `bgm` | `folder: string`, `mode: 'random' | '指定'` | `setFolder(p)`, `setMode(m)` | BGM Panel (Ph16) |
| `export` | `outputDir: string`, `format: string` | `setOutputDir(p)`, `setFormat(f)` | Export Panel (Ph16) |

---

## Phase 14 Visual Assets (Minimal)

This phase creates only 4 Vue files with visual output:

| File | Visual Content | States |
|------|---------------|--------|
| `App.vue` | `<router-view />` only | No visual state |
| `DefaultLayout.vue` | `v-app-bar` (app title), `v-main` (router-view slot), optional `v-navigation-drawer` wrapper | 1 state (loaded) |
| `HomeView.vue` | Welcome heading, body text, mode description | 1 state (loaded) |
| `NotFoundView.vue` | 404 error code, error message, "返回首页" link | 1 state (loaded) |

**No empty states, loading states, error states, or interactive states** needed in Phase 14. These arrive in Phases 15-17.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
