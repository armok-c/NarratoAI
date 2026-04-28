---
phase: 5
slug: script-management
status: active
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-28
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --lib script` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib script`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | SCRP-01 | — | N/A | unit | `cargo test --lib script::load` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | SCRP-01 | — | N/A | unit | `cargo test --lib script::serde` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | SCRP-03 | — | N/A | unit | `cargo test --lib script::validate` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 2 | SCRP-02 | — | N/A | unit | `cargo test --lib script::save` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | SCRP-02 | — | N/A | unit | `cargo test --lib script::edit` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/script/` — module with ScriptClip, OstType, ScriptError, ValidationError types
- [ ] Test fixture: `resource/scripts/2026-0416-112716.json` — existing Python sample for compat tests
- [ ] `serde` + `serde_json` + `serde_repr` in Cargo.toml — if not already present

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Python-generated JSON loads correctly in Rust | SCRP-01 | Requires cross-language file comparison | Load `resource/scripts/2026-0416-112716.json`, verify all fields round-trip |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
