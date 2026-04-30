---
phase: 11-extended-features
plan: 01
type: execute
wave: 1
subsystem: "config, audio, youtube, material"
tags: ["infrastructure", "module-scaffold", "config-extension"]
requires: []
provides: ["audio-module", "youtube-module", "material-module", "audio-section-config"]
affects: ["Cargo.toml", "config.example.toml", "lib.rs", "config/types.rs", "config/defaults.rs"]
tech-stack:
  added: ["symphonia 0.5 (mp3/flac/pcm/wav/vorbis)", "md-5 0.10"]
  patterns: ["serde config section", "spawn_blocking subprocess", "thiserror domain errors"]
key-files:
  created:
    - src/audio/mod.rs
    - src/audio/normalizer.rs
    - src/audio/volume.rs
    - src/youtube/mod.rs
    - src/youtube/downloader.rs
    - src/material/mod.rs
    - src/material/searcher.rs
    - src/material/downloader.rs
  modified:
    - Cargo.toml
    - src/config/types.rs
    - src/config/defaults.rs
    - config.example.toml
    - src/lib.rs
decisions:
  - symphonia feature set: mp3, flac, pcm, wav, vorbis (no AAC -- deferred)
  - AudioSection uses deny_unknown_fields + #[serde(default)] matching existing FramesSection pattern
  - AudioSection default target_lufs: -23.0 (EBU R128 broadcast standard)
  - Module stubs contain placeholder functions to avoid Rust unused-module warnings
duration: 375s
completed_date: "2026-04-30"
commits:
  - "71e4256: feat(11-extended-features): add symphonia + md-5 deps, AudioSection, pexels/pixabay keys"
  - "720fcb4: feat(11-extended-features): create module stubs and lib.rs exports"
---

# Phase 11 Plan 01: Extended Features Infrastructure

## One-Liner

Cargo.toml dependency additions (symphonia + md-5), AudioSection config struct with 12 fields, AppSection extension with pexels/pixabay API key arrays, config.example.toml [audio] section, and 8 module stub files across 3 new top-level modules (audio/youtube/material) with lib.rs exports -- all passing cargo check.

## Objective

Establish infrastructure for all four Phase 11 extended feature modules: add symphonia + md-5 dependencies, extend config types (AudioSection struct, Pexels/Pixabay API key arrays in AppSection), create module directories/stubs/mod.rs files for audio, youtube, and material modules, update lib.rs exports, update config.example.toml, and ensure cargo check passes.

## Tasks

| # | Name | Type | Status | Commit |
|---|------|------|--------|--------|
| 1 | Add dependencies, extend config types, update example config | auto | Done | `71e4256` |
| 2 | Create module stubs + lib.rs export + verify compile | auto | Done | `720fcb4` |

## Verification

- `cargo check` -- PASS (no errors, only pre-existing `get_azure_voices` dead_code warning)
- `cargo test --lib config::types::tests` -- 5/5 PASS (all config tests including new [audio] section assertions)
- `cargo test --lib` -- 319/321 PASS, 1 pre-existing failure (`ffmpeg::hwaccel::tests::test_detect_encoders_format` -- system-dependent FFmpeg hardware encoder detection, unrelated to this plan)

### Acceptance Criteria Check

| Criterion | Status |
|-----------|--------|
| `cargo check` zero error | PASS |
| symphonia in Cargo.toml | PASS |
| pub struct AudioSection in types.rs | PASS |
| pexels_api_keys in types.rs | PASS |
| impl Default for AudioSection in defaults.rs | PASS |
| [audio] in config.example.toml | PASS |
| pub mod audio in lib.rs | PASS |
| pub mod normalizer in audio/mod.rs | PASS |
| pub mod volume in audio/mod.rs | PASS |
| pub mod downloader in youtube/mod.rs | PASS |
| pub mod searcher in material/mod.rs | PASS |
| pub mod downloader in material/mod.rs | PASS |

## Key Architecture Decisions

1. **AudioSection design**: Follows existing `FramesSection` pattern -- `#[serde(deny_unknown_fields)]` + all fields `#[serde(default)]`. 12 fields covering LUFS normalization, volume presets, sample rate/channels/bitrate, crossfade, and BG fadeout.

2. **Module stub design**: Each stub file contains a `pub fn *_placeholder() -> bool { true }` function to prevent Rust's `unused module` warning. Real implementations will replace these in Plans 2-4.

3. **symphonia features**: Enabled only `["mp3", "flac", "pcm", "wav", "vorbis"]` -- no AAC. AAC can be added if RMS fallback encounters AAC-encoded audio in practice.

4. **API key arrays**: `pexels_api_keys: Vec<String>` and `pixabay_api_keys: Vec<String>` in `AppSection` support multi-key rotation (round-robin by atomic counter in future plans).

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

| File | Function | Reason |
|------|----------|--------|
| src/audio/normalizer.rs | `normalize_placeholder()` | Intentional -- real implementation in 11-02-PLAN.md |
| src/audio/volume.rs | `volume_placeholder()` | Intentional -- real implementation in 11-02-PLAN.md |
| src/youtube/downloader.rs | `youtube_placeholder()` | Intentional -- real implementation in 11-03-PLAN.md |
| src/material/searcher.rs | `search_placeholder()` | Intentional -- real implementation in 11-04-PLAN.md |
| src/material/downloader.rs | `download_placeholder()` | Intentional -- real implementation in 11-04-PLAN.md |

## Deferred Issues

1. **ffmpeg::hwaccel::tests::test_detect_encoders_format** -- Pre-existing test failure unrelated to this plan. Requires FFmpeg with hardware encoder support on the test machine. Noted here as out-of-scope; does not affect this plan's correctness.

## Threat Surface

No new threat flags introduced. The `AudioSection` config extension follows existing `#[serde(deny_unknown_fields)]` pattern (mitigates T-11-01). symphonia and md-5 are mature libraries with no known CVEs affecting this usage (T-11-02 accepted risk).

## Self-Check: PASSED

All artifact files verified:
- Cargo.toml -- symphonia dependency present
- src/config/types.rs -- AudioSection struct, pexels_api_keys, pixabay_api_keys present
- src/config/defaults.rs -- impl Default for AudioSection present
- config.example.toml -- [audio] section with 11 fields present
- src/lib.rs -- pub mod audio, youtube, material present
- src/audio/mod.rs -- pub mod normalizer, volume present
- src/youtube/mod.rs -- pub mod downloader present
- src/material/mod.rs -- pub mod searcher, downloader present
- All 8 stub files exist

All commits verified in git log:
- `71e4256` -- confirmed
- `720fcb4` -- confirmed
