---
phase: 11
slug: extended-features
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | built-in `#[test]` + `#[tokio::test]` |
| **Config file** | Cargo.toml declares `[dev-dependencies]` |
| **Quick run command** | `cargo test -p narratoai-core --lib -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-core --lib` (unit tests only, < 10s)
- **After every plan wave:** Run `cargo test` (all unit + non-ignored integration tests)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | EXTD-01 | T-11-01 | CLI argument injection prevention via args array | unit | `cargo test analyze_lufs` | ❌ W0 | ⬜ pending |
| 11-01-02 | 01 | 1 | EXTD-01 | T-11-01 | N/A | unit | `cargo test get_audio_rms` | ❌ W0 | ⬜ pending |
| 11-01-03 | 01 | 1 | EXTD-01 | T-11-01 | N/A | integration `#[ignore]` | `cargo test normalize_lufs -- --ignored` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 1 | EXTD-02 | T-11-01 | URL validation before CLI subprocess | unit | `cargo test resolution_matching` | ❌ W0 | ⬜ pending |
| 11-02-02 | 02 | 1 | EXTD-02 | T-11-01 | Use Command::args() not shell string | unit | `cargo test invalid_url` | ❌ W0 | ⬜ pending |
| 11-03-01 | 03 | 1 | EXTD-03 | T-11-02 | API origin validation | unit | `cargo test pexels_search` | ❌ W0 | ⬜ pending |
| 11-03-02 | 03 | 1 | EXTD-03 | T-11-02 | N/A | unit | `cargo test cache_key_md5` | ❌ W0 | ⬜ pending |
| 11-04-01 | 04 | 1 | EXTD-04 | — | N/A | unit | `cargo test volume_profiles` | ❌ W0 | ⬜ pending |
| 11-04-02 | 04 | 1 | EXTD-04 | — | N/A | unit | `cargo test volume_validation` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/fixtures/audio/` — test audio files (small WAV/MP3) for RMS tests
- [ ] `tests/fixtures/json/` — JSON fixtures for loudnorm output, Pexels response, Pixabay response, yt-dlp --dump-json
- [ ] `wiremock` in dev-dependencies for HTTP mock in material tests
- [ ] `symphonia` in dev-dependencies for RMS test execution
- [ ] test module stubs in `src/audio/`, `src/youtube/`, `src/material/`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| LUFS normalization audible quality | EXTD-01 | Subjective audio quality | Play output file, verify no distortion |
| yt-dlp actual download to disk | EXTD-02 | Requires real YouTube URL + network | Run `cargo test youtube_integration -- --ignored` |
| Pexels/Pixabay API live response | EXTD-03 | Requires valid API keys | Run `cargo test material_integration -- --ignored` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
