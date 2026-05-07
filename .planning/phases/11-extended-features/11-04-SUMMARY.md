---
phase: 11-extended-features
plan: 04
type: execute
wave: 2
subsystem: "material"
tags: ["pexels", "pixabay", "material-search", "video-download", "md5-cache", "ffprobe-validation"]
requires: ["01-module-scaffold"]
provides: ["material-searcher", "material-downloader"]
affects: ["src/material/searcher.rs", "src/material/downloader.rs", "Cargo.toml"]
tech-stack:
  added: ["reqwest blocking feature (0.13)"]
  patterns: ["reqwest blocking Client with proxy", "MD5 URL cache key", "temp file + atomic rename + ffprobe validation"]
key-files:
  created: []
  modified:
    - narratoai-core/src/material/searcher.rs
    - narratoai-core/src/material/downloader.rs
    - Cargo.toml
decisions:
  - reqwest blocking Client used instead of async (sync functions in library context; tests use #[test] not #[tokio::test])
  - search_videos_pixabay uses large/medium/small/tiny variant cascade instead of resolution matching (Pixabay does not provide resolution-filtered results)
  - choose_variant() extracted as standalone helper function to avoid lifetime issues with closure captures
  - download_videos uses HashSet<String> for URL dedup (not sorted — order preservation not required)
  - ffprobe validation in save_video uses existing probe_audio() (treats video as audio container for duration check)
duration: 17m
completed_date: "2026-04-30"
commits:
  - "9b26a47: feat(11-04): implement material downloader — MD5 cache, save_video, download_videos"
  - "26bb4f0: test(11-04): add failing tests for material downloader"
  - "4cd1ea1: feat(11-04): implement material searcher — MaterialError, Pexels/Pixabay search, key rotation"
  - "f557904: test(11-04): add failing tests for material searcher"
---

# Phase 11 Plan 04: Pexels/Pixabay Material Search & Download Summary

## One-Liner

Pexels/Pixabay material search and download module implemented: MaterialError enum with 6 Chinese variants, Pexels/Pixabay API response types with serde Deserialize, AtomicUsize-based API key round-robin rotation, reqwest blocking Client with proxy support, MD5 URL cache key (strip query string), temp-file + atomic rename + ffprobe-validate save flow, and orchestrated search-dedup-download with audio_duration limit breakout.

## Objective

Implement Pexels/Pixabay material search and download capabilities (EXTD-03) by replacing stubs in `src/material/searcher.rs` and `src/material/downloader.rs` with full implementations. Functions are 1:1 aligned with Python `app/services/material.py`: `get_api_key()` key rotation, `search_videos_pexels()`/`search_videos_pixabay()` API calls with resolution/duration filtering, `save_video()` with MD5 URL cache and ffprobe validation, `download_videos()` orchestration with dedup and duration limit.

## Tasks

| # | Name | Type | Status | TDD Gate | Commit |
|---|------|------|--------|----------|--------|
| 1 | Material searcher — MaterialError, API types, Pexels/Pixabay search, key rotation | tdd=auto | Done | RED `f557904` / GREEN `4cd1ea1` | `4cd1ea1` |
| 2 | Material downloader — MD5 cache, save_video, download_videos, ffprobe validation | tdd=auto | Done | RED `26bb4f0` / GREEN `9b26a47` | `9b26a47` |

## Verification

- `cargo test --lib material::searcher::tests` -- 11/11 PASS
- `cargo test --lib material::downloader::tests` -- 5/5 PASS
- `cargo test --lib material::` -- 16/16 PASS (all material tests, merging with jianying::material::* for total 23/23)
- `cargo test --lib` -- 335/337 PASS, 1 pre-existing failure (`ffmpeg::hwaccel::tests::test_detect_encoders_format` -- system-dependent, unrelated)
- `cargo check` -- zero errors (1 pre-existing warning `get_azure_voices` dead_code, unrelated)

### Acceptance Criteria Check

| Criterion | Status |
|-----------|--------|
| `pub enum MaterialError` with 6 variants | PASS — 1 match |
| `pub fn get_api_key` with AtomicUsize | PASS — 1 match |
| `pub fn search_videos_pexels` | PASS — 1 match |
| `pub fn search_videos_pixabay` | PASS — 1 match |
| `pexels.com/videos/search` URL in code | PASS — 2 matches |
| `pixabay.com/api/videos` URL in code | PASS — 2 matches |
| `AtomicUsize` for key rotation | PASS — 11 matches |
| `pub struct PexelsSearchResponse` | PASS — 1 match |
| `pub struct PixabaySearchResponse` | PASS — 1 match |
| `pub fn compute_cache_key` (MD5) | PASS — 1 match |
| `pub fn save_video` | PASS — 1 match |
| `pub fn download_videos` | PASS — 1 match |
| `Md5` usage for cache | PASS — 2 matches |
| `probe_audio` for ffprobe validation | PASS — 2 matches |
| `rename(&temp` atomic rename | PASS — 1 match |
| `total_duration >= audio_duration` limit break | PASS — 1 match |

## Key Architecture Decisions

1. **reqwest blocking Client**: The material search functions are synchronous (no `async`), using `reqwest::blocking::Client`. The `"blocking"` feature was added to `Cargo.toml`. This matches the library context where callers may not be in an async runtime, and avoids the need for `#[tokio::test]` in unit tests. The blocking Client supports all reqwest features used (proxy, headers, timeouts, JSON deserialization).

2. **Pixabay variant cascade**: Pixabay API returns video variants (large/medium/small/tiny) as optional fields rather than a flat list. The implementation selects the best available variant via ordering preference (large > medium > small > tiny). Unlike Pexels, Pixabay does not return per-video-file resolution metadata, so resolution filtering is done at the `search_videos_pexels` level only.

3. **choose_variant() as standalone function**: The closure-based implementation of `choose_variant` caused Rust lifetime errors due to conflicting borrows of `or_else` chaining. Extracting it as a standalone `fn` resolved the lifetime issue cleanly.

4. **MD5 URL cache key**: `compute_cache_key()` strips the URL query string before hashing (matching Python behavior exactly). This ensures that CDN URLs with temporally-varying tokens produce consistent cache keys. Cache hit reuses existing files on disk, avoiding redundant downloads for repeated tasks.

5. **ffprobe validation**: `save_video()` uses the existing `probe_audio()` function from `src/ffmpeg/probe.rs` to validate downloaded video files. While `probe_audio` is named for audio, ffprobe's `show_entries format=duration` works on any container format (MP4, WebM, etc.) and validates basic file integrity by checking duration > 0.

6. **AtomicUsize key rotation**: `get_api_key()` uses `AtomicUsize::fetch_add(1, Ordering::Relaxed)` with modulo for round-robin rotation. This is an improvement over the Python version (which used a non-thread-safe module-level counter) and prevents race conditions in async/multi-threaded contexts.

## Deviations from Plan

None — plan executed exactly as written. Both stubs from 11-01 are now replaced with full implementations.

### TDD Gate Compliance

Both tasks follow RED/GREEN cycle:

| Task | RED commit | GREEN commit |
|------|-----------|-------------|
| 1. searcher | `f557904` test(11-04): add failing tests for material searcher | `4cd1ea1` feat(11-04): implement material searcher |
| 2. downloader | `26bb4f0` test(11-04): add failing tests for material downloader | `9b26a47` feat(11-04): implement material downloader |

No REFACTOR phase needed — code is clean with no warnings.

## Known Stubs

Previous stubs from 11-01 are now resolved for the material module:

| File | Before | After |
|------|--------|-------|
| src/material/searcher.rs | `search_placeholder()` stub | Full implementation (470 lines) |
| src/material/downloader.rs | `download_placeholder()` stub | Full implementation (284 lines) |

Remaining stubs in `src/audio/normalizer.rs`, `src/audio/volume.rs`, `src/youtube/downloader.rs` are intentional — real implementation in plans 11-02 and 11-03.

## Deferred Issues

1. **ffmpeg::hwaccel::tests::test_detect_encoders_format** — Pre-existing test failure unrelated to this plan. Requires FFmpeg with hardware encoder support. Documented here for awareness; does not affect plan correctness.

## Threat Surface

No new threat flags introduced beyond the plan's threat model. The `save_video()` function mitigates T-11-03 (DoS via large download) with a 300-second request timeout. File paths for ffprobe validation are constructed internally from `save_dir + cache_key.mp4`, not from user input (mitigates T-11-04). API response types use serde `Deserialize` with strict field matching (mitigates T-11-01 spoofing from malformed API responses).

## Self-Check: PASSED

All created/modified files verified:

- `src/material/searcher.rs` — 470 lines, all required types and functions present
- `src/material/downloader.rs` — 284 lines, all required functions present
- `Cargo.toml` — reqwest blocking feature added

All commits verified in git log:
- `f557904` — RED searcher — confirmed
- `4cd1ea1` — GREEN searcher — confirmed
- `26bb4f0` — RED downloader — confirmed
- `9b26a47` — GREEN downloader — confirmed

All `material::` tests pass: 16/16 (searcher: 11, downloader: 5). 23/23 including existing `jianying::material::*` tests.
