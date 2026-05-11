---
phase: 07
slug: sde-pipeline
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-11
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `narratoai-core/Cargo.toml` |
| **Quick run command** | `cargo test --lib sde` (44 tests, <1s) |
| **Full suite command** | `cargo test --lib` (565 tests, ~5s) |
| **Estimated runtime** | ~1s (quick) / ~5s (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib sde`
- **After every plan wave:** Run `cargo test --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | SDE-01 | T-7-02 | SdeError Display hides sensitive data | unit | `cargo test --lib sde::error` | ✅ | ✅ green |
| 07-01-02 | 01 | 1 | SDE-01 | T-7-01 | SdeRequest.validate() rejects invalid params | unit | `cargo test --lib sde::types` | ✅ | ✅ green |
| 07-01-03 | 01 | 1 | SDE-02 | T-7-03 | parse_srt_timestamp strict format validation | unit | `cargo test --lib subtitle::timestamp` | ✅ | ✅ green |
| 07-01-04 | 01 | 1 | SDE-01 | — | mod.rs module declaration | compile | `cargo check --lib` | ✅ | ✅ green |
| 07-02-01 | 02 | 1 | SDE-03 | — | plot_analysis v1.0 template registered | unit | `cargo test --lib prompt::register` | ✅ | ✅ green |
| 07-02-02 | 02 | 1 | SDE-03 | — | script_generation v2.0 template registered | unit | `cargo test --lib prompt::register` | ✅ | ✅ green |
| 07-02-03 | 02 | 1 | SDE-03 | — | OutputFormat::Text variant + validator | unit | `cargo test --lib prompt::register` | ✅ | ✅ green |
| 07-03-01 | 03 | 2 | SDE-02 | T-7-06 | 5-step encoding detection chain, fixed regex patterns | unit | `cargo test --lib subtitle::parser` | ✅ | ✅ green |
| 07-03-02 | 03 | 2 | SDE-03 | T-7-07, T-7-08 | repair_json pure-memory, SdeError wraps LLM errors | unit | `cargo test --lib sde::script_gen` | ✅ | ✅ green |
| 07-04-01 | 04 | 3 | SDE-01 | — | 5 documentary step functions pub(crate) | compile | `cargo check --lib` | ✅ | ✅ green |
| 07-04-02 | 04 | 3 | SDE-01 | — | run_sde() 9-step orchestrator with guardrails | unit | `cargo test --lib sde::pipeline` | ✅ | ✅ green |
| 07-04-03 | 04 | 3 | SDE-01 | — | Full pipeline E2E with real LLM + FFmpeg | integration | `cargo test --test sde_real_test` | ✅ | ⚠️ manual |

*Status: ✅ green · ❌ red · ⚠️ manual (requires external services)*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No Wave 0 installs needed.

- [x] `narratoai-core/src/sde/error.rs` — 5 unit tests for SdeError Display
- [x] `narratoai-core/src/sde/types.rs` — 5 unit tests for SdeRequest/SdePipelineState/SdeProgressStep
- [x] `narratoai-core/src/sde/script_gen.rs` — 28 unit tests for repair_json + parse_script
- [x] `narratoai-core/src/sde/pipeline.rs` — 8 unit tests for run_sde orchestration
- [x] `narratoai-core/src/subtitle/parser.rs` — 55 unit tests for encoding detection + SRT/ASS parsing
- [x] `narratoai-core/src/prompt/register.rs` — 4 unit tests for prompt registration

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SDE full pipeline E2E output | SDE-01 | Requires real LLM API key, video file, FFmpeg | `cargo test --test sde_real_test` with valid config.toml + video assets |
| OST=0/1 audio mixing correctness | SDE-01 | Requires audio playback or spectral analysis | Run `sde_real_test`, inspect output video audio tracks |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-11

---

## Validation Audit 2026-05-11

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Unit tests verified green | 103 (44 SDE + 55 subtitle + 4 prompt) |
| Integration tests (manual) | 1 (sde_real_test) |
| Structural deviations | 3 (documented in 07-VERIFICATION.md, all non-blocking) |

**Note:** Three structural deviations from plan were identified in VERIFICATION.md (clip.rs/audio.rs inlined into pipeline.rs, subtitle.rs extracted to standalone module, 'play original' text prefix check replaced by OST field). All are functionally equivalent and do not affect requirement coverage.

---

_Generated: 2026-05-11_
_Validator: Claude (gsd-nyquist-auditor)_
