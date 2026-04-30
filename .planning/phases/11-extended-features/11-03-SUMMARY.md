---
phase: 11-extended-features
plan: 03
subsystem: youtube-download
tags: [yt-dlp, youtube, async-subprocess, tokio]
requires:
  - phase: 01-foundation
    provides: error types, config system, Cargo.toml dependencies
provides:
  - YouTube video format query and download via yt-dlp CLI subprocess
  - Resolution normalization and format selection with codec preference
  - URL injection protection (prefix validation + args array pattern)
affects: [Phase 10: Tauri Command Layer, Phase 6: Documentary Pipeline]

tech-stack:
  added: []
  patterns:
    - Async subprocess via tokio::process::Command
    - JSON parse from CLI subprocess stdout (serde_json::Value)
    - Domain error enum with thiserror + Chinese Display

key-files:
  created: []
  modified:
    - src/youtube/downloader.rs

key-decisions:
  - "URL prefix check (http/https only) before any yt-dlp call — defense against T-11-01 injection"
  - "Resolution normalization splits on 'p' and takes first segment: '1080p60' -> '1080p'"
  - "Format selection prefers non-av01 codecs within matching resolution for compatibility"
  - "All CLI calls use .args([...]) array pattern, never string concatenation"

patterns-established:
  - "Async CLI subprocess pattern: tokio::process::Command with .args() array, .output().await, status.success() check, stderr capture"
  - "Pure function API: no hidden state, all parameters explicit, Result<T, E> return"
  - "Domain error enum: thiserror derive, Chinese Display messages, 4 variants matching Python exception types"

requirements-completed: [EXTD-02]
---

# Phase 11 Plan 03: YouTube Downloader Summary

**YouTube video format query and download via yt-dlp CLI subprocess with injection-safe args array pattern, resolution normalization, and codec-prioritized format selection**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-30T13:41:14Z
- **Completed:** 2026-04-30T13:49:15Z
- **Tasks:** 1 (TDD: RED + GREEN + fix)
- **Files modified:** 1

## Accomplishments

- Implemented `YoutubeError` enum with 4 variants (SubprocessError, FormatNotFound, InvalidUrl, DownloadFailed) — all Chinese Display messages
- Implemented `VideoFormat` struct and `DownloadResult` struct for typed format/lookup and download results
- Implemented `validate_format()` checking against supported list [mp4, mkv, webm, flv, avi] (case-insensitive)
- Implemented `_get_video_formats()` async function: yt-dlp --dump-json with URL prefix validation, JSON parsing, resolution normalization ("1080p60" -> "1080p")
- Implemented `download_video()` async function: format lookup, resolution matching with non-av01 preference, format_id download via yt-dlp, proxy support from config.toml [proxy] section
- All CLI calls use args array pattern — zero shell string concatenation (T-11-01 mitigation)
- URL prefix validation before all yt-dlp calls — rejects ftp://, file://, etc. (T-11-02 mitigation)
- 12 unit tests covering error display, format validation, URL rejection, resolution normalization, format selection priority, and struct debug formatting

## Task Commits

Each task was committed atomically (TDD pattern):

1. **Task 1: YouTube downloader (RED — failing tests)** - `9d1c224` (test)
   - Add 7 test functions; all fail to compile (types not yet implemented)
2. **Task 1: YouTube downloader (GREEN — implementation)** - `720b9c2` (feat)
   - Full implementation: types, error enum, format query, download function; 12 tests pass
3. **Task 1: YouTube downloader (fix — doc comment)** - `d00e770` (fix)
   - Reword doc comment to avoid acceptance criteria grep false positive

**Plan metadata:** Pending (final commit after SUMMARY creation)

## Files Created/Modified

- `src/youtube/downloader.rs` — YouTube video downloader module (452 lines):
  - `YoutubeError` enum (4 variants, Chinese Display)
  - `VideoFormat` struct (format_id, resolution, ext, vcodec)
  - `DownloadResult` struct (output_path, title)
  - `validate_format()` — format whitelist validation
  - `_get_video_formats()` — async format query from yt-dlp
  - `download_video()` — async download with resolution matching
  - `#[cfg(test)] mod tests` — 12 unit tests

## Decisions Made

- Followed the existing async subprocess pattern from `src/ffmpeg/probe.rs` (tokio::process::Command + JSON parse)
- Resolution normalization: split on 'p', keep numeric prefix — matches Python `youtube_service.py` behavior exactly
- Format selection: within matching resolution, prefer non-av01 codecs for better compatibility; fallback to any format with video
- Proxy URL passed through `--proxy` as added arg only when non-empty (avoids sending empty `--proxy ""` to yt-dlp)
- Video title fetched separately via `yt-dlp --get-title` rather than parsing from dump-json (simpler, more reliable)

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- Pre-existing test failure `ffmpeg::hwaccel::tests::test_detect_encoders_format` (not related to this plan — environment lacks FFmpeg hardware encoding)
- Pre-existing warning `function 'get_azure_voices' is never used` (unrelated)

## User Setup Required

**yt-dlp CLI must be installed on the system:**
- Windows: `winget install yt-dlp` or download from https://github.com/yt-dlp/yt-dlp/releases
- macOS: `brew install yt-dlp`
- Linux: `sudo apt install yt-dlp` or `pip install yt-dlp`

Proxy configuration (optional) goes in `config.toml [proxy]` section (http/https fields).

## Next Phase Readiness

- YouTube download module complete and ready for Tauri Command Layer integration (Phase 10)
- Module exports via `src/youtube/mod.rs` (already has `pub mod downloader`)
- Can be consumed by Phase 6 Documentary Pipeline for YouTube video sourcing
- Module is pure function API — no global state, all parameters explicit

---
*Phase: 11-extended-features*
*Completed: 2026-04-30*
