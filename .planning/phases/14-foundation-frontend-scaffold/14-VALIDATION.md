---
phase: 14
slug: foundation-frontend-scaffold
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-12
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest (unit) + vue-tsc (typecheck) + cargo test (ts-rs generation) |
| **Config file** | `vitest.config.ts` — created in Wave 1 if tauriInvoke tests exist |
| **Quick run command** | `npx vue-tsc --noEmit` |
| **Full suite command** | `npx vue-tsc --noEmit && npx vitest run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vue-tsc --noEmit`
- **After every plan wave:** Run full suite if tests exist
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | FNDT-01 | T-T01 | N/A — scaffold only | decision | checkpoint: version select | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | FNDT-01/02/03 | T-T01 | N/A — scaffold creation | build | `npx vue-tsc --noEmit` | ❌ W0 | ⬜ pending |
| 14-01-03 | 01 | 1 | FNDT-01/02/03 | T-T03 | N/A — Tauri integration | manual | `npm run dev` (dev server start) | ❌ W0 | ⬜ pending |
| 14-02-01 | 02 | 2 | FNDT-05 | T-T04 | BigInt overflow prevention in serializeBigIntArg | unit | `npx vitest run --reporter=verbose` | ❌ W0 | ⬜ pending |
| 14-02-02 | 02 | 2 | FNDT-04 | — | N/A — Store skeletons | import | Import all 9 stores in Node REPL | ❌ W0 | ⬜ pending |
| 14-02-03 | 02 | 2 | FNDT-05 | — | N/A — API module wrappers | build | `npx vue-tsc --noEmit` | ❌ W0 | ⬜ pending |
| 14-03-01 | 03 | 2 | FNDT-06 | — | N/A — type export only | build | `cargo test` (triggers ts-rs generation) | ❌ W0 | ⬜ pending |
| 14-03-02 | 03 | 2 | TDET-03 | — | N/A — verification only | grep | `grep -r 'assert!(true)' narratoai-core/tests/` (零匹配) | ✅ | ⬜ verified |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `package.json` — add `"typecheck": "vue-tsc --noEmit"` script
- [ ] `package.json` — add `"test": "vitest run"` script
- [ ] `vitest.config.ts` — create if tauriInvoke unit tests exist (FNDT-05)
- [ ] `tests/useTauri.test.ts` — stub for BigInt serialization + sensitive field filtering tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Vite dev server starts without errors | FNDT-01 | Browser HMR check | Run `npm run dev`, open http://localhost:5173, verify "欢迎使用 NarratoAI" renders in Vuetify |
| Hash routing navigates to HomeView + 404 | FNDT-03 | Visual route verification | Navigate to `/#/` → HomeView, `/#/nonexistent` → 404 页面 |
| 9 Pinia stores instantiate with correct initial state | FNDT-04 | DevTools inspection | Open Vue DevTools, verify all 9 stores exist in Pinia tab with correct defaults |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
