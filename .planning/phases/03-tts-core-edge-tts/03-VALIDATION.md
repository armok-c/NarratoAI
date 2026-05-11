---
phase: 03
slug: tts-core-edge-tts
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-11
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for TTS Core + Edge-TTS.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust cargo test (lib + integration) |
| **Config file** | narratoai-core/Cargo.toml |
| **Quick run command** | `cargo test --lib tts:: 2>&1 \| tail -5` |
| **Full suite command** | `cargo test --lib && cargo test --test tts_test` |
| **Estimated runtime** | ~3 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib tts::`
- **After every plan wave:** Run `cargo test --lib && cargo test --test tts_test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | TTS-01 | — | TTSError 5 variants, Chinese messages | unit | `cargo test --lib error::tests::test_tts_error` | ✅ | ✅ green |
| 03-01-02 | 01 | 1 | TTS-01 | — | WordBoundary/TtsOutput field integrity | unit | `cargo test --lib tts::tests` | ✅ | ✅ green |
| 03-01-02 | 01 | 1 | TTS-02 | T-03-01 | synthesize() router, UnknownEngine dispatch | unit | `cargo test --lib tts::tests` | ✅ | ✅ green |
| 03-02-01 | 02 | 2 | TTS-03 | T-03-03 | SSML XML escaping, rate/pitch conversion | unit | `cargo test --lib edge_tts::tests` | ✅ | ✅ green |
| 03-02-02 | 02 | 2 | TTS-03 | T-03-04 | WebSocket connect, binary parse, retry | unit | `cargo test --lib edge_tts::tests` | ✅ | ✅ green |
| 03-03-01 | 03 | 3 | TTS-01 | — | Types/errors via public API | integration | `cargo test --test tts_test` | ✅ | ✅ green |
| 03-03-01 | 03 | 3 | TTS-02 | T-03-01 | Router dispatch via public API | integration | `cargo test --test tts_test` | ✅ | ✅ green |
| 03-03-01 | 03 | 3 | TTS-03 | — | WordBoundary 100ns semantics | integration | `cargo test --test tts_test` | ✅ | ✅ green |

---

## Test Coverage Summary

| Requirement | Inline (lib) Tests | Integration Tests | Network-Required | Status |
|-------------|--------------------|-------------------|------------------|--------|
| TTS-01 (trait/types/errors) | 8 (5 TTSError + 3 tts core) | 4 (types + errors) | 0 | COVERED |
| TTS-02 (router dispatch) | 2 (unknown engine) | 3 (router + signature) | 0 | COVERED |
| TTS-03 (Edge-TTS engine) | 34 (SSML/rate/pitch/parse/engine) | 1 (100ns semantics) | 1 (edge_tts_real_test) | COVERED |

**Totals:** 44 inline tests + 8 integration tests + 1 network test (ignored by default)

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No additional Wave 0 setup needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Edge-TTS 生成音频的实际可播放性 | TTS-03 | 需人工验证 MP3 文件可被播放器打开且中文语音清晰 | `cargo test --test edge_tts_real_test`（需网络），然后手动播放输出的 MP3 |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (N/A — no MISSING)
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

---

## Validation Audit 2026-05-11

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

**Gap resolved:** `tests/tts_test.rs` was orphaned at workspace root (not compiled by Cargo). Moved to `narratoai-core/tests/tts_test.rs` and updated API from 6-arg to 8-arg signature. 8/8 integration tests now pass.

**Approval:** approved 2026-05-11
