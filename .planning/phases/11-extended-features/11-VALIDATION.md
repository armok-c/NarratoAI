---
phase: 11
slug: extended-features
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-30
audited: 2026-05-12
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | built-in `#[test]` + `#[tokio::test]` |
| **Config file** | Cargo.toml declares `[dev-dependencies]` |
| **Quick run command** | `cargo test --lib -- --nocapture` |
| **Full suite command** | `cargo test --lib` |
| **Integration tests** | `cargo test --test audio_integration --test youtube_integration --test material_integration` |
| **Estimated runtime** | ~2 seconds (unit), integration tests require external services |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib` (unit tests only, < 5s)
- **After every plan wave:** Run `cargo test` (all unit + non-ignored integration tests)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Tests | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------|--------|
| 11-01-01 | 01 | 1 | EXTD-01 | T-11-01 | CLI argument injection prevention via args array | unit | `cargo test --lib audio::normalizer::tests` | 7 | ✅ green |
| 11-01-02 | 01 | 1 | EXTD-01 | T-11-01 | N/A | unit | `cargo test --lib audio::normalizer::tests` | 7 | ✅ green |
| 11-01-03 | 01 | 1 | EXTD-01 | T-11-01 | N/A | integration `#[ignore]` | `cargo test --test audio_integration -- --ignored` | 2 | ✅ green |
| 11-02-01 | 02 | 1 | EXTD-02 | T-11-01 | URL validation before CLI subprocess | unit | `cargo test --lib youtube::downloader::tests` | 12 | ✅ green |
| 11-02-02 | 02 | 1 | EXTD-02 | T-11-01 | Use Command::args() not shell string | unit | `cargo test --lib youtube::downloader::tests` | 12 | ✅ green |
| 11-03-01 | 03 | 1 | EXTD-03 | T-11-02 | API origin validation | unit | `cargo test --lib material::searcher::tests` | 11 | ✅ green |
| 11-03-02 | 03 | 1 | EXTD-03 | T-11-02 | N/A | unit | `cargo test --lib material::downloader::tests` | 5 | ✅ green |
| 11-04-01 | 04 | 1 | EXTD-04 | — | N/A | unit | `cargo test --lib audio::volume::tests` | 16 | ✅ green |
| 11-04-02 | 04 | 1 | EXTD-04 | — | N/A | unit | `cargo test --lib audio::volume::tests` | 16 | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Test Summary

| Module | Unit Tests | Integration Tests (#[ignore]) | Total |
|--------|-----------|-------------------------------|-------|
| audio::normalizer | 7 | 2 | 9 |
| audio::volume | 16 | — | 16 |
| youtube::downloader | 12 | 2 | 14 |
| material::searcher | 11 | — | 11 |
| material::downloader | 5 | 2 | 7 |
| **Total** | **51** | **6** | **57** |

Full suite result: **572 passed, 0 failed, 1 ignored** (2026-05-12)

---

## Wave 0 Requirements

- [x] `tests/fixtures/` — not needed; unit tests use tempfile/std::env::temp_dir
- [x] JSON fixtures for loudnorm/Pexels/Pixabay responses embedded in test functions
- [x] `reqwest::blocking` in dependencies for material HTTP tests
- [x] `symphonia` in dependencies for RMS test execution
- [x] All test modules verified in `src/audio/`, `src/youtube/`, `src/material/`
- [x] Integration test stubs in `tests/audio_integration.rs`, `tests/youtube_integration.rs`, `tests/material_integration.rs`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| LUFS normalization audible quality | EXTD-01 | Subjective audio quality | Play output file, verify no distortion |
| yt-dlp actual download to disk | EXTD-02 | Requires real YouTube URL + network | Run `cargo test --test youtube_integration -- --ignored` |
| Pexels/Pixabay API live response | EXTD-03 | Requires valid API keys | Run `cargo test --test material_integration -- --ignored` |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** complete

---

## Validation Audit 2026-05-12

| Metric | Count |
|--------|-------|
| Requirements total | 9 |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| COVERED | 9 |
| Manual-Only | 3 |

All 9 requirements have automated verification. 3 behaviors require manual testing (external services). Phase 11 is Nyquist-compliant.
