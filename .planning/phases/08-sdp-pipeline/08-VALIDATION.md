---
phase: 08
slug: sdp-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-05
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | built-in `#[test]` |
| **Config file** | Cargo.toml dev-dependencies |
| **Quick run command** | `cargo test -p narratoai-core -- sdp` |
| **Full suite command** | `cargo test -p narratoai-core` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-core -- sdp`
- **After every plan wave:** Run `cargo test -p narratoai-core`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | SDP-01 | N/A | subtitle parse + encoding detect | unit | `cargo test -p narratoai-core -- subtitle` | ❌ W0 | ⬜ pending |
| 08-03-01 | 03 | 2 | SDP-01 | N/A | LLM JSON repair (reuses Phase 7) | unit | `cargo test -p narratoai-core -- sdp::script_gen` | ❌ W0 | ⬜ pending |
| 08-03-02 | 03 | 2 | SDP-01 | N/A | merge_script OST=1 | unit | `cargo test -p narratoai-core -- sdp::script_gen` | ❌ W0 | ⬜ pending |
| 08-04-01 | 04 | 3 | SDP-02 | N/A | sdp_step_clip (reuses sde) | unit | `cargo test -p narratoai-core -- sde::clip` | ✅ | ⬜ pending |
| 08-04-02 | 04 | 3 | SDP-02 | N/A | sdp_step_concat | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ W0 | ⬜ pending |
| 08-04-03 | 04 | 3 | SDP-02 | N/A | sdp_step_composite (no TTS) | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ W0 | ⬜ pending |
| 08-04-04 | 04 | 3 | SDP-02 | N/A | OST validation (all = 1) | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 1 | SDP-02 | N/A | input validation (files exist, non-empty, OST=1) | unit | `cargo test -p narratoai-core -- sdp::types` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/sdp/script_gen.rs` — 单元测试覆盖 repair_json 复用 + merge_script + parse_script
- [ ] `src/sdp/pipeline.rs` — 单元测试覆盖 run_sdp 编排 + step_concat + step_composite
- [ ] `src/sdp/types.rs` — 单元测试覆盖 validate + default + ProgressStep order
- [ ] `src/sdp/error.rs` — 单元测试覆盖中文错误消息
- [ ] `src/subtitle/mod.rs` — 单元测试覆盖 parse_subtitle_file (从 SDE 迁移)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SDP 端到端视频输出质量 | SDP-01, SDP-02 | 视频内容的正确性无法自动化校验 | 用真实视频+字幕运行 run_sdp，检查输出视频的片段时序和音频是否正确 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
