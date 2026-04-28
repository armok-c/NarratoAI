---
phase: 02
slug: llm-service-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `Cargo.toml` (test dependencies section) |
| **Quick run command** | `cargo test -p narratoai --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | LLM-01 | — | Provider routing by name | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-02 | — | Non-streaming text completion | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-03 | — | SSE streaming token output | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-04 | — | Vision analysis with base64 images | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-05 | — | Structured error types | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-06 | — | Retry with max_retries config | unit | `cargo test llm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LLM-07 | — | Proxy configuration passthrough | unit | `cargo test llm` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/llm_test.rs` — stubs for LLM-01 through LLM-07
- [ ] `tests/common/mod.rs` — shared fixtures (mock LLM server, test config builder)
- [ ] `Cargo.toml` — add `async-openai` (with `chat-completion` feature), `image`, `base64` crates

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real API endpoint connectivity | LLM-01 | Requires valid API keys and network access | Configure `config.toml` with test credentials, run `cargo test -- --ignored` for integration tests |
| Streaming SSE behavior under network interruption | LLM-03 | Network conditions not reproducible in unit tests | Manual test with real provider, disconnect network mid-stream, verify error propagation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
