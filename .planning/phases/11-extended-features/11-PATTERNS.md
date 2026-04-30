# Phase 11: Extended Features — Pattern Map

**Mapped:** 2026-04-30
**Files classified:** 15 (8 new, 4 modified, 3 test files)
**Analogs found:** 15 / 15

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/audio/mod.rs` | config | N/A | `src/visual/mod.rs` | exact (same role) |
| `src/audio/normalizer.rs` | utility | file-I/O, batch (FFmpeg subprocess + symphonia decode) | `src/ffmpeg/command.rs`, `src/ffmpeg/probe.rs` | role-match (spawn_blocking + subprocess) |
| `src/audio/volume.rs` | utility | transform (pure data + functions) | `src/script/mod.rs`, `src/script/types.rs` | role-match (pure functions + config structs) |
| `src/youtube/mod.rs` | config | N/A | `src/visual/mod.rs` | exact (same role) |
| `src/youtube/downloader.rs` | service | request-response (CLI subprocess) | `src/ffmpeg/probe.rs` | exact (std::process::Command + JSON parse) |
| `src/material/mod.rs` | config | N/A | `src/visual/mod.rs` | exact (same role) |
| `src/material/searcher.rs` | service | request-response (HTTP) | `src/llm/openai_compatible.rs` | role-match (reqwest + API key) |
| `src/material/downloader.rs` | utility/service | file-I/O, CRUD (HTTP download + caching) | `src/script/mod.rs` (save_script), `src/ffmpeg/probe.rs` | partial-match (file write + validation) |
| `src/config/types.rs` | config | N/A | `src/config/types.rs` (existing) | exact (extend same file) |
| `src/config/defaults.rs` | config | N/A | `src/config/defaults.rs` (existing) | exact (extend same file) |
| `src/lib.rs` | config | N/A | `src/lib.rs` (existing) | exact (extend same file) |
| `Cargo.toml` | config | N/A | `Cargo.toml` (existing) | exact (extend same file) |
| `tests/audio_integration.rs` | test | N/A | `tests/ffmpeg_test.rs` | role-match (FFmpeg-dependent tests) |
| `tests/youtube_integration.rs` | test | N/A | `tests/script_test.rs` | role-match (func tests) |
| `tests/material_integration.rs` | test | N/A | `tests/llm_test.rs` (wiremock pattern) | role-match (HTTP mock tests) |

## Pattern Assignments

### `src/audio/mod.rs` (config, N/A)

**Analog:** `src/visual/mod.rs` (lines 1-4)

```rust
// Analog: src/visual/mod.rs lines 1-4
pub mod error;
pub mod types;
pub mod frame_extractor;
pub mod analyzer;
```

**Pattern to follow:**
```rust
pub mod normalizer;
pub mod volume;
```

---

### `src/audio/normalizer.rs` (utility, file-I/O)

**Analog (FFmpeg subprocess):** `src/ffmpeg/probe.rs` (lines 1-7, 22-54)

**Imports pattern** (lines 1-5):
```rust
use std::path::Path;
use std::process::Command;

use crate::error::FFmpegError;
```

**Subprocess + JSON parse pattern** (lines 22-56):
```rust
// Analog: src/ffmpeg/probe.rs lines 22-56 — std::process::Command + serde_json parse
pub fn probe_video(path: &Path) -> Result<VideoInfo, FFmpegError> {
    let ffprobe_bin = ffmpeg_sidecar::ffprobe::ffprobe_path();

    let output = match Command::new(&ffprobe_bin)
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path.as_os_str())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(FFmpegError::BinaryNotFound);
            }
            return Err(FFmpegError::SpawnFailed(e.to_string()));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FFmpegError::ExecutionError(format!(
            "ffprobe exited with code {:?}: {}",
            output.status.code(),
            stderr,
        )));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| FFmpegError::OutputParseError(e.to_string()))?;
    // ...extract fields from json...
    Ok(VideoInfo { ... })
}
```

**Analog (spawn_blocking):** `src/ffmpeg/command.rs` (lines 48-148)

**spawn_blocking async wrapper pattern** (lines 15-24):
```rust
// Analog: src/ffmpeg/command.rs lines 15-24 — generic spawn_blocking wrapper
pub async fn run_ffmpeg<F, R>(blocking_fn: F) -> Result<R, FFmpegError>
where
    F: FnOnce() -> Result<R, FFmpegError> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(blocking_fn)
        .await
        .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
}
```

**Error type pattern:**
```rust
// Analog: src/error.rs lines 23-39 — thiserror + Chinese Display
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("LUFS 分析失败: {0}")]
    LoudnormAnalysisFailed(String),

    #[error("LUFS 标准化失败: {0}")]
    LoudnormNormalizeFailed(String),

    #[error("RMS 回退失败: {0}")]
    RmsFallbackFailed(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无效音量值: {0}")]
    InvalidVolume(String),
}
```

---

### `src/audio/volume.rs` (utility, transform)

**Analog (pure data structs):** `src/script/types.rs` (lines 1-2, 44-66)

**Imports pattern:**
```rust
use serde::{Deserialize, Serialize};
```

**Core struct pattern** (lines 1-2, 30-33):
```rust
// Analog: src/config/types.rs lines 182-192 — serde config struct + Default
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioSection {
    #[serde(default)]
    pub target_lufs: f64,
    #[serde(default)]
    pub max_peak: f64,
    #[serde(default)]
    pub tts_volume: f64,
    // ... more fields
}
```

**Analog (Default impl):** `src/config/defaults.rs` (lines 103-111)

```rust
// Analog: src/config/defaults.rs lines 103-111
impl Default for AudioSection {
    fn default() -> Self {
        Self {
            target_lufs: -23.0,
            max_peak: -1.0,
            enable_smart_volume: false,
            // etc.
        }
    }
}
```

**VolumeConfig struct pattern:**
```rust
// RESEARCH.md lines 503-521 — VolumeConfig with validate()
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeConfig {
    pub tts_volume: f64,
    pub original_volume: f64,
    pub bgm_volume: f64,
}

impl VolumeConfig {
    pub fn validate(&self) -> Result<(), AudioError> {
        // validation logic
    }
}
```

---

### `src/youtube/mod.rs` (config, N/A)

**Analog:** `src/visual/mod.rs` (lines 1-4)

```rust
// Analog: src/visual/mod.rs lines 1-4
pub mod error;
pub mod types;
pub mod frame_extractor;
pub mod analyzer;
```

**Pattern to follow:**
```rust
pub mod downloader;
```

---

### `src/youtube/downloader.rs` (service, CLI subprocess)

**Analog:** `src/ffmpeg/probe.rs` (lines 1-7, 22-56, 110-157)

**Imports pattern** (lines 1-4):
```rust
use std::path::Path;
use std::process::Command;

use crate::error::FFmpegError;
```

**Core subprocess pattern** (lines 22-44):
```rust
// Analog: src/ffmpeg/probe.rs lines 22-44 — Command::new + args + output
pub fn probe_video(path: &Path) -> Result<VideoInfo, FFmpegError> {
    let ffprobe_bin = ffmpeg_sidecar::ffprobe::ffprobe_path();
    let output = match Command::new(&ffprobe_bin)
        .args([...])
        .arg(path.as_os_str())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(FFmpegError::BinaryNotFound);
            }
            return Err(FFmpegError::SpawnFailed(e.to_string()));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FFmpegError::ExecutionError(format!(...)));
    }
    // parse JSON output...
}
```

**Async subprocess pattern** (from RESEARCH.md lines 381-418):
```rust
// Usage pattern: tokio::process::Command for async yt-dlp calls
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
    // ... parse formats ...
}
```

**Error type pattern** (from `src/script/error.rs` lines 1-20):
```rust
// Analog: src/script/error.rs lines 4-20
#[derive(thiserror::Error, Debug)]
pub enum YoutubeError {
    #[error("yt-dlp 子进程错误: {0}")]
    SubprocessError(String),

    #[error("未找到匹配的格式: {0}")]
    FormatNotFound(String),

    #[error("无效的 URL: {0}")]
    InvalidUrl(String),

    #[error("下载失败: {0}")]
    DownloadFailed(String),
}
```

---

### `src/material/mod.rs` (config, N/A)

**Analog:** `src/visual/mod.rs` (lines 1-4)

```rust
// Analog: src/visual/mod.rs lines 1-4
pub mod error;
pub mod types;
pub mod frame_extractor;
pub mod analyzer;
```

**Pattern to follow:**
```rust
pub mod searcher;
pub mod downloader;
```

---

### `src/material/searcher.rs` (service, HTTP request-response)

**Analog:** `src/llm/openai_compatible.rs` (lines 1-10, 34-50)

**Imports pattern** (lines 1-6):
```rust
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use backoff::ExponentialBackoff;
use tokio::sync::Semaphore;

use crate::error::LLMError;
```

**API response struct** (from RESEARCH.md lines 526-588):
```rust
// Pexels response types — serde Deserialize with rename_all
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
```

**Error type pattern:**
```rust
// Analog: src/error.rs — thiserror + Chinese Display
#[derive(thiserror::Error, Debug)]
pub enum MaterialError {
    #[error("API 请求失败: {0}")]
    ApiRequest(String),

    #[error("API 认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("文件验证失败: {0}")]
    ValidationFailed(String),
}
```

---

### `src/material/downloader.rs` (utility/service, file-I/O)

**Analog (file write + temp file):** `src/script/mod.rs` (lines 131-154)

**File save with temp file pattern** (lines 144-154):
```rust
// Analog: src/script/mod.rs lines 144-154 — temp file + atomic rename
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    validate(script)?;
    let json = serde_json::to_string_pretty(script).map_err(ScriptError::JsonParse)?;
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, json.as_bytes()).map_err(ScriptError::Io)?;
    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path); // best-effort cleanup
        return Err(ScriptError::Io(e));
    }
    Ok(())
}
```

**Analog (ffprobe validation):** `src/ffmpeg/probe.rs` (lines 114-157)
```rust
// Analog: src/ffmpeg/probe.rs lines 114-157 — ffprobe audio validation
pub fn probe_audio(path: &Path) -> Result<f64, FFmpegError> {
    let ffprobe_bin = ffmpeg_sidecar::ffprobe::ffprobe_path();
    let output = match Command::new(&ffprobe_bin)
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0"])
        .arg(path.as_os_str())
        .output()
    { ... };
    // parse duration...
}
```

---

### `src/config/types.rs` (modify — add AudioSection, extend AppSection)

**Analog:** Existing `src/config/types.rs` patterns

**New section pattern** (lines 182-203, existing FramesSection):
```rust
// Follow existing pattern: FramesSection (lines 193-203)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FramesSection {
    #[serde(default)]
    pub frame_interval_input: u64,
    #[serde(default)]
    pub vision_batch_size: u32,
    #[serde(default)]
    pub vision_max_concurrency: u32,
}
```

**New AudioSection** (same pattern):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioSection {
    #[serde(default)]
    pub target_lufs: f64,
    #[serde(default)]
    pub max_peak: f64,
    #[serde(default)]
    pub enable_audio_normalization: bool,
    // ...11 fields per D-19...
}
```

**Extend AppConfig** (lines 4-27):
```rust
// Add to existing AppConfig struct (lines 4-27)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    // ...existing fields...
    #[serde(default)]
    pub audio: AudioSection,
    // material section not needed as config keys live in [app] section
}
```

**Extend AppSection** (lines 30-59):
```rust
// Add to existing AppSection struct (lines 30-59)
pub struct AppSection {
    // ...existing fields...
    #[serde(default)]
    pub pexels_api_keys: Vec<String>,
    #[serde(default)]
    pub pixabay_api_keys: Vec<String>,
}
```

---

### `src/config/defaults.rs` (modify — add AudioSection defaults)

**Analog:** `src/config/defaults.rs` lines 103-111

```rust
// Follow FramesSection Default pattern (lines 103-111)
impl Default for AudioSection {
    fn default() -> Self {
        Self {
            target_lufs: -23.0,
            max_peak: -1.0,
            enable_audio_normalization: false,
            enable_smart_volume: false,
            tts_volume: 1.0,
            original_volume: 1.0,
            bgm_volume: 0.5,
            sample_rate: 44100,
            channels: 2,
            bitrate: 192_000,
            crossfade_duration: 0.0,
            bgm_fade_out: 0.0,
        }
    }
}
```

---

### `src/lib.rs` (modify — add pub mod declarations)

**Analog:** `src/lib.rs` lines 1-9

```rust
// Analog: src/lib.rs lines 1-9
pub mod config;
pub mod error;
pub mod tts;
pub mod llm;
pub mod ffmpeg;
pub mod jianying;
pub mod script;
pub mod prompt;
pub mod visual;
```

**Add to this list:**
```rust
pub mod audio;
pub mod youtube;
pub mod material;
```

---

### `Cargo.toml` (modify — add symphonia, md-5)

**Analog:** Existing dependency blocks

```toml
# Add under [dependencies] (lines 10-32)
symphonia = { version = "0.5", features = ["mp3", "flac", "pcm", "wav", "vorbis"] }
md-5 = "0.10"
```

---

## Shared Patterns

### Error Enum Pattern
**Source:** `src/error.rs` (lines 1-40), `src/script/error.rs` (lines 4-20), `src/visual/error.rs` (lines 5-21)
**Apply to:** All new domain error enums (AudioError, YoutubeError, MaterialError)

```rust
// Core pattern: thiserror derive + Chinese Display + #[from] for common types
#[derive(thiserror::Error, Debug)]
pub enum AudioError {
    #[error("LUFS 分析失败: {0}")]
    LoudnormAnalysisFailed(String),

    #[error("LUFS 标准化失败: {0}")]
    LoudnormNormalizeFailed(String),

    #[error("RMS 回退失败: {0}")]
    RmsFallbackFailed(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无效音量值: {0}")]
    InvalidVolume(String),
}
```

Error unit tests pattern (from `src/error.rs` lines 150-218):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_chinese() {
        let err = AudioError::LoudnormAnalysisFailed("test".into());
        let msg = err.to_string();
        assert!(msg.contains("LUFS 分析失败"), "消息应包含中文: {}", msg);
    }
    // one test per variant
}
```

### Config Section Pattern
**Source:** `src/config/types.rs` + `src/config/defaults.rs`
**Apply to:** AudioSection, AppSection extensions

```rust
// types.rs: serde struct with #[serde(default)] on each field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioSection {
    #[serde(default)]
    pub target_lufs: f64,
    // ...
}

// defaults.rs: Default impl
impl Default for AudioSection {
    fn default() -> Self {
        Self {
            target_lufs: -23.0,
            // ...
        }
    }
}
```

### CLI Subprocess Pattern
**Source:** `src/ffmpeg/probe.rs` (lines 22-56)
**Apply to:** `src/youtube/downloader.rs`

Both sync (std::process::Command) and async (tokio::process::Command) variants. Always use argument array, never shell strings. Check stdout for JSON parsing, check status.success() for exit code, capture stderr for error messages.

### Pure Function API Pattern
**Source:** `src/script/mod.rs` lines 10-129, `src/ffmpeg/probe.rs` lines 22-108
**Apply to:** All new utility/service functions

No hidden state, no self, no global. All parameters explicit. Return Result<T, E> with domain-specific error types.

```rust
// Pattern: pure function, explicit params, Result return
pub fn normalize_lufs(input: &Path, output: &Path, target_lufs: f64, max_peak: f64) -> Result<(), AudioError>
```

### Unit Test Pattern
**Source:** `src/script/mod.rs` (lines 156-608), `src/ffmpeg/probe.rs` (lines 160-194)
**Apply to:** All new modules

Unit tests in `#[cfg(test)] mod tests` at bottom of each file. Test Chinese error messages, test error cases (nonexistent files, invalid input), test pure function behavior with temp files.

### Integration Test Pattern
**Source:** `tests/ffmpeg_test.rs` (lines 1-8, 9-34), `tests/llm_test.rs` (lines 1-13, 19-47)

Integration tests in `tests/` directory. External dependency detection at top (`find_ffmpeg()` pattern). Create test fixtures with ffmpeg lavfi. Mock HTTP with wiremock for API tests.

```rust
// tests/ffmpeg_test.rs lines 9-34 — find external binary helper
fn find_ffmpeg() -> PathBuf {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return PathBuf::from("ffmpeg");
        }
    }
    let cached = ffmpeg_sidecar::paths::ffmpeg_path();
    if cached.exists() {
        return cached;
    }
    panic!("FFmpeg 不可用。请安装 FFmpeg...");
}

// tests/llm_test.rs lines 19-47 — wiremock for HTTP mocking
#[tokio::test]
async fn test_registry_register_and_get() {
    let mut registry = Registry::new();
    let provider = create_test_provider("http://localhost:0/v1");
    registry.register("test_provider", Arc::new(provider));
    let got = registry.get("test_provider");
    assert!(got.is_ok());
}
```

## No Analog Found

None — all 15 files have close matches in the existing codebase.

## Metadata

**Analog search scope:** `src/` (all Rust source), `tests/` (all test files)
**Files scanned:** 45+ source files including all Rust modules and test files
**Pattern extraction date:** 2026-04-30
