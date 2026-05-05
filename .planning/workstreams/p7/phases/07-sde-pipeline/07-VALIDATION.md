---
phase: 7
slug: sde-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-05
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust native) |
| **Config file** | none — Wave 0 installs test modules |
| **Quick run command** | `cargo test --lib sde` |
| **Full suite command** | `cargo test --lib` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib sde`
- **After every plan wave:** Run `cargo test --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 0 | SDE-01 | T-7-01 | SdeError Display is Chinese | unit | `cargo test sde::error` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 0 | SDE-02 | — | encoding_rs detects GBK without panic | unit | `cargo test sde::subtitle` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | SDE-03 | T-7-02 | LLM API key never logged | unit | `cargo test sde::script_gen` | ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 1 | SDE-03 | T-7-03 | JSON repair handles 7 fallback levels | unit | `cargo test sde::script_gen::repair_json` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 2 | SDE-01 | — | run_sde() end-to-end with mock LLM | integration | `cargo test sde::pipeline` | ❌ W0 | ⬜ pending |
| 07-03-02 | 03 | 2 | SDE-02 | — | SRT/ASS parsing correctness | integration | `cargo test sde::subtitle` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/sde/error.rs` — SdeError enum with Chinese Display messages
- [ ] `src/sde/subtitle.rs` — encoding detection + SRT/ASS text extraction
- [ ] `src/sde/types.rs` — SdeRequest + SdePipelineState + SdeProgressStep
- [ ] `src/sde/mod.rs` — module public API surface

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| E2E with real LLM | SDE-03 | Requires API key + network | Run `run_sde()` with real subtitle file, verify script JSON output has valid OST tags |
| Subtitle encoding auto-detection | SDE-02 | Requires real GBK/UTF-16 files | Test with 3 subtitle files: UTF-8, GBK, UTF-16LE |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
