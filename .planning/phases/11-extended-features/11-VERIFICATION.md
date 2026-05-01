---
phase: 11-extended-features
verified: 2026-04-30T15:00:00Z
status: gaps_found
score: 35/35 must-haves verified
overrides_applied: 0
overrides: []
gaps:
  - truth: "RMS 回退 fallback 路径中 get_audio_rms() 计算 dB 值时使用 i16::MAX (32767) 作为 f32 采样参考电平，导致 RMS 值偏低约 90 dB，回退时产生巨量增益放大"
    status: failed
    reason: "src/audio/normalizer.rs:288 使用 20.0 * (rms / (i16::MAX as f64)).log10()，但 f32 采样范围是 [-1.0, 1.0]，参考电平应为 1.0。当 loudnorm 两遍标准化失败触发 RMS 回退时，gain_db = target_lufs - rms_db 会产生约 +90 dB 的过放大，导致音频削波/失真"
    artifacts:
      - path: "src/audio/normalizer.rs"
        issue: "第 288 行：get_audio_rms() 将 i16::MAX (32767) 作为 f32 采样的参考电平，正确值应为 1.0。此 bug 由代码审查发现 (CR-01)"
    missing:
      - "将 288 行 `20.0 * (rms / (i16::MAX as f64)).log10()` 改为 `20.0 * rms.log10()`（f32 采样参考电平为 1.0）"
---

# Phase 11: Extended Features Verification Report

**Phase Goal:** 补齐 Python 版的辅助功能——音频标准化、YouTube 下载、Pexels 素材搜索、智能音量控制
**Verified:** 2026-04-30T15:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

Total must-haves from all 5 plans: 35 truths across infrastructure (5), audio normalization (10), YouTube (7), material (8), integration tests (4).

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Cargo.toml includes symphonia + md-5 dependencies | VERIFIED | `Cargo.toml:37-38` -- symphonia 0.5 with mp3/flac/pcm/wav/vorbis, md-5 0.10 |
| 2 | config parses [audio] section and target fields | VERIFIED | `src/config/types.rs:206-231` -- AudioSection with 12 fields; `src/config/types.rs:344-349` test assertions pass |
| 3 | config parses pexels_api_keys and pixabay_api_keys | VERIFIED | `src/config/types.rs:62-64` -- Vec<String> fields in AppSection; test assertion at line 348-349 passes |
| 4 | lib.rs exports pub mod audio/youtube/material | VERIFIED | `src/lib.rs:10-12` -- pub mod audio; pub mod youtube; pub mod material; |
| 5 | All 3 module subdirectories have mod.rs + submodule stubs, cargo check passes | VERIFIED | `src/audio/mod.rs` (normalizer+volume), `src/youtube/mod.rs` (downloader), `src/material/mod.rs` (searcher+downloader) -- all exist; `cargo check` 0 errors |
| 6 | AudioError enum has 6+ variants, Chinese Display | VERIFIED | `src/audio/normalizer.rs:25-43` -- 6 variants, all Chinese Display |
| 7 | LoudnormData deserializes from FFmpeg loudnorm JSON, includes target_offset | VERIFIED | `src/audio/normalizer.rs:57-96` -- serde struct with all 10 fields including target_offset |
| 8 | analyze_lufs() calls FFmpeg loudnorm subprocess, parses JSON | VERIFIED | `src/audio/normalizer.rs:122-157` -- Command::new(ffmpeg) with loudnorm filter, stderr JSON extraction |
| 9 | normalize_lufs() performs two-pass loudnorm (analysis->apply), includes target_offset | VERIFIED | `src/audio/normalizer.rs:163-215` -- calls analyze_lufs then builds second-pass with measured values including offset |
| 10 | get_audio_rms() decodes via symphonia -> PCM f32 -> RMS dB | VERIFIED | `src/audio/normalizer.rs:221-293` -- symphonia decode loop, RMS calculation (note: formula bug at line 288) |
| 11 | calculate_volume_adjustment() computes TTS and original gain | VERIFIED | `src/audio/normalizer.rs:300-310` -- pure function, clamp [0.1,2.0]/[0.1,3.0] |
| 12 | normalize_audio_for_mixing() orchestrates and returns output PathBuf | VERIFIED | `src/audio/normalizer.rs:316-356` -- loudnorm first, RMS fallback, output path generation |
| 13 | VolumeConfig type-safe struct with tts/original/bgm volume | VERIFIED | `src/audio/volume.rs:19-23` -- Debug+Clone+Copy+PartialEq struct |
| 14 | 4 volume functions: get_optimized_volumes/apply_volume_profile/get_recommended_volumes_for_content/validate_volume | VERIFIED | `src/audio/volume.rs:107-213` -- all 4 functions with full implementations |
| 15 | Volume profile values 1:1 match Python AudioConfig class | VERIFIED | `src/audio/volume.rs:136-163` -- balanced/voice_focused/original_focused/quiet_background profiles hardcoded |
| 16 | YoutubeError enum has 4+ variants, Chinese Display | VERIFIED | `src/youtube/downloader.rs:14-30` -- 4 variants, all Chinese Display |
| 17 | download_video() async via tokio::process::Command calling yt-dlp | VERIFIED | `src/youtube/downloader.rs:170-272` -- pub async fn, tokio::process::Command |
| 18 | _get_video_formats() parses yt-dlp --dump-json output | VERIFIED | `src/youtube/downloader.rs:87-144` -- async fn, serde_json::Value parsing of formats array |
| 19 | Resolution matching supports "1080p60" -> "1080p" normalization | VERIFIED | `src/youtube/downloader.rs:127-132` (in _get_video_formats) and `186-191` (in download_video) |
| 20 | Output format validated against allowed list (mp4/mkv/webm/flv/avi) | VERIFIED | `src/youtube/downloader.rs:62-73` -- validate_format(), case-insensitive |
| 21 | Proxy read from config.toml [proxy] section and passed to yt-dlp --proxy | VERIFIED | `src/youtube/downloader.rs:99-103` (in _get_video_formats) and `241-244` (in download_video) |
| 22 | All CLI calls use args array, no shell strings | VERIFIED | `src/youtube/downloader.rs` -- all Command::new().args([...]) patterns; grep for `shell(true)` returns 0 |
| 23 | MaterialError enum has 4+ variants, Chinese Display | VERIFIED | `src/material/searcher.rs:20-38` -- 6 variants, all Chinese Display |
| 24 | search_videos_pexels() calls Pexels API /videos/search via reqwest | VERIFIED | `src/material/searcher.rs:185-246` -- reqwest::blocking::Client, pexels.com/videos/search |
| 25 | search_videos_pixabay() calls Pixabay API /api/videos/ via reqwest | VERIFIED | `src/material/searcher.rs:251-310` -- reqwest::blocking::Client, pixabay.com/api/videos/ |
| 26 | get_api_key() rotates from Vec<String> with AtomicUsize | VERIFIED | `src/material/searcher.rs:128-137` -- AtomicUsize::fetch_add, round-robin |
| 27 | Pexels/Pixabay API response types with serde deserialization | VERIFIED | `src/material/searcher.rs:44-106` -- PexelsSearchResponse, PixabaySearchResponse with serde |
| 28 | save_video() uses MD5(url_without_query) cache key, skips if exists | VERIFIED | `src/material/downloader.rs:24-30` (compute_cache_key), `45-122` (save_video with cache hit check) |
| 29 | download_videos() orchestrates search+download+duration sum, stops after audio_duration | VERIFIED | `src/material/downloader.rs:134-212` -- search + dedup + download + duration break |
| 30 | Downloaded video validated by ffprobe (duration > 0) | VERIFIED | `src/material/downloader.rs:102-118` -- ffprobe_audio() validation, cleanup on failure |
| 31 | tests/audio_integration.rs exists with #[ignore] tests | VERIFIED | `tests/audio_integration.rs` -- 2 #[ignore] tests with find_ffmpeg |
| 32 | tests/youtube_integration.rs exists with #[ignore] tests | VERIFIED | `tests/youtube_integration.rs` -- 2 #[ignore] tests with find_ytdlp |
| 33 | tests/material_integration.rs exists with #[ignore] tests | VERIFIED | `tests/material_integration.rs` -- 2 #[ignore] tests with env var checks |
| 34 | All 3 integration test files compile | VERIFIED | `cargo test --test audio_integration --test youtube_integration --test material_integration --no-run` compiles 3 executables, 0 errors |
| 35 | VolumeConfig validate() checks range [0.0, 2.0] | VERIFIED | `src/audio/volume.rs:27-41` -- validate() returns AudioError::InvalidVolume on out-of-range |

**Score:** 35/35 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `Cargo.toml` | symphonia + md-5 deps | VERIFIED | Line 37-38: symphonia 0.5 (mp3/flac/pcm/wav/vorbis), md-5 0.10 |
| `src/config/types.rs` | AudioSection + pexels/pixabay keys | VERIFIED | AudioSection 12 fields (line 206-231), pexels_api_keys/pixabay_api_keys in AppSection (62-64) |
| `src/config/defaults.rs` | AudioSection Default impl | VERIFIED | Impl Default for AudioSection at line 111-128 |
| `src/audio/mod.rs` | pub mod normalizer; pub mod volume; | VERIFIED | Line 1-2 |
| `src/audio/normalizer.rs` | AudioError + LoudnormData + 5 norm functions + tests | VERIFIED | 451 lines, all 6 functions public, 6 unit tests |
| `src/audio/volume.rs` | VolumeConfig + ProcessingConfig + MixingConfig + 4 volume functions + tests | VERIFIED | 323 lines, all structs and functions, 13 unit tests |
| `src/youtube/mod.rs` | pub mod downloader; | VERIFIED | Line 1 |
| `src/youtube/downloader.rs` | YoutubeError + VideoFormat + download_video + validate_format + tests | VERIFIED | 453 lines, all required types and functions, 12 unit tests |
| `src/material/mod.rs` | pub mod searcher; pub mod downloader; | VERIFIED | Line 1-2 |
| `src/material/searcher.rs` | MaterialError + API types + Pexels/Pixabay search + key rotation + tests | VERIFIED | 471 lines, all types and functions, 11 unit tests |
| `src/material/downloader.rs` | compute_cache_key + save_video + download_videos + tests | VERIFIED | 285 lines, all required functions, 5 unit tests |
| `config.example.toml` | [audio] section | VERIFIED | Lines 183-218, 11 fields with Chinese comments |
| `tests/audio_integration.rs` | #[ignore] FFmpeg tests | VERIFIED | 33 lines, find_ffmpeg + 2 #[ignore] tests |
| `tests/youtube_integration.rs` | #[ignore] yt-dlp tests | VERIFIED | 55 lines, find_ytdlp + 2 #[ignore] tests |
| `tests/material_integration.rs` | #[ignore] API tests | VERIFIED | 27 lines, env var checks + 2 #[ignore] tests |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `src/config/types.rs` | Cargo.toml | serde Deserialize | VERIFIED | AudioSection has #[derive(Deserialize)] |
| `src/lib.rs` | `src/audio/mod.rs` | pub mod audio | VERIFIED | `src/lib.rs:10` |
| `src/audio/normalizer.rs::analyze_lufs` | std::process::Command(ffmpeg) | FFmpeg loudnorm filter | VERIFIED | `src/audio/normalizer.rs:130` |
| `src/audio/normalizer.rs::get_audio_rms` | symphonia::default::get_probe | symphonia PCM decode | VERIFIED | `src/audio/normalizer.rs:232` |
| `src/audio/volume.rs::apply_volume_profile` | VolumeConfig | 4-profile match | VERIFIED | `src/audio/volume.rs:136-163` - balanced/voice_focused/original_focused/quiet_background |
| `src/youtube/downloader.rs::download_video` | tokio::process::Command("yt-dlp") | yt-dlp CLI subprocess | VERIFIED | `src/youtube/downloader.rs:228-239` |
| `src/youtube/downloader.rs::download_video` | config.toml [proxy] | --proxy arg | VERIFIED | `src/youtube/downloader.rs:241-244` |
| `src/material/searcher.rs::search_videos_pexels` | reqwest::Client -> pexels.com/videos/search | HTTP GET | VERIFIED | `src/material/searcher.rs:196-210` |
| `src/material/downloader.rs::save_video` | reqwest::Client | HTTP download | VERIFIED | `src/material/downloader.rs:71-76` |
| `src/material/downloader.rs::save_video` | md-5::Md5 | MD5 cache key | VERIFIED | `src/material/downloader.rs:26-29` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `src/audio/normalizer.rs::get_audio_rms` | rms_db | symphonia PCM decode + formula | BUGGY | FLOWING_WITH_BUG -- formula uses i16::MAX reference for f32 samples, producing values ~90 dB too low |
| `src/audio/normalizer.rs::normalize_audio_for_mixing` | gain_db = target_lufs - rms_db | rms_db from get_audio_rms | BUGGY on fallback | OVER_AMPLIFIES -- when loudnorm fails and RMS fallback triggers, gain is ~+90 dB too high |
| `src/audio/normalizer.rs::normalize_lufs` | FFmpeg output file | FFmpeg loudnorm two-pass | PRODUCES REAL DATA | FLOWING -- loudnorm subprocess writes normalized audio file |
| `src/material/searcher.rs::get_api_key` | api_key | AtomicUsize counter -> Vec<String> index | PRODUCES REAL DATA | FLOWING -- returns key from array |
| `src/material/downloader.rs::save_video` | downloaded file | reqwest HTTP response bytes | PRODUCES REAL DATA (when network available) | FLOWING -- temp file + atomic rename + ffprobe validate |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| EXTD-01 | 11-02-PLAN, 11-05-PLAN | 音频响度标准化 (LUFS-based loudness normalization) | VERIFIED | `normalizer.rs`: two-pass loudnorm (analyze_lufs + normalize_lufs) + RMS fallback (get_audio_rms) + calculate_volume_adjustment + normalize_audio_for_mixing orchestration |
| EXTD-02 | 11-03-PLAN, 11-05-PLAN | YouTube 视频下载 (yt-dlp subprocess) | VERIFIED | `downloader.rs`: download_video async, _get_video_formats, validate_format, resolution normalization, proxy support, all CLI via args array |
| EXTD-03 | 11-04-PLAN, 11-05-PLAN | Pexels/Pixabay 素材搜索和下载 | VERIFIED | `searcher.rs`: search_videos_pexels, search_videos_pixabay, key rotation with AtomicUsize; `downloader.rs`: save_video with MD5 cache + ffprobe validate, download_videos orchestration |
| EXTD-04 | 11-02-PLAN, 11-05-PLAN | 智能音量控制 (content-type presets) | VERIFIED | `volume.rs`: VolumeConfig + 4 functions (get_optimized_volumes, apply_volume_profile, get_recommended_volumes_for_content, validate_volume) + ProcessingConfig + MixingConfig |

All 4 requirement IDs are satisfied across all 5 plans. No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `src/audio/normalizer.rs` | 288 | RMS calculation uses i16::MAX (32767) as reference for f32 samples (range [-1.0, 1.0]) | BLOCKER (CR-01) | RMS fallback path produces values ~90 dB too low; when triggered in normalize_audio_for_mixing(), gain_db = target_lufs - rms_db produces ~+90 dB over-amplification causing audio clipping/distortion. Fix: change to `20.0 * rms.log10()` |
| `src/material/searcher.rs` | 174-176 | urlencoding helper only encodes spaces, misses & = % # | WARNING (WR-01) | Search terms containing & (AT&T, C++) produce malformed API URLs with truncated query params |
| `src/youtube/downloader.rs` | 127-132, 186-191 | Resolution normalization logic duplicated across two functions | WARNING (WR-05) | If normalization needs to change, both locations must be updated |
| `src/youtube/downloader.rs` | 258-267 | Video title extraction spawns a second yt-dlp process wastefully | WARNING (WR-06) | Title already available in --dump-json output from format query |
| `src/youtube/downloader.rs` | 269 | Download does not verify output file exists after yt-dlp succeeds | WARNING (WR-07) | Returns path pointing to potentially non-existent file |
| `tests/audio_integration.rs` | 23, 31 | Stub tests contain `assert!(true, "存根...")` -- pass vacuously | WARNING (WR-09) | No real assertions; creates false sense of coverage |
| `tests/youtube_integration.rs` | 15-31 | Dead platform-detection code in find_ytdlp (where/which branches unreachable) | WARNING (WR-10) | `#[cfg(windows)]` and `#[cfg(not(windows))]` branches never executed |
| `src/audio/volume.rs` | 55, 86 | ProcessingConfig::default() and MixingConfig::default() are inherent methods, not Default trait impls | INFO (IN-04) | Cannot be used generically with T::default() |
| `config.example.toml` | 87, 123-124, 151-152 | Pre-existing config mismatches (soulvoice.voice_uri, doubaotts.ak/sk, ui.doubaotts_voice_type) not defined in structs | WARNING | Pre-existing from Phase 12; not introduced by Phase 11. These fields would cause parse errors with deny_unknown_fields |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Cargo check with zero errors | `cargo check` | 0 errors (1 pre-existing warning) | PASS |
| Audio normalizer unit tests | `cargo test --lib audio::normalizer::tests` | 6/6 passed | PASS |
| Audio volume unit tests | `cargo test --lib audio::volume::tests` | 13/13 passed | PASS |
| YouTube downloader unit tests | `cargo test --lib youtube::downloader::tests` | 12/12 passed | PASS |
| Material searcher unit tests | `cargo test --lib material::searcher::tests` | 11/11 passed | PASS |
| Material downloader unit tests | `cargo test --lib material::downloader::tests` | 5/5 passed | PASS |
| Integration test files compile | `cargo test --test audio_integration --test youtube_integration --test material_integration --no-run` | Compiled 3 executables | PASS |
| Config test with [audio] section assertions | `cargo test --lib config::types::tests` | All pass (verified earlier) | PASS |

### Human Verification Required

None. All checks are programmatic.

### Gaps Summary

**CR-01 (BLOCKER): RMS calculation uses wrong reference level**

File: `src/audio/normalizer.rs:288`

Bug: The `get_audio_rms()` function decodes audio to f32 PCM samples (range [-1.0, 1.0]) but computes RMS dB using `i16::MAX` (32767) as the reference level. The correct reference for f32 samples is 1.0 (full scale). This produces RMS dB values approximately 90 dB too low.

Impact: When `normalize_audio_for_mixing()` triggers the RMS fallback path (i.e., when FFmpeg loudnorm two-pass fails), `gain_db = target_lufs - rms_db` produces a gain of approximately +90 dB, causing massive over-amplification and audio clipping/distortion.

Fix:
```rust
// Line 288: change reference from i16::MAX to 1.0 (full scale for f32 samples)
let rms_db = if rms > 0.0 {
    20.0 * rms.log10()  // rms is already in [0, 1] range for f32 samples
} else {
    -60.0 // 静音
};
```

The primary loudnorm two-pass path (`normalize_lufs()` -> `analyze_lufs()`) works correctly. This bug only affects the RMS fallback path. The feature module is complete and functional for the primary use case, but the fallback is broken.

**WARNING items for awareness:**
- WR-01: urlencoding helper too simplistic (&, =, % encoding missing) -- affects Pexels and Pixabay search with unusual search terms
- WR-05: Resolution normalization duplicated in two functions -- refactoring opportunity
- WR-06: Redundant yt-dlp --get-title subprocess -- performance concern
- WR-07: No output file existence verification after download -- robustness concern
- WR-09: Integration test stubs contain no real assertions -- known design tradeoff
- WR-10: Dead code in find_ytdlp helper -- cleanup opportunity
- Pre-existing config.example.toml mismatches (soulvoice.voice_uri, doubaotts.ak/sk, ui.doubaotts_voice_type) -- introduced by Phase 12, not Phase 11

---

_Verified: 2026-04-30T15:00:00Z_
_Verifier: Claude (gsd-verifier)_
