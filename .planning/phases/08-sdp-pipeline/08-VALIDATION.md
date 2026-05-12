---
phase: 08
slug: sdp-pipeline
status: audited
nyquist_compliant: false
wave_0_complete: true
created: 2026-05-05
audited: 2026-05-12
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | built-in `#[test]` |
| **Config file** | Cargo.toml dev-dependencies |
| **Quick run command** | `cargo test -p narratoai-core -- sdp::` |
| **Full suite command** | `cargo test -p narratoai-core` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-core -- sdp::`
- **After every plan wave:** Run `cargo test -p narratoai-core`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Tests Found | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | SDP-01 | N/A | subtitle parse + encoding detect | unit | `cargo test -- subtitle::` | 51 tests (parser: 30, error: 4, timestamp: 10, types: 2, sde re-export: 5) | ✅ COVERED |
| 08-02-01 | 02 | 1 | SDP-01 | T-08-003 | prompt template registration | unit | `cargo test -- prompt::register` | 4 tests (subtitle_analysis + plot_extraction assertions) | ✅ COVERED |
| 08-03-01 | 03 | 2 | SDP-01 | T-08-004 | LLM JSON repair (reuses Phase 7) | unit | `cargo test -- sde::script_gen` | 19 SDE tests covering repair_json 6-step + parse_script | ✅ COVERED (indirect) |
| 08-03-02 | 03 | 2 | SDP-01 | N/A | merge_script OST=1 | unit | `cargo test -- sdp::script_gen` | 12 tests (7 merge_script + 5 helpers) | ✅ COVERED |
| 08-04-01 | 04 | 3 | SDP-02 | N/A | sdp_step_clip (reuses sde) | unit | `cargo test -- sdp::clip` | 2 tests (precise correction + fallback) | ✅ COVERED |
| 08-04-02 | 04 | 3 | SDP-02 | T-08-006 | sdp_step_concat | unit | `cargo test -- sdp::pipeline` | 1 test (validation only) | ⚠️ PARTIAL |
| 08-04-03 | 04 | 3 | SDP-02 | N/A | sdp_step_composite (no TTS) | unit | N/A | 0 tests | ❌ MISSING |
| 08-04-04 | 04 | 3 | SDP-02 | T-08-008 | OST validation (all = 1) | unit | `cargo test -- sdp::pipeline` | 3 tests (all-ost1, non-ost1, mixed) | ✅ COVERED |
| 08-types | 03 | 2 | SDP-01 | N/A | SdpRequest validate + defaults + ProgressStep | unit | `cargo test -- sdp::types` | 7 tests (validate, defaults, progress, state) | ✅ COVERED |
| 08-error | 03 | 2 | SDP-01 | N/A | SdpError 中文错误消息 | unit | `cargo test -- sdp::error` | 6 tests (all variants) | ✅ COVERED |

*Status: ✅ COVERED · ⚠️ PARTIAL · ❌ MISSING · 🔧 MANUAL-ONLY*

---

## Wave 0 Requirements

- [x] `src/sdp/script_gen.rs` — 单元测试覆盖 repair_json 复用 (间接 via SDE 19 tests) + merge_script (7 tests) + helpers (5 tests)
- [x] `src/sdp/types.rs` — 单元测试覆盖 validate (3 tests) + default (2 tests) + ProgressStep order (1 test) + state init (1 test)
- [x] `src/sdp/error.rs` — 单元测试覆盖中文错误消息 (6 tests, 全部 6 变体)
- [x] `src/subtitle/mod.rs` — 单元测试覆盖 parse_subtitle_file (51 tests, 含 parser/timestamp/error/types)
- [ ] `src/sdp/pipeline.rs` — step_concat: 仅 validation test; step_composite: 无单元测试 (需 FFmpeg)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SDP 端到端视频输出质量 | SDP-01, SDP-02 | 视频内容的正确性无法自动化校验 | 用真实视频+字幕运行 run_sdp，检查输出视频的片段时序和音频是否正确 |
| sdp_step_composite FFmpeg 合成 | SDP-02 | FFmpeg filter_complex + amix 需要实际 FFmpeg 运行环境 + 视频/音频文件 | 准备拼接后的 combined_concat.mp4 + BGM 文件，运行 step_composite 检查输出视频的音频混音效果 |
| sdp_step_concat 文件拼接 | SDP-02 | FFmpeg concat demuxer 需要实际视频片段文件 | 准备裁剪后的视频片段 + concat_list.txt，运行 step_concat 检查拼接输出 |

---

## Test Counts Summary

| Module | Test Count | Status |
|--------|-----------|--------|
| sdp::error | 6 | ✅ all green |
| sdp::types | 7 | ✅ all green |
| sdp::script_gen | 12 | ✅ all green |
| sdp::clip | 2 | ✅ all green |
| sdp::pipeline | 7 | ✅ all green |
| **SDP total** | **34** | **all green** |
| subtitle::* | 51 | ✅ all green |
| sde::script_gen | 19 | ✅ all green (repair_json indirect) |
| prompt::register | 4 | ✅ all green |
| documentary::* | 59 | ✅ all green (no regression) |

---

## Validation Audit 2026-05-12

| Metric | Count |
|--------|-------|
| Requirements mapped | 10 |
| COVERED | 8 |
| PARTIAL | 1 (step_concat — validation test only) |
| MISSING | 1 (step_composite — needs FFmpeg, moved to Manual-Only) |
| Escalated to Manual-Only | 3 (e2e video quality, step_composite, step_concat) |

### Gap Details

**08-04-02 (step_concat) — PARTIAL:**
- Existing: `test_sdp_concat_validation_no_video_fails` (validates error on missing clips)
- Missing: concat list path escaping test, concat file content format test
- Mitigation: concat logic is straightforward (path replace + file write), low risk; full behavior verified in manual e2e test

**08-04-03 (step_composite) — MISSING:**
- No unit test exists for FFmpeg filter_complex composition
- Reason: requires actual FFmpeg binary + video/audio files; cannot be unit-tested in CI
- Mitigation: composite logic follows documentary::step_composite pattern (59 tests); FFmpeg command structure validated by code review

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies (2 items in Manual-Only for FFmpeg-dependent steps)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all COVERED references
- [x] No watch-mode flags
- [x] Feedback latency < 60s (~30s for full suite)
- [ ] `nyquist_compliant: true` — blocked by 1 PARTIAL + 1 MISSING (both FFmpeg-dependent, moved to Manual-Only)

**Approval:** partial — 8/10 requirements automated, 2 FFmpeg-dependent items in Manual-Only
