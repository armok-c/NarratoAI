---
phase: 11-extended-features
reviewed: 2026-04-30T10:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - config.example.toml
  - src/audio/mod.rs
  - src/audio/normalizer.rs
  - src/audio/volume.rs
  - src/config/defaults.rs
  - src/config/types.rs
  - src/lib.rs
  - src/material/downloader.rs
  - src/material/mod.rs
  - src/material/searcher.rs
  - src/tts/common.rs
  - src/tts/doubaotts.rs
  - src/tts/soulvoice.rs
  - src/youtube/downloader.rs
  - src/youtube/mod.rs
  - tests/audio_integration.rs
  - tests/material_integration.rs
  - tests/youtube_integration.rs
findings:
  critical: 4
  warning: 10
  info: 8
  total: 22
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-04-30T10:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

This review covers 19 Rust source and configuration files implementing audio processing (LUFS normalization, smart volume), material search/download (Pexels, Pixabay), TTS engines (Doubao, SoulVoice), YouTube download via yt-dlp, and configuration types/defaults.

Four BLOCKER issues were found: a critical RMS calculation formula bug that would massively over-amplify audio in the normalization fallback path, and three config struct/example mismatches that make `config.example.toml` unparseable by the Rust code due to `#[serde(deny_unknown_fields)]`. Additionally, ten WARNING issues span incomplete URL encoding, ignored resolution constraints, inconsistent proxy error handling, and code duplication.

---

## Critical Issues

### CR-01: RMS calculation uses i16::MAX reference for f32 audio samples -- gain calculation off by ~90 dB

**File:** `src/audio/normalizer.rs:288`
**Issue:** The `get_audio_rms()` function decodes audio to f32 PCM samples (range [-1.0, 1.0]) but computes RMS dB using `i16::MAX` (32767) as the reference level. The correct reference for f32 samples is 1.0 (full scale). This produces RMS dB values approximately 90 dB too low.

For a typical full-scale sine wave (RMS = 0.707):
- Buggy: `20 * log10(0.707 / 32767) = -93.3 dBFS`
- Correct: `20 * log10(0.707) = -3.0 dBFS`

This feeds into `normalize_audio_for_mixing()` where `gain_db = target_lufs - rms_db` (line 338). With the buggy value:
- Actual: `gain_db = -20.0 - (-93.3) = +73.3 dB` (massive over-amplification)
- Correct: `gain_db = -20.0 - (-3.0) = -17.0 dB` (proper attenuation)

This affects the RMS fallback path -- when FFmpeg loudnorm fails, audio will be amplified to clipping/distortion levels with no protection.

**Fix:**
```rust
// Line 288: change reference from i16::MAX to 1.0 (full scale for f32 samples)
let rms_db = if rms > 0.0 {
    20.0 * rms.log10()  // rms is already in [0, 1] range for f32 samples
} else {
    -60.0 // 静音
};
```

### CR-02: config.example.toml `[soulvoice]` has `voice_uri` field undefined in struct -- parse error with deny_unknown_fields

**File:** `config.example.toml:87` and `src/config/types.rs:115-116`
**Issue:** `config.example.toml` specifies `voice_uri` under `[soulvoice]`:
```toml
[soulvoice]
voice_uri = "speech:mcg3fdnx:..."
```
But `SoulVoiceSection` (types.rs line 113) is annotated with `#[serde(deny_unknown_fields)]` and does not contain a `voice_uri` field. Any user copying the example config verbatim will get a parse error from `toml::from_str()`. The `deny_unknown_fields` attribute is specifically designed to catch such mismatches, but the example file itself violates the schema.

Even if `deny_unknown_fields` were removed, `voice_uri` would be silently ignored, meaning users set a config value with no effect -- also wrong.

**Fix:** Either add `voice_uri` to the `SoulVoiceSection` struct and use it in the SoulVoice TTS engine, or remove `voice_uri` from `config.example.toml`.

### CR-03: config.example.toml `[doubaotts]` has `ak`/`sk` fields undefined in struct -- parse error with deny_unknown_fields

**File:** `config.example.toml:123-124` and `src/config/types.rs:164-165`
**Issue:** `config.example.toml` specifies `ak` and `sk` under `[doubaotts]`:
```toml
[doubaotts]
ak = ""
sk = ""
```
But `DoubaoTTSSection` (types.rs line 162) is annotated with `#[serde(deny_unknown_fields)]` and does not contain `ak` or `sk` fields. The example config cannot be parsed by the Rust code. Moreover, the Doubao engine code (doubaotts.rs) only uses `appid` and `token` for authentication, so `ak`/`sk` would serve no purpose even if added.

**Fix:** Remove `ak` and `sk` from `config.example.toml`. If these were intended for authentication, the engine code needs to be updated to use them.

### CR-04: config.example.toml `[ui]` has `doubaotts_voice_type`/`doubaotts_rate` fields undefined in struct -- parse error with deny_unknown_fields

**File:** `config.example.toml:151-152` and `src/config/types.rs:69`
**Issue:** `config.example.toml` specifies `doubaotts_voice_type` and `doubaotts_rate` under `[ui]`:
```toml
doubaotts_voice_type = "BV700_V2_streaming"
doubaotts_rate = 1.0
```
But `UiSection` (types.rs line 68) is annotated with `#[serde(deny_unknown_fields)]` and does not contain either field. The example config cannot be parsed by the Rust code.

**Fix:** Add `doubaotts_voice_type` and `doubaotts_rate` fields to `UiSection`, or remove them from `config.example.toml`.

---

## Warnings

### WR-01: urlencoding function only encodes spaces, omitting other special characters

**File:** `src/material/searcher.rs:174-176`
**Issue:** The `urlencoding` helper only replaces spaces with `+` but does not encode `&`, `=`, `%`, `#`, or other URL-significant characters. A search term containing `&` (e.g., "AT&T", "C++", "foo & bar") would produce a malformed API URL:

`https://api.pexels.com/videos/search?query=AT&T&per_page=20`

The `&` would be interpreted as a query parameter separator, corrupting the request. Both Pexels (line 196) and Pixabay (line 262) URLs are affected.

**Fix:** Use a proper URL encoding library or encode the query parameter before insertion. Replace the custom function with `urlencoding` crate usage or percent-encode at least `&`, `=`, `%`, `#`, `+`.

```rust
// Simple fix (not comprehensive):
fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
     .replace('&', "%26")
     .replace('=', "%3D")
     .replace('%', "%25")
     .replace('#', "%23")
}
// Better: use the `url` crate or `percent_encoding` crate
```

### WR-02: Pixabay search ignores target resolution constraints

**File:** `src/material/searcher.rs:252-258`
**Issue:** `search_videos_pixabay` accepts `_target_width` and `_target_height` (both prefixed with underscore, signaling intent to ignore them) and never applies any resolution filter. The `choose_variant` helper (line 283) simply picks large > medium > small > tiny by availability with no regard for the caller's requested resolution. In contrast, `search_videos_pexels` (line 233) does correctly filter by exact width/height.

This means callers requesting `1920x1080` videos from Pixabay may receive 640x360 videos, and the system has no way to know.

**Fix:** Implement resolution filtering for Pixabay. At minimum, filter variants that are "close enough" to the target, or fall back to the available variant with a warning.

```rust
// Add resolution preference scoring in choose_variant:
fn choose_variant(videos: &PixabayVideos, target_w: u32, target_h: u32) -> Option<&PixabayVideoVariant> {
    let candidates = [&videos.large, &videos.medium, &videos.small, &videos.tiny];
    // Pick the variant with the highest resolution that does not exceed target
    candidates.iter().filter_map(|c| c.as_ref()).min_by_key(|v| {
        let dw = (v.width as i64 - target_w as i64).abs();
        let dh = (v.height as i64 - target_h as i64).abs();
        dw + dh
    }).copied()
}
```

### WR-03: TTS common build_client silently falls back to unproxied client on config error

**File:** `src/tts/common.rs:56-60`
**Issue:** When `reqwest::Client::builder().build()` fails (e.g., invalid proxy URL), `build_client()` silently falls back to `reqwest::Client::new()` with no proxy, logging only a warning. This means a misconfigured proxy in the TTS engines is silently bypassed. In contrast, `material/searcher.rs:153-154` correctly returns `Err(MaterialError::ApiRequest(...))` for invalid proxy config, which is the right behavior.

Callers of TTS's `build_client` assume proxy is applied, but it may not be.

**Fix:** Propagate the error instead of silently falling back:

```rust
pub fn build_client(proxy_config: &ProxyConfig) -> Result<reqwest::Client, TTSError> {
    let mut builder = reqwest::Client::builder();
    builder = proxy_config.apply_to_client(builder);
    builder.build().map_err(|e| TTSError::ConnectionFailed(
        format!("HTTP 客户端创建失败: {}", e)
    ))
}
```

This requires updating the callers in `soulvoice.rs:24` and `doubaotts.rs:25` to handle `Result`.

### WR-04: SoulVoice engine ignores pitch parameter silently

**File:** `src/tts/soulvoice.rs:100`
**Issue:** The `synthesize()` method receives `_pitch: f64` (prefixed with underscore) and never passes it to the API. The SoulVoice API request (line 59) only sends `"speed": rate`. Callers expecting pitch adjustment to work across all TTS engines will get silent failure from SoulVoice. Doubaotts correctly uses pitch (doubaotts.rs:55: `"pitch_ratio": pitch`).

**Fix:** Check if the SoulVoice API supports pitch; if so, add it to the request body. If not, document the limitation and log a warning when a non-default pitch is requested.

```rust
// At line 59, if pitch != 1.0:
if (pitch - 1.0).abs() > f64::EPSILON {
    tracing::warn!("SoulVoice 不支持 pitch 参数，忽略 pitch={}", pitch);
}
```

### WR-05: Resolution normalization logic duplicated across two functions

**File:** `src/youtube/downloader.rs:127-132` and `src/youtube/downloader.rs:186-191`
**Issue:** The identical resolution normalization logic ("1080p60" -> "1080p") is implemented twice: once in `_get_video_formats` (lines 127-132) and again in `download_video` (lines 186-191). If the normalization needs to change, both locations must be updated. The second usage (line 186) normalizes the input resolution, and the first (line 127) normalizes the format_note values from yt-dlp.

**Fix:** Extract to a shared helper function:

```rust
fn normalize_resolution(resolution: &str) -> String {
    if let Some(pos) = resolution.find('p') {
        format!("{}p", &resolution[..pos])
    } else {
        resolution.to_string()
    }
}
```

### WR-06: Video title extraction spawns a second yt-dlp process wastefully

**File:** `src/youtube/downloader.rs:258-267`
**Issue:** After downloading the video, title extraction spawns a completely separate `yt-dlp --get-title` process. The `_get_video_formats` function already calls `yt-dlp --dump-json` which returns the full video metadata including the `title` field. The title could be extracted from that JSON response and cached/returned alongside the format list, eliminating the redundant subprocess.

**Fix:** Return the title from `_get_video_formats` (or add a parallel method), or extract `title` from the `--dump-json` output directly.

```rust
// In _get_video_formats, extract title as well:
let title = info["title"].as_str().unwrap_or("unknown_title").to_string();
```

### WR-07: YouTube download does not verify output file exists after download

**File:** `src/youtube/downloader.rs:269`
**Issue:** After the yt-dlp subprocess reports success (line 252), the code constructs `output_path` synthetically via string formatting (line 269) without checking if the file actually exists. If yt-dlp succeeds but writes to a different path (e.g., due to `--output` template quirks, special characters in the URL, or yt-dlp renaming the output), `download_video` returns a `DownloadResult` pointing to a non-existent file.

**Fix:** After download, verify the file exists. If missing, attempt to find the actual output file from yt-dlp's verbose output, or fail.

```rust
if !output_path.exists() {
    // Fallback: parse stderr for yt-dlp's actual output path
    return Err(YoutubeError::DownloadFailed(
        "下载完成后未找到输出文件".into()
    ));
}
```

### WR-08: Two-pass loudnorm analysis uses hardcoded I=-23 regardless of caller's target_lufs

**File:** `src/audio/normalizer.rs:128`
**Issue:** `analyze_lufs()` always sets the loudnorm analysis target to `I=-23:TP=-1:LRA=7`. The caller of `normalize_lufs()` can pass a different `target_lufs`, but the first-pass analysis always measures against -23 LUFS. For targets significantly different from -23 (e.g., -14 for YouTube), the measured values from the first pass may not be optimal for the second pass. While FFmpeg's loudnorm filter handles modest deviations, the analysis target should match the final target.

**Fix:** Pass `target_lufs` and `max_peak` through to `analyze_lufs`:

```rust
pub fn analyze_lufs(input: &Path, target_lufs: f64, max_peak: f64) -> Result<LoudnormData, AudioError> {
    // ...
    let filter = format!("loudnorm=I={}:TP={}:LRA=7:print_format=json", target_lufs, max_peak);
    // ...
}
```

### WR-09: Stub tests contain no assertions -- pass vacuously

**File:** `tests/audio_integration.rs:23`, `tests/audio_integration.rs:31`, `tests/youtube_integration.rs:53`
**Issue:** Three integration tests (`test_normalize_lufs_two_pass`, `test_get_audio_rms_with_real_file`, `test_get_video_formats`) contain only `assert!(true, "存根...")`. They pass regardless of whether the underlying code works. While acceptable as placeholders during development, they create a false sense of test coverage. A test that cannot fail is worse than no test.

**Fix:** Either implement real test logic, or convert them to `#[ignore]` with a `todo!()` body to fail fast if accidentally un-ignored:

```rust
#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    todo!("集成测试：使用 ffmpeg 生成测试音频 → normalize_lufs → 验证输出");
}
```

### WR-10: find_ytdlp test helper contains dead platform-detection code

**File:** `tests/youtube_integration.rs:15-31`
**Issue:** The `find_ytdlp()` function checks `yt-dlp --version` first (lines 9-14) and returns immediately on success. If that fails, it panics at line 31 with "yt-dlp 不可用". The `#[cfg(windows)]` and `#[cfg(not(windows))]` branches (lines 15-29) that call `where`/`which` are unreachable dead code -- control never reaches them, because the panic at line 31 always fires first.

**Fix:** Either remove the dead branches or restructure to check `where`/`which` BEFORE panicking:

```rust
fn find_ytdlp() -> String {
    if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return "yt-dlp".to_string();
        }
    }
    // Check PATH via platform-specific tools
    #[cfg(windows)]
    let find_cmd = "where";
    #[cfg(not(windows))]
    let find_cmd = "which";
    if let Ok(output) = Command::new(find_cmd).arg("yt-dlp").output() {
        if output.status.success() {
            return "yt-dlp".to_string();
        }
    }
    panic!("yt-dlp 不可用。...");
}
```

---

## Info

### IN-01: Unnecessary `speed_ratio` variable alias

**File:** `src/tts/doubaotts.rs:37`
**Issue:** `let speed_ratio = rate;` creates a one-use alias that adds no value. The `rate` parameter should be used directly at line 53.

**Fix:** Replace `speed_ratio` with `rate` on line 53 and remove line 37.

### IN-02: Unnecessary leading underscore on private function name

**File:** `src/youtube/downloader.rs:87`
**Issue:** Function `_get_video_formats` uses a leading underscore, which is a Python convention for "private." In Rust, the function is already private (no `pub`) -- the underscore provides no signal. Rust uses `pub(crate)` for crate-internal visibility, not naming conventions.

**Fix:** Rename to `get_video_formats_inner` or just `get_video_formats`.

### IN-03: References to internal rule IDs in comments (T-11-01, D-10, etc.)

**File:** `src/youtube/downloader.rs:85, 164, 165`
**Issue:** Multiple comments reference internal security/testing rules ("per T-11-01", "per T-11-02", "per D-10", "per D-26", "per D-27") that have no meaning outside the design documents. These become stale as documents evolve and are confusing to readers who do not have access to the planning artifacts.

**Fix:** Replace with self-contained explanations of why the code is written that way.

### IN-04: `ProcessingConfig::default()` is an inherent method, not a `Default` trait impl

**File:** `src/audio/volume.rs:55`
**Issue:** `ProcessingConfig::default()` and `MixingConfig::default()` are defined as inherent methods rather than implementing the `Default` trait. This means they cannot be used generically (e.g., `T::default()` where T is a config type) and `Default::default()` will not compile. Same issue for `MixingConfig::default()` at line 86.

**Fix:** Implement the `Default` trait instead:

```rust
impl Default for ProcessingConfig {
    fn default() -> Self {
        Self { /* ... */ }
    }
}
```

### IN-05: `LoudnormData` accessor methods named differently from raw JSON field names

**File:** `src/audio/normalizer.rs:72`
**Issue:** The struct fields are named after the raw ffmpeg loudnorm JSON keys (`input_i`, `input_lra`, etc.), but the accessor methods are named conceptually (`measured_I()`, `measured_LRA()`, etc.). A reader must trace through to understand that `measured_I()` reads `self.input_i`. With `#[allow(non_snake_case)]` on the impl block (line 70), Rust's naming lints are suppressed.

**Fix:** Either rename the struct fields to match the method names and use `#[serde(alias = ...)]` for deserialization, or rename the methods to match the fields.

### IN-06: Pixabay API key sent as URL query parameter

**File:** `src/material/searcher.rs:262`
**Issue:** The Pixabay API key is appended as a URL query parameter (`&key={}`). While this is the documented Pixabay API mechanism, it means the API key will appear in any URL logs, error messages, or network traces. The Pexels API (line 208) correctly uses the `Authorization` header instead.

**Fix:** This is by design for Pixabay's API, but add a warning comment that logging `api_url` could leak the key.

### IN-07: Negative LUFS values printed without formatting care

**File:** `src/audio/normalizer.rs:181`
**Issue:** The `loudnorm` filter string uses `{}` format for `target_lufs` and `max_peak` which are f64 values. Values like `-23.0` become `-23` or `-23.0` in the format string, which is fine for FFmpeg. However, `max_peak` is always `-1.0` currently, and if NaN were somehow passed, the format string would contain "NaN" which would confuse FFmpeg. This is more of a robustness concern.

**Fix:** Validate inputs before constructing the filter string.

### IN-08: `save_video` leaves `.tmp` files on crash during download

**File:** `src/material/downloader.rs:86`
**Issue:** If the process crashes after creating the `.tmp` file but before `rename()` on line 98, the `.tmp` file is orphaned on disk. On the next run with the same URL, the cache check (line 59) looks for `.mp4` extension, not `.tmp`, so the orphan persists. Not a correctness bug, but leads to disk clutter over time.

**Fix:** Clean up stale `.tmp` files before writing, or use a well-known temp directory.

---

_Reviewed: 2026-04-30T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
