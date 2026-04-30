---
phase: 12
slug: additional-tts-engines
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | none — Cargo.toml dev-dependencies |
| **Quick run command** | `cargo test --lib tts -- --nocapture` |
| **Full suite command** | `cargo test --lib` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib tts -- --nocapture`
- **After every plan wave:** Run `cargo test --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | TTS-06 | — | N/A | unit | `cargo test --lib tts::soulvoice::tests` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | TTS-09 | — | N/A | unit | `cargo test --lib tts::doubaotts::tests` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 2 | TTS-07 | — | N/A | unit | `cargo test --lib tts::qwen_tts::tests` | ❌ W0 | ⬜ pending |
| 12-02-02 | 02 | 2 | TTS-08 | — | N/A | unit | `cargo test --lib tts::indextts2::tests` | ❌ W0 | ⬜ pending |
| 12-03-01 | 03 | 3 | TTS-04 | T-12-01 | Azure REST API over HTTPS, key in Ocp-Apim header | unit | `cargo test --lib tts::azure_speech::tests` | ❌ W0 | ⬜ pending |
| 12-03-02 | 03 | 3 | TTS-05 | T-12-02 | TC3-HMAC-SHA256签名, key不记录日志 | unit | `cargo test --lib tts::tencent_tts::tests` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/tts/soulvoice.rs` — unit tests with mocked HTTP response
- [ ] `src/tts/doubaotts.rs` — unit tests with mocked HTTP response + base64 decode
- [ ] `src/tts/qwen_tts.rs` — unit tests with mocked HTTP response + audio URL download
- [ ] `src/tts/indextts2.rs` — unit tests with mocked multipart upload
- [ ] `src/tts/azure_speech.rs` — unit tests with mocked SSML POST + smart routing
- [ ] `src/tts/tencent_tts.rs` — unit tests with mocked TC3 signature + JSON response

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Qwen exact REST endpoint | TTS-07 | Python SDK封装，原始HTTP端点需实际API调用验证 | 使用有效dashscope API key发送请求，确认返回200和音频数据 |
| IndexTTS2 multipart字段名 | TTS-08 | 自托管服务，字段名取决于服务端版本 | 针对运行中的IndexTTS2实例发送multipart请求 |
| All engine integration | TTS-04~09 | 需要真实API密钥 | `cargo test --lib tts -- --ignored` 运行 `#[ignore]` 标记的集成测试 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
