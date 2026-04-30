---
phase: 12-additional-tts-engines
verified: 2026-04-30T18:30:00Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
gaps: []
---

# Phase 12: Additional TTS Engines Verification Report

**Phase Goal:** 实现 6 个额外 TTS 引擎（SoulVoice、Doubao、Qwen TTS、IndexTTS2、Azure Speech、Tencent TTS），使 Rust 版 NarratoAI 的 TTS 路由层达到与 Python 版的功能对等

**Verified:** 2026-04-30T18:30:00Z
**Status:** passed
**Re-verification:** No (initial verification)

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Azure Speech TTS 引擎通过 REST API 生成语音音频 | VERIFIED | `src/tts/azure_speech.rs` — AzureSpeechEngine 实现 POST `{region}.tts.speech.microsoft.com/cognitiveservices/v1`，SSML XML 请求体，Ocp-Apim-Subscription-Key 认证，原始音频字节写入文件。12 单元测试通过 |
| 2 | Tencent TTS 引擎生成语音音频 | VERIFIED | `src/tts/tencent_tts.rs` — TencentTtsEngine 实现 TC3-HMAC-SHA256 手动签名，POST 到 `tts.tencentcloudapi.com`，base64 Audio 解码，Subtitles 转 WordBoundary。7 单元测试通过 |
| 3 | SoulVoice TTS 引擎生成语音音频 | VERIFIED | `src/tts/soulvoice.rs` — SoulVoiceEngine 实现 POST 到配置的 api_url，Bearer 认证，返回音频字节写入文件。5 单元测试通过 |
| 4 | Qwen TTS 引擎生成语音音频 | VERIFIED | `src/tts/qwen_tts.rs` — QwenTtsEngine 实现两步流程：先 POST 到 DashScope REST API 获取 audio URL，再 GET 下载音频字节。duration 从响应解析。5 单元测试通过 |
| 5 | IndexTTS2 语音克隆引擎用参考音频生成语音 | VERIFIED | `src/tts/indextts2.rs` — IndexTts2Engine 实现 multipart/form-data 上传参考音频，120s 超时。5 单元测试通过 |
| 6 | Doubao TTS 引擎生成语音音频 | VERIFIED | `src/tts/doubaotts.rs` — DoubaoTtsEngine 实现 POST 到 `openspeech.bytedance.com/api/v1/tts`，Bearer;{token} 认证，base64 解码。7 单元测试通过 |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/tts/common.rs` | 共享工具模块 (>=80 行) | VERIFIED | 181 行 — ProxyConfig/retry_loop/build_client/write_audio_bytes/parse_engine_prefix + 6 单元测试 |
| `src/tts/soulvoice.rs` | SoulVoice 引擎 (>=80 行) | VERIFIED | 218 行 — 完整 REST API 实现 + 5 单元测试 |
| `src/tts/doubaotts.rs` | Doubao 引擎含 base64 解码 (>=100 行) | VERIFIED | 332 行 — 完整实现 + 7 单元测试 |
| `src/tts/qwen_tts.rs` | Qwen TTS 引擎 (>=120 行) | VERIFIED | 261 行 — 二次下载模式 + 5 单元测试 |
| `src/tts/indextts2.rs` | IndexTTS2 引擎 multipart (>=140 行) | VERIFIED | 251 行 — multipart 文件上传 + 5 单元测试 |
| `src/tts/azure_speech.rs` | Azure Speech 引擎 (>=150 行) | VERIFIED | 468 行 — SSML/智能回退 + 12 单元测试 |
| `src/tts/tencent_tts.rs` | Tencent TTS 引擎 TC3 签名 (>=200 行) | VERIFIED | 428 行 — TC3-HMAC-SHA256 + 7 单元测试 |
| `src/tts/mod.rs` | 路由器含所有 6 新分支 | VERIFIED | 230 行 — 8 个 mod 声明 + 7 个 match 分支 (edge_tts + 6 新 + 回退) + 3 单元测试 |
| `Cargo.toml` | 正确的依赖 | VERIFIED | reqwest(json+multipart), hmac/sha2/hex/chrono/base64/regex |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `soulvoice.rs` | `common.rs` | build_client/retry_loop/parse_engine_prefix/write_audio_bytes | WIRED | soulvoice.rs:24/106/32/82 调用 common:: 函数 |
| `doubaotts.rs` | `common.rs` | build_client/retry_loop/write_audio_bytes | WIRED | doubaotts.rs:25/130/106 调用 common:: 函数 |
| `qwen_tts.rs` | `common.rs` | build_client/retry_loop/parse_engine_prefix/write_audio_bytes | WIRED | qwen_tts.rs:26/133/34/110 调用 common:: 函数 |
| `indextts2.rs` | `common.rs` | build_client/retry_loop/parse_engine_prefix/write_audio_bytes | WIRED | indextts2.rs:26/121/33/90 调用 common:: 函数 |
| `azure_speech.rs` | `common.rs` | build_client/retry_loop/write_audio_bytes | WIRED | azure_speech.rs:201/306/265 调用 common:: 函数 |
| `tencent_tts.rs` | `common.rs` | build_client/retry_loop/write_audio_bytes/parse_engine_prefix | WIRED | tencent_tts.rs:34/248/215/46 调用 common:: 函数 |
| `mod.rs` | `common.rs` | ProxyConfig::from_proxy | WIRED | mod.rs:100/106/112/118/127/141 使用 common::ProxyConfig |
| `mod.rs` | `azure_speech.rs` | should_use_azure_services | WIRED | mod.rs:124 调用 azure_speech::should_use_azure_services |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `soulvoice.rs` | audio_bytes | reqwest response.bytes() | Yes — 从 REST API 获取原始音频字节 | FLOWING |
| `doubaotts.rs` | audio_bytes | base64 decode of response["data"] | Yes — 从 REST API base64 字段解码 | FLOWING |
| `qwen_tts.rs` | audio_bytes | reqwest GET audio_url | Yes — 二次下载模式 | FLOWING |
| `indextts2.rs` | audio_bytes | reqwest multipart response.bytes() | Yes — 从自托管服务获取 | FLOWING |
| `azure_speech.rs` | audio_bytes | reqwest POST response.bytes() | Yes — 从 Azure REST API 获取 | FLOWING |
| `tencent_tts.rs` | audio_bytes | base64 decode of resp["Audio"] | Yes — 从 Tencent REST API base64 字段解码 | FLOWING |
| `tencent_tts.rs` | word_boundaries | resp["Subtitles"] array | Yes — 从 API 响应解析，单位 ms 转 100ns | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| cargo build succeeds | `cargo build --lib` | 编译成功, 0 errors, 1 warning (get_azure_voices unused, 预期行为) | PASS |
| All TTS lib tests pass | `cargo test --lib tts` | 78 passed, 0 failed, 1 ignored (integration) | PASS |
| Common utility tests pass | `cargo test --lib tts::common::tests` | 6/6 passed | PASS |
| SoulVoice tests pass | `cargo test --lib tts::soulvoice::tests` | 5/5 passed | PASS |
| Doubao tests pass | `cargo test --lib tts::doubaotts::tests` | 7/7 passed | PASS |
| Qwen tests pass | `cargo test --lib tts::qwen_tts::tests` | 5/5 passed | PASS |
| IndexTTS2 tests pass | `cargo test --lib tts::indextts2::tests` | 5/5 passed | PASS |
| Azure Speech tests pass | `cargo test --lib tts::azure_speech::tests` | 12/12 passed | PASS |
| Tencent tests pass | `cargo test --lib tts::tencent_tts::tests` | 7/7 passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TTS-04 | 12-03-PLAN.md | Azure Speech TTS 引擎实现 (REST API) | SATISFIED | `src/tts/azure_speech.rs` — 完整实现包括 SSML XML、Ocp-Apim-Subscription-Key 认证、-V2 后缀剥离、智能回退逻辑 |
| TTS-05 | 12-03-PLAN.md | Tencent TTS 引擎实现 | SATISFIED | `src/tts/tencent_tts.rs` — 完整 TC3-HMAC-SHA256 签名实现，含 Subtitles 转 WordBoundary |
| TTS-06 | 12-01-PLAN.md | SoulVoice TTS 引擎实现 | SATISFIED | `src/tts/soulvoice.rs` — REST API 实现，Bearer 认证 |
| TTS-07 | 12-02-PLAN.md | Qwen TTS 引擎实现 | SATISFIED | `src/tts/qwen_tts.rs` — DashScope REST API + 二次 URL 下载 |
| TTS-08 | 12-02-PLAN.md | IndexTTS2 语音克隆引擎实现 | SATISFIED | `src/tts/indextts2.rs` — multipart/form-data 参考音频上传 + 推理参数 |
| TTS-09 | 12-01-PLAN.md | Doubao TTS 引擎实现 | SATISFIED | `src/tts/doubaotts.rs` — REST API + Bearer;{token} 认证 + base64 解码 |

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| No anti-patterns detected | — | — | — |

### Intentional Stubs (Documented in Summaries)

| Artifact | Field | Reason |
|----------|-------|--------|
| soulvoice.rs:85-88 | duration: 0.0, word_boundaries: vec![] | SoulVoice API 不返回时间长度/词边界 |
| doubaotts.rs:109-113 | duration: 0.0, word_boundaries: vec![] | Doubao API 不返回时间长度/词边界 |
| qwen_tts.rs:113-116 | word_boundaries: vec![] | Qwen TTS API 不支持词边界 |
| indextts2.rs:93-97 | word_boundaries: vec![], duration: 0.0 | IndexTTS2 不支持词边界 |
| azure_speech.rs:270-273 | word_boundaries: vec![], duration: 0.0 | REST API 不支持词边界 (Python SDK 版可支持但 Rust 用 REST) |
| tencent_tts.rs:218-221 | duration: 0.0 | Subtitles 不直接包含总时长 |

These match the Python version's behavior — only Edge-TTS supports word boundaries.

### Gaps Summary

No gaps found. Phase 12 goal is fully achieved.

### Deferred Items

None — all required artifacts are implemented and verified.

---

_Verified: 2026-04-30T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
