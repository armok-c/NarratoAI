---
phase: 04
slug: prompt-system-visual-analyzer
status: compliant
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-30
audited: 2026-05-12
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | none (Cargo.toml `[dev-dependencies]`) |
| **Quick run command** | `cargo test -p narratoai-core --lib prompt:: visual:: -- --nocapture` |
| **Full suite command** | `cargo test -p narratoai-core --lib -- --nocapture && cargo clippy -p narratoai-core -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p narratoai-core --lib prompt:: visual:: -- --nocapture`
- **After every plan wave:** `cargo test -p narratoai-core --lib -- --nocapture && cargo clippy -p narratoai-core -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | PRMP-01 | N/A | N/A | unit | `cargo test -p narratoai-core --lib prompt::registry::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-01-02 | 01 | 1 | PRMP-02 | N/A | N/A | unit | `cargo test -p narratoai-core --lib prompt::template::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-01-03 | 01 | 1 | PRMP-04 | T-04-01 | 缺失变量返回错误而非空字符串 | unit | `cargo test -p narratoai-core --lib prompt::template::tests::test_missing_variable_error -- --nocapture` | ✅ | ✅ COVERED |
| 04-02-01 | 02 | 1 | VISL-01 | T-04-03 | FFmpeg 参数通过类型安全 API 构建 | unit | `cargo test -p narratoai-core --lib visual::frame_extractor::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-02-02 | 02 | 1 | VISL-02 | T-04-06 | deny_unknown_fields 防注入 | unit | `cargo test -p narratoai-core --lib visual::types::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-02-03 | 02 | 1 | VISL-03 | N/A | N/A | unit | `cargo test -p narratoai-core --lib visual::types::tests::test_batch_analysis_result_roundtrip -- --nocapture` | ✅ | ✅ COVERED |
| 04-03-01 | 03 | 2 | PRMP-03 | N/A | N/A | unit | `cargo test -p narratoai-core --lib prompt::register::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-03-02 | 03 | 2 | PRMP-04 | T-04-05 | validate_output 对 JSON/文案/剧情做格式校验 | unit | `cargo test -p narratoai-core --lib prompt::validators::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-03-03 | 03 | 2 | PRMP-01 | N/A | PromptManager 统一外观接口 | unit | `cargo test -p narratoai-core --lib prompt::manager::tests -- --nocapture` | ✅ | ✅ COVERED |
| 04-04-01 | 04 | 2 | VISL-02 | T-04-06 | parse_and_retry 剥离 markdown 代码块 | unit | `cargo test -p narratoai-core --lib visual::analyzer::tests -- --nocapture` | ✅ | ✅ COVERED |

*Status: ⬜ pending · ✅ COVERED · ❌ MISSING · ⚠️ PARTIAL*

---

## Wave 0 Requirements

- [x] `src/prompt/template.rs` — `#[cfg(test)] mod tests` with 14 tests (variable substitution + filters + error cases)
- [x] `src/prompt/registry.rs` — `#[cfg(test)] mod tests` with 7 registry CRUD operation tests
- [x] `src/visual/types.rs` — `#[cfg(test)] mod tests` with 10 mock JSON deserialization tests
- [x] `src/visual/frame_extractor.rs` — `#[cfg(test)] mod tests` with 9 tests (naming + fallback + unit tests)
- [x] `tests/llm_wiremock_test.rs` — integration tests for LLM provider (10 wiremock tests covering analyze_images)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 视觉分析质量 (场景覆盖、忠实度、细节密度) | VISL-02 | 需要人对实际视频的分析结果做质量判断 | 对 2 段测试视频 (10s简单+30s复杂) 的输出做人工审查 |
| 跨帧时间连贯性 | VISL-02 | 相邻帧对象列表的语义一致性需要人工判断 | 抽检标记的相邻帧对 (Jaccard < 0.2)，判定是否真的矛盾 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ✅ passed

---

## Validation Audit 2026-05-12

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total prompt tests | 57 |
| Total visual tests | 32 |
| Total lib tests | 572 passed, 1 ignored |

### Audit Notes

- Original VALIDATION.md had 4 test name mismatches (cosmetic): `test_missing_var` → `test_missing_variable_error`, `test_categories` → `test_list_categories`, `test_json_schema` → `test_deserialize_full_json`, `test_serde_roundtrip` → `test_batch_analysis_result_roundtrip`. All corrected.
- Plan/Wave columns were misaligned for rows 04-02-01 through 04-03-03. Remapped to actual PLAN dependencies.
- Added 3 missing rows: Plan 02 visual types (VISL-02/VISL-03), Plan 03 register/validators/manager coverage.
- Wave 0 integration test reference updated to existing `tests/llm_wiremock_test.rs` (Phase 2 wiremock tests covering the LlmProvider interface that Phase 4 depends on).
