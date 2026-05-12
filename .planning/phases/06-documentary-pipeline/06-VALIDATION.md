---
phase: 6
slug: documentary-pipeline
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-02
audited: 2026-05-12
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `narratoai-core/Cargo.toml` |
| **Quick run command** | `cargo test --lib documentary` |
| **Full suite command** | `cargo test -p narratoai-core` |
| **Integration command** | `cargo test --test documentary_integration_test` |
| **Estimated runtime** | ~2 seconds (unit), ~15 seconds (integration) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib documentary`
- **After every plan wave:** Run `cargo test -p narratoai-core`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Test Count | File | Status |
|---------|------|------|-------------|-----------|------------|------|--------|
| 06-01-01 | 01 | 1 | DOCS-01, DOCS-03 | unit | 35 | error.rs (14), timestamp.rs (21) | ✅ |
| 06-01-02 | 01 | 1 | FFMP-07 | unit | 5 | subtitle.rs (5) | ✅ |
| 06-02-01 | 02 | 2 | FFMP-02~08, DOCS-01 | unit | 9 | pipeline.rs (3), clip.rs (3), audio.rs (3) | ✅ |
| 06-02-02 | 02 | 2 | FFMP-08 | unit | (included above) | pipeline.rs | ✅ |
| 06-03-01 | 03 | 2 | DOCS-02 | unit | 10 | script_gen.rs (10) | ✅ |
| 06-03-02 | 03 | 2 | DOCS-02 | unit | (included above) | script_gen.rs | ✅ |
| 06-04-01 | 04 | 3 | All 10 reqs | integration | 9 | documentary_integration_test.rs (9) | ✅ |
| 06-04-02 | 04 | 3 | All 10 reqs | verification | — | (test run) | ✅ |

**Totals: 60 unit tests + 9 integration tests = 69 tests, 0 failures.**

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Requirement Traceability

| Req | Description | Covered By | Tests |
|-----|-------------|------------|-------|
| FFMP-02 | 视频裁剪（OST 分支） | clip.rs: clip_ost0/1/2 | 3 clip tests + pipeline tests |
| FFMP-03 | TTS 音频生成集成 | pipeline.rs: step_tts | pipeline unit tests |
| FFMP-04 | 音频合并（TTS 定位） | audio.rs: merge_audio_files | 3 audio tests |
| FFMP-05 | 视频片段拼接 | pipeline.rs: step_concat | pipeline unit tests |
| FFMP-06 | 最终合成（音频+字幕烧录） | pipeline.rs: step_composite | pipeline unit tests |
| FFMP-07 | 字幕生成（SRT） | subtitle.rs: generate_srt_from_word_boundaries | 5 subtitle tests + integration test |
| FFMP-08 | 流水线编排 | pipeline.rs: run_documentary | 3 pipeline tests + integration test |
| DOCS-01 | 纪录片模块结构 | mod.rs: 9 sub-modules | error tests (14) |
| DOCS-02 | 帧分析→文案生成 | script_gen.rs: generate_documentary_script | 10 script_gen tests |
| DOCS-03 | 错误处理 | error.rs: PipelineError 14 variants | 14 error tests |

---

## Wave 0 Requirements

- [x] `narratoai-core/tests/documentary_integration_test.rs` — 9 integration tests
- [x] `narratoai-core/tests/common/mod.rs` — ffmpeg_available() helper

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Final video plays correctly in media player | DOCS-01 | Requires visual/audio verification | Play output video, verify narration syncs with frames |
| SRT subtitle timing matches audio | FFMP-04 | Requires human perception check | Play with subtitles enabled, verify word timing |

---

## Audit Findings (2026-05-12)

### Fixed

1. **Integration test discoverability** — `tests/documentary_integration_test.rs` and `tests/common/mod.rs` were in root workspace `tests/` dir (no root package), making them invisible to Cargo. Copied to `narratoai-core/tests/`. 9 tests now discoverable and passing.

### Observations (not blocking)

1. Root `tests/` directory has 8 orphaned test files from other phases (audio_integration, config_test, ffmpeg_test, llm_test, material_integration, script_test, youtube_integration) — same discoverability issue, not Phase 6 scope.

---

## Code Review History

| Iter | Date | Critical | Warning | Info | Status |
|------|------|----------|---------|------|--------|
| 1-9 | 2026-05-07/08 | various | various | various | partial → found |
| 10 | 2026-05-08 | 0 | 0 | 5 | fixes applied (IN-01 dead code, IN-02 ProgressStep, IN-04 SRT seq) |
| 11 | 2026-05-08 | 0 | 0 | 2 | clean — IN-03/IN-05 skip maintained |

---

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all references
- [x] No watch-mode flags
- [x] Feedback latency < 15s (well under 120s threshold)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved (2026-05-12)
