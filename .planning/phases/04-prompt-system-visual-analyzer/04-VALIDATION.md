---
phase: 04
slug: prompt-system-visual-analyzer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | none (Cargo.toml `[dev-dependencies]`) |
| **Quick run command** | `cargo test -p narratoai-core prompt::tests -- --nocapture` |
| **Full suite command** | `cargo test -p narratoai-core -- --nocapture && cargo clippy -p narratoai-core -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p narratoai-core prompt::tests && cargo test -p narratoai-core visual::tests`
- **After every plan wave:** `cargo test -p narratoai-core -- --nocapture && cargo clippy -p narratoai-core -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | PRMP-01 | N/A | N/A | unit | `cargo test prompt::registry::tests` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | PRMP-02 | N/A | N/A | unit | `cargo test prompt::template::tests` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | PRMP-04 | T-04-01 | 缺失变量返回错误而非空字符串 | unit | `cargo test prompt::template::tests::test_missing_var` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 1 | PRMP-03 | N/A | N/A | unit | `cargo test prompt::registry::tests::test_categories` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | PRMP-01 | N/A | N/A | unit | `cargo test prompt::manager::tests` | ❌ W0 | ⬜ pending |
| 04-03-01 | 03 | 2 | VISL-01 | N/A | N/A | integration | `cargo test visual::frame_extractor::tests` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 2 | VISL-02 | T-04-02 | JSON markdown代码块剥离防注入 | unit | `cargo test visual::types::tests::test_json_schema` | ❌ W0 | ⬜ pending |
| 04-03-03 | 03 | 2 | VISL-03 | N/A | N/A | unit | `cargo test visual::types::tests::test_serde_roundtrip` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/prompt/template.rs` — `#[cfg(test)] mod tests` with golden file comparisons
- [ ] `src/prompt/registry.rs` — `#[cfg(test)] mod tests` with registry CRUD operations
- [ ] `src/visual/types.rs` — `#[cfg(test)] mod tests` with 10 mock JSON deserialization tests
- [ ] `src/visual/frame_extractor.rs` — `#[cfg(test)] mod tests` with integration test (requires FFmpeg)
- [ ] `tests/` — integration tests for full visual analysis pipeline (mock LLM)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 视觉分析质量 (场景覆盖、忠实度、细节密度) | VISL-02 | 需要人对实际视频的分析结果做质量判断 | 对 2 段测试视频 (10s简单+30s复杂) 的输出做人工审查 |
| 跨帧时间连贯性 | VISL-02 | 相邻帧对象列表的语义一致性需要人工判断 | 抽检标记的相邻帧对 (Jaccard < 0.2)，判定是否真的矛盾 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
