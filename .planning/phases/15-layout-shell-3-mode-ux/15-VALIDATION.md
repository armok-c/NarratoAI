---
phase: 15
slug: layout-shell-3-mode-ux
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-13
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.x |
| **Config file** | `vitest.config.ts` (Phase 14 installed) |
| **Quick run command** | `npx vitest run --reporter=verbose` |
| **Full suite command** | `npx vitest run` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run --reporter=verbose`
- **After every plan wave:** Run `npx vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | — | — | Rust sysinfo crate compiles, command returns valid JSON | unit (Rust) | `cargo test -p narratoai -- commands::system` | ❌ W0 | ⬜ pending |
| 15-02-01 | 02 | 1 | LOUT-01 | — | WindowControls renders 3 buttons with correct icons | unit | `npx vitest run src/components/__tests__/WindowControls` | ❌ W0 | ⬜ pending |
| 15-02-02 | 02 | 1 | LOUT-01 | — | AppHeader renders title + logo + buttons, drag starts on mousedown | unit | `npx vitest run src/components/__tests__/AppHeader` | ❌ W0 | ⬜ pending |
| 15-03-01 | 03 | 1 | LOUT-02, LOUT-04, LOUT-05 | — | DefaultLayout renders AppHeader + v-main + SystemMonitorBar + SettingsDrawer | unit | `npx vitest run src/layouts/__tests__/DefaultLayout` | ❌ W0 | ⬜ pending |
| 15-03-02 | 03 | 1 | LOUT-04 | — | SettingsDrawer renders header/content/footer, v-switch toggles theme | unit | `npx vitest run src/components/__tests__/SettingsDrawer` | ❌ W0 | ⬜ pending |
| 15-03-03 | 03 | 1 | LOUT-04 | — | SettingSection renders title/icon/slot, collapsible behavior works | unit | `npx vitest run src/components/__tests__/SettingSection` | ❌ W0 | ⬜ pending |
| 15-04-01 | 04 | 1 | LOUT-03, MODE-02 | — | HomeView renders 60/40 split, ModeSelector card in upper-right | unit | `npx vitest run src/views/__tests__/HomeView` | ❌ W0 | ⬜ pending |
| 15-04-02 | 04 | 1 | MODE-01 | — | ModeStore.setMode persists to localStorage, resetPipeline clears pipeline state | unit | `npx vitest run src/stores/__tests__/mode` | ❌ W0 | ⬜ pending |
| 15-05-01 | 05 | 1 | LOUT-06 | — | SystemMonitorBar renders CPU/RAM text, polls every 2s, cleans up interval | unit | `npx vitest run src/components/__tests__/SystemMonitorBar` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/components/__tests__/WindowControls.test.ts` — stubs for LOUT-01 window control buttons
- [ ] `src/components/__tests__/AppHeader.test.ts` — stubs for LOUT-01 custom titlebar
- [ ] `src/components/__tests__/SettingsDrawer.test.ts` — stubs for LOUT-04 settings drawer
- [ ] `src/components/__tests__/SettingSection.test.ts` — stubs for LOUT-04 reusable card
- [ ] `src/components/__tests__/SystemMonitorBar.test.ts` — stubs for LOUT-06 system monitor
- [ ] `src/views/__tests__/HomeView.test.ts` — stubs for LOUT-03, MODE-02 home view
- [ ] `src/layouts/__tests__/DefaultLayout.test.ts` — stubs for LOUT-02, LOUT-05 layout
- [ ] `src/stores/__tests__/mode.test.ts` — stubs for MODE-01 mode store
- [ ] `src-tauri/src/commands/system.rs` test — Rust unit test for get_system_stats command

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Window drag by header mousedown | LOUT-01 | Browser test env has no Tauri window, native drag unavailable | Launch Tauri dev, verify header drag moves window |
| WindowControls close exits process | LOUT-01 | Test env cannot verify process exit | Launch Tauri dev, verify close button exits app |
| Theme toggle localStorage persistence across page reload | LOUT-05 | Vitest clears localStorage between tests by default | Manual: toggle theme, refresh page (F5), verify theme persists |
| SystemMonitorBar 2s poll interval with real Tauri command | LOUT-06 | Requires running Tauri backend for get_system_stats | Launch Tauri dev, verify CPU/RAM values update every 2s |
| ModeSelector v-select dropdown interaction | MODE-02 | Vuetify v-select rendering needs jsdom + Vuetify plugin setup | Verify in Tauri dev: select each mode, verify ModeStore updates |
| Tauri capabilities permissions (window APIs) | LOUT-01 | Permissions changes are config-level, not testable in unit tests | Verify Tauri dev window controls work (minimize/maximize/close) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
