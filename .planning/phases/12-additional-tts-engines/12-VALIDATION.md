---
phase: 12
slug: additional-tts-engines
status: audit-complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-30
audited: 2026-05-12
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
| 12-01-01 | 01 | 1 | TTS-06 | — | N/A | unit | `cargo test --lib tts::soulvoice::tests` | ✅ narratoai-core/src/tts/soulvoice.rs | ✅ green |
| 12-01-02 | 01 | 1 | TTS-09 | — | N/A | unit | `cargo test --lib tts::doubaotts::tests` | ✅ narratoai-core/src/tts/doubaotts.rs | ✅ green |
| 12-02-01 | 02 | 2 | TTS-07 | — | N/A | unit | `cargo test --lib tts::qwen_tts::tests` | ✅ narratoai-core/src/tts/qwen_tts.rs | ✅ green |
| 12-02-02 | 02 | 2 | TTS-08 | — | N/A | unit | `cargo test --lib tts::indextts2::tests` | ✅ narratoai-core/src/tts/indextts2.rs | ✅ green |
| 12-03-01 | 03 | 3 | TTS-04 | T-12-01 | Azure REST API over HTTPS, key in Ocp-Apim header | unit | `cargo test --lib tts::azure_speech::tests` | ✅ narratoai-core/src/tts/azure_speech.rs | ✅ green |
| 12-03-02 | 03 | 3 | TTS-05 | T-12-02 | TC3-HMAC-SHA256签名, key不记录日志 | unit | `cargo test --lib tts::tencent_tts::tests` | ✅ narratoai-core/src/tts/tencent_tts.rs | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `src/tts/soulvoice.rs` — unit tests with mocked HTTP response (5 tests)
- [x] `src/tts/doubaotts.rs` — unit tests with mocked HTTP response + base64 decode (8 tests)
- [x] `src/tts/qwen_tts.rs` — unit tests with mocked HTTP response + audio URL download (5 tests)
- [x] `src/tts/indextts2.rs` — unit tests with mocked multipart upload (6 tests)
- [x] `src/tts/azure_speech.rs` — unit tests with mocked SSML POST + smart routing (13 tests)
- [x] `src/tts/tencent_tts.rs` — unit tests with mocked TC3 signature + JSON response (7 tests)

---

## Audit Results (2026-05-12)

### Test Execution

```
cargo test --lib tts -- --nocapture
test result: ok. 93 passed; 0 failed; 1 ignored; 0 measured; 479 filtered out
```

### Per-File Audit

| File | Lines | Tests | Requirement | Key Behaviors Verified |
|------|-------|-------|-------------|----------------------|
| `common.rs` | 205 | 6 | shared | parse_engine_prefix (3 cases), ProxyConfig (2 cases), write_audio_bytes |
| `soulvoice.rs` | 234 | 5 | TTS-06 | engine_new, success, empty_text, http_error, parse_prefix |
| `doubaotts.rs` | 379 | 8 | TTS-09 | engine_new, empty_text, missing_config, ak_sk_not_supported, auth_header_format, base64_response, business_code_check, payload_structure |
| `qwen_tts.rs` | 330 | 5 | TTS-07 | engine_new, success, empty_text, no_audio_url, parse_prefix |
| `indextts2.rs` | 275 | 6 | TTS-08 | engine_new, success, empty_text, missing_ref_audio, parse_prefix, default_config |
| `azure_speech.rs` | 525 | 13 | TTS-04 | should_use_azure_services (5 cases), extract_azure_lang, get_azure_voices, engine_new, success, empty_text, empty_config, v2_suffix_stripped |
| `tencent_tts.rs` | 458 | 7 | TTS-05 | engine_new, success, empty_text, api_error, empty_audio, parse_voice_prefix, tc3_signature |
| `mod.rs` | 235 | 3 | routing | mock_engine_trait, unknown_engine, unknown_engine_message |
| **Total** | — | **93** | — | — |

### Delta Since Original Verification (2026-04-30)

- doubaotts.rs: 7→8 tests (+test_auth_header_format, +test_business_code_check; reorganized)
- indextts2.rs: 5→6 tests (+test_default_config)
- azure_speech.rs: 12→13 tests (+test_azure_speech_empty_config)
- All files gained implementation refinements (line count increases across the board)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Qwen exact REST endpoint | TTS-07 | Python SDK封装，原始HTTP端点需实际API调用验证 | 使用有效dashscope API key发送请求，确认返回200和音频数据 |
| IndexTTS2 multipart字段名 | TTS-08 | 自托管服务，字段名取决于服务端版本 | 针对运行中的IndexTTS2实例发送multipart请求 |
| All engine integration | TTS-04~09 | 需要真实API密钥 | `cargo test --lib tts -- --ignored` 运行 `#[ignore]` 标记的集成测试 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ✅ passed — all 6 requirements COVERED, 93 tests verified
