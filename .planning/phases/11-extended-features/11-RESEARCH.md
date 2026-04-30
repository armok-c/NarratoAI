# Phase 11: Extended Features — Research

**Researched:** 2026-04-30
**Domain:** Audio processing, external CLI integration, HTTP API integration, volume control
**Confidence:** HIGH

## Summary

Phase 11 implements four independent auxiliary features for the Rust rewrite: (1) LUFS-based audio loudness normalization with FFmpeg two-pass loudnorm and symphonia RMS fallback, (2) YouTube video download via yt-dlp CLI subprocess, (3) Pexels/Pixabay stock video search and download via reqwest HTTP client, and (4) smart volume control with preset profiles and content-type recommendations.

All four features are self-contained modules that depend only on Phase 1 infrastructure (FFmpeg subprocess, config system, error types). The Python reference implementations exist in `app/services/audio_normalizer.py`, `app/config/audio_config.py`, `app/services/youtube_service.py`, and `app/services/material.py` — these must be 1:1 functionally replicated in Rust.

**Primary recommendation:** Use FFmpeg-sidecar (existing) for loudnorm two-pass, symphonia 0.5.5 for PCM decode fallback, tokio::process::Command for yt-dlp, and reqwest (existing) for Pexels/Pixabay HTTP API calls. Add `md-5` crate for URL cache key hashing.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Two-pass FFmpeg loudnorm — first pass analysis, second pass application.
- **D-02:** target_lufs configurable with default -23.0, from new `[audio]` config section.
- **D-03:** FFmpeg loudnorm failure fallback to pure math RMS normalization via symphonia.
- **D-04:** RMS fallback uses symphonia (pure Rust audio decoder), no external audio tools.
- **D-05:** New `src/audio/` module: normalizer.rs + volume.rs.
- **D-06:** Pure function API with explicit parameters, no hidden state.
- **D-07:** All 5 functions 1:1 replicate Python: `analyze_lufs()`, `normalize_lufs()`, `get_audio_rms()`, `calculate_volume_adjustment()`, `normalize_audio_for_mixing()`.
- **D-08:** yt-dlp via CLI subprocess (`tokio::process::Command`), user must install yt-dlp.
- **D-09:** Full format/resolution matching: `_get_video_formats()` then manual format_id download.
- **D-10:** No download progress callback.
- **D-11:** New `src/youtube/` module, pure function API: `download_video(url, resolution, output_format, rename) -> Result<(PathBuf, String)>`.
- **D-12:** Pexels + Pixabay dual source support.
- **D-13:** Multi API key rotation from config `[app]` section arrays.
- **D-14:** Combined `download_videos()` orchestration + fine-grained `search_videos()` / `save_video()`.
- **D-15:** New `src/material/` module. MD5 URL cache key. reqwest HTTP client.
- **D-16:** Volume profiles hardcoded (balanced/voice_focused/original_focused/quiet_background), config overrides in `[audio]` section.
- **D-17:** `src/audio/normalizer.rs` + `src/audio/volume.rs`. Shared `AudioError`.
- **D-18:** All 4 volume functions 1:1 replicate Python.
- **D-19:** New `[audio]` config section with 11 fields.
- **D-20:** Processing/Mixing config structs defined in Rust, FFmpeg crossfade deferred to Phase 6.
- **D-21:** `VolumeConfig` struct with type-safe fields.
- **D-22:** `MixingConfig` struct definition + config reading, FFmpeg implementation deferred to Phase 6.
- **D-23:** Four new top-level modules: `src/audio/`, `src/youtube/`, `src/material/`.
- **D-24:** Config types extend `src/config/types.rs`: `AudioSection`, extend `AppSection` for pexels/pixabay keys.
- **D-25:** `AudioError`, `YoutubeError`, `MaterialError` per module.
- **D-26:** thiserror derive + Chinese Display messages.
- **D-27:** Error variants align with Python: LoudnormAnalysisFailed, LoudnormNormalizeFailed, RmsFallbackFailed, FileNotFound, SubprocessError, FormatNotFound, InvalidUrl, DownloadFailed.
- **D-28:** Unit tests mock external calls, integration tests with `#[ignore]`.
- **D-29:** Unit tests in `#[cfg(test)] mod tests`, integration tests in `tests/`.
- **D-30:** yt-dlp proxy from `[proxy]` config, passed as `--proxy` argument.
- **D-31:** reqwest proxy from `[proxy]` config via `reqwest::Proxy`.

### Claude's Discretion
- Error enum variant count and definition details
- Function signature parameters (default value handling)
- Cargo.toml new dependencies (symphonia feature flags)
- Test case design and mock implementation
- `[audio]` config field exact names and types
- `download_videos()` orchestration internal decomposition

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EXTD-01 | Audio loudness normalization (LUFS) | FFmpeg loudnorm two-pass pattern documented; symphonia 0.5.5 API for RMS fallback verified |
| EXTD-02 | YouTube video download (yt-dlp) | yt-dlp CLI format selection via `-F`/`-f` flags documented; tokio::process::Command pattern from ffmpeg probe.rs available |
| EXTD-03 | Pexels material search/download | Pexels API endpoint/response format verified; reqwest already in Cargo.toml; proxy integration via `reqwest::Proxy` |
| EXTD-04 | Smart volume control | Volume profile values from Python AudioConfig documented; `VolumeConfig` struct pattern defined |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Audio loudness normalization | Audio module | FFmpeg subprocess | `src/audio/normalizer.rs` orchestrates FFmpeg loudnorm calls and RMS fallback; loudnorm execution uses ffmpeg-sidecar (Phase 1) |
| YouTube download | YouTube module | OS subprocess | `src/youtube/` wraps yt-dlp CLI calls; yt-dlp is a system dependency like FFmpeg |
| Material search/download | Material module | HTTP client | `src/material/` calls Pexels/Pixabay REST APIs via reqwest; all network logic contained in module |
| Smart volume control | Audio module | Config system | `src/audio/volume.rs` defines profiles as code constants; config overrides from `[audio]` section |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ffmpeg-sidecar | 2.5.1 | FFmpeg CLI wrapper for loudnorm | Already in Cargo.toml; existing spawn_blocking pattern in `src/ffmpeg/command.rs` [VERIFIED: Cargo.toml] |
| symphonia | 0.5.5 | Pure Rust audio decode for RMS fallback | Latest stable release (2025-10); supports MP3, WAV, FLAC, OGG; no C dependencies [VERIFIED: crate registry] |
| reqwest | 0.13.3 | HTTP client for Pexels/Pixabay API | Already in Cargo.toml; supports proxy via `reqwest::Proxy` [VERIFIED: Cargo.toml] |
| tokio | 1.52.1 | Async runtime for yt-dlp subprocess | Already in Cargo.toml; `tokio::process::Command` for CLI subprocess [VERIFIED: Cargo.toml] |
| md-5 | 0.10.6 | MD5 hashing for video URL cache key | Pure Rust, lightweight, replaces Python `hashlib.md5()` [VERIFIED: crate registry] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | JSON deserialization for loudnorm output and API responses | Already in Cargo.toml; loudnorm JSON parsing, Pexels/Pixabay API response parsing |
| serde_json | 1.0.140 | JSON value manipulation for API responses | Already in Cargo.toml; flexible JSON querying for API responses with unknown structures |
| thiserror | 2.0.18 | Error enum derivation | Already in Cargo.toml; AudioError/YoutubeError/MaterialError |
| tracing | 0.1.44 | Logging | Already in Cargo.toml; replace loguru with tracing macros |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| symphonia (RMS) | pydub equivalent (none in Rust) | No viable Rust alternative for pure-Rust audio decode. symphonia is the only mature option. |
| yt-dlp subprocess | youtube-dl-exec Rust crate | yt-dlp is the active fork (youtube-dl is deprecated). Direct subprocess gives full control over format matching. |
| reqwest raw | pexels-api crate (0.0.5) | pexels-api crate is immature (v0.0.5) with minimal API coverage. Raw reqwest calls are more maintainable. |
| md-5 crate | sha2 crate with SHA-256 | Python uses MD5 for URL cache keys. Must use MD5 for cache key compatibility. |

**Installation (Cargo.toml additions):**
```toml
symphonia = { version = "0.5", features = ["mp3", "flac", "pcm", "wav", "vorbis"] }
md-5 = "0.10"
```

## Architecture Patterns

### Audio Normalization Flow (EXTD-01)
```
input_audio
    |
    v
normalize_audio_for_mixing(input, output_dir, target_lufs)
    |
    v
normalize_lufs(input, output, target_lufs, max_peak)
    |
    +-- Pass 1: ffmpeg -af loudnorm=I=X:TP=Y:LRA=7:print_format=json -f null -
    |       parse JSON from stderr -> measured_I, measured_LRA, measured_TP, measured_thresh
    |
    +-- Pass 2: ffmpeg -af loudnorm=I=X:TP=Y:LRA=7:measured_I=...:measured_TP=... -ar 44100 -ac 2 output
    |       on success -> return Ok
    |
    +-- Fallback (if pass 1 or pass 2 fails):
            get_audio_rms(path)    // symphonia decode -> PCM f32 -> RMS dB
            calculate_volume_adjustment(target_lufs, measured_rms)
            -> apply gain and re-encode via FFmpeg
```

### yt-dlp Download Flow (EXTD-02)
```
download_video(url, resolution, output_format, rename)
    |
    +-- validate_format(output_format)
    |
    +-- _get_video_formats(url)
    |       yt-dlp --dump-json URL  (get info without downloading)
    |       parse JSON -> extract formats[]
    |       normalize resolution strings ("1080p60" -> "1080p")
    |       match target resolution, find correct format_id
    |
    +-- download with matched format_id
            yt-dlp -f FORMAT_ID+bestaudio --merge-output-format OUTPUT_FMT -o OUTPUT_TEMPLATE URL
            on rename-> use custom filename
            on no rename-> use task_id + title
```

### Material Search Flow (EXTD-03)
```
search_videos(search_term, minimum_duration, video_aspect, source)
    |
    +-- get_api_key("pexels_api_keys"|"pixabay_api_keys") [round-robin by global counter]
    |
    +-- if pexels:
    |       GET https://api.pexels.com/videos/search?query=X&per_page=20&orientation=Y
    |       Header: Authorization: API_KEY
    |       parse JSON.response.videos[].video_files[] -> match width/height -> MaterialInfo
    |
    +-- if pixabay:
            GET https://pixabay.com/api/videos/?q=X&video_type=all&per_page=50&key=API_KEY
            parse JSON.response.hits[].videos.{large,medium,small} -> match resolution -> MaterialInfo

save_video(video_url, save_dir)
    |
    +-- url_without_query = video_url.split('?')[0]
    +-- url_hash = MD5(url_without_query)
    +-- video_path = save_dir / "vid-{url_hash}.mp4"
    +-- if exists and size > 0: return path (cache hit)
    +-- else: HTTP GET -> write to path -> validate with ffprobe -> return path

download_videos(task_id, search_terms[], source, aspect, audio_duration, max_clip_duration)
    |
    +-- for each search_term: search_videos() -> collect unique items
    +-- shuffle if random mode
    +-- for each item: save_video() -> track total_duration -> stop when >= audio_duration
    +-- return Vec<String> of paths
```

### Smart Volume Profile Flow (EXTD-04)
```
get_optimized_volumes(video_type: &str) -> VolumeConfig
    base: tts=0.8, original=1.3, bgm=0.3
    video_type overrides:
      educational   -> tts=0.9, original=0.8, bgm=0.2
      entertainment -> tts=0.8, original=1.2, bgm=0.4
      news          -> tts=1.0, original=0.6, bgm=0.1

apply_volume_profile(profile_name: &str) -> VolumeConfig
    balanced        -> tts=0.8, original=1.2, bgm=0.3
    voice_focused   -> tts=1.0, original=0.7, bgm=0.2
    original_focused -> tts=0.7, original=1.5, bgm=0.2
    quiet_background -> tts=0.8, original=1.3, bgm=0.1

get_recommended_volumes_for_content(content_type: &str) -> VolumeConfig
    mixed          -> tts=0.8, original=1.3, bgm=0.3
    voice_only     -> tts=1.0, original=0.5, bgm=0.2
    original_heavy -> tts=0.6, original=1.6, bgm=0.1
    music_video    -> tts=0.7, original=1.8, bgm=0.0
```

### Recommended Project Structure
```
src/
├── audio/
│   ├── mod.rs           # pub mod normalizer; pub mod volume;
│   ├── normalizer.rs    # analyze_lufs, normalize_lufs, get_audio_rms, calculate_volume_adjustment, normalize_audio_for_mixing
│   └── volume.rs        # VolumeConfig, PROCESSING_CONFIG, MIXING_CONFIG, profiles, get_optimized_volumes, apply_volume_profile, etc.
├── youtube/
│   ├── mod.rs           # pub mod downloader;
│   └── downloader.rs    # download_video, _get_video_formats, _validate_format
├── material/
│   ├── mod.rs           # pub mod searcher; pub mod downloader;
│   ├── searcher.rs      # search_videos_pexels, search_videos_pixabay, get_api_key
│   └── downloader.rs    # save_video, download_videos
├── config/
│   └── types.rs         # + AudioSection, MaterialSection, extend AppSection with pexels_api_keys/pixabay_api_keys
└── lib.rs               # + pub mod audio; pub mod youtube; pub mod material;
```

### Pattern 1: FFmpeg loudnorm via spawn_blocking (EXTD-01)
**What:** Execute two-pass FFmpeg loudnorm analysis and normalization using the existing spawn_blocking pattern from `src/ffmpeg/command.rs`. First pass analyzes audio and returns JSON with measured loudness parameters. Second pass applies precise normalization using measured values.

**When to use:** When normalizing audio files to a target LUFS level. The two-pass approach is industry standard for EBU R128 compliance and preserves dynamics better than single-pass.

**Example — first pass analysis:**
```rust
// Source: Derived from ffmpeg-sidecar API + Python `audio_normalizer.py:analyze_audio_lufs()`
use ffmpeg_sidecar::command::FfmpegCommand;

fn analyze_lufs_sync(input: &Path) -> Result<LoudnormData, AudioError> {
    let mut cmd = FfmpegCommand::new();
    cmd.input(input.to_string_lossy().as_ref())
        .args(["-af", "loudnorm=I=-23:TP=-1:LRA=7:print_format=json"])
        .args(["-f", "null"])
        .output("-");  // discard output, analysis goes to stderr

    let child = cmd.spawn().map_err(|e| AudioError::LoudnormAnalysisFailed(e.to_string()))?;
    let iter = child.iter().map_err(|e| AudioError::LoudnormAnalysisFailed(e.to_string()))?;

    let mut json_lines = Vec::new();
    let mut in_json = false;
    for event in iter {
        match event {
            FfmpegEvent::Log(_, msg) => {
                if msg.trim() == "{" { in_json = true; }
                if in_json { json_lines.push(msg); }
            }
            _ => {}
        }
    }

    let json_str = json_lines.join("\n");
    let data: LoudnormData = serde_json::from_str(&json_str)
        .map_err(|e| AudioError::LoudnormAnalysisFailed(e.to_string()))?;
    Ok(data)
}
```

**Example — second pass normalization:**
```rust
// Source: Derived from Python `audio_normalizer.py:normalize_audio_lufs()`
fn normalize_lufs_sync(
    input: &Path,
    output: &Path,
    target_lufs: f64,
    measured: &LoudnormData,
) -> Result<(), AudioError> {
    let mut cmd = FfmpegCommand::new();
    cmd.input(input.to_string_lossy().as_ref())
        .overwrite()
        .args(["-af", &format!(
            "loudnorm=I={}:TP=-1:LRA=7:measured_I={}:measured_LRA={}:measured_TP={}:measured_thresh={}",
            target_lufs,
            measured.input_i, measured.input_lra,
            measured.input_tp, measured.input_thresh,
        )])
        .args(["-ar", "44100", "-ac", "2"])
        .output(output.to_string_lossy().as_ref());

    let mut child = cmd.spawn().map_err(|e| AudioError::LoudnormNormalizeFailed(e.to_string()))?;
    let iter = child.iter().map_err(|_| AudioError::LoudnormNormalizeFailed("spawn failed".into()))?;

    for event in iter {
        if let FfmpegEvent::Error(e) = event {
            tracing::error!("loudnorm error: {}", e);
        }
    }
    child.wait().map_err(|e| AudioError::LoudnormNormalizeFailed(e.to_string()))?;
    Ok(())
}
```

### Pattern 2: symphonia PCM decode for RMS (EXTD-01 fallback)
**What:** When FFmpeg loudnorm analysis fails, decode audio to PCM f32 samples using symphonia, compute RMS in dB, apply gain factor.

**When to use:** FFmpeg loudnorm fails on unusual audio formats or corrupted files. symphonia handles more formats gracefully.

**Example:**
```rust
// Source: symphonia GETTING_STARTED.md v0.5 (WebSearch, 2026-04-30)
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

fn get_audio_rms(path: &Path) -> Result<f64, AudioError> {
    let src = std::fs::File::open(path)
        .map_err(|e| AudioError::FileNotFound(path.display().to_string()))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let probed = symphonia::default::get_probe()
        .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| AudioError::RmsFallbackFailed(format!("probe failed: {}", e)))?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::RmsFallbackFailed("no audio track".into()))?;

    let codec_params = &track.codec_params;
    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::RmsFallbackFailed(format!("decoder init: {}", e)))?;

    let track_id = track.id;
    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(_) => break,
        };
        if packet.track_id() != track_id { continue; }

        if let Ok(audio_buf) = decoder.decode(&packet) {
            let spec = *audio_buf.spec();
            let cap = audio_buf.capacity() as u64;
            let buf = sample_buf.get_or_insert_with(|| SampleBuffer::<f32>::new(cap, spec));
            buf.copy_interleaved_ref(audio_buf);
            all_samples.extend_from_slice(buf.samples());
        }
    }

    let sum_sq: f64 = all_samples.iter().map(|s| (*s as f64) * (*s as f64)).sum();
    let rms = (sum_sq / all_samples.len() as f64).sqrt();

    if rms > 0.0 {
        // Assuming 16-bit reference level for dB calculation
        let rms_db = 20.0 * (rms / (i16::MAX as f64)).log10();
        Ok(rms_db)
    } else {
        Ok(-60.0) // silence
    }
}
```

### Pattern 3: yt-dlp subprocess (EXTD-02)
**What:** Use `tokio::process::Command` to spawn yt-dlp. The `tokio::process::Command` pattern is analogous to `std::process::Command` in `src/ffmpeg/probe.rs`, but async.

**When to use:** All YouTube download operations. yt-dlp must be pre-installed by user.

**Example:**
```rust
// Source: tokio::process::Command docs (tokio.rs) + Python youtube_service.py
use tokio::process::Command;

async fn get_video_formats(url: &str, proxy: Option<&str>) -> Result<Vec<VideoFormat>, YoutubeError> {
    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--dump-json", "--no-warnings", "--quiet", url]);
    if let Some(proxy_url) = proxy {
        cmd.args(["--proxy", proxy_url]);
    }

    let output = cmd.output().await
        .map_err(|e| YoutubeError::SubprocessError(e.to_string()))?;

    if !output.status.success() {
        return Err(YoutubeError::SubprocessError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| YoutubeError::SubprocessError(e.to_string()))?;

    let formats = info["formats"].as_array()
        .ok_or_else(|| YoutubeError::FormatNotFound("no formats array".into()))?;

    // Parse and filter formats (replicate Python resolution normalization)
    let mut result = Vec::new();
    for fmt in formats {
        let resolution = fmt["format_note"].as_str().unwrap_or("N/A").to_string();
        let base_res = resolution.split('p').next().map(|s| format!("{}p", s)).unwrap_or_default();
        let vcodec = fmt["vcodec"].as_str().unwrap_or("none");
        result.push(VideoFormat {
            format_id: fmt["format_id"].as_str().unwrap_or("").to_string(),
            resolution: base_res,
            ext: fmt["ext"].as_str().unwrap_or("").to_string(),
            vcodec: vcodec.to_string(),
        });
    }
    Ok(result)
}
```

### Anti-Patterns to Avoid
- **Blocking symphonia decode in async context:** symphonia is CPU-bound. Wrap decode in `tokio::task::spawn_blocking()`.
- **Shell string yt-dlp commands:** Use `Command::new("yt-dlp").args(&[...])` with argument array. Never use `shell(true)` with user-provided URLs.
- **Unvalidated config proxy URLs:** Validate proxy URL format before passing to reqwest or yt-dlp. Malformed proxy can crash the subprocess.
- **Unbounded download loops:** `download_videos()` must enforce `total_duration > audio_duration` break condition to prevent infinite download on very short search terms.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Audio decode to PCM | Custom format parser | symphonia 0.5.5 | Handles MP3/AAC/WAV/FLAC/OGG/Vorbis codec edge cases, container demuxing, sample format conversion |
| YouTube download | Custom YouTube API client | yt-dlp CLI | yt-dlp handles authentication, format negotiation, throttling, geo-restriction, and DRM. Re-implementing is infeasible |
| HTTP client with proxy | Raw TCP socket | reqwest::Proxy | reqwest already supports HTTP/HTTPS/SOCKS5 proxies, connection pooling, timeouts, and TLS |
| JSON parser | Manual string parsing | serde_json | serde provides type-safe deserialization for loudnorm JSON output and Pexels/Pixabay API responses |
| MD5 hashing | Manual SHA-256 or ad-hoc | md-5 crate | Python uses MD5 for URL cache keys. Must match for potential cross-compatibility. md-5 is pure Rust and audited |

**Key insight:** External dependencies (yt-dlp, FFmpeg) handle the most complex protocol and format negotiation logic. Rust code focuses on orchestration, error handling, and configuration routing — not on reimplementing format decoders or HTTP clients.

## Common Pitfalls

### Pitfall 1: FFmpeg loudnorm two-pass offset error
**What goes wrong:** The second pass normalization produces audio that does not reach the target LUFS, often off by 0.5-1.5 LU.
**Why it happens:** The first-pass `target_offset` value is not being captured and passed to the second pass. The FFmpeg dev explicitly states offset is "crucial for precise integrated loudness."
**How to avoid:** In the first pass, parse `target_offset` from the JSON output. In the second pass, add `offset=<value>` parameter. The `offset` is pre-limiter, so it does not affect true peak.
**Warning signs:** Output audio sounds quieter or louder than expected after normalization.

### Pitfall 2: yt-dlp format matching with "1080p60" and similar suffixes
**What goes wrong:** Resolution strings like "1080p60" (1080p at 60fps) fail exact string matching against "1080p".
**Why it happens:** YouTube yt-dlp returns `format_note` values with framerate suffixes. Python code normalizes by splitting on 'p'.
**How to avoid:** Always normalize resolution by extracting the number before 'p' and re-appending 'p': `"1080p60".split('p')[0] + "p"` -> `"1080p"`.
**Warning signs:** Downloaded video defaults to worst resolution (144p/240p) because requested resolution is not found.

### Pitfall 3: Pexels/Pixabay API key rotation race condition
**What goes wrong:** Multiple concurrent calls to `get_api_key()` using a global counter produce duplicate or skipped keys.
**Why it happens:** Python uses a module-level `requested_count` with simple increment, not thread-safe. In Rust's async context, non-atomic increment causes race conditions.
**How to avoid:** Use `AtomicUsize` with `fetch_add(1, Ordering::Relaxed)` and modulo. Or use `Arc<Mutex<usize>>` for simplicity. Note: Python version is not thread-safe either, so exact replication means `AtomicUsize` is acceptable improvement.
**Warning signs:** Intermittent 401/403 errors from API; rate limiting despite having valid keys.

### Pitfall 4: symphonia feature flag explosion
**What goes wrong:** Cargo compile times increase dramatically or certain audio formats fail to decode.
**Why it happens:** symphonia's default features include many format/codec bundles. Adding `mp3`, `flac`, `aac`, `isomp4` increases dependency tree.
**How to avoid:** Enable only needed features: `["mp3", "flac", "pcm", "wav", "vorbis"]`. PCM decode is needed even for MP3 files (symphonia first demuxes, then decodes PCM). Avoid `"all"` or `"all-codecs"` unless all formats are needed.
**Warning signs:** Compile times exceeding 5 minutes for the symphonia dependency tree.

### Pitfall 5: reqwest proxy format mismatch with yt-dlp
**What goes wrong:** reqwest accepts `http://user:pass@host:port` format but yt-dlp `--proxy` may need different syntax or SOCKS5 prefix.
**Why it happens:** reqwest::Proxy and yt-dlp `--proxy` have minor syntax differences (e.g., SOCKS5: `socks5://host:port` for reqwest, `socks5://host:port` for yt-dlp same).
**How to avoid:** Store clean proxy URL in config (no authentication in URL), pass directly to both reqwest::Proxy::http() and yt-dlp `--proxy`. If authentication is needed, it must be documented as URL-encoded.
**Warning signs:** yt-dlp downloads fail with proxy errors while reqwest API calls succeed, or vice versa.

## Code Examples

### Loudnorm JSON Response Parsing (EXTD-01)
```rust
// Source: FFmpeg loudnorm filter documentation (ffmpeg.org)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoudnormData {
    pub input_i: String,       // e.g., "-27.61"
    pub input_lra: String,     // e.g., "18.06"
    pub input_tp: String,      // e.g., "-4.47"
    pub input_thresh: String,  // e.g., "-39.20"
    pub target_offset: String, // e.g., "0.58"
    pub output_i: String,
    pub output_lra: String,
    pub output_tp: String,
    pub output_thresh: String,
    pub normalization_type: String,
}

impl LoudnormData {
    /// Parse string values to f64 for use in second-pass filter parameters.
    pub fn measured_I(&self) -> f64 { self.input_i.parse().unwrap_or(0.0) }
    pub fn measured_LRA(&self) -> f64 { self.input_lra.parse().unwrap_or(0.0) }
    pub fn measured_TP(&self) -> f64 { self.input_tp.parse().unwrap_or(0.0) }
    pub fn measured_thresh(&self) -> f64 { self.input_thresh.parse().unwrap_or(0.0) }
    pub fn offset(&self) -> f64 { self.target_offset.parse().unwrap_or(0.0) }
}
```

### VolumeConfig Struct (EXTD-04)
```rust
// Source: Python AudioConfig DEFAULT_VOLUMES + profiles (audio_config.py)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeConfig {
    pub tts_volume: f64,
    pub original_volume: f64,
    pub bgm_volume: f64,
}

impl VolumeConfig {
    pub fn validate(&self) -> Result<(), AudioError> {
        let min = 0.0;
        let max = 2.0;
        if self.tts_volume < min || self.tts_volume > max {
            return Err(AudioError::InvalidVolume(format!("tts_volume={}", self.tts_volume)));
        }
        // ... similar for original_volume, bgm_volume
        Ok(())
    }
}
```

### Pexels API Response (EXTD-03)
```rust
// Source: Pexels API docs (pexels.com/api/documentation)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PexelsSearchResponse {
    pub page: u32,
    pub per_page: u32,
    pub total_results: u32,
    pub videos: Vec<PexelsVideo>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideo {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    pub url: String,
    pub video_files: Vec<PexelsVideoFile>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoFile {
    pub id: u64,
    pub quality: String,         // "hd" or "sd"
    pub file_type: String,       // "video/mp4"
    pub width: u32,
    pub height: u32,
    pub link: String,
}
```

### Pixabay API Response (EXTD-03)
```rust
// Source: Pixabay API docs (pixabay.com/api/docs)
#[derive(Debug, Deserialize)]
pub struct PixabaySearchResponse {
    pub total: u32,
    pub total_hits: u32,
    pub hits: Vec<PixabayHit>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayHit {
    pub id: u64,
    pub duration: u32,
    pub videos: PixabayVideos,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideos {
    pub large: Option<PixabayVideoVariant>,
    pub medium: Option<PixabayVideoVariant>,
    pub small: Option<PixabayVideoVariant>,
    pub tiny: Option<PixabayVideoVariant>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoVariant {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| symphonia 0.4.x | symphonia 0.5.5 | October 2025 | API changes: `SampleBuffer::copy_interleaved_ref` vs `copy_interleaved_typed`. 0.5 also stabilizes MP4/ISOBMFF support |
| yt-dlp 2023 | yt-dlp 2026.02.21 | Ongoing | Format sorting API added `-S` flag. Python code uses old `format_id` matching — Rust should match same behavior |
| ffmpeg-sidecar 1.x | ffmpeg-sidecar 2.5.1 | 2025 | Transitioned from bundled binary approach to managed download. Already in use by Phase 1 |

**Deprecated/outdated:**
- **Pydub (Python):** Python's `audio_normalizer.py` falls back to pydub for RMS. Rust has no pydub equivalent — symphonia replaces this.
- **moviepy (Python):** Python uses moviepy for audio volume manipulation. Rust uses FFmpeg + symphonia only.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | symphonia 0.5.5's `SampleBuffer::copy_interleaved_ref` correctly converts decoded audio to f32 interleaved | Standard Stack | Low — API is documented and stable across 0.5.x |
| A2 | ffmpeg-sidecar 2.5.1 can capture loudnorm JSON output from stderr via `FfmpegEvent::Log` | Pattern 1 | Medium — ffmpeg-sidecar event model may not surface all stderr lines. Alternative: use `std::process::Command` directly |
| A3 | `tokio::process::Command` supports Windows paths correctly | Pattern 3 | Medium — Windows CLI path handling may differ. Test with actual yt-dlp install on Windows |
| A4 | reqwest's `Client::builder().proxy()` handles the config proxy URL format without transformation | Pattern: Material Search | Low — reqwest proxy docs show simple URL passing works |

## Open Questions (RESOLVED)

1. **ffmpeg-sidecar stderr capture for loudnorm JSON** (RESOLVED)
   - What we know: `FfmpegEvent::Log(level, msg)` captures FFmpeg log output. loudnorm `print_format=json` outputs to stderr.
   - What's unclear: Whether ffmpeg-sidecar exposes all stderr lines as `FfmpegEvent::Log` or some go to `FfmpegEvent::Error`. The clipped JSON may need special handling.
   - Recommendation: Implement first pass using `std::process::Command` capturing stderr directly (as `src/ffmpeg/probe.rs` does for ffprobe). This is safer than relying on ffmpeg-sidecar event filtering. Fall back to ffmpeg-sidecar only if integration issues arise.
   - **RESOLVED:** Plan 01 and Plan 02 both use `std::process::Command` directly (not ffmpeg-sidecar events) for loudnorm stderr capture, consistent with the `src/ffmpeg/probe.rs` pattern.

2. **yt-dlp JSON output parsing for format list** (RESOLVED)
   - What we know: `yt-dlp --dump-json URL` outputs a single JSON object with a `formats` array. Python uses `yt_dlp.YoutubeDL` internal API for this.
   - What's unclear: The --dump-json output structure for the formats array may differ from Python API's `info.get('formats', [])` in edge cases (DASH manifests, adaptive formats).
   - Recommendation: Test format parsing with a range of YouTube URLs and validate against Python output. Use `serde_json::Value` for flexible access initially.
   - **RESOLVED:** Plan 03 uses `serde_json::Value` for flexible format list parsing, handling edge cases in DASH manifests and adaptive formats gracefully.

3. **symphonia default features** (RESOLVED)
   - What we know: symphonia default enables flac, mkv, ogg, pcm, vorbis, wav. MP3 requires explicit `"mp3"` feature.
   - What's unclear: Whether AAC support is needed for RMS fallback (TTS outputs are typically WAV/MP3, but downloaded audio could be AAC).
   - Recommendation: Enable `"mp3"` and optionally `"aac"` features. Fallback should handle the most common formats. AAC can be added when a need arises.
   - **RESOLVED:** Plan 02 uses `features = ["mp3", "flac", "pcm", "wav", "vorbis"]` — MP3 explicitly enabled; AAC deferred until a concrete need arises.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| yt-dlp | YouTube download (EXTD-02) | Yes | 2026.02.21 | — (must be installed by user) |
| FFmpeg | Audio normalization (EXTD-01) | Bundled via ffmpeg-sidecar | 2.5.1 | ffmpeg-sidecar auto-downloads |
| Rust toolchain | All | Yes | — | — |

**Missing dependencies with no fallback:**
- None — all required tools are either available (yt-dlp) or bundled (FFmpeg via ffmpeg-sidecar)

## Validation Architecture

> Note: This section describes the test infrastructure plan. Since Phase 11 is a domain module phase (not Wave 0 foundation), existing test patterns apply.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | built-in `#[test]` + `#[tokio::test]` |
| Config file | none (Cargo.toml declares `[dev-dependencies]`) |
| Quick run command | `cargo test -p narratoai-core --lib audio:: -- --nocapture` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXTD-01 | `analyze_lufs()` returns measured loudness data | unit (mocked FFmpeg) | `cargo test analyze_lufs` | Wave 0 |
| EXTD-01 | `normalize_lufs()` produces output with correct target LUFS | integration `#[ignore]` | `cargo test normalize_lufs -- --ignored` | Wave 0 |
| EXTD-01 | RMS fallback returns valid dB value for WAV/MP3 | unit | `cargo test get_audio_rms` | Wave 0 |
| EXTD-02 | `download_video()` matches resolution strings correctly | unit | `cargo test resolution_matching` | Wave 0 |
| EXTD-02 | Invalid URL returns YoutubeError | unit (mocked yt-dlp) | `cargo test invalid_url` | Wave 0 |
| EXTD-03 | Pexels search response parsing is correct | unit (mock HTTP) | `cargo test pexels_search` | Wave 0 |
| EXTD-03 | MD5 cache key generation matches Python | unit | `cargo test cache_key_md5` | Wave 0 |
| EXTD-04 | Volume profile values match Python defaults | unit | `cargo test volume_profiles` | Wave 0 |
| EXTD-04 | Volume validation clamps out-of-range values | unit | `cargo test volume_validation` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib` (unit tests only, < 10s)
- **Per wave merge:** `cargo test` (all unit + non-ignored integration tests)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Create test audio fixture files (small WAV/MP3) in `tests/fixtures/` for RMS tests
- [ ] Create JSON fixture files for loudnorm output, Pexels response, Pixabay response, yt-dlp --dump-json output
- [ ] Add `wiremock` (already in dev-dependencies) for HTTP mock in material tests
- [ ] `symphonia` added to dev-dependencies for RMS test execution

*(If no gaps: "None — existing test infrastructure covers all phase requirements")*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes | Validate yt-dlp URL format (must be http/https) before passing to CLI. Validate proxy URLs from config |
| V6 Cryptography | No | MD5 is used only for cache key generation, not for security |
| V8 File Upload/Download | Yes | Pexels/Pixabay video downloads: validate file size, verify via ffprobe, check content-type if available |

### Known Threat Patterns for This Phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| CLI argument injection (yt-dlp) | Tampering | Use `Command::args()` with argument array, NOT shell string. Never pass unvalidated user input as shell arguments |
| Arbitrary URL download (Pexels/Pixabay) | Spoofing | Only download from pexels.com/pixabay.com CDN URLs returned by their API — validated by response origin |
| Proxy credential leak | Information Disclosure | Document that proxy URL format should not include credentials in config file; use env vars instead |

## Sources

### Primary (HIGH confidence)
- [VERIFIED: Cargo.toml] — ffmpeg-sidecar 2.5.1, reqwest 0.13, tokio 1.52.1, serde 1.0.228, thiserror 2.0.18
- [VERIFIED: crate registry] — symphonia 0.5.5 stable release, md-5 0.10.6
- [VERIFIED: WebFetch Pexels openpublicapis] — Pexels API /videos/search endpoint format, response JSON structure, rate limits
- [VERIFIED: WebFetch pixabay openpublicapis] — Pixabay API /api/videos/ endpoint format, response JSON structure (hits[].videos.{large,medium,small,tiny})

### Secondary (MEDIUM confidence)
- [CITED: github.com/pdeljanov/Symphonia GETTING_STARTED.md] — symphonia decode loop pattern with SampleBuffer<f32>
- [CITED: ffmpeg.org ffmpeg-filters.html] — loudnorm two-pass filter parameters: measured_I, measured_LRA, measured_TP, measured_thresh, offset
- [CITED: github.com/yt-dlp/yt-dlp] — yt-dlp CLI --dump-json, -f format selection, --proxy support
- [CITED: Python audio_normalizer.py] — Python reference: two-pass loudnorm, RMS via pydub, calculate_volume_adjustment
- [CITED: Python youtube_service.py] — Python reference: _get_video_formats, resolution normalization ("1080p60" -> "1080p"), manual format_id download
- [CITED: Python material.py] — Python reference: Pexels/Pixabay search, save_video with MD5 cache, download_videos orchestration
- [CITED: Python audio_config.py] — Python reference: VolumeConfig values, profiles, content types, PROCESSING_CONFIG, MIXING_CONFIG

### Tertiary (LOW confidence)
- [ASSUMED] — ffmpeg-sidecar 2.5.1 `FfmpegEvent::Log` exposes loudnorm JSON output (not verified against actual FFmpeg run)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crate versions verified against registry; dependencies already in Cargo.toml except symphonia + md-5
- Architecture: HIGH — patterns derived from existing Phase 1 code (ffmpeg probe.rs, ffmpeg command.rs) and Python reference implementations
- Pitfalls: MEDIUM — based on known FFmpeg loudnorm offset issue (documented by ffmpeg dev) and Python code review; yt-dlp format matching tested against Python behavior

**Research date:** 2026-04-30
**Valid until:** 2026-05-30 (standard stack stable; yt-dlp version may update but CLI API stable)
