---
phase: 02
slug: llm-service-layer
status: audit_complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-28
audited: 2026-05-12
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `Cargo.toml` (test dependencies section) |
| **Quick run command** | `cargo test -p narratoai-core --test llm_wiremock_test` |
| **Full suite command** | `cargo test -p narratoai-core --lib -- llm && cargo test -p narratoai-core --test llm_wiremock_test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-core --test llm_wiremock_test`
- **After every plan wave:** Run `cargo test -p narratoai-core`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01 | 01 | 1 | LLM-07 | — | LLMError enum with 9 variants + From impl | unit | `cargo test -p narratoai-core --lib -- llm` | ✅ | ✅ green |
| 01-02 | 01 | 1 | LLM-04 | — | LlmProvider trait with async methods | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-01 | 02 | 1 | LLM-01 | — | Registry register/get/list/ProviderNotFound | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-01 | 02 | 1 | LLM-04 | — | image_to_base64_data_url preprocessing | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-02 | 02 | 1 | LLM-02 | — | Non-streaming text completion | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-02 | 02 | 1 | LLM-03 | — | SSE streaming token output | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-02 | 02 | 1 | LLM-04 | — | Vision analysis with base64 images | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-02 | 02 | 1 | LLM-05 | — | JSON response_format fallback | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 02-02 | 02 | 1 | LLM-05 | — | OpenAI error code mapping (401/429/400) | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| 03-01 | 03 | 1 | LLM-01 | — | register_all_providers factory + edge cases | unit | `cargo test -p narratoai-core --lib -- llm` | ✅ | ✅ green |
| 03-02 | 03 | 1 | LLM-01..07 | — | Wiremock integration test suite | integration | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| AUDIT | — | — | LLM-07 | — | Proxy config accepted with valid URL | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |
| AUDIT | — | — | LLM-07 | — | Proxy config rejected with empty URL | unit | `cargo test -p narratoai-core --test llm_wiremock_test` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `narratoai-core/tests/llm_wiremock_test.rs` — 10 integration tests covering LLM-01 through LLM-07
- [x] `narratoai-core/src/llm/test_utils.rs` — shared fixtures (test JPEG generation)
- [x] `narratoai-core/src/error.rs` — 9 LLMError unit tests (Chinese messages)
- [x] `narratoai-core/src/llm/register.rs` — 2 unit tests for register_all_providers

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real API endpoint connectivity | LLM-01 | Requires valid API keys and network access | Configure `config.toml` with test credentials, run `cargo test -p narratoai-core --test llm_real_api_test` |
| Streaming SSE behavior under network interruption | LLM-03 | Network conditions not reproducible in unit tests | Manual test with real provider, disconnect network mid-stream, verify error propagation |

---

## Validation Audit 2026-05-12

| Metric | Count |
|--------|-------|
| Gaps found | 3 |
| Resolved | 3 |
| Escalated | 0 |

**Gaps resolved:**
1. `tests/llm_test.rs` unreachable (workspace root, not a package) → moved to `narratoai-core/tests/llm_wiremock_test.rs` + updated `analyze_images` signature
2. LLM-07 proxy configuration passthrough — no test → added `test_proxy_configuration_accepted` + `test_proxy_empty_url_rejected`
3. VALIDATION.md stale (draft, TBD task IDs, wrong test paths) → fully updated

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ✅ compliant
