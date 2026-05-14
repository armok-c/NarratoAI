---
phase: 16
slug: configuration-panels-ffmpeg-sidecar-core-tech-debt
status: draft
tool: none (Vuetify 4)
preset: not applicable
created: 2026-05-13
---

# Phase 16 — UI Design Contract

> Visual and interaction contract for Phase 16 (Configuration Panels + FFmpeg Sidecar + Core Tech Debt). Builds on Phase 14 design system and Phase 15 SettingsDrawer/SettingSection infrastructure. Adds 7 configuration panels, Local Draft persistence, validation UX, and snackbar feedback.

---

## Design System

| Property | Value | Source |
|----------|-------|--------|
| Tool | none | Vuetify 4, not shadcn |
| Component library | Vuetify 4.x | Phase 14 UI-SPEC, `src/plugins/vuetify.ts` |
| Icon library | @mdi/font 7.4+ | Phase 14 UI-SPEC |
| Font | Roboto + Chinese fallback (via Vuetify zhHans locale) | Phase 14 UI-SPEC |
| Locale | zhHans (Chinese Simplified) | Phase 14 UI-SPEC |
| Component framework | Vue 3.5 Composition API + TypeScript 6 | Phase 14 scaffold |
| State management | Pinia 3 (Composition API style) | Phase 14 scaffold |
| Native dialog | @tauri-apps/plugin-dialog for folder selection | Phase 14 RESEARCH.md |
| Shell sidecar | @tauri-apps/plugin-shell (new install) + tauri-plugin-shell (Cargo, new install) | Phase 16 CONTEXT.md D-30 |

---

## 1. Layout & Spacing

### 1.1 Spacing Scale (inherited from Phase 14 UI-SPEC)

All Phase 16 components use the established 8px grid via Vuetify spacing utility classes:

| Token | Value | Vuetify Utility | Usage in Phase 16 |
|-------|-------|-----------------|-------------------|
| xs | 4px | `pa-1` / `ga-1` | Icon-to-text gap in panel headers, inline help icon spacing |
| sm | 8px | `pa-2` / `ga-2` | Compact field spacing within panels, header action button margins |
| md | 16px | `pa-4` / `ga-4` | Panel content padding, field vertical gap, section-to-section gap |
| lg | 24px | `pa-6` / `ga-6` | Drawer content padding from edges, major section breaks |
| xl | 32px | `pa-8` / `ga-8` | Footer top margin, header bottom padding |
| 2xl | 48px | `pa-12` | Not used in Phase 16 panels |
| 3xl | 64px | `pa-16` | Not used in Phase 16 panels |

**Exceptions for Phase 16:**
- **SettingSection header min-height:** 48px (inherited from Phase 15)
- **SettingsDrawer width:** 420px (inherited from Phase 15, established in LOUT-04)
- **SettingsDrawer scrollbar width:** 6px (inherited from Phase 15 D-30)
- **Drawer header/footer divider border:** 1px at `rgba(var(--v-theme-on-surface), 0.12)` (header) and `0.08` (footer) — inherited from Phase 15

### 1.2 SettingsDrawer Content Layout

The drawer Content area is reorganized from a single SettingSection into a vertical stack of 7 collapsing SettingSection panels:

```
┌─────────────────────────────────┐
│ Header: [icon] 设置        [X] │  ← flex-shrink: 0, border-bottom
├─────────────────────────────────┤
│ Content (scrollable)            │  ← flex: 1, overflow-y: auto
│                                 │
│ ┌─ LLM Vision ────────────[▼]─┐│  ← SettingSection, outlined variant
│ │ Provider [v-select]         ││     header-actions: "重置" btn
│ │ Model     [v-text-field]    ││     default expanded: true
│ │ API Key   [password + show] ││     gap between fields: 12px
│ │ Base URL  [v-text-field]    ││
│ │ [测试连接] btn              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ LLM Text ─────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ TTS ──────────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ BGM ──────────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ Export ───────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ 网络代理 ────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││
│ └─────────────────────────────┘│
│  gap: 12px                      │
│ ┌─ 模式参数 ────────────[▼]─┐│  ← default expanded: false
│ │ ... fields ...              ││  title/content changes per mode
│ └─────────────────────────────┘│
│                                 │
├─────────────────────────────────┤
│ Footer                          │  ← flex-shrink: 0
│ [保存设置] btn (primary,block)  │     border-top divider
│ 重置全部 (link text, center)    │
│ vX.X.X (version, caption)       │
└─────────────────────────────────┘
```

**Panel-to-panel gap:** 12px (wa-3 equivalent, recommended for dense settings layouts).

**Panel content padding (within SettingSection):**
- Header: 12px 16px (inherited from Phase 15 SettingSection)
- Body: 0 16px 16px 16px (inherited from Phase 15 SettingSection)
- Field vertical spacing within a panel: 12px between adjacent fields

### 1.3 Field Layout Within Panels

Each field within a panel uses full width (no inline/grid layout):

```
┌──────────────────────────────────────┐
│  Label                                │
│  [v-text-field / v-select / etc.]    │  ← full width, variant="outlined"
│                                       │     density="compact"
│  ┌──────────────────────────────────┐ │
│  │ help text or tooltip indicator   │ │  ← text-caption or mdi-help-circle-outline
│  └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

**Exceptions:**
- BGM folder path: `[v-text-field (readonly)] [v-btn "浏览"]` — inline flex row, field takes remaining width
- Export output directory: Same pattern as BGM
- BGM random/specified mode: v-switch + conditional v-select — inline layout
- Network proxy enable/disable: v-switch at top, fields below (grayed when disabled)
- Mode param dynamic fields: mode-dependent field set, always full-width stacked

### 1.4 Mode-Based Panel Visibility

| Panel | Documentary | SDE | SDP |
|-------|-------------|-----|-----|
| LLM Vision | Visible | Hidden (v-if) | Hidden (v-if) |
| LLM Text | Visible | Visible | Visible |
| TTS | Visible | Visible | Hidden (v-if) |
| BGM | Visible | Visible | Visible |
| Export | Visible | Visible | Visible |
| Network Proxy | Visible | Visible | Visible |
| Mode Params | Visible ("帧参数") | Visible ("短剧参数") | Visible ("混剪参数") |

**Visibility mechanism:** `v-if="showVisionPanel"` computed from `useModeStore().currentMode` — hidden panels are fully removed from DOM, not just hidden (per D-02 specificity). Transition via `<Transition>` wrapping the panel group is NOT needed; each SettingSection's `v-expand-transition` handles its own collapse animation when re-mounted.

### 1.5 FFmpeg Sidecar Binary Layout (non-UI)

Directory structure (influences bundle layout, not visual UI):

```
src-tauri/binaries/
  ffmpeg-x86_64-pc-windows-msvc.exe
  ffprobe-x86_64-pc-windows-msvc.exe
  ffmpeg-aarch64-apple-darwin
  ffprobe-aarch64-apple-darwin
  ffmpeg-x86_64-unknown-linux-gnu
  ffprobe-x86_64-unknown-linux-gnu
```

No FFmpeg management UI. This is behind-the-scenes packaging.

---

## 2. Typography & Copywriting

### 2.1 Typography Scale (inherited from Phase 14 UI-SPEC)

| Role | Vuetify Class | Size | Weight | Line Height | Usage in Phase 16 |
|------|---------------|------|--------|-------------|-------------------|
| Body | `text-body-2` | 14px | 400 (regular) | 1.5 | Field labels, section descriptions, snackbar messages |
| Label | `text-subtitle-2` | 14px | 500 (medium) | 1.2 | Panel section titles (SettingSection title), field group headers |
| Heading | `text-h6` | 20px | 600 (semibold) | 1.2 | SettingsDrawer header title ("设置"), panel headings |
| Display | `text-h5` | 24px | 600 (semibold) | 1.2 | Not used in Phase 16 panels |
| Caption | `text-caption` | 12px | 400 (regular) | 1.5 | Version text, helper descriptions, field hint text |
| Button | `text-button` | 14px | 500 (medium) | — | Save button, reset button, test connection button |

**Phase 16 specific typography notes:**
- **Provider recommended model hint:** `text-caption text-medium-emphasis` — 12px muted text below Model field, shown only when a preset Provider is selected
- **Validation error icon badge:** No custom text; use v-badge with `color="error"` dot on the SettingSection title area, number of issues in the badge (`color="error" variant="dot"` for single issue, or `color="error"` with content for multiple)
- **"未保存的更改" indicator:** `text-body-2 text-warning` — orange-colored text next to save button
- **Field placeholders:** `text-body-2` at `text-disabled` opacity — all placeholder text in Chinese

### 2.2 Copywriting Contract

#### Global CTAs

| Element | Copy | Location | Action |
|---------|------|----------|--------|
| Save button | 保存设置 | SettingsDrawer Footer, primary elevated v-btn full-width | Calls `save_config` Tauri command |
| Reset per-panel | 重置 | Each SettingSection header-actions slot | Restores that panel's fields to defaults from `get_config()` |
| Reset all | 重置全部 | SettingsDrawer Footer, link-style text below save button | Opens confirmation dialog, clears localStorage draft + resets all Pinia stores |
| Test connection | 测试连接 | Each LLM panel footer, outlined v-btn | Calls `test_llm_connection` Tauri command |
| Browse folder | 浏览 | BGM/Export panels, inline v-btn next to readonly path field | Opens `@tauri-apps/plugin-dialog` `open({directory: true})` |
| Show/hide API key | (eye icon toggle) | API Key/Secret fields, v-text-field `append-inner` icon | Toggles `type="password"` / `type="text"` |

#### Confirmation Dialogs

| Scenario | Dialog Title | Dialog Body | Actions |
|----------|-------------|-------------|---------|
| Mode switch with unsaved changes | — | 未保存的更改将丢失，是否继续？ | [取消] [继续] |
| Reset all confirmation | — | 确认重置全部设置？这将清除所有未保存的更改。 | [取消] [确认重置] |
| External config conflict | — | 配置文件已被外部修改，是否覆盖？ | [取消] [覆盖] |
| Loading config failure | — | 配置加载失败 | Snackbar with [重试] action button |

#### Empty States

| Scenario | Copy | Display |
|----------|------|---------|
| Edge-TTS voices not loaded | 音色列表加载中... | v-autocomplete loading state |
| Edge-TTS voices load failed | 音色列表加载失败，请检查网络或代理设置 | Error alert below engine selector with [重试] button |
| BGM folder not selected | 尚未选择 BGM 文件夹 | v-text-field shows placeholder "点击浏览选择 BGM 文件夹" |
| Export directory not selected | 尚未选择输出目录 | v-text-field shows placeholder "点击浏览选择输出目录" |
| Provider "自定义" selected | 请输入 Provider 名称和 Base URL | Model field placeholder shows "输入自定义模型名称"，no recommended model hint shown |
| Mode params (first load) | 读取模式参数中... | SettingSection loading skeleton shown |

#### Error States

| Scenario | Copy | Display |
|----------|------|---------|
| get_config() Tauri command fails | 配置加载失败，请重试或重启应用 | v-snackbar error, red background, 5s timeout, retry action button |
| save_config() Tauri command fails | 配置保存失败: {error detail} | v-snackbar error, red background, 5s timeout |
| test_llm_connection fails | 连接失败: {error detail} | v-snackbar error, red background, 5s timeout |
| Validation errors on save | {N} 项配置不完整 | v-snackbar warning, orange background, 4s timeout. Affected panels get red badge dot on their SettingSection title. |
| Edge-TTS voice list load fails | 语音列表加载失败: {error detail} | Inline error message (text-body-2 text-error) below engine selector + [重试] button |
| Edge-TTS proxy connection fails | (silent) | No frontend feedback; `tracing::warn` in Rust backend only (D-27) |
| LocalStorage full (draft save) | (silent) | No user-facing feedback (D-24: silent degradation) |
| LocalStorage draft restore fails | (silent) | No user-facing feedback (D-24: silent skip) |

#### Success States

| Scenario | Copy | Display |
|----------|------|---------|
| save_config() succeeds | 设置已保存 | v-snackbar success, green background, 3s timeout |
| test_llm_connection succeeds | 连接成功 | v-snackbar success, green background, 3s timeout |
| Edge-TTS voices loaded | (no message) | v-autocomplete populates with voice list, loading state removed |

#### Destructive Actions

| Action | Confirmation Required | Copy |
|--------|----------------------|------|
| Reset all settings | Yes (v-dialog confirm) | 确认重置全部设置？这将清除所有未保存的更改。 [取消] [确认重置] |
| Switch mode with unsaved changes | Yes (v-dialog confirm) | 未保存的更改将丢失，是否继续？ [取消] [继续] |
| Overwrite externally modified config | Yes (v-dialog confirm) | 配置文件已被外部修改，是否覆盖？ [取消] [覆盖] |

**No direct destructive actions in Phase 16** (no delete, no stop, no clear) — those arrive in Phase 17.

#### Panel-Specific Copy

**LLM Vision Panel:**
- Title: "视觉模型"
- Icon: `mdi-robot`
- Fields: Provider (v-select, label: "提供商"), Model (v-text-field, label: "模型"), API Key (password, label: "API 密钥"), Base URL (v-text-field, label: "接口地址", placeholder: "留空使用默认")
- Test button: "测试连接"

**LLM Text Panel:**
- Title: "文本模型"
- Icon: `mdi-chat-processing`
- Fields: Same structure as LLM Vision
- Test button: "测试连接"

**TTS Panel:**
- Title: "语音合成"
- Icon: `mdi-text-to-speech`
- Engine selector: v-select, label: "TTS 引擎", options: Edge-TTS / Azure / 腾讯云 / SoulVoice / 阿里通义 / IndexTTS2 / 豆包
- Dynamic engine params per engine (see Section 3)

**BGM Panel:**
- Title: "背景音乐"
- Icon: `mdi-music-note`
- Fields: Folder path (readonly + browse), Mode switch (随机 / 指定)
- Mode Params: When "指定", v-select listing audio files in folder

**Export Panel:**
- Title: "导出设置"
- Icon: `mdi-file-export`
- Fields: Output directory (readonly + browse), Format (v-select: MP4 / MKV / MOV)

**Network Proxy Panel:**
- Title: "网络代理"
- Icon: `mdi-earth`
- Fields: Enable switch (v-switch "启用代理"), HTTP proxy address (v-text-field, placeholder: "留空使用环境变量"), HTTPS proxy address (v-text-field, placeholder: "留空使用环境变量")

**Mode Params Panel:**
- Title: Dynamic per mode (Documentary: "帧参数", SDE: "短剧参数", SDP: "混剪参数")
- Icon: `mdi-tune`
- Documentary fields: 帧间隔 (v-text-field, number, label: "帧间隔"), 视觉批处理大小 (v-text-field, number, label: "视觉批处理大小")
- SDE fields: 剧名 (v-text-field, label: "剧名"), 温度 (v-slider 0~2, label: "温度")
- SDP fields: 自定义片段数 (v-text-field, number, label: "自定义片段数"), 最短时长 (v-text-field, number, label: "最短时长(秒)"), 最长时长 (v-text-field, number, label: "最长时长(秒)")

---

## 3. Color & Theming

### 3.1 Color Tokens (inherited from Phase 14 UI-SPEC, verified against `src/plugins/vuetify.ts`)

All Phase 14 color tokens carry forward unchanged. No new color tokens are introduced.

#### Light Theme

| Role | Hex | Vuetify Token | Phase 16 Usage |
|------|-----|---------------|----------------|
| Dominant (60%) | `#F8FAFC` | `background` | Drawer outer background (via v-navigation-drawer default) |
| Secondary (30%) | `#FFFFFF` | `surface` | SettingSection card backgrounds, v-text-field input background |
| Secondary variant | `#F1F5F9` | `surface-variant` | Panel header hover background, disabled field backgrounds |
| Accent (10%) | `#2563EB` | `primary` | "保存设置" button, active panel indicator, focus rings |
| Destructive | `#DC2626` | `error` | Validation red dot badge, test_llm_connection failure snackbar |
| Success | `#16A34A` | `success` | Save success snackbar, test_llm_connection success snackbar |
| Warning | `#D97706` | `warning` | "未保存的更改" indicator text, partial validation warning badge |
| Informational | `#0284C7` | `info` | Help tooltip icons, informational snackbar |
| On-dominant | `#334155` | `on-background` | Drawer content text |
| On-secondary | `#1E293B` | `on-surface` | Panel header titles, field labels, input text |
| Muted | (derived) | `text-medium-emphasis` | Helper text, placeholder text, version number |

#### Dark Theme

| Role | Hex | Vuetify Token | Phase 16 Usage |
|------|-----|---------------|----------------|
| Dominant (60%) | `#0F172A` | `background` | Drawer background |
| Secondary (30%) | `#1E293B` | `surface` | SettingSection card backgrounds |
| Secondary variant | `#334155` | `surface-variant` | Panel header hover background, disabled fields |
| Accent (10%) | `#3B82F6` | `primary` | Same as light — primary actions |
| Destructive | `#EF4444` | `error` | Same roles as light |
| Success | `#22C55E` | `success` | Same roles as light |
| Warning | `#F59E0B` | `warning` | Same roles as light |
| Informational | `#0EA5E9` | `info` | Same roles as light |
| On-dominant | `#E2E8F0` | `on-background` | Drawer content text |
| On-secondary | `#F1F5F9` | `on-surface` | Panel header titles, field labels |

### 3.2 Semantic Color Contracts for Phase 16

| Semantic | Color | Vuetify Token | Target |
|----------|-------|---------------|--------|
| Save success snackbar | Green | `success` | Background of v-snackbar |
| Save failure snackbar | Red | `error` | Background of v-snackbar |
| Validation warning badge | Red | `error` | v-badge dot on SettingSection title area |
| Unsaved changes text | Orange | `warning` | `<span class="text-warning">` next to save button |
| Test connection success | Green | `success` | v-snackbar background |
| Test connection failure | Red | `error` | v-snackbar background |
| Help/description icons | Muted blue | `text-info` | mdi-help-circle-outline icon fill |
| Disabled proxy fields | Inherited | `v-field--disabled` | Vuetify's built-in disabled styling at reduced opacity |
| Special characters | — | `text-medium-emphasis` | All field placeholders and helper text |
| Loading skeleton | Inherited | `v-skeleton-loader` default | Vuetify's built-in skeleton styling with panel variant |

### 3.3 Accent Reserved-For (extended from Phase 14 for Phase 16)

Accent (primary) color remains reserved exclusively for:

1. **Global save button** (`color="primary"` elevated v-btn in Footer) — only primary button in the drawer
2. **Active/focused form controls** — v-text-field focus ring, v-select focused highlight, v-switch active track
3. **Save button loading state** — v-progress-linear indeterminate bar during save
4. **Panel header active state** — Not used (panels use outlined variant, no active indicator)
5. **Test connection button** — Not primary; uses `variant="outlined"` with no explicit color (inherits on-surface)

**Primary color is NOT used for:**
- Per-panel reset buttons (use `variant="text" size="small"` with no color)
- Browse folder buttons (use `variant="outlined"` with no color)
- Panel header icons (inherits on-surface text color)
- Field labels and helper text (use on-surface / text-medium-emphasis)
- Version text (use text-caption at reduced opacity)

---

## 4. Component States (Loading, Empty, Error, Edge Cases)

### 4.1 SettingSection Loading State

**When it shows:** During initial `get_config()` load from Tauri backend (D-32). Per-panel loading state controlled by `loading` prop on `SettingSection`.

**Visual:**
- Vuetify `v-skeleton-loader` with `type="paragraph"` inside SettingSection content area (matching SmartEdit SettingSection implementation)
- SettingSection header remains visible with title and icon
- Chevron icon rendered but section is collapsed during loading (user can expand to see skeleton)
- Loading state duration: user-perceptible (>=300ms). If load completes in <300ms, skeleton is suppressed to avoid flash.

**Transition:** After config loads, skeleton fades out and real fields appear via Vue reactivity. No explicit fade animation needed (Vue handles DOM swap).

**Per-panel load conditions:**
- All panels load simultaneously from single `get_config()` call
- Panel loading state = `configStore.loading` (shared)
- After first successful load, subsequent SettingsDrawer opens do not re-show skeletons (Pinia cache from D-32 preload)

### 4.2 Empty States Per Panel

| Panel | Empty Condition | Visual |
|-------|----------------|--------|
| LLM Vision | Provider empty after load | v-select shows placeholder "选择提供商" |
| LLM Text | Provider empty after load | Same as Vision |
| TTS | No engine selected after load | v-select shows placeholder "选择 TTS 引擎" |
| TTS Edge-TTS voices | Voice list not yet loaded | v-autocomplete shows loading spinner, no results |
| BGM | No folder selected | v-text-field shows placeholder, path area empty |
| Export | No output directory selected | Same as BGM |
| Network Proxy | No proxy addresses | Fields empty, placeholder visible |
| Mode Params | No params loaded | Fields show placeholder or default values |

### 4.3 Error States Per Panel

| Panel | Error Condition | Behavior |
|-------|----------------|----------|
| All panels | `get_config()` fails | Snackbar "配置加载失败，请重试或重启应用" + retry button. Code defensively shows empty/default fields so panel remains operable. |
| LLM Vision/Text | `test_llm_connection` fails | Red snackbar with error detail. No field-level error indication. |
| TTS | `get_edge_tts_voices` fails | Inline error text below engine selector + [重试] button. Snakebar NOT shown (inline is less intrusive). |
| BGM | Folder dialog cancelled | No action taken. Folder path unchanged. |
| Export | Folder dialog cancelled | Same as BGM. |
| Any panel | `save_config` fails with external config conflict | Warning dialog: "配置文件已被外部修改，是否覆盖？" |
| Any panel | `save_config` fails with validation error | Snackbar "{N} 项配置不完整" + red badges on affected panel titles |
| Any panel | `save_config` fails with network/server error | Red snackbar with error message |

### 4.4 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| User opens SettingsDrawer while `get_config()` still loading | All panels show skeleton loaders. Save button disabled. |
| API Key returned from backend is null/empty | Field shows empty (no filler text). Type="password" toggle works. |
| User clicks "保存设置" with no changes made | Save button disabled via `:disabled="!hasChanges"`. Button not clickable. |
| User changes a field, then navigates to another panel within drawer | Changes retained in Pinia state. No auto-save. |
| User opens SettingsDrawer, changes fields, closes drawer (no save) | Draft auto-saved to localStorage on close (D-20). Draft restored on next SettingsDrawer open. |
| User opens SettingsDrawer, closes, reopens, and saves | Save succeeds, localStorage draft cleared, snackbar success. |
| User has unsaved draft, then mode switch triggers | Confirm dialog: "未保存的更改将丢失，是否继续？" Cancel = abort switch |
| Edge-TTS engine selected but voice list not yet loaded | Engine selector shows selected engine. Voice autocomplete shows loading. |
| User switches TTS engine while fields have values | Engine parameters retained in memory per D-14. Switching back restores previous values. |
| BGM "指定" mode selected but folder not chosen yet | v-select shows "请先选择 BGM 文件夹" placeholder and is disabled. |
| Network proxy fields empty but "启用" switch is ON | Fields show placeholders. Backend uses env vars as fallback. |
| localStorage draft JSON corrupted | Restore silently fails (D-24). App proceeds with backend config. |
| Tauri non-dev mode (production build) | Dynamic imports still work. `getVersion()` in dev fallback shows "v0.1.0". All `@tauri-apps/*` imports wrapped in try/catch. |
| App start with draft newer than backend config | Draft overlays backend config (D-22 optimistic strategy). Save triggers conflict check. |

---

## 5. Interaction & Animation

### 5.1 Accordion / Collapse Behavior

| Property | Value | Source |
|----------|-------|--------|
| Panel collapse mechanism | Each `SettingSection` manages its own expanded state via `v-expand-transition` | D-01, Phase 15 SettingSection component |
| Multiple open panels allowed | Yes — no single-expand restriction | D-04 |
| Default open panel | Only LLM Vision (index 0, `defaultExpanded: true`), all others folded | D-04 |
| Expand indicator | Chevron icon: `mdi-chevron-down` (collapsed) / `mdi-chevron-up` (expanded), 20px | Phase 15 SettingSection |
| Expand transition | `v-expand-transition` (Vuetify built-in, CSS height animation) | Phase 15 SettingSection |
| Chevron rotation | CSS `transition: transform 0.2s ease` on `.setting-section-chevron` | Phase 15 SettingSection |
| Header click area | Entire header row, `cursor: pointer`, hover background `rgba(on-surface, 0.04)` | Phase 15 SettingSection |
| Panel persistence | Collapse state saved to `localStorage('settingsPanelStates')` | D-04 |

**localStorage schema for panel states:**
```typescript
// localStorage key: 'settingsPanelStates'
// Value type:
interface SettingsPanelStates {
  llmVision: boolean    // default: true
  llmText: boolean      // default: false
  tts: boolean          // default: false
  bgm: boolean          // default: false
  export: boolean       // default: false
  networkProxy: boolean // default: false
  modeParams: boolean   // default: false
}
```

**Panel hide/show animation:** No explicit animation on panel DOM insertion/removal. When mode changes and `v-if` condition flips, SettingSection either mounts (in its default collapsed state) or unmounts (instantly removed). The `v-expand-transition` within only animates the panel body expand/collapse, not the mounting/unmounting.

### 5.2 Save Flow

```
User clicks "保存设置"
  → Save button enters loading state (:loading=true)
  → Field validation (frontend): check for empty required fields
    → If validation fails: show warning snackbar + red badges on affected panels
  → Call save_config Tauri command (incremental — only changed fields)
    → On success:
      → Snackbar "设置已保存" (green, 3s)
      → Clear localStorage('settingsDraft')
      → Update Pinia stores with saved values as "original" for change detection
      → Remove "未保存的更改" indicator
      → Save button transitions to disabled (no changes)
    → On conflict (external modification):
      → Dialog: "配置文件已被外部修改，是否覆盖？"
      → User confirms → re-send save_config with overwrite flag
      → User cancels → dialog closes, no changes persisted
    → On error:
      → Snackbar with error detail (red, 5s)
      → Fields retain user's edits (not reverted)
      → Draft remains in localStorage
  → Save button exits loading state
```

### 5.3 Test Connection Flow

```
User clicks "测试连接" in an LLM panel
  → Button enters loading state (:loading=true, disabled)
  → Call test_llm_connection Tauri command with current panel's Provider/Model/API Key/Base URL
    → This is a forward-looking verification attempt
    → On success:
      → Snackbar "连接成功" (green, 3s)
    → On failure:
      → Snackbar "连接失败: {error detail}" (red, 5s)
  → Button exits loading state
```

### 5.4 Local Draft Auto-Save Flow

```
Trigger: SettingsDrawer close (X button, scrim click, gear toggle)
  → Debounce: none (immediate on close)
  → Read all panel fields from Pinia stores
  → Serialize to JSON with updatedAt timestamp
  → Write to localStorage('settingsDraft')
  → On localStorage failure: silent degradation (try/catch, no user feedback)

Trigger: App startup (App.vue onMounted after get_config() succeeds)
  → Check localStorage('settingsDraft')
  → If draft exists AND draft.updatedAt > config.loadedAt:
    → Restore draft values into Pinia stores (optimistic overlay)
    → Set isDirty=true (show "未保存的更改" indicator)
  → If draft exists AND draft.updatedAt <= config.loadedAt:
    → Clear localStorage('settingsDraft') (stale draft)
  → If restore parse fails: silent skip (try/catch)

Trigger: save_config succeeds
  → Clear localStorage('settingsDraft')
  → Set isDirty=false
  → Remove "未保存的更改" indicator
```

### 5.5 API Key Masking Toggle

| Interaction | Behavior |
|-------------|----------|
| Default display | Type="password", masked: `••••••••••••` |
| Show toggle click | Changes to type="text", reveals full key (plaintext from store state) |
| Hide toggle click | Changes back to type="password" |
| Backend API Key display format | Desensitized at Rust level: `sk-...****...xyz` (frontend receives masked string from `get_config()` — never full key from backend) |

**Note:** The API Key stored in Pinia state is the FULL key (received from backend via `get_config()` in desensitized form on initial load, OR from user input). The toggle simply changes the HTML input `type` attribute. When the user edits the field, the full value is visible in the input.

### 5.6 Snackbar Specifications

| Property | Success | Error | Warning | Info |
|----------|---------|-------|---------|------|
| Color | `success` | `error` | `warning` | `info` |
| Timeout | 3000ms | 5000ms | 4000ms | 3000ms |
| Location | `top` | `top` | `top` | `top` |
| Close button | No | No | No | No |
| Action button | None | [重试] for config load failures | None | None |
| Multi-line support | Short messages only (<60 chars) | Error details may wrap (2 lines max) | Short messages | Short messages |

**Snackbar z-index:** Vuetify default (must appear above v-navigation-drawer). Since v-navigation-drawer has z-index 1004 (Vuetify default), snackbar with `location="top"` sits above the drawer when the drawer is open. Test to ensure this works — if snackbar appears behind drawer, explicitly set snackbar content z-index.

### 5.7 Folder Browser Interaction

```
User clicks "浏览" button
  → Call `open({ directory: true, multiple: false, title: '选择文件夹' })` from @tauri-apps/plugin-dialog
  → OS native folder picker opens (modal)
  → User selects folder and clicks "选择文件夹" / "Open"
    → Returned path populates the readonly v-text-field
    → Pinia store updated with new path
  → User clicks "取消" / cancels dialog
    → No change to current path
    → No user feedback needed (dialog dismissal is sufficient)
```

### 5.8 Provider Selection → Model Suggestion

```
User selects a preset Provider from v-select dropdown (e.g., "OpenAI")
  → A watcher on `provider` computes the recommended model
  → A <div class="text-caption text-medium-emphasis mt-1"> element appears below Model field
  → Copy: "推荐: gpt-4o" (from PROVIDER_PRESETS map)
  → Model field is NOT auto-filled (user must type or paste)
  → If user switches to "自定义" → recommended model hint disappears
  → If user switches Provider → hint updates to new Provider's recommended model
```

### 5.9 TTS Engine Switching (Preserve Params)

```
User selects "Azure" from TTS engine v-select
  → store.setEngine('azure') updates Pinia
  → Dynamic fields component re-renders with Azure's config fields
  → Azure fields read from store.engineConfigs['azure'] (may be populated or empty)
  → If user had previously edited Azure config in this session:
    → Fields restore previous values from memory (store preserves engineConfigs per engine)
  → If user switches back to "Edge-TTS":
    → Edge-TTS fields restore previous values from store.engineConfigs['edge_tts']
```

### 5.10 Polling / Cache / Refresh

| Action | Mechanism | Rate | User Facing |
|--------|-----------|------|-------------|
| Edge-TTS voice list cache | Pinia cache on first load in session | Cached for session lifetime | v-autocomplete shows cached list immediately on subsequent opens |
| Edge-TTS voice list refresh | Manual via [重试] button | On demand | Button visible only after load failure |
| Config preload | App.vue onMounted single call | Once on app start | Transparent — panels show skeleton if not yet loaded |
| Panel state persistence | localStorage write on collapse/expand change | Deferred (no debounce needed — low frequency) | Transparent |

---

## 6. Accessibility

### 6.1 Keyboard Navigation

| Element | Keyboard Target | Behavior |
|---------|----------------|----------|
| SettingsDrawer open/close | AppHeader gear button | Enter/Space to toggle |
| Panel expand/collapse | SettingSection header | Enter/Space to toggle (header is a `<div>` with `@click`, must be focusable with `tabindex="0"` and `role="button"` or `aria-expanded`) |
| Save button | v-btn | Enter/Space to activate |
| All form fields | v-text-field / v-select / v-switch / v-btn | Native Vuetify keyboard support |
| "重置" per panel | Header actions v-btn (text variant) | Tab to focus, Enter/Space to activate |
| "重置全部" | Link-style text in Footer | Tab to focus, Enter/Space to activate |
| API Key show/hide | Icon button in field append-inner | Tab to focus, Enter/Space to toggle |
| "浏览" folder | v-btn next to path field | Tab to focus, Enter/Space to open dialog |
| Snackbar dismiss | Snackbar close button (if present) | Tab to focus, Enter/Space to dismiss |
| Confirmation dialogs | v-dialog buttons | Auto-focus on primary action (confirm), Tab to navigate between actions. Escape cancels. |
| Mode switch dialog | Same as confirmation dialog | Focus on [取消] first (defensive — prevents accidental data loss) |
| Test connection button | Outlined v-btn in panel | Tab to focus, Enter/Space to activate |

### 6.2 ARIA Attributes

| Element | Attribute | Value |
|---------|-----------|-------|
| SettingsDrawer | `role="dialog"` | Vuetify v-navigation-drawer has native ARIA roles |
| SettingsDrawer | `aria-label` | "设置面板" |
| SettingSection header (collapsible) | `role="button"` | Must be added if using `<div @click>` pattern |
| SettingSection header (collapsible) | `tabindex="0"` | Must be added for keyboard focus |
| SettingSection header (collapsible) | `aria-expanded` | `"true"` / `"false"` bound to expanded state |
| SettingSection content | `role="region"` | Wrap collapsible content area |
| SettingSection content | `aria-labelledby` | Reference to header title element ID |
| Each LLM Panel test button | `aria-label` | "测试 {panelTitle} 连接" |
| Save button | `aria-label` | "保存设置" |
| Per-panel reset button | `aria-label` | "重置 {panelTitle} 设置" |
| API Key show/hide toggle | `aria-label` | "显示 API 密钥" / "隐藏 API 密钥" (dynamic based on current state) |
| Browse folder button | `aria-label` | "浏览文件夹" |
| Validation error badge | `role="status"` | Use on the red badge element |
| Validation error badge | `aria-live="polite"` | Announce to screen readers |

### 6.3 Focus Management

| Action | Focus Target | Rationale |
|--------|-------------|-----------|
| SettingsDrawer opens | First interactive element (LLM Vision Provider v-select) or the close button | Quick keyboard access to first setting |
| SettingsDrawer closes | AppHeader gear button | Return focus to trigger element |
| Panel expands | First field within the expanded panel | User can immediately edit |
| Dialog opens | Primary action button or [取消] for destructive confirmations | Keyboard-accessible default action |
| Dialog closes | The element that triggered the dialog | Return focus to trigger |
| Snackbar appears | No focus change (passive notification) | — |
| Snackbar with action button | Action button receives focus | User can immediately retry |
| "重置" per panel | First field in the panel | User can see values restored |
| "重置全部" confirmation completes | Close button or save button | User proceeds with next action |

### 6.4 Color Contrast

All color combinations inherit from Phase 14 Vuetify theme which meets WCAG AA standards:
- Text on `on-background` / `on-surface`: Contrast ratio >= 4.5:1 against their respective backgrounds
- `text-medium-emphasis`: Opacity 0.6 of on-surface, used for non-critical text only (placeholders, captions)
- Error/success/warning text: Used sparingly, always on appropriate contrasting background (snackbar uses inverse text for its color — Vuetify default behavior)
- Validation red dot badge: Small element (8px dot), color contrast for indicator alone is acceptable per WCAG non-text contrast

### 6.5 Reduced Motion

- `prefers-reduced-motion: reduce` — Vuetify's `v-expand-transition` and `v-snackbar` slide transitions should respect reduced motion via Vuetify's built-in `$transition-duration` override at the theme level
- Chevron rotation animation (0.2s) — short enough to be non-nauseating, but if reduced motion is detected, disable CSS `transition` entirely
- No Phase 16 animations are essential for understanding content — all are decorative expansion/collapse effects

---

## 7. Registry Safety

**Not applicable.** Phase 16 uses Vuetify 4 (npm package), not shadcn. No third-party component registries.

| Registry | Packages Used | Safety Gate |
|----------|--------------|-------------|
| npm | Vuetify 4.x, @mdi/font, @tauri-apps/plugin-dialog, @tauri-apps/plugin-shell | npm provenance, published packages, version-pinned in package.json |
| Cargo (crates.io) | tauri-plugin-shell, sysinfo (for get_system_stats) | crates.io provenance, version-pinned in Cargo.toml |
| Tauri sidecar binary | FFmpeg 7.1 static builds | Manually sourced binary, managed via git-lfs. Not an npm/Cargo package. No registry involved. |

---

## 8. File Organization & Component Map

```
src/components/
  SettingsDrawer.vue                  ← MODIFY: content → 7 panels, footer → save+reset
  SettingSection.vue                   ← ALREADY EXISTS (Phase 15), reuse as-is
  config/
    LlmVisionPanel.vue                ← NEW: Vision LLM config (CONF-01)
    LlmTextPanel.vue                  ← NEW: Text LLM config (CONF-01)
    TtsPanel.vue                      ← NEW: TTS engine + params (CONF-02)
    BgmPanel.vue                      ← NEW: BGM folder + mode (CONF-03)
    ExportPanel.vue                   ← NEW: Output dir + format (CONF-04)
    ModeParamsPanel.vue               ← NEW: Mode-specific params (CONF-05)
    NetworkProxyPanel.vue             ← NEW: Proxy settings (TDET-01)
    ProviderPresets.ts                ← NEW: Provider → recommendedModel + defaultBaseUrl mapping

src/composables/
  useConfig.ts                        ← NEW: get_config()/save_config() wrapper + draft management
  useTheme.ts                         ← EXISTS (Phase 14), no change

src/stores/
  llm.ts                              ← EXTEND: full fields + load/save/test/reset actions
  tts.ts                              ← EXTEND: engine + voice list + per-engine params
  bgm.ts                              ← EXTEND: folder + mode + audio file list
  export.ts                           ← EXTEND: output dir + format + load/save
  mode.ts                             ← EXTEND: params (frameInterval, batchSize, dramaName, temperature, clipCount) — no change to currentMode logic needed

src-tauri/src/commands/
  config.rs                           ← NEW: 5 commands (get_config, save_config, test_llm_connection, get_edge_tts_voices, get_system_proxy)
  mod.rs                              ← MODIFY: add `pub mod config;` and register 5 commands

src-tauri/src/
  lib.rs                              ← MODIFY: add .plugin(tauri_plugin_shell::init())

src-tauri/
  tauri.conf.json                     ← MODIFY: add externalBin ["binaries/ffmpeg", "binaries/ffprobe"] + shell plugin
  capabilities/default.json           ← MODIFY: add shell:allow-execute, shell:allow-spawn permissions

src-tauri/binaries/                    ← NEW: FFmpeg + ffprobe platform binaries

src/types/generated/                   ← VERIFY: ts-rs generated AppConfig types exist; manual fallback if not
```

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
