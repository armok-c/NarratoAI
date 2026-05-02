---
phase: 6
slug: documentary-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-02
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --lib documentary` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds (unit), ~120 seconds (integration with FFmpeg) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib documentary`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | FFMP-02~08, DOCS-01~03 | — | N/A | unit | `cargo test --lib documentary` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/documentary_pipeline_test.rs` — stubs for FFMP-02~08, DOCS-01~03
- [ ] `tests/common/mod.rs` — shared fixtures (test video generation, mock TTS provider)

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Final video plays correctly in media player | DOCS-01 | Requires visual/audio verification | Play output video, verify narration syncs with frames |
| SRT subtitle timing matches audio | FFMP-04 | Requires human perception check | Play with subtitles enabled, verify word timing |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
