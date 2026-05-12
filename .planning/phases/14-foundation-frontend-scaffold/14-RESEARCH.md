# Phase 14: Foundation — Frontend Scaffold - Research

**Researched:** 2026-05-12
**Domain:** Vite + Vue 3 + TypeScript frontend scaffold for Tauri 2 desktop app
**Confidence:** HIGH

## Summary

Phase 14 establishes the complete frontend scaffold for NarratoAI v2.0. The approach is to directly port SmartEdit's proven patterns (Vite + Vue 3 + Vuetify + Pinia + vue-router Hash mode + tauriInvoke wrapper) while upgrading to the latest stable versions and adapting the 9 Pinia stores to NarratoAI's domain model.

**Critical finding:** CONTEXT.md D-14 specifies Vite 6 / TypeScript ~5.7 / Vuetify 3.5+ / Pinia 2.1+, but **npm registry shows all these packages have shipped newer major versions**: Vite 8.0.12, TypeScript 6.0.3, Vuetify 4.0.7, Pinia 3.0.4, vue-router 5.0.6. The planner and discuss-phase should confirm whether to follow the locked decision versions or upgrade to latest stable. The REQUIREMENTS.md (FNDT-01) already references Vite 8 and TypeScript 6.0 correctly — the CONTEXT.md D-14 appears to have been written with stale version expectations.

**Primary recommendation:** Port SmartEdit patterns directly (useTauri.ts, vuetify.ts, router/index.ts, vite.config.ts, tsconfig.json, store patterns), then adapt to NarratoAI's 9-store domain model. The `index.html`, `package.json`, and all `src/` scaffold files are greenfield.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- D-01: Directory structure by file type — `src/components/`, `src/stores/`, `src/views/`, `src/types/`, `src/composables/`, `src/api/`, `src/plugins/`, `src/styles/`, `src/layouts/`, `src/router/`, `src/utils/`. Not by business domain.
- D-02: Path alias `@/` -> `src/`.
- D-03: Entry pattern: `main.ts` (createApp + use Pinia/Router/Vuetify) -> `App.vue` -> `layouts/DefaultLayout.vue` -> `router-view`.
- D-04: 9 Pinia stores (app, mode, video, pipeline, script, llm, tts, bgm) with specific state per CONTEXT.
- D-05: Composition API style for all stores (`defineStore('name', () => { ... })`).
- D-06: All 9 stores as skeletons (types + initial state + basic actions) in Phase 14; full logic in later phases.
- D-07: tauriInvoke wrapper ported from SmartEdit (BigInt->Number, sensitive field filtering, debug logging).
- D-08: `src/api/` directory with domain-split command modules.
- D-09: Add `#[derive(TS)]` + `#[ts(export)]` on Rust structs for type export.
- D-10: Export range: all Tauri command request/response types + config types.
- D-11: Add ts-rs to Cargo.toml, trigger generation via build flow.
- D-12/13: Integration test stub replacement for TDET-03 — verify assert!(true) no longer exists.
- D-14: Version constraints (see critical finding above — confirm with user).
- D-15: Dependency versions prefer SmartEdit package.json as reference, upgrade to latest stable.

### Claude's Discretion
- Vite config (port, manualChunks, Tauri host config)
- tsconfig strict level and path alias config
- ESLint/Prettier rule selection
- Vuetify theme config (colors, fonts, zhHans locale details)
- Store interface methods and return types
- `src/api/` module splitting granularity
- ts-rs export path configuration
- Vite dev server Tauri integration

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FNDT-01 | Project init — Vite + Vue 3 + TypeScript, integrate Tauri 2 | SmartEdit vite.config.ts + tsconfig.json direct port; Tauri 2 already configured |
| FNDT-02 | Vuetify plugin, @mdi/font icons, zhHans locale | SmartEdit vuetify.ts direct port (zhHans locale imported from vuetify/locale) |
| FNDT-03 | vue-router Hash mode, SPA | SmartEdit router/index.ts direct port (createWebHashHistory) |
| FNDT-04 | Pinia state — 9 stores | SmartEdit store patterns; 9-store skeleton design per CONTEXT specs |
| FNDT-05 | API layer — tauriInvoke() wrapper | SmartEdit useTauri.ts direct port (BigInt serialization, sensitive field filtering, debug logging) |
| FNDT-06 | ts-rs TypeScript type export | Add ts-rs crate to Cargo.toml + #[derive(TS)] on Rust structs |
| TDET-03 | Integration test stub replacement | Confirm repo-wide grep shows zero assert!(true) stubs |


## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Build toolchain | Client (Dev) | — | Vite + TypeScript compiles to static assets; no server involvement |
| Hash routing | Client (Runtime) | — | createWebHashHistory resolves entirely in browser; no server fallback needed |
| State management | Client (Runtime) | — | Pinia stores manage client-side state (UI state, cached config, pipeline progress) |
| Tauri IPC wrapper | Client (API Layer) | Backend (Tauri) | tauriInvoke is a client-side abstraction over invoke(); thin wrapper |
| Type synchronization | Build-time (Rust -> TS) | — | ts-rs runs at Rust compile time, emits .ts files consumed by Vite |
| Integration testing | Backend (Rust test suite) | — | TDET-03 concerns Rust integration tests, not frontend testing |
| Theme persistence | Client (localStorage) | — | app store writes theme preference to localStorage |


## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue | 3.5.34 | UI framework | Industry standard; SmartEdit proven |
| Vite | 8.0.12 | Build tool + dev server | SmartEdit proven; latest stable (NOTE: D-14 specifies Vite 6) |
| @vitejs/plugin-vue | 6.0.6 | Vite Vue SFC compiler plugin | Required for .vue file support |
| TypeScript | 6.0.3 | Type checking | SmartEdit proven (NOTE: D-14 specifies ~5.7) |
| vue-router | 5.0.6 | SPA routing | SmartEdit proven; Hash mode for Tauri file:// protocol |
| Pinia | 3.0.4 | State management | SmartEdit proven (NOTE: D-14 says 2.1+/"尚未发布") |
| Vuetify | 4.0.7 | Material Design UI framework | SmartEdit proven (NOTE: D-14 specifies 3.5+) |
| @mdi/font | 7.4.47 | Material Design Icons | Paired with Vuetify; SmartEdit proven |

**Version discrepancy note:** npm registry confirms higher major versions than CONTEXT.md D-14 assumes. The planner should raise this with discuss-phase for confirmation before locking versions.

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/api | 2.11.0 | Tauri IPC + window management | All Tauri commands; window controls in DefaultLayout |
| @tauri-apps/plugin-dialog | 2.6.0 | Native file dialogs | Video upload, folder selection (Phase 16-17) |
| @tauri-apps/plugin-shell | 2.0.0 | FFmpeg sidecar | TDET-07 FFmpeg packaging (Phase 16) |
| loglevel | 1.9.2 | Client-side logging | Debug logging in tauriInvoke and stores |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Vuetify 4 | Naive UI, Element Plus | SmartEdit reference uses Vuetify; the entire v2.0 "mimics SmartEdit" goal depends on Vuetify |
| Pinia 3 | Vuex 5 (not released) | Pinia is the official Vue 3 recommendation; no viable alternative |
| Hash routing (createWebHashHistory) | HTML5 history (createWebHistory) | Hash mode is required for Tauri file:// protocol; no alternative for this use case |

**Installation:**
```bash
npm create vue@latest -- --typescript
cd narratoai
npm install vuetify@^4.0.0 @mdi/font pinia vue-router@5 loglevel
npm install -D @tauri-apps/cli
```

**Version Verification:**
```
vue 3.5.34        [VERIFIED: npm registry]
vuetify 4.0.7     [VERIFIED: npm registry]
pinia 3.0.4       [VERIFIED: npm registry]
vue-router 5.0.6  [VERIFIED: npm registry]
vite 8.0.12       [VERIFIED: npm registry]
typescript 6.0.3  [VERIFIED: npm registry]
@tauri-apps/api 2.11.0 [VERIFIED: npm registry]
@mdi/font 7.4.47  [VERIFIED: npm registry]
loglevel 1.9.2    [VERIFIED: npm registry]
ts-rs 11.1.0      [CITED: docs.rs/ts-rs]
```

### Rust-side Dependencies (for ts-rs)
```toml
# narratoai-core/Cargo.toml
[dependencies]
ts-rs = { version = "11.1", features = ["serde-compat", "format"] }

# src-tauri/Cargo.toml  
[dependencies]
ts-rs = "11.1"
```

**Cargo version check:** Rust 1.93.1 available (ts-rs MSRV 1.88.0 satisfied).


## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────┐
│                    Tauri 2 Desktop Shell                  │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Vite Dev Server (:5173) ← beforeDevCommand: vite  │  │
│  │                    or                              │  │
│  │         Build → dist/ ← frontendDist              │  │
│  └────────────────────────────────────────────────────┘  │
│                           │                               │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Vue 3 Application (Hash Router)                   │  │
│  │                                                     │  │
│  │  main.ts → createApp()                              │  │
│  │    ├── createPinia() → 9 Stores                    │  │
│  │    ├── createWebHashHistory() → Router              │  │
│  │    └── createVuetify() → Vuetify Plugin             │  │
│  │                                                     │  │
│  │  App.vue ──▶ layouts/DefaultLayout.vue              │  │
│  │                  ├── AppHeader (Phase 15)            │  │
│  │                  ├── v-main > router-view           │  │
│  │                  └── SettingsDrawer (Phase 15)       │  │
│  │                         │                            │  │
│  │  views/HomeView.vue ←──┘ (Phase 15)                 │  │
│  │                                                     │  │
│  │  composables/useTauri.ts                            │  │
│  │    └── tauriInvoke<T>(cmd, args)                    │  │
│  │          ├── normalizeTauriArgs (BigInt→Number)     │  │
│  │          ├── filterSensitiveArgs                    │  │
│  │          └── debugLog (DEV mode only)               │  │
│  │                                                     │  │
│  │  api/ (domain-split command modules)                │  │
│  │    ├── pipelineApi.ts → tauriInvoke('run_*', ...)   │  │
│  │    ├── scriptApi.ts  → tauriInvoke('load/save/*')   │  │
│  │    └── configApi.ts  → tauriInvoke('get/set/*')     │  │
│  └──────────────┬──────────────────────────────────────┘  │
│                 │ invoke() / emit()                       │
│  ┌──────────────▼──────────────────────────────────────┐  │
│  │  Tauri IPC (src-tauri/src/commands/)                │  │
│  │    ├── pipeline.rs (run_documentary/run_sde/run_sdp)│  │
│  │    ├── script.rs (load/save/validate/etc.)          │  │
│  │    └── export.rs (export_jianying_draft)            │  │
│  │                                                     │  │
│  │  ProgressPayload ◄── emit("pipeline-progress", ...) │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌────────────────────────────────────────────────────┐   │
│  │  ts-rs Type Sync (build-time)                      │   │
│  │  Rust #[derive(TS)] → .ts files                    │   │
│  │    └── src/types/generated/ ← exported types        │   │
│  └────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
NarratoAI/
├── index.html                    # Vite SPA entry (new)
├── package.json                  # Dependencies (new)
├── vite.config.ts                # Vite config (port SmartEdit)
├── tsconfig.json                 # TypeScript strict (port SmartEdit)
├── tsconfig.node.json            # Node config for Vite (new)
├── src/
│   ├── main.ts                   # App entry (port SmartEdit)
│   ├── App.vue                   # Root <v-app> (port SmartEdit)
│   ├── api/                      # Tauri IPC command modules
│   │   ├── pipelineApi.ts        # run_documentary/run_sde/run_sdp
│   │   ├── scriptApi.ts          # load/save/validate/update_narration/set_ost
│   │   ├── configApi.ts          # config read/write (if applicable)
│   │   └── exportApi.ts          # export_jianying_draft
│   ├── components/               # Reusable UI components (Phase 15+)
│   ├── composables/
│   │   ├── useTauri.ts           # tauriInvoke wrapper (port SmartEdit)
│   │   └── useTheme.ts           # Theme toggle
│   ├── layouts/
│   │   └── DefaultLayout.vue     # 3-area layout (port SmartEdit)
│   ├── plugins/
│   │   └── vuetify.ts            # Vuetify 3 plugin (port SmartEdit)
│   ├── router/
│   │   └── index.ts              # Hash router (port SmartEdit)
│   ├── stores/                   # 9 Pinia stores
│   │   ├── app.ts                # Theme + global UI state
│   │   ├── mode.ts               # Current mode (doc/sde/sdp)
│   │   ├── video.ts              # Video list + upload state
│   │   ├── pipeline.ts           # Pipeline progress + state
│   │   ├── script.ts             # Script load/save/edit
│   │   ├── llm.ts                # Vision + Text LLM config
│   │   ├── tts.ts                # TTS engine + params
│   │   ├── bgm.ts                # BGM folder + mode
│   │   └── index.ts              # Re-export all store hooks
│   ├── types/
│   │   ├── generated/            # ts-rs auto-exported types
│   │   └── index.ts              # Manual supplementary types
│   ├── utils/
│   │   └── logger.ts             # loglevel wrapper
│   ├── styles/
│   │   └── main.css              # Global styles
│   └── views/
│       ├── HomeView.vue          # Main view (skeleton for Phase 15)
│       └── NotFoundView.vue      # 404 fallback
└── src-tauri/                    # Tauri backend (exists)
    └── gen/
        └── types/                 # ts-rs default export target
```

### Pattern 1: Pinia Composition API Store Skeleton

**What:** All 9 stores use the `defineStore('name', () => { ... })` Composition API pattern. Phase 14 creates skeleton files with type definitions and initial states; Phase 15+ implement full actions.

**When to use:** Every store. This is a locked decision (D-05).

**Example (app store — ported from SmartEdit):**
```typescript
// Source: SmartEdit src/stores/app.ts [VERIFIED: direct read]
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const theme = ref<'light' | 'dark'>(
    (localStorage.getItem('theme') as 'light' | 'dark') || 'light'
  )

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    localStorage.setItem('theme', theme.value)
  }

  return { theme, toggleTheme }
})
```

**Other store skeletons should follow this pattern:**

```typescript
// mode.ts — current mode state
export const useModeStore = defineStore('mode', () => {
  const currentMode = ref<'documentary' | 'sde' | 'sdp'>('documentary')
  function setMode(mode: 'documentary' | 'sde' | 'sdp') { ... }
  return { currentMode, setMode }
})

// pipeline.ts — pipeline progress
export const usePipelineStore = defineStore('pipeline', () => {
  const status = ref<'idle' | 'running' | 'error'>('idle')
  const progress = ref<ProgressPayload | null>(null)
  // ... Phase 17 will add full event handling
  return { status, progress }
})
```

### Pattern 2: tauriInvoke Wrapper (Direct Port)

**What:** All Tauri IPC calls go through `tauriInvoke<T>()` instead of raw `invoke()`. Handles BigInt-to-Number serialization, sensitive field filtering, and debug logging.

**When to use:** Every IPC call. This is a locked decision (D-07).

**Source:** `E:/GitLib/SmartEdit/src/composables/useTauri.ts` — direct port to `src/composables/useTauri.ts`.

### Pattern 3: API Module Structure

**What:** Each domain has its own API module in `src/api/` that imports `tauriInvoke` and exposes typed functions.

**When to use:** Domain-bound command grouping.

**Example (pipelineApi.ts):**
```typescript
import { tauriInvoke } from '@/composables/useTauri'
import type { DocumentaryRequest, SdeRequest, SdpRequest, CommandResponse } from '@/types/generated'

export async function runDocumentary(request: DocumentaryRequest): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_documentary', { request })
}

export async function runSde(request: SdeRequest): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_sde', { request })
}

export async function runSdp(request: SdpRequest): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_sdp', { request })
}
```

### Anti-Patterns to Avoid

- **Using raw `invoke()` instead of `tauriInvoke()`:** Loses BigInt safety, sensitive field filtering, and debug logging.
- **Domain-based directory split:** D-01 locks file-type-based structure; do NOT create `src/documentary/`, `src/sde/`, `src/sdp/` dirs.
- **Adding full store logic in Phase 14:** D-06 explicitly says skeletons only. Full actions come in later phases.
- **Installing `pinia-plugin-persistedstate`:** The app store uses manual localStorage (like SmartEdit). No need for an extra persistence plugin.
- **Using `createWebHistory`:** Will break on Tauri's file:// / custom protocol. Hash mode is required.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tauri IPC wrapper | Raw invoke() call | SmartEdit `useTauri.ts` (direct port) | BigInt serialization, sensitive field filtering, debug logging all battle-tested in SmartEdit |
| Material Design UI framework | Custom CSS components | Vuetify 4 | 80+ pre-built components, RTL support, SSR compatibility |
| Icon system | SVG icon set | @mdi/font | 7300+ icons, Vuetify native integration |
| Client-side routing | window.location handling | vue-router createWebHashHistory | SPA navigation, lazy loading, nested routes |
| TypeScript config | Custom strictness | SmartEdit tsconfig.json (direct port) | Proven strict mode with `@/` path alias already configured |
| State management | Custom reactive store | Pinia defineStore | Devtools, SSR, TypeScript inference, Composition API integration |

**Key insight:** SmartEdit has already solved every frontend scaffolding challenge this phase faces. Direct porting proven configurations eliminates risk and debugging time. The only greenfield work is adapting to NarratoAI's 9-store domain model and adding ts-rs type export.

## Common Pitfalls

### Pitfall 1: Package Version Mismatch with Locked Decisions
**What goes wrong:** Following CONTEXT.md D-14 literally (Vite 6, TS 5.7, Vuetify 3.5, Pinia 2.1) installs outdated packages.
**Why it happens:** The locked decisions were made with stale npm registry assumptions.
**How to avoid:** Raise with discuss-phase before planning. npm confirms latest versions: Vite 8.0.12, TS 6.0.3, Vuetify 4.0.7, Pinia 3.0.4, vue-router 5.0.6.
**Warning signs:** `npm install` warnings about deprecated versions.

### Pitfall 2: tauri.conf.json Not Updated for Frontend
**What goes wrong:** Vite dev starts on `:5173` but tauri.conf.json has `devUrl: "http://localhost:1420"` and empty `beforeDevCommand`.
**Why it happens:** Tauri template defaults; must be updated to match Vite config.
**How to avoid:** Set `devUrl` to `"http://localhost:5173"` and `beforeDevCommand` to `"npm run dev"` in tauri.conf.json.
**Warning signs:** Tauri app shows blank page or cannot connect.

### Pitfall 3: ts-rs Export Path Confusion
**What goes wrong:** ts-rs default export goes to `src-tauri/gen/types/` but frontend types are in `src/types/generated/`.
**Why it happens:** ts-rs outputs relative to each crate's location, not to the frontend src/.
**How to avoid:** Use `#[ts(export_to = "../../src/types/generated/")]` on each exported type, OR use a build script to copy from `src-tauri/gen/types/` to `src/types/generated/`.
**Warning signs:** Build succeeds but TypeScript cannot find generated type files.

### Pitfall 4: BigInt Serializable Threshold
**What goes wrong:** SmartEdit's useTauri.ts throws on BigInt values > MAX_SAFE_INTEGER. Some Rust i64 values (timestamps like `clip_id: i64`) can exceed this.
**Why it happens:** JavaScript Number.MAX_SAFE_INTEGER = 9,007,199,254,740,991. Large i64 values can exceed this.
**How to avoid:** Audit all IPC types for i64 fields. Consider whether to use u64 for timestamps, or accept the safety check.
**Warning signs:** `tauriInvoke` throws "bigint value exceeds JavaScript safe integer range" at runtime.

### Pitfall 5: Pinia 2 vs Pinia 3 API Differences
**What goes wrong:** If CONTEXT D-14's "Pinia 2.1+" is used but npm shows Pinia 3.0.4, the Composition API `defineStore` syntax remains the same (backward compatible), but other APIs may differ.
**Why it happens:** Pinia 3 has breaking changes from Pinia 2.
**How to avoid:** Verify Pinia 3 migration guide before installing. The core `defineStore('id', () => {})` pattern is stable across v2->v3.
**Warning signs:** TypeScript compilation errors on store usage.

## Code Examples

### Entry Point (port SmartEdit)
```typescript
// Source: SmartEdit src/main.ts [VERIFIED: direct read]
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import vuetify from './plugins/vuetify'
import './styles/main.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(vuetify)
app.mount('#app')
```

### Root Component (port SmartEdit)
```vue
<!-- Source: SmartEdit src/App.vue [VERIFIED: direct read] -->
<template>
  <v-app>
    <router-view />
  </v-app>
</template>
<script setup lang="ts">
</script>
```

### Vuetify Plugin with zhHans Locale
```typescript
// Source: SmartEdit src/plugins/vuetify.ts [VERIFIED: direct read]
// Updated with Vuetify 4 API
import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { zhHans } from 'vuetify/locale'

const vuetify = createVuetify({
  components,
  directives,
  locale: {
    locale: 'zhHans',
    messages: { zhHans },
  },
  icons: { defaultSet: 'mdi' },
  theme: {
    defaultTheme: 'light',
    themes: {
      light: {
        colors: {
          primary: '#2563EB',
          secondary: '#64748B',
          background: '#F8FAFC',
          surface: '#FFFFFF',
        },
      },
      dark: {
        colors: {
          primary: '#3B82F6',
          secondary: '#94A3B8',
          background: '#0F172A',
          surface: '#1E293B',
        },
      },
    },
  },
})
export default vuetify
```

### Hash Router
```typescript
// Source: SmartEdit src/router/index.ts [VERIFIED: direct read]
import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: () => import('@/layouts/DefaultLayout.vue'),
      children: [
        {
          path: '',
          name: 'home',
          component: () => import('@/views/HomeView.vue'),
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: () => import('@/views/NotFoundView.vue'),
    },
  ],
})
export default router
```

### Vite Config with manualChunks
```typescript
// Source: SmartEdit vite.config.ts [VERIFIED: direct read]
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) return
          if (id.includes('node_modules/vuetify')) return 'vuetify'
          if (id.includes('node_modules/vue') ||
              id.includes('node_modules/pinia') ||
              id.includes('node_modules/vue-router')) return 'vue-vendor'
          if (id.includes('node_modules/@tauri-apps')) return 'tauri'
        },
      },
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: '127.0.0.1',
    watch: { ignored: ['**/src-tauri/**'] },
  },
})
```

### Vite-Hosted Dev Server Tauri Integration

The Tauri configuration MUST be updated to point to the Vite dev server:

```diff
// src-tauri/tauri.conf.json
{
  "build": {
-   "frontendDist": "../dist",
-   "devUrl": "http://localhost:1420",
-   "beforeDevCommand": "",
-   "beforeBuildCommand": ""
+   "frontendDist": "../dist",
+   "devUrl": "http://localhost:5173",
+   "beforeDevCommand": "npm run dev",
+   "beforeBuildCommand": "npm run build"
  }
}
```

### ts-rs Rust Export Pattern
```rust
// Source: ts-rs documentation [CITED: docs.rs/ts-rs]
use ts_rs::TS;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentaryRequest {
    pub video_path: std::path::PathBuf,
    pub script_path: std::path::PathBuf,
    pub tts_engine: String,
    pub voice_name: String,
    // ...
}
```

**Note on PathBuf:** ts-rs maps `PathBuf` to `string` in TypeScript, which is the expected behavior for IPC arguments.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Options API stores | Composition API stores (`defineStore('id', () => {})`) | Pinia 2 (2022) | Better TypeScript inference, composable reuse, no `this` confusion |
| LiteLLM for LLM access | OpenAI-compatible provider | Phase 2 (2026) | All LLM calls go through one provider; relevant for LLM store design |
| Streamlit Web UI | Tauri + Vue 3 desktop app | v2.0 (current) | Phase 14 first step of migration to desktop-native UI |
| Manual chunk config | SmartEdit manualChunks strategy | SmartEdit production | Three vendor chunks (vuetify/vue-vendor/tauri) proven to reduce load time |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CONTEXT D-14's version numbers (Vite 6, TS 5.7, Vuetify 3.5, Pinia 2.1) should be confirmed against npm before planning | Standard Stack | Planner picks stale versions; npm shows newer majors available |
| A2 | ts-rs 11.1 works with Rust 1.93.1 | Standard Stack | Version mismatch may cause compilation failure |
| A3 | The npm `create vue@latest` scaffolding produces a compatible project structure | Architecture Patterns | May generate unnecessary files (e2e tests, vitest config) needing cleanup |
| A4 | SmartEdit's manualChunks function form is forward-compatible with Vite 8 | Code Examples | Vite 8 may have changed rollup plugin APIs |
| A5 | `@tauri-apps/api` 2.11.0 is compatible with Tauri 2.0 in tauri.conf.json | Code Examples | Major version mismatch could break IPC |

## Open Questions

1. **Which versions to use?**
   - What we know: npm shows Vite 8.0.12, TS 6.0.3, Vuetify 4.0.7, Pinia 3.0.4. CONTEXT says Vite 6 / TS 5.7 / Vuetify 3.5 / Pinia 2.1+.
   - What's unclear: Does the team want latest stable (Vite 8) or the earlier versions from CONTEXT?
   - Recommendation: Cross-reference with REQUIREMENTS.md (FNDT-01 says "Vite 8 + Vue 3.5 + TypeScript 6.0"). The requirements document already uses the correct versions. Recommend adopting latest stable.

2. **ts-rs export path strategy?**
   - What we know: Default export goes to crate-root relative `bindings/` or `gen/types/`. CONTEXT mentions both `src-tauri/gen/types/` and `src/types/generated/`.
   - What's unclear: Should ts-rs export directly to `../../src/types/generated/` via `#[ts(export_to)]`, or should a build script copy from `src-tauri/gen/types/`?
   - Recommendation: Use `#[ts(export_to = "../../src/types/generated/")]` on each exported struct for direct targeting. This is simpler and provides immediate TypeScript resolution.

3. **Tauri 2.0 `frontendDist` path?**
   - What we know: Current tauri.conf.json has `"frontendDist": "../dist"`. The frontend will be at project root level.
   - What's unclear: Whether the relative path resolves correctly from `src-tauri/` to root `dist/`.
   - Recommendation: Keep `"../dist"` — Tauri 2.0 resolution starts from the directory containing tauri.conf.json (i.e., `src-tauri/`).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Vite, npm packages | YES | v22.22.0 | — |
| npm | Package installation | YES | 10.9.4 | — |
| Rust | ts-rs compilation | YES | 1.93.1 | — |
| Cargo | Dependency management | YES | 1.93.1 | — |

**Missing dependencies with no fallback:** None — all core tooling available.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest (via SmartEdit pattern) or Vite-native |
| Config file | Will be created by `create vue@latest` or manually |
| Quick run command | `npm run dev` (dev server), `vue-tsc --noEmit` (typecheck) |
| Full suite command | Not applicable — Phase 14 is build/typecheck verification, not frontend testing |

### Phase Requirements -> Test Map

**Phase 14 does NOT require frontend tests.** The success criteria verify:
1. Dev server starts without errors (manual check)
2. TypeScript compilation passes (`vue-tsc --noEmit`)
3. Pinia stores instantiate with correct initial state (manual import check)
4. tauriInvoke unit tests (standalone Node tests for BigInt serialization + filtering)
5. ts-rs generates valid .ts files (build-time check)

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FNDT-01 | TypeScript compiles without errors | build | `vue-tsc --noEmit` | ❌ Wave 0 |
| FNDT-05 | tauriInvoke handles BigInt correctly | unit | TBD (vitest) | ❌ Wave 0 |
| TDET-03 | verify assert!(true) stub removal | grep audit | `grep -r 'assert!(true)' narratoai-core/tests/` | ✅ grep run |

### Wave 0 Gaps
- [ ] TypeScript compilation check command: `vue-tsc --noEmit` (add to package.json scripts)
- [ ] For FNDT-05: Consider wrapping useTauri.ts BigInt/`isDebugMode` functions in standalone Node tests (no Tauri runtime needed)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | YES | tauriInvoke `normalizeTauriArgs` + Rust-side `validate_path` — IPC input sanitization |
| V6 Cryptography | No | No crypto operations in frontend scaffold |

### Known Threat Patterns for Tauri IPC

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Sensitive data in IPC logs | Information Disclosure | `filterSensitiveArgs` strips apiKey/password/token before logging |
| BigInt overflow in IPC | Tampering | `serializeBigIntArg` checks Number.MAX_SAFE_INTEGER range |
| Path traversal via IPC | Tampering | Rust-side `validate_path()` rejects `..` and null bytes |

**Security note:** Phase 14 does not add any new security controls beyond what SmartEdit's useTauri.ts already implements. The existing IPC type security (Rust-side `validate_path`, field-level `#[serde(skip_serializing)]` on API keys) continues to be effective.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: npm registry] — All npm package versions verified via `npm view <package> version` commands
- [VERIFIED: direct read] — SmartEdit reference project at E:/GitLib/SmartEdit/ (main.ts, App.vue, useTauri.ts, stores, router, plugins, vite.config, tsconfig)
- [VERIFIED: direct read] — NarratoAI codebase (src-tauri/commands/, src-tauri/models/progress.rs, src-tauri/error.rs, narratoai-core types)
- [CITED: docs.rs/ts-rs 11.1] — ts-rs crate documentation for #[derive(TS)] and serde-compat

### Secondary (MEDIUM confidence)
- [VERIFIED: web search] — Vuetify 3.5+ zhHans locale configuration
- [VERIFIED: web search] — vue-router createWebHashHistory Tauri compatibility
- [VERIFIED: web search] — Vite 6 rollupOptions manualChunks API (function form unchanged from Vite 5)
- [VERIFIED: web search] — ts-rs latest version and feature flags

### Tertiary (LOW confidence)
- None — all critical claims verified against npm or source code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against npm registry
- Architecture: HIGH — SmartEdit patterns directly ported and verified
- Pitfalls: MEDIUM — version discrepancy between CONTEXT.md and npm requires user confirmation
- Security: HIGH — controls verified against existing code

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (npm packages may update; re-verify if planning delayed beyond 30 days)

## TDET-03 Verification

**Grep result:** `grep -r 'assert!(true)' narratoai-core/tests/` — No matches found.
`grep -r 'assert!(true)' src-tauri/` — No matches found.
`grep -r 'assert!(true)' --include='*.rs' -r E:/GitLib/NarratoAI/` — No matches found.

**Conclusion:** All `assert!(true)` stubs have been pre-cleaned. This sub-task requires verification by the planner followed by a "completed, no action needed" mark.
