# Research Summary: NarratoAI v2.0 Tauri + Vue 3 Frontend

**Domain:** AI video narration + automated editing desktop application — frontend (Vue 3 + Vuetify 3 + Tauri 2.0) + tech debt cleanup
**Researched:** 2026-05-12
**Overall confidence:** HIGH

## Executive Summary

NarratoAI v2.0 adds a Vue 3 + Vuetify 3 frontend to the existing Rust/Tauri 2.0 desktop backend, replacing the old Streamlit web UI and cleaning six known tech debt items from v1.0. The backend is already shipping (23,532 LOC Rust core + 1,077 LOC Tauri command layer, 573 passing tests). The milestone delivers a native desktop UI that closely mimics the SmartEdit reference project's layout, and resolves architectural debt that limits production readiness.

### Key Finding: 3-Mode UX is the Central Design Challenge

The single most important architectural decision for v2.0 is how the three pipeline modes (Documentary/SDE/SDP) share a single UI. SmartEdit handles only one mode (short drama narration). NarratoAI must extend to three, with:

- **Different settings per mode** — Vision LLM config only in Documentary; TTS tab hidden entirely in SDP; mode-specific params (frame interval, drama name, custom clip count)
- **Different pipeline commands** — Three separate Rust commands with different parameter types and different step counts (6/9/3)
- **Different progress visualization** — Step names differ per mode; script preview rendering differs per mode
- **Shared infrastructure** — VideoTable, LogPanel, AppHeader, BGM/Export panels are mode-independent

The research recommends: a `modeStore` Pinia store as the single source of truth, `v-if` on settings tabs based on computed visible panels, mode-keyed config storage in stores, and a `startPipeline()` dispatch function that routes to the correct Tauri command.

### Key Finding: Tech Debt Has Variable Frontend Impact

Of the six tech debt items, only two directly block frontend features: FFmpeg sidecar bundling (needed for all pipeline output) and Edge-TTS proxy tunnel (needed for TTS behind corporate proxies). Three items have zero frontend impact: Tauri State lock, VisualAnalyzer facade, and integration test stubs. Two items are optional frontend features: YouTube/Pexels command exposure and ConfigManager hot-reload notification.

## Key Findings

**Stack:** Vue 3.5 + Vuetify 3.12 + Pinia 3.0 + vue-router 5.0 + TypeScript 6.0 + Vite 8.0 (Rolldown). All libraries verified via npm registries. SmartEdit reference confirms patterns. Key risk: Vite 8 Rolldown breaking changes — `build.rolldownOptions` replaces `build.rollupOptions`, `advancedChunks` replaces `manualChunks`.

**Architecture:** 3-layer: Vue SPA → 13 Tauri IPC commands → Rust library. Single-page app with hash routing (`createWebHashHistory()`). Mode-aware dynamic panel switching. 9 Pinia stores (app, mode, video, pipeline, script, llm, tts, bgm). ts-rs auto-generated TypeScript types for IPC boundary. No HTTP calls from frontend.

**Features:** 17 P1 features for MVP. Core differentiator is 3-mode switch with dynamic settings panels. Mode selection lives inside the LLM/Mode settings tab, not in AppHeader. TTS tab hidden in SDP mode (no narration). Vision LLM config hidden in SDE/SDP modes. FFmpeg sidecar bundled (no management panel).

**Critical pitfall:** Tauri 2.0 capability-based permission system differs from v1 allowlist — forgetting to add `shell:allow-execute` + sidecar scope in `capabilities/default.json` causes silent FFmpeg failures. Second: Tauri State read lock held across async pipelines blocks all writes to PromptManager/Registry for minutes.

## Implications for Roadmap

Based on research, suggested phase structure for v2.0:

**Phase 1: Foundation (Frontend Scaffold + Tech Debt: Test Stubs)**
- Vite 8 + Vue 3 project init, Vuetify plugin, vue-router (hash), Pinia stores (all 9), API layer (tauriInvoke wrapper), ts-rs type generation, directory structure
- Replace integration test stubs with real tests, wire `_max_retries` parameter

**Phase 2: Layout Shell + 3-Mode UX Switching**
- AppHeader, DefaultLayout, HomeView flex layout, ModeStore, ModeSelector, SettingsDrawer with dynamic tabs, dark/light theme toggle, SystemMonitorBar

**Phase 3: Configuration Panels + FFmpeg Sidecar + Core Tech Debt**
- LLM/Mode tab (vision+text dual model), TTS panel (7 engines), BGM panel, Export panel, mode-specific panels, FFmpeg sidecar bundling, Edge-TTS proxy tunnel, VisualAnalyzer facade completion

**Phase 4: Pipeline Controls + Progress + Script Preview**
- VideoTable with status/actions, Upload/Start/Stop/Clear buttons, mode-aware pipeline dispatch, pipeline progress events, ScriptPreviewDialog (mode-aware rendering), LogPanel

**Phase 5: Remaining Tech Debt + Polish**
- Tauri State read lock optimization, YouTube/Pexels command exposure, ConfigManager hot-reload activation, end-to-end integration testing, edge case handling

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All libraries verified via npm registries. Version compatibility matrix cross-checked. SmartEdit confirms patterns. |
| Features | HIGH | Feature landscape mapped from SmartEdit reference + Rust backend types + PROJECT.md spec. |
| Architecture | HIGH | Three-layer architecture derived from SmartEdit's working implementation. |
| Pitfalls | MEDIUM | Code-level analysis is HIGH; Tauri 2.0-specific items rely on web research. |

## Open Questions

- FFmpeg static binary acquisition channel per target triple (decision needed in Phase 1)
- ts-rs type regeneration pipeline integration (pre-commit vs CI step)
- Vite 8 Rolldown `advancedChunks` API surface verification
- SystemMonitorBar Tauri event source (new `sysinfo`-based command vs existing plugin)
- Pinia 3 TypeScript inference patterns (verify during Phase 1 scaffold)
- Edge-TTS Windows proxy auto-detection extent (manual URL vs PAC auto-detect)

## Research Files

| File | Purpose |
|------|---------|
| `.planning/research/STACK.md` | Technology recommendations with versions, rationale, anti-features, sidecar config |
| `.planning/research/FEATURES.md` | Feature landscape: 13 table stakes, 8 differentiators, 6 anti-features, 3-mode UX mapping |
| `.planning/research/ARCHITECTURE.md` | 3-layer architecture, project structure, 5 patterns, 7 anti-patterns, data flows |
| `.planning/research/PITFALLS.md` | 20 critical pitfalls, 15-item checklist, recovery strategies, phase mapping |

---
*Research summary for: NarratoAI v2.0 Tauri + Vue 3 Frontend*
*Researched: 2026-05-12*
