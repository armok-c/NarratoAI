# Technology Stack

**Project:** NarratoAI Rust Rewrite
**Researched:** 2026-05-12
**Confidence:** HIGH

## Overview

This document covers the full stack. **Part I** documents the already-shipped Rust/Tauri 2.0 backend (v1.0, for context). **Part II** specifies the NEW frontend additions for v2.0 (Vue 3 + Vuetify 3 SPA on Tauri 2.0).

---

## Part I: Existing Backend (v1.0 -- Shipped)

### Core Language & Build

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 2024 edition | Primary language | Memory safety, zero-cost abstractions, async support, excellent FFmpeg/process interop. |
| Cargo | bundled | Build system | Standard Rust toolchain. Cargo.lock for reproducible builds. |

### Async Runtime

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio | 1.x | Async runtime | De facto standard. Multi-threaded scheduler, fs, process, net, sync, time, macros. |
| futures | 0.3.x | Async utilities | Stream, Sink, FutureExt, StreamExt. Needed for FFmpeg output and WebSocket message streaming. |

### HTTP Client

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| reqwest | 0.12.x | Async HTTP client | JSON, multipart, stream, TLS, proxy. Features: `json`, `multipart`, `stream`, `rustls-tls`. |

### Video/Audio Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| ffmpeg-sidecar | 2.x | FFmpeg CLI wrapper | Subprocess approach matches Python pattern. Builder API, auto-download, ffprobe integration. No FFI complexity. |

### WebSocket (Edge-TTS)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio-tungstenite | 0.26.x | Async WebSocket client | Edge-TTS requires WebSocket to Microsoft's speech synthesis service. |

### Serialization & Configuration

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| serde | 1.x | Serialization framework | Foundation for all data serialization. Derive macros for JSON, TOML. |
| serde_json | 1.x | JSON handling | Script files, LLM API, TTS API payloads. |
| toml | 0.8.x | TOML configuration | Direct serde integration for config loading. |

### Error Handling

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| anyhow | 1.x | Application error handling | Ergonomic error chaining with `.context()`. Used in app/binary layer. |
| thiserror | 2.x | Library error types | Derive macro for custom error enums in library crates. |

### Logging & Diagnostics

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tracing | 0.1.x | Structured logging | Spans and events. `#[instrument]` macro. Async-aware context. |
| tracing-subscriber | 0.3.x | Log output configuration | `env-filter`, `fmt` features for console output. |

### Image Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| image | 0.25.x | Image manipulation | Frame extraction validation, thumbnail generation for vision analyzer. |

### Utility Libraries

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| uuid | 1.x | Unique identifiers | Task IDs, job tracking. v4 for random IDs. |
| chrono | 0.4.x | Date/time handling | Timestamps in logs, task state, SRT subtitle timing. |
| base64 | 0.22.x | Base64 encoding | Encoding images for vision LLM API requests. |
| tempfile | 3.x | Temporary files/directories | Intermediate video/audio files during pipeline processing. |
| regex | 1.x | Regular expressions | Parsing FFmpeg output, subtitle timestamp extraction. |
| which | 7.x | Locate FFmpeg binary | Cross-platform binary finder. |

---

## Part II: New Frontend Stack (v2.0)

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Vue 3 | ^3.5.34 | Frontend framework | Composition API + TypeScript-first. Current stable as of May 6, 2026. `<script setup>` SFC syntax is the standard. Reactive system, teleport, suspense. |
| Vuetify 3 | ^3.12.5 | Material Design component library | Project explicitly targets Vuetify 3 (not 4). 3.12.x is the latest 3.x line, still maintained. Components: v-data-table, v-navigation-drawer, v-select, v-switch, v-card for all planned layouts. v3 is battle-tested and stable. |
| Pinia | ^3.0.4 | State management | Pinia 3 is the current major (v3.0.4, Nov 2025). Breaking changes from v2: no `defineStore({id:...})` object syntax, Vue 2 dropped, TypeScript 4.5+ required. New project should use string syntax `defineStore('id', ...)` anyway so migration is trivial. |
| vue-router | ^5.0.6 | Client-side routing | Vue Router 5 (v5.0.6, Apr 2026) merged file-based routing into core with **no breaking changes from v4**. Tauri must use `createWebHashHistory()` -- the `/app#/...` hash avoids 404 on `tauri://localhost`. v5 still supports this identically to v4. |
| TypeScript | ~6.0.3 | Type checking | TypeScript 6.0.3 (Mar 2026) is latest stable ("last JS-based TS", before Go-native 7.0). Strict mode for type safety. |
| Vite | ^8.0.10 | Build tool | Vite 8 (Mar 2026) ships Rolldown unified bundler replacing esbuild+Rollup. 10-30x faster builds. Tauri interacts via `devUrl` + `frontendDist` only -- no Vite version dependency. |
| @mdi/font | ^7.4.47 | Material Design Icons | Standard icon set for Vuetify. SmartEdit reference uses v7.4.x. Over 7,000+ icons. |

### Tauri Integration Packages

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| @tauri-apps/api | ^2.5.0 | Tauri core JS API | `invoke()` for IPC calls. `Event`/`listen()` for progress events. `getCurrentWebviewWindow()` for window management. Core package slimmed down in v2 -- only includes `core`, `path`, `event`, `window/webviewWindow`. |
| @tauri-apps/plugin-shell | ^2.2.1 | Shell + sidecar execution | **Required for FFmpeg sidecar bundling.** `Command.sidecar()` to spawn bundled FFmpeg. `Command.create()` for CLI commands. Must also add `tauri-plugin-shell = "2"` to Cargo.toml. |
| @tauri-apps/plugin-dialog | ^2.6.0 | Native file dialogs | File open/save dialogs for video file selection, export path chooser. Maps to Rust `tauri-plugin-dialog = "2"`. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| loglevel | ^1.9.2 | Frontend logging | SmartEdit reference uses it. Console logging bridge between frontend and Rust tracing backend. Use for frontend-side debug logs. |
| vite-plugin-vuetify | ^2.1.2 | Vuetify Vite plugin | Auto-import Vuetify components per-file for tree-shaking. Alternative: import all components globally (SmartEdit pattern). Either works; plugin is more tree-shakeable. |
| @vue/test-utils | ^2.4.6 | Vue component testing | If unit tests needed. Mount/shallowMount components in vitest. Stable v2 line. |
| jsdom | ^28.1.0 | DOM environment for vitest | Required by vitest for component tests that need DOM. |

### Development Tools

| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| @tauri-apps/cli | ^2.5.0 | Tauri build commands | `npx tauri dev`, `npx tauri build`. Wraps Rust `cargo-tauri` binary via NAPI bindings. |
| @vitejs/plugin-vue | ^6.0.5 | Vite Vue SFC plugin | Compatible with Vite 8 stable. Handles `.vue` SFC compilation with `<script setup>` and `<template>` transforms. |
| vue-tsc | ^3.2.2 | Vue TypeScript checker | Type-check `.vue` files. v3 is the current major (v3.2.2 latest). Used in `vue-tsc --noEmit && vite build` for type-safe builds. |
| sass | ^1.99.0 | SCSS compiler | Dart Sass is the only actively maintained Sass implementation. Required by Vuetify for SASS variable overrides (theme customization). |
| vitest | ^4.1.1 | Unit test runner | Compatible with Vite 8 (v4.1.1). Test tags, `aroundEach`/`aroundAll` hooks. Use `jsdom` as environment for Vue component tests. |

### npm Installation

```bash
# Core dependencies
npm install vue@^3.5.34
npm install vuetify@^3.12.5
npm install pinia@^3.0.4
npm install vue-router@^5.0.6
npm install @mdi/font@^7.4.47

# Tauri integrations
npm install @tauri-apps/api@^2.5.0
npm install @tauri-apps/plugin-shell@^2.2.1
npm install @tauri-apps/plugin-dialog@^2.6.0

# Supporting
npm install loglevel@^1.9.2

# Dev dependencies
npm install -D vite@^8.0.10
npm install -D @vitejs/plugin-vue@^6.0.5
npm install -D vue-tsc@^3.2.2
npm install -D typescript@~6.0.3
npm install -D sass@^1.99.0
npm install -D @tauri-apps/cli@^2.5.0
npm install -D vite-plugin-vuetify@^2.1.2

# Test dev dependencies (if needed)
npm install -D vitest@^4.1.1
npm install -D @vue/test-utils@^2.4.6
npm install -D jsdom@^28.1.0
```

### Cargo.toml Additions (for Tauri plugins)

The existing Cargo.toml at `src-tauri/` must add these dependencies for the new frontend:

```toml
# In src-tauri/Cargo.toml, add to existing [dependencies]:
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
```

The `tauri` crate is already at `tauri = { version = "2", features = ["tray-icon", "devtools"] }` and does not need changes.

---

## Alternatives Considered

| Recommended | Alternative | Why Alternative Was Not Chosen |
|-------------|-------------|-------------------------------|
| Vuetify 3 | Vuetify 4 (v4.0.6, Feb 2026) | Project spec explicitly targets Vuetify 3. Vuetify 4 is active but v3 is still maintained. Sticking with v3 matches SmartEdit reference and reduces risk from new v4 breaking changes. |
| vue-router 5 | vue-router 4 | v5 has zero breaking changes for hash-route Tauri apps. Only change is built-in file-based routing which doesn't affect explicit route definitions. New project should start on current major. |
| Pinia 3 | Pinia 2 | Pinia 3 drops `defineStore({id:...})` object syntax and Vue 2. New project uses string syntax anyway, so no migration cost. Pinia 3 requires Vue 3.3+ and TS 4.5+ -- both satisfied. |
| Vite 8 | Vite 6 or 7 | Vite 8 (Rolldown) is latest stable. 10-30x faster builds. New project avoids migration pain. Only config change: `build.rollupOptions` -> `build.rolldownOptions` (or use compat layer). |
| TypeScript 6 | TypeScript 5.x | TS 6 is the "last JS-based TypeScript" before Go-native 7.0. Backward compatible with 5.x syntax. New project should adopt current stable. |
| npm | pnpm or yarn | SmartEdit reference uses npm. No package manager-specific features needed for this project. npm works fine. |
| Manual Vuetify imports | Auto-import via vite-plugin-vuetify | Both work. SmartEdit uses manual global imports (`* as components`). For a new project, vite-plugin-vuetify gives better tree-shaking. If bundle size is not a concern, manual imports are simpler. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `unplugin-vue-router` | Vue Router 5 has file-based routing built-in. But this project uses explicit route definitions with hash history, so file-based routing is unnecessary anyway. | vue-router's `createRouter` with explicit routes |
| `@tauri-apps/plugin-fs` | File system access should go through Rust IPC commands, not direct frontend FS access. The Rust backend owns all file operations. | Rust `#[tauri::command]` + `invoke()` |
| `@tauri-apps/plugin-http` | All HTTP calls (LLM, TTS) are handled by the Rust backend. Frontend should never make direct HTTP requests. | Rust reqwest + `invoke()` |
| `axios` or `fetch` | Same reason: all data flows through Tauri IPC. No HTTP calls from the frontend. | `import { invoke } from '@tauri-apps/api/core'` |
| `element-plus` or `ant-design-vue` | Another component library adds weight and conflicts with Vuetify. Stick with Vuetify 3 for consistency. | Vuetify 3 |
| `naive-ui` | Same as above. Project spec requires Vuetify. | Vuetify 3 |
| `node-sass` | Deprecated. LibSass is no longer maintained. Dart Sass (`sass` package) is the only active implementation. | `sass` (Dart Sass) |
| `pinia-plugin-persistedstate` | State is managed by the Rust backend, not frontend localStorage. Frontend only caches transient UI state. | Rust backend `State` + `invoke()` |
| `packages for FFmpeg management UI` | Spec explicitly says: "FFmpeg as Tauri sidecar (no management UI)". No ffmpegwasm, no flv.js, no video.js, no web-based transcoder UI. | FFmpeg Tauri sidecar via `@tauri-apps/plugin-shell` |
| Nuxt / SSR / SSG | This is a Tauri desktop app with no server-side rendering. Nuxt adds unnecessary complexity (file system routes, hydration, etc.). | Vue 3 SPA with `createWebHashHistory()` |
| Tailwind CSS | Vuetify already provides a complete design system. Adding Tailwind creates class conflicts and duplicates styling concerns. Overrides are done via Vuetify SASS variables. | Vuetify SASS variables + theme customization |
| ESLint / Prettier (unless explicitly wanted) | SmartEdit uses them but they are dev-ergonomic choices, not stack requirements. If the project doesn't have a lint/style mandate, skip them to reduce config overhead. | Add only if needed |
| `playwright` (e2e) | e2e testing not in scope for this milestone. Tauri e2e tests are complex (need `tauri-driver`). Defer until frontend is stable. | Add in later milestone if needed |

---

## FFmpeg Sidecar Bundling

FFmpeg must be bundled as a Tauri 2.0 sidecar (not management UI, not auto-download on first run).

### Configuration (`src-tauri/tauri.conf.json`)

```json
{
  "bundle": {
    "externalBin": ["binaries/ffmpeg"]
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

### Binary Placement

| Target Platform | Path | Example |
|-----------------|------|---------|
| Windows x64 | `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` | `ffmpeg-x86_64-pc-windows-msvc.exe` |
| macOS x64 | `src-tauri/binaries/ffmpeg-x86_64-apple-darwin` | `ffmpeg-x86_64-apple-darwin` |
| macOS ARM | `src-tauri/binaries/ffmpeg-aarch64-apple-darwin` | `ffmpeg-aarch64-apple-darwin` |
| Linux x64 | `src-tauri/binaries/ffmpeg-x86_64-unknown-linux-gnu` | `ffmpeg-x86_64-unknown-linux-gnu` |

Get target triple: `rustc -vV | grep host | cut -d' ' -f2`

### Capabilities (`src-tauri/capabilities/default.json`)

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-execute",
    "shell:allow-spawn",
    "dialog:default",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
    "core:window:allow-toggle-maximize",
    "core:window:allow-is-maximized"
  ]
}
```

Note: This replaces the existing `capabilities/default.json` from v1.0 which only had basic permissions.

### Frontend Invocation (from JS)

```typescript
import { Command } from '@tauri-apps/plugin-shell'

const command = Command.sidecar('binaries/ffmpeg', [
  '-i', inputPath,
  '-c:v', 'libx264',
  outputPath,
])
const output = await command.execute()
```

### Rust Side (`app_handle`)

```rust
use tauri_plugin_shell::ShellExt;

let sidecar = app_handle.shell().sidecar("ffmpeg")
    .args(["-i", input, output])
    .spawn()?;
```

---

## Vite 8 Configuration for Tauri 2.0

The `vite.config.ts` must be adapted from the SmartEdit reference (Vite 5) for Vite 8 (Rolldown):

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vuetify from 'vite-plugin-vuetify'
import { resolve } from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [
    vue(),
    vuetify({ autoImport: true }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    // Vite 8 uses rolldownOptions (not rollupOptions)
    rolldownOptions: {
      output: {
        // manualChunks deprecated, use advancedChunks or keep compat
        advancedChunks: {
          groups: [
            { name: 'vuetify', test: /[\\/]node_modules[\\/]vuetify[\\/]/ },
            { name: 'vue-vendor', test: /[\\/]node_modules[\\/](vue|pinia|vue-router)[\\/]/ },
            { name: 'tauri', test: /[\\/]node_modules[\\/]@tauri-apps[\\/]/ },
          ],
        },
      },
    },
    target: process.env.TAURI_ENV_PLATFORM === 'windows'
      ? 'chrome105'
      : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
})
```

---

## Version Compatibility Matrix

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| vue@3.5.x | vuetify@3.12.x, pinia@3.x, vue-router@5.x | All on same Vue 3 ecosystem |
| vite@8.0.x | @vitejs/plugin-vue@6.0.5, vite-plugin-vuetify@2.1.x | Vite 8 (Rolldown) supported from plugin-vue 6.0.5 |
| vitest@4.1.x | vite@8.0.x, jsdom@28.x | Vitest 4.1.1+ supports Vite 8 stable |
| vuetify@3.12.x | vite-plugin-vuetify@2.1.x, @mdi/font@7.x | Standard Vuetify 3 ecosystem |
| @tauri-apps/api@2.5.x | tauri@2.x, @tauri-apps/plugin-shell@2.x | All on Tauri 2 plugin system |
| vue-tsc@3.2.x | typescript@6.x, vue@3.5.x | vue-tsc v3 works with TS 6 |
| sass@1.99.x | vuetify@3.12.x | Dart Sass -- only active implementation |
| pinia@3.0.x | vue@3.3+, typescript@4.5+ | Pinia 3 requires Vue 3.3+ minimum |
| vue-router@5.x | vue@3.x | No breaking changes from v4 |

---

## Sources

- Vue 3 npm: `vue@3.5.34` (May 6, 2026) -- [npm](https://www.npmjs.com/package/vue)
- Vuetify 3 npm: `vuetify@3.12.5` (current 3.x) -- [npm](https://www.npmjs.com/package/vuetify)
- Vuetify 3 LTS policy -- [vuetifyjs.com](https://next.vuetifyjs.com/en/introduction/long-term-support/)
- Pinia 3.0 Release -- [github.com/vuejs/pinia](https://github.com/vuejs/pinia/releases/tag/v3.0.0)
- Vue Router 5 Migration -- [router.vuejs.org](https://router.vuejs.org/guide/migration/v4-to-v5.html)
- TypeScript 6.0 Announcement -- [devblogs.microsoft.com](https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/)
- Vite 8 Announcement -- [vite.dev](https://vite.dev/blog/announcing-vite8)
- Tauri 2 Sidecar Docs -- [v2.tauri.app](https://v2.tauri.app/develop/sidecar/)
- @tauri-apps/plugin-shell docs -- [v2.tauri.org.cn](https://v2.tauri.org.cn/reference/javascript/shell/)
- Vitest 4.1 Release -- [vitest.dev](https://vitest.dev/blog/vitest-4-1)
- SmartEdit reference project at `E:/GitLib/SmartEdit` -- dependency versions and patterns
- Dart Sass releases -- [github.com/sass/dart-sass](https://newreleases.io/project/github/sass/dart-sass/release/1.99.0)

---

*Stack research for: NarratoAI v2.0 frontend (Vue 3 + Vuetify 3 + Tauri 2.0)*
*Researched: 2026-05-12*
