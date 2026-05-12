# Architecture Research: Vue 3 Frontend Integration with NarratoAI Rust/Tauri Backend

**Domain:** Desktop video narration application (Tauri 2.0 + Vue 3 + Vuetify 3)
**Researched:** 2026-05-12
**Context:** v2.0 milestone — adding Vue 3 frontend to existing Rust/Tauri 2.0 app, plus cleaning v1.0 tech debt
**Reference:** SmartEdit project (E:\GitLib\smartEdit\) for proven Vue 3 + Tauri integration patterns
**Confidence:** HIGH (architecture derived from SmartEdit's working implementation + existing NarratoAI Tauri command layer)

## Standard Architecture

### System Overview

The system is a Tauri 2.0 desktop app with a clear three-layer separation: Vue 3 frontend, Tauri IPC command bridge, and Rust library backend. The frontend never calls the Rust library directly — all communication goes through typed Tauri invoke() calls and event listeners.

```
┌──────────────────────────────────────────────────────────────────┐
│                      VUE 3 FRONTEND (Vite)                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────────┐  │
│  │   Views  │ │Components│ │  Stores  │ │   API Layer         │  │
│  │ HomeView │ │ VideoTbl │ │ (Pinia)  │ │ tauriInvoke() wrap  │  │
│  │          │ │ LogPanel │ │ app/llm/ │ │ event listeners     │  │
│  │          │ │ Settings │ │ tts/bgm/ │ │ (typed, validated)  │  │
│  │          │ │ ScriptDlg│ │ mode/    │ │                     │  │
│  │          │ │ SysMonBar│ │ pipeline/│ │                     │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────────┘  │
│                             │ invoke/event                        │
├─────────────────────────────┼────────────────────────────────────┤
│                   TAURI IPC BRIDGE                                │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  13 #[tauri::command] functions (3 groups)                │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌────────────────────┐  │    │
│  │  │  pipeline/  │ │   script/   │ │     export/        │  │    │
│  │  │ run_doc      │ │ load_script │ │ export_jianying    │  │    │
│  │  │ run_sde      │ │ save_script │ │                    │  │    │
│  │  │ run_sdp      │ │ validate    │ │                    │  │    │
│  │  │ gen_doc_scr  │ │ get_info    │ │                    │  │    │
│  │  │ gen_sdp_scr  │ │ update_narr │ │                    │  │    │
│  │  │              │ │ set_ost     │ │                    │  │    │
│  │  │              │ │ update_ts   │ │                    │  │    │
│  │  └─────────────┘ └─────────────┘ └────────────────────┘  │    │
│  │  Tauri State: AppConfig, Registry, PromptManager          │    │
│  │  Events: pipeline-progress, backend-log                   │    │
│  └──────────────────────────────────────────────────────────┘    │
│                             | call                                │
├─────────────────────────────┼────────────────────────────────────┤
│                    RUST LIBRARY (narratoai-core)                   │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐        │
│  │config│ │ llm  │ │ tts  │ │ffmpeg│ │pipeline│ │script│        │
│  │      │ │      │ │(7eng)│ │      │ │doc/sde│ │      │        │
│  │      │ │      │ │      │ │      │ │/sdp   │ │      │        │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘        │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                  │
│  │prompt│ │visual│ │audio │ │youtube│ │jianying│                │
│  │      │ │      │ │norm  │ │       │ │export  │                │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘                  │
└──────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Layer | Component | Responsibility | Typical Implementation |
|-------|-----------|----------------|------------------------|
| Frontend | Views | Page-level layout, event orchestration, store coordination | `HomeView.vue` — assemblies VideoTable + tabs + LogPanel |
| Frontend | Components | Reusable UI pieces, single responsibility | Each `.vue` file owns one feature (VideoTable, LogPanel, SettingsDrawer, etc.) |
| Frontend | Stores (Pinia) | Client-side state, local draft cache, API action dispatch | Composition API `defineStore()` with `ref()` for reactive state |
| Frontend | API Layer | Typed wrappers around `tauriInvoke()`, parameter normalization | Function-per-command in `src/api/` files |
| Frontend | Composables | Shared reactive logic not tied to a single component | `useTheme()`, `useTauri()`, `useDebugLog()` |
| IPC Bridge | Commands | `#[tauri::command]` facades, path validation, progress callbacks | 13 commands in `src-tauri/src/commands/` |
| IPC Bridge | State | Shared Rust state injected via `app.manage()` | `AppConfig`, `Arc<RwLock<Registry>>`, `Arc<RwLock<PromptManager>>` |
| Backend | Library | All business logic, pipeline orchestration, FFmpeg/TTS/LLM | `narratoai-core` crate, 23,532 LOC |

## Recommended Project Structure

```
src/
├── api/                    # 1. Tauri invoke() wrappers (one file per domain)
│   ├── pipelineCommands.ts # run_documentary, run_sde, run_sdp, generate_script commands
│   ├── scriptCommands.ts   # load/save/validate script, edit operations
│   ├── exportCommands.ts   # export_jianying_draft
│   ├── configCommands.ts   # (future) config load/save if exposed as commands
│   └── index.ts            # barrel export
│
├── components/             # 2. Reusable Vue components
│   ├── AppHeader.vue       # Custom titlebar (40px), minimize/maximize/close + menu
│   ├── VideoTable.vue      # File list with status, script preview, delete
│   ├── LogPanel.vue        # Scrollable log output with auto-scroll + clear/export
│   ├── SettingsDrawer.vue  # Right-side drawer (420px) with all configuration panels
│   ├── ScriptPreviewDialog.vue  # Modal showing generated script before pipeline run
│   ├── ScriptGenerationForm.vue # Script gen params (temperature, max_tokens, models)
│   ├── SystemMonitorBar.vue     # CPU/RAM usage bar (inline in button row)
│   ├── ModeSelector.vue   # Three-mode dropdown/segmented-button (NEW for NarratoAI)
│   ├── DocumentaryPanel.vue# Documentary-specific settings (frame interval, vision batch)
│   ├── SdePanel.vue        # SDE-specific settings (subtitle parser options)
│   ├── SdpPanel.vue        # SDP-specific settings (clip count, temperature)
│   ├── LlmPanel.vue        # LLM config (vision + text dual model)
│   ├── TtsPanel.vue        # TTS engine selection, voice, speed, proxy
│   ├── BgmPanel.vue        # BGM file select, folder browsing, volume
│   ├── ExportPanel.vue     # Export profile (Speed/Balanced/Quality), full load toggle
│   └── __tests__/          # Component-level tests (Vitest + vue-test-utils)
│
├── composables/            # 3. Shared composition functions
│   ├── useTauri.ts         # tauriInvoke() wrapper with bigint handling + logging
│   ├── useTheme.ts         # Light/dark theme toggle + localStorage persistence
│   ├── usePipelineProgress.ts  # Listen to pipeline-progress events, expose reactive state
│   └── useDebugLog.ts      # Backend log listener, TTS event logger, DebugLogger class
│
├── layouts/                # 4. Page layout components
│   └── DefaultLayout.vue   # AppHeader + v-main (router-view) + SettingsDrawer
│
├── plugins/                # 5. Vue plugin initialization
│   └── vuetify.ts          # Vuetify createVuetify() with zhHans locale, MDI icons, theme
│
├── router/                 # 6. Vue Router configuration
│   └── index.ts            # Single route (hash history): / → DefaultLayout → HomeView
│
├── stores/                 # 7. Pinia stores (composition API style)
│   ├── app.ts              # Theme, mode preference, app-level flags
│   ├── mode.ts             # --- NEW --- Selected mode (documentary|sde|sdp), mode config
│   ├── pipeline.ts         # --- NEW --- Pipeline execution state, task_id, progress
│   ├── video.ts            # Video file list, script files, upload state
│   ├── script.ts           # Active script data, editing operations, validation
│   ├── llm.ts              # LLM config (dual model: vision + text), connection test state
│   ├── tts.ts              # TTS config (engine select, voice list, local draft pattern)
│   └── bgm.ts              # BGM config (folder browse, file list, selection)
│
├── types/                  # 8. TypeScript type definitions
│   └── generated/          # Auto-generated by ts-rs from Rust types
│       ├── index.ts        # Barrel re-export of all generated types
│       ├── Script.ts       # Script (Vec<ScriptClip>)
│       ├── ScriptClip.ts   # ScriptClip with OstType, timestamps, pipeline fields
│       ├── OstType.ts      # NarrationOnly | OriginalSound | Mixed
│       ├── AppConfig.ts    # Full app config structure
│       ├── CommandResponse.ts
│       ├── ProgressPayload.ts
│       ├── ScriptInfo.ts
│       └── ... (ts-rs auto-generates all types exposed by Tauri commands)
│
├── utils/                  # 9. Utility functions
│   ├── logger.ts           # loglevel instance (alternative to custom DebugLogger)
│   ├── scriptPreview.ts    # Parse Script JSON → preview data structure
│   └── resourceColors.ts   # Color mapping for resource usage display
│
├── views/                  # 10. Page views
│   └── HomeView.vue        # Primary (and only) view — orchestrates all components
│
├── styles/                 # 11. Global CSS overrides
│   ├── slider.css          # Vuetify slider custom styling
│   └── tooltip-fix.css     # Vuetify tooltip CSS workarounds
│
├── App.vue                 # Root component: <v-app><router-view /></v-app>
├── main.ts                 # Entry point: createApp + Pinia + Router + Vuetify + init
└── vite-env.d.ts           # Vite environment type declarations
```

### Structure Rationale

- **`api/` files are thin wrappers, not services:** Each file exports plain async functions that call `tauriInvoke()`. No classes, no business logic, no state. This keeps the IPC boundary completely transparent and testable. SmartEdit's pattern: `tauriInvoke<T>(command, args)` with bigint normalization and debug logging.

- **Stores use "local draft" pattern:** Each config store has both `config` (DB-persisted) and `localDraft` (unsaved edits). `saveLocalDraftToDb()` saves draft to backend. This prevents losing unsaved edits and enables change detection (`draftDirty` flag). SmartEdit's BGM and TTS stores both use this pattern successfully.

- **Generated types from ts-rs guarantee sync:** All TypeScript types in `types/generated/` are auto-generated from Rust structs. This eliminates the most common class of frontend-backend bugs: type mismatch. The Rust types used in `#[tauri::command]` parameters and return values must be annotated with `#[derive(TS)]`.

- **Flat component tree favors single view:** Like SmartEdit, NarratoAI is a single-page desktop app. Deep route nesting (e.g., /settings/llm, /settings/tts) adds no value. A single `HomeView` with tabbed settings panel is simpler and matches desktop UX conventions.

- **Composables over mixins:** All shared logic uses Vue 3 Composition API composables. No mixins, no Options API where shared logic is needed. `usePipelineProgress()` encapsulates the entire event listener lifecycle (onMounted registration, onUnmounted cleanup).

## Architectural Patterns

### Pattern 1: Three-Mode Dynamic Panel Switching

**What:** The HomeView settings area switches between three mode-specific panels (Documentary, SDE, SDP) based on the user's mode selection, while sharing common panels (LLM, TTS, BGM, Export) across all modes.

**When to use:** When the app has multiple operational modes that share 80% of the UI but differ in 20% of the configuration parameters and pipeline behavior.

**Trade-offs:**
- (+) Shared panels reduce code duplication — LLM, TTS, BGM configs are mode-agnostic
- (+) New mode can be added by creating one panel component + one pipeline command
- (-) Mode-specific panels must be carefully scoped to avoid leaking mode logic into shared code
- (-) SettingsDrawer becomes mode-aware, which adds conditional rendering complexity

**Implementation:**

```typescript
// stores/mode.ts — dictates visible settings panels
export type PipelineMode = 'documentary' | 'sde' | 'sdp'

export const useModeStore = defineStore('mode', () => {
  const currentMode = ref<PipelineMode>('documentary')
  const modeLabel = computed(() => MODE_LABELS[currentMode.value])

  // Which panels to show in HomeView settings tabs
  const visiblePanels = computed(() => ({
    // Common panels (always shown)
    common: ['llm', 'tts', 'bgm', 'export'] as const,
    // Mode-specific panel (shown based on current mode)
    modeSpecific: currentMode.value,
  }))

  return { currentMode, modeLabel, visiblePanels }
})
```

```vue
<!-- HomeView.vue — v-tabs with dynamic v-window-item -->
<v-tabs v-model="settingsTab" density="compact">
  <v-tab value="llm"><v-icon>mdi-brain</v-icon> 模式</v-tab>
  <v-tab value="tts"><v-icon>mdi-volume-high</v-icon> 配音</v-tab>
  <v-tab value="bgm"><v-icon>mdi-music</v-icon> BGM</v-tab>
  <!-- If documentary mode selected, show documentary tab -->
  <v-tab v-if="modeStore.currentMode === 'documentary'" value="documentary">
    <v-icon>mdi-camera</v-icon> 画面分析
  </v-tab>
  <!-- If SDE/SDP mode selected, show subtitle config tab -->
  <v-tab v-if="modeStore.currentMode !== 'documentary'" value="subtitle">
    <v-icon>mdi-subtitles</v-icon> 字幕设置
  </v-tab>
  <v-tab value="export"><v-icon>mdi-export</v-icon> 导出</v-tab>
</v-tabs>

<v-window v-model="settingsTab">
  <!-- Common panels (always shown) -->
  <v-window-item value="llm"> <LlmPanel /> </v-window-item>
  <v-window-item value="tts"> <TtsPanel /> </v-window-item>
  <v-window-item value="bgm"> <BgmPanel /> </v-window-item>
  <!-- Mode-specific panels -->
  <v-window-item value="documentary"> <DocumentaryPanel /> </v-window-item>
  <v-window-item value="subtitle"> <SubtitleConfigPanel /> </v-window-item>
  <v-window-item value="export"> <ExportPanel /> </v-window-item>
</v-window>
```

### Pattern 2: Tauri Command Invocation Wrapper (tauriInvoke)

**What:** A unified wrapper around `@tauri-apps/api/core:invoke()` that handles bigint serialization, sensitive field filtering, debug logging, and error normalization.

**When to use:** Every single Tauri IPC call should go through this wrapper. Never call `invoke()` directly from components or stores.

**Trade-offs:**
- (+) Centralized logging makes debugging IPC issues trivial
- (+) Bigint handling prevents silent data corruption (Rust bigint → JS Number truncation)
- (+) Sensitive field filtering prevents API keys from appearing in dev console logs
- (-) One extra indirection layer; negligible performance impact for desktop IPC

**Example:**
```typescript
// composables/useTauri.ts (adapted from SmartEdit)
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  debugLog(`[tauriInvoke] ${command}`, args || '')
  const normalizedArgs = normalizeTauriArgs(args)
  const result = await invoke<T>(command, normalizedArgs)
  debugLog(`[tauriInvoke] ${command} OK`, result)
  return result
}
```

### Pattern 3: Pipeline Progress Event Stream

**What:** The backend emits pipeline progress events via Tauri's `app.emit()` through a closure passed into the pipeline orchestrator. The frontend listens to these events and updates a reactive progress store.

**When to use:** Any long-running pipeline operation (documentary 6-step, SDE 9-step, SDP 3-step) that needs real-time UI feedback.

**Trade-offs:**
- (+) Real-time progress without polling — no busy-waiting
- (+) Backend sends typed `ProgressPayload` with step index, percentage, and message
- (+) Cleanly separates backend progress tracking from frontend display
- (-) Event listener lifecycle must be managed (register on mount, unregister on unmount)
- (-) Events can be lost if frontend is not listening at the moment of emission (unlikely for desktop)

**Implementation:**

```typescript
// composables/usePipelineProgress.ts
export function usePipelineProgress() {
  const pipelineStore = usePipelineStore()
  const progress = reactive<ProgressPayload>({
    pipeline_type: '', task_id: '', step_name: '',
    percent: 0, message: '', step_index: 0, total_steps: 0,
    status: 'idle',
  })

  onMounted(async () => {
    await listen<ProgressPayload>('pipeline-progress', (event) => {
      Object.assign(progress, event.payload)
      pipelineStore.updateProgress(event.payload)
    })
  })

  onUnmounted(() => { /* unlisten */ })

  return { progress }
}
```

### Pattern 4: Local Draft with Conditional Save

**What:** Config stores maintain a `localDraft` ref that holds unsaved user edits. The user can freely modify draft values without triggering backend writes. Only explicit actions (saving, starting pipeline) persist the draft.

**When to use:** Any configuration form where the user might make multiple changes before committing, and where unsaved changes should survive tab switching.

**Trade-offs:**
- (+) Prevents partial saves, reduces IPC calls
- (+) User can switch between LLM/TTS/BGM tabs without losing unsaved edits
- (+) Change detection (`draftDirty`) enables dirty-state warnings and save prompts
- (-) More complex store logic — must synchronize `config` ↔ `localDraft` on load/save
- (-) Risk of confusing users: unsaved changes in one tab affect pipeline but aren't persisted

**Example (from SmartEdit's tts.ts):**
```typescript
export const useTtsStore = defineStore('tts', () => {
  const config = ref<TtsConfig>(defaultTtsConfig)    // persisted state
  const localDraft = ref<TtsConfig>(defaultTtsConfig)  // unsaved edits

  async function loadConfig() {
    const loaded = await ttsApi.getTtsConfig()
    config.value = loaded
    localDraft.value = { ...loaded } // sync draft on load
  }

  async function saveLocalDraftToDb() {
    await saveConfig(localDraft.value) // persist draft
  }
})
```

### Pattern 5: Mode-Specific Pipeline Dispatch

**What:** The "Start" button in HomeView dispatches to one of three Tauri commands based on `modeStore.currentMode`. The pipeline store tracks the active task_id and routes incoming progress events to the correct UI representation.

**When to use:** When multiple backend pipelines share a common frontend progress display but have different step counts and step names.

**Implementation:**

```typescript
// api/pipelineCommands.ts
export async function startPipeline(mode: PipelineMode, request: RequestData) {
  switch (mode) {
    case 'documentary':
      return tauriInvoke<CommandResponse>('run_documentary', { request })
    case 'sde':
      return tauriInvoke<CommandResponse>('run_sde', { request })
    case 'sdp':
      return tauriInvoke<CommandResponse>('run_sdp', { request })
  }
}
```

The progress callback on the Rust side already includes `pipeline_type` and `step_name`, so the frontend can render mode-specific step names without mode-aware progress tracking:

```typescript
// Use pipeline_type from ProgressPayload to format display
const stepLabels: Record<string, string[]> = {
  documentary: ['加载脚本', '语音生成', '视频裁剪', '音频合成', '片段拼接', '最终合成'],
  sde: ['解析字幕', '剧情分析', '脚本生成', '加载脚本', '语音生成', '视频裁剪', '音频合成', '片段拼接', '最终合成'],
  sdp: ['视频裁剪', '片段拼接', '最终合成'],
}
```

## Data Flows

### Pipeline Execution Flow

This is the most critical data flow — the complete lifecycle from user clicking "Start" to output video.

```
User clicks "开始" button
    │
    ▼
HomeView.handleStart()
    │
    ├── 1. Save all config drafts to backend
    │   └── llmStore.saveLocalDraftToDb()
    │   └── ttsStore.saveLocalDraftToDb()
    │   └── bgmStore.saveLocalDraftToDb()
    │
    ├── 2. Validate preconditions
    │   └── modeStore.currentMode === 'documentary'?
    │       → video_path must exist, script_path writable
    │   └── modeStore.currentMode === 'sde'?
    │       → subtitle_path + video_path must exist
    │   └── modeStore.currentMode === 'sdp'?
    │       → subtitle_path + video_path must exist
    │
    ├── 3. Build request from config
    │   └── Collect video_path, script_path, bgm_path, output_dir
    │   └── Collect LLM model names, TTS engine, OST defaults
    │
    ├── 4. Dispatch to mode-specific command
    │   └── tauriInvoke('run_documentary', { request, config })
    │   └── Returns: CommandResponse { output_video_path }
    │
    ├── 5. Progress events stream asynchronously
    │   └── Backend calls app_handle.emit("pipeline-progress", payload)
    │   └── Frontend listener updates pipelineStore.progress reactively
    │   └── LogPanel appends messages from payload.message
    │
    └── 6. Final event: status="success" or status="error"
        └── pipelineStore.activeTaskId cleared
        └── Snackbar notification with result/error message
```

### Script Edit Flow

The user can preview and edit the generated script before pipeline execution:

```
User clicks "预览脚本" on a video row
    │
    ▼
HomeView.handleViewScript(videoId)
    │
    ├── tauriInvoke('load_script', { path: scriptPath })
    │   └── Returns: Script (Vec<ScriptClip>)
    │
    ├── Parse into preview model
    │   └── scriptPreview utility: extract timestamps, narrations, OST types
    │
    ├── Display in ScriptPreviewDialog
    │   └── User edits narration, OST type, timestamp
    │       ├── tauriInvoke('update_narration', { script, index, text })
    │       ├── tauriInvoke('set_ost', { script, index, ost })
    │       └── tauriInvoke('update_timestamp', { script, index, timestamp })
    │
    └── User exports script
        └── tauriInvoke('save_script', { script, path })
```

### Config Load Flow

On application startup, the frontend loads all configuration asynchronously:

```
App startup (main.ts mounted)
    │
    └── HomeView onMounted
        │
        ├── llmStore.loadConfig()
        │   └── tauriInvoke('load_llm_config')  // (future Tauri command — see tech debt)
        │
        ├── ttsStore.loadConfig()
        │   └── tauriInvoke('get_tts_config')
        │
        ├── bgmStore.loadConfig()
        │   └── tauriInvoke('get_bgm_config')
        │
        ├── Sync pipeline status
        │   └── tauriInvoke('get_pipeline_status')
        │
        └── Register event listeners
            ├── listen('pipeline-progress', ...)
            └── listen('backend-log', ...)
```

### Three-Mode Switching Flow

When user changes the mode in ModeSelector:

```
User selects mode in ModeSelector dropdown
    │
    ▼
modeStore.currentMode = 'sde'
    │
    ├── HomeView re-computes visible settings panels
    │   └── DocumentaryPanel hidden, SubtitleConfigPanel shown
    │
    ├── SettingsDrawer updates mode-specific sections
    │   └── Documentary params (frame_interval, vision_batch) hidden
    │   └── SDE params (subtitle parser, temperature) shown
    │
    ├── "Start" button handler updates dispatch target
    │   └── Now calls run_sde() instead of run_documentary()
    │
    └── VideoTable columns may adjust
        └── (If needed: show/hide OST column based on mode)
```

## Scaling Considerations

| Dimension | Consideration | Approach |
|-----------|--------------|----------|
| Pinia Stores | Store proliferation | One store per domain concept. If a store exceeds ~200 lines, extract sub-stores (e.g., split `pipeline.ts` into `pipeline.ts` + `progress.ts`). |
| Event Listeners | Memory leaks from forgotten unlisteners | `usePipelineProgress()` composable wraps `onMounted`/`onUnmounted` lifecycle. Never register `listen()` outside composables or stores. |
| ts-rs Types | Keeping generated types in sync | Run `cargo test` in src-tauri on Rust changes, which regenerates TS files. Commit generated files to catch type drift in CI. |
| Vite Build | Bundle size for Chinese users | Configure manualChunks for Vuetify (large component library), Vue vendor, and Tauri API to separate cache-friendly bundles. |
| Tauri Commands | Too many commands → confusing API surface | Keep the 15-command surface. Do NOT expose individual library functions — group calls into coarse pipeline commands. |

## Integration Points

### Tauri Command Mapping

| Frontend API File | Tauri Command | Method Signature (Frontend → Backend) |
|-------------------|---------------|---------------------------------------|
| `pipelineCommands.ts` | `run_documentary` | `(request: DocumentaryRequest) => Promise<CommandResponse>` |
| `pipelineCommands.ts` | `run_sde` | `(request: SdeRequest) => Promise<CommandResponse>` |
| `pipelineCommands.ts` | `run_sdp` | `(request: SdpRequest) => Promise<CommandResponse>` |
| `pipelineCommands.ts` | `generate_documentary_script` | `(request: ScriptGenRequest) => Promise<Script>` |
| `pipelineCommands.ts` | `generate_sdp_script` | `(subtitlePath, temperature, customClips) => Promise<Script>` |
| `scriptCommands.ts` | `load_script` | `(path: string) => Promise<Script>` |
| `scriptCommands.ts` | `save_script` | `(script: Script, path: string) => Promise<void>` |
| `scriptCommands.ts` | `validate_script` | `(script: Script) => Promise<void>` |
| `scriptCommands.ts` | `get_script_info` | `(script: Script) => Promise<ScriptInfo>` |
| `scriptCommands.ts` | `update_narration` | `(script: Script, index: number, text: string) => Promise<Script>` |
| `scriptCommands.ts` | `set_ost` | `(script: Script, index: number, ost: OstType) => Promise<Script>` |
| `scriptCommands.ts` | `update_timestamp` | `(script: Script, index: number, timestamp: string) => Promise<Script>` |
| `exportCommands.ts` | `export_jianying_draft` | `(script, videoPath, draftPath, name, width, height) => Promise<string>` |

**NOTE:** SmartEdit has additional commands (`get_tts_config`, `save_tts_config`, etc.) that NarratoAI does not yet expose. Some require adding new Tauri commands on the Rust side. These are marked as tech debt.

### Event Stream Mappings

| Event Name | Payload Type | Direction | Purpose |
|------------|-------------|-----------|---------|
| `pipeline-progress` | `ProgressPayload` | Backend → Frontend | Real-time pipeline step progress (used by `usePipelineProgress`) |
| `backend-log` | `LogEvent` | Backend → Frontend | Debug log streaming to F12 console (used by `useDebugLog`) |

### Tech Debt Integration Points

The following tech debt items directly affect frontend architecture or require new Tauri commands:

| Tech Debt | Frontend Impact | Resolution Path |
|-----------|----------------|-----------------|
| **Edge-TTS proxy tunnel** | TTS panel needs proxy URL config field (already in config) | Implement proxy tunnel in `edge_tts.rs` — frontend just passes config |
| **Tauri State read lock** | No direct frontend impact, but long pipelines freeze UI if backend hangs | Refactor pipeline commands to extract Arc<dyn LlmProvider> before .await |
| **Integration test stubs** | No frontend impact | Replace `assert!(true)` in Rust integration tests |
| **`_max_retries` unwired** | Frontend could expose retry count in settings | Wire parameter through TTS pipeline; add optional field to settings |
| **YouTube/Pexels unexposed** | Frontend can't download YouTube/Pexels from UI | Add new Tauri commands for YouTube download + Pexels search |
| **ConfigManager hotloading** | Frontend doesn't need real-time config updates for v2.0 | Config is loaded at startup and saved explicitly; hotloading is future work |
| **VisualAnalyzer facade** | Documentary mode depends on this — needed for script gen | Facade already exists in `visual::analyzer` — just needs testing |

## Anti-Patterns

### Anti-Pattern 1: Direct `invoke()` Calls in Components

**What people do:** Importing and calling `invoke()` directly in `.vue` component `<script>` blocks.

**Why it's wrong:** Makes components untestable without mocking `@tauri-apps/api/core`. No centralized error handling. Bigint serialization bugs replicated everywhere.

**Do this instead:** Always go through `tauriInvoke()` in `src/api/` files. Components call `store.action()` which calls `api.command()`. Never import `invoke` in a `.vue` file.

### Anti-Pattern 2: Mixing Mode Logic and Shared Config Stores

**What people do:** Putting mode-specific fields (e.g., `frameInterval` for documentary, `temperature` for SDE) into the shared `LlmConfig` store, then conditionally showing/hiding them in all components.

**Why it's wrong:** Bloats the shared config schema with optional fields that are only relevant to one mode. Every mode's fields leak into all components, increasing coupling.

**Do this instead:** Mode-specific params belong in mode-specific panels and the mode-specific request structs. Shared stores (LLM, TTS, BGM) remain mode-agnostic. The pipeline command receives the merged config + mode-specific params as a single request object.

### Anti-Pattern 3: Polling for Progress Instead of Listening for Events

**What people do:** `setInterval(() => invoke('get_progress', { taskId }), 1000)` to poll backend for pipeline progress.

**Why it's wrong:** Wastes IPC bandwidth. Cannot detect events between polling intervals. Backend must maintain a separate progress query endpoint.

**Do this instead:** Use Tauri's event system (`app.emit()` from Rust, `listen()` from frontend). The existing `ProgressPayload` struct and `pipeline-progress` event already implement this correctly. The `usePipelineProgress()` composable encapsulates the listener lifecycle.

### Anti-Pattern 4: Single God Store

**What people do:** One massive Pinia store containing all application state (videos, configs, progress, theme, etc.).

**Why it's wrong:** Pinia stores with 500+ lines become untestable and impossible to reason about. Importing any state from the store pulls in all dependencies. Refactoring a single field requires understanding the entire store.

**Do this instead:** One store per domain concern. SmartEdit's 7 stores (app, llm, video, script, tts, bgm) is the right granularity. Adding `mode.ts` and `pipeline.ts` for NarratoAI brings it to 9 stores — still manageable.

### Anti-Pattern 5: Ignoring ts-rs Type Drift

**What people do:** Manually writing TypeScript interfaces that mirror Rust structs, or not checking generated types into version control.

**Why it's wrong:** Types inevitably drift. A Rust field rename without regenerating TS types silently corrupts IPC data. Manual TS types are never maintained.

**Do this instead:** Use `ts-rs` derive macro (`#[derive(TS)]`) on all Rust types that cross the IPC boundary. Run `cargo test` (which invokes the ts-rs export) as a pre-commit hook. Check generated `.ts` files into git so CI can detect drift.

## Command Layer Organization

The existing 13 Tauri commands are sufficient for v2.0. Here is how the frontend uses each group:

### Pipeline Commands (5 commands)

These are the primary interaction point. All three pipeline modes go through their respective `run_*` command. Script generation commands (`generate_documentary_script`, `generate_sdp_script`) are called before pipeline execution to produce editable scripts.

```
Frontend flow:

1. User selects mode → modeStore.currentMode = 'sde'
2. User clicks "预览脚本" → generate_documentary_script (or generate_sdp_script)
3. ScriptPreviewDialog shows result, user edits
4. User clicks "开始" → run_sde (or run_documentary / run_sdp)
5. Progress events flow → pipelineStore.progress updates
6. Pipeline completes → CommandResponse received
```

### Script Commands (7 commands)

All script editing operations are stateless (immutable updates on the Rust side). The frontend sends the full Script array, the backend returns a new Script array with the edit applied. This is clean but means the frontend must hold the full script in memory.

```
Optimization for large scripts (>500 clips):
- Consider sending only the edited clip index + new value
- For v2.0 MVP, full-script passing is fine (scripts are typically 20-200 clips)
```

### Export Command (1 command)

`export_jianying_draft` is a standalone operation triggered from the ScriptPreviewDialog or after pipeline completion. It requires the script, video path, and draft metadata.

## Sources

- SmartEdit project (E:\GitLib\smartEdit\) — Working Vue 3 + Vuetify 3 + Tauri 2.0 reference architecture, 7 Pinia stores, 13 components, API layer, composables
- NarratoAI src-tauri source (E:\GitLib\NarratoAI\src-tauri\src\) — 13 existing Tauri commands across 3 modules (pipeline, script, export)
- NarratoAI narratoai-core (E:\GitLib\NarratoAI\narratoai-core\src\) — Rust library with config, llm, tts, ffmpeg, pipeline modules
- Tauri 2.0 documentation: State management, Command patterns, Channel API, Event system
- Vuetify 3 documentation: v-tabs, v-window, navigation-drawer, theming, zhHans locale
- Pinia documentation: defineStore, Composition API style, plugin registration
- vue-router documentation: createWebHashHistory (Tauri requirement)

---
*Architecture research for: NarratoAI v2.0 Vue 3 Frontend Integration*
*Researched: 2026-05-12*
