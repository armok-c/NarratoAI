# Technology Stack

**Project:** NarratoAI Rust Rewrite
**Researched:** 2026-04-27
**Context:** Python v0.7.8 complete backend rewrite to Rust + Tauri 2.0

## Recommended Stack

### Core Language & Build

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 2024 edition | Primary language | Memory safety, zero-cost abstractions, async support, excellent FFmpeg/process interop. Edition 2024 is current stable. |
| Cargo | bundled | Build system, dependency management | Standard Rust toolchain. Replaces pip + requirements.txt with proper lock file (Cargo.lock). |

### Async Runtime

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio | 1.x (latest) | Async runtime | De facto standard. Provides full feature set: multi-threaded scheduler, fs, process, net, sync, time, macros. Uses IOCP on Windows (this project targets Windows). Features needed: `full` or select `rt-multi-thread`, `fs`, `process`, `net`, `sync`, `time`, `macros`. |
| futures | 0.3.x | Async utilities | Stream, Sink, FutureExt, StreamExt. Needed for processing async streams from FFmpeg output and WebSocket messages. |

### HTTP Client (LLM API + TTS API)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| reqwest | 0.12.x | Async HTTP client | Most popular Rust HTTP client. Built on tokio + hyper. Supports JSON, form, multipart, proxy, TLS, streaming, cookies, SOCKS. Required features: `json`, `multipart`, `stream`, `rustls-tls` (or `native-tls`). Replaces Python's `requests` + `openai` SDK. |

### Video/Audio Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| ffmpeg-sidecar | 2.x | FFmpeg CLI wrapper | Wraps FFmpeg as subprocess (matches Python's pattern). Builder API like `std::process::Command`. Auto-download capability. Iterator interface for streaming output. ffprobe integration. No FFI complexity, no C dependency, no build issues. **Better than ffmpeg-next for this project** because Python already uses subprocess pattern and ffmpeg-sidecar is simpler to build/debug. |
| which | 7.x | Locate FFmpeg binary | Cross-platform binary finder. Used to locate system-installed FFmpeg/ffprobe before falling back to auto-download. |

### WebSocket (Edge-TTS)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio-tungstenite | 0.26.x | Async WebSocket client | Edge-TTS requires WebSocket connection to Microsoft's speech synthesis service. Built on tungstenite + tokio. Supports TLS via native-tls or rustls. `connect_async()` for connections. `WebSocketStream` implements `Stream + Sink` for ergonomic message handling. |

### Serialization & Configuration

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| serde | 1.x | Serialization framework | Foundation for all data serialization in Rust. Derive macros for JSON, TOML, and custom formats. |
| serde_json | 1.x | JSON handling | Script files are JSON. LLM API responses are JSON. TTS API payloads are JSON. Replaces Python's json module + pydantic serialization. |
| toml | 0.8.x | TOML configuration | Direct `serde` integration. `from_str`/`to_string`/`to_string_pretty`. Replaces Python's `toml`/`tomli`. Config schema derives `Serialize`/`Deserialize`. |

### Error Handling

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| anyhow | 1.x | Application error handling | Ergonomic error chaining for the application layer. `context()` for adding context to errors. Used in binary/app code where specific error types are not needed. |
| thiserror | 2.x | Library error types | Derive macro for defining custom error enums. Used in library crates (tts, llm, pipeline) where callers need to match on specific error variants. Replaces Python's inconsistent mix of `ValueError`, custom exceptions, and silent failures. |

### Logging & Diagnostics

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tracing | 0.1.x | Structured logging + diagnostics | Replaces Python's loguru. Spans and events for structured output. `#[instrument]` attribute for automatic function tracing. Ecosystem: `tracing-subscriber` (output formatting), `tracing-appender` (file logging), `tracing-futures` (async integration). |
| tracing-subscriber | 0.3.x | Log output configuration | Configures formatting, filtering, output destinations. `fmt::layer()` for console, `fmt::layer().with_writer(file)` for file output. |

### Image Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| image | 0.25.x | Image manipulation | Replaces Python's Pillow. Frame extraction validation, thumbnail generation, format conversion. Needed for vision analyzer keyframe processing before sending to LLM. |

### Tauri Integration

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri | 2.x | Desktop application framework | Command layer between Rust backend and Vue 3 frontend. `#[tauri::command]` for exposing Rust functions. `State` for shared state management. `Channel` API for streaming progress. `Event` system for notifications. Produces native installers (MSI/DMG/AppImage). |
| tauri-build | 2.x | Build script support | Tauri build integration for Cargo. |

### Utility Libraries

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| uuid | 1.x | Unique identifiers | Task IDs, job tracking. v4 for random IDs. |
| chrono | 0.4.x | Date/time handling | Timestamps in logs, task state, SRT subtitle timing. |
| base64 | 0.22.x | Base64 encoding | Encoding images for vision LLM API requests (base64 image payloads in OpenAI-compatible API). |
| tempfile | 3.x | Temporary files/directories | Intermediate video/audio files during pipeline processing. Auto-cleanup on drop. |
| regex | 1.x | Regular expressions | Parsing FFmpeg output, log parsing, subtitle timestamp extraction. |

## TTS Engine Implementation Strategy

Each Python TTS engine maps to a specific Rust implementation approach. All TTS engines will implement a common `TtsProvider` trait.

| Engine | Protocol | Rust Implementation | Difficulty | Config Section |
|--------|----------|---------------------|------------|----------------|
| edge_tts | WebSocket to Microsoft Edge | tokio-tungstenite -- implement Edge-TTS WebSocket protocol (connect, send SSML config, receive binary audio chunks). Protocol is well-documented in Python edge-tts library. | High | `[ui]` edge_* |
| azure_speech | Native SDK + REST | reqwest HTTP POST to Azure Cognitive Services REST API. Avoid native SDK bindings (maintainability concern). Use the REST v1 endpoint directly. | Medium | `[azure]` |
| tencent_tts | Tencent Cloud SDK | reqwest HTTP POST with Tencent Cloud credential signing (TC3-HMAC-SHA256). Implement signing algorithm in pure Rust. | Medium | `[tencent]` |
| soulvoice | HTTP POST (Bearer token) | reqwest HTTP POST with JSON body and Authorization header. Simplest engine. | Low | `[soulvoice]` |
| tts_qwen | DashScope API | reqwest HTTP POST to DashScope endpoint. JSON request/response with Alibaba-specific headers. | Low | `[tts_qwen]` |
| indextts2 | HTTP multipart (voice clone) | reqwest multipart POST. Upload reference audio + text, receive synthesized audio. Local deployment endpoint. | Medium | `[indextts2]` |
| doubaotts | HTTP POST JSON (Bearer token) | reqwest HTTP POST with JSON body and Authorization header. Similar to soulvoice. | Low | `[doubaotts]` |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| FFmpeg binding | ffmpeg-sidecar (CLI wrapper) | ffmpeg-next (FFI bindings) | ffmpeg-next links to libav* C libraries -- complex build, cross-compilation issues, requires system FFmpeg dev headers. ffmpeg-sidecar uses the same subprocess approach Python already uses, with auto-download. No FFI = no C dependency = reliable builds. |
| FFmpeg binding | ffmpeg-sidecar | ffmpeg-cli (another wrapper) | Less maintained, fewer features. ffmpeg-sidecar has Iterator interface, auto-download, ffprobe integration. |
| HTTP client | reqwest | hyper | hyper is lower-level (raw HTTP/1.1, HTTP/2). reqwest provides JSON, multipart, cookies, proxy out of the box. For an application (not a framework), reqwest is the right abstraction level. |
| HTTP client | reqwest | ureq | ureq is synchronous. This project needs async for concurrent TTS, LLM calls, and progress streaming. |
| TLS backend | rustls-tls | native-tls | rustls is pure Rust, no OpenSSL dependency. Better cross-compilation story. However, if Windows certificate store integration is needed, native-tls may be preferable. Use rustls as default, native-tls as fallback. |
| Serialization | serde + serde_json | simd-json | simd-json is faster but requires exact ownership semantics and is less ergonomic. serde_json is the ecosystem standard. Performance is not the bottleneck for JSON handling in this project. |
| Logging | tracing | log + env_logger | log is the older, simpler facade. tracing provides spans, structured data, and async-aware context. For a pipeline with multiple concurrent operations, tracing's span model is significantly more useful. |
| Error handling | anyhow + thiserror | eyre + thiserror | eyre provides better error reports but adds dependency weight. anyhow is sufficient and more widely used. |
| Image | image | resvg / turbojpeg | image is general-purpose (PNG, JPEG, WebP, etc.). resvg is SVG-only. turbojpeg is JPEG-only. image covers all formats needed for vision analyzer frame processing. |

## Cargo.toml Skeleton

```toml
[package]
name = "narratoai"
version = "0.1.0"
edition = "2024"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# HTTP client (LLM + TTS APIs)
reqwest = { version = "0.12", features = ["json", "multipart", "stream", "rustls-tls"] }

# Video/audio processing
ffmpeg-sidecar = "2"

# WebSocket (edge-tts)
tokio-tungstenite = { version = "0.26", features = ["rustls-tls-native-roots"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
anyhow = "1"
thiserror = "2"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# Image processing
image = "0.25"

# Utilities
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.22"
tempfile = "3"
regex = "1"
which = "7"

# Tauri (command layer)
tauri = { version = "2", optional = true }

[features]
default = []
tauri = ["dep:tauri"]
```

## Installation & Setup

```bash
# Prerequisites
# 1. Rust toolchain (rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. FFmpeg (required, system-installed)
#    Windows: winget install FFmpeg
#    Or: ffmpeg-sidecar auto-downloads on first run

# Build
cargo build --release

# Build with Tauri support (for desktop app)
cargo build --release --features tauri

# Run tests
cargo test

# Run
cargo run --release
```

## Confidence Assessment

| Recommendation | Confidence | Source |
|----------------|------------|--------|
| tokio as async runtime | HIGH | De facto standard, verified via docs.rs, zero credible alternatives |
| reqwest as HTTP client | HIGH | De facto standard, verified via docs.rs, features match all needs |
| ffmpeg-sidecar over ffmpeg-next | HIGH | Verified via docs.rs, matches Python subprocess pattern, avoids FFI complexity |
| tokio-tungstenite for edge-tts | HIGH | Only viable async WebSocket client, verified via docs.rs |
| serde + toml for configuration | HIGH | Ecosystem standard, verified via docs.rs |
| tracing for logging | HIGH | Standard choice, verified via docs.rs, async-aware |
| TTS engine implementation strategies | MEDIUM | Derived from Python code analysis of voice.py + engine files. Edge-TTS WebSocket protocol details need verification against Microsoft's actual protocol. Azure REST API approach needs endpoint verification. |
| Tauri 2.0 command patterns | MEDIUM | Verified via docs.rs. API may have minor changes between 2.0.x versions. |
| Version numbers | MEDIUM | Latest versions fetched from docs.rs metadata where available. Some may be approximate by 1-2 minor versions. Cargo will resolve to exact versions via Cargo.lock. |

## Sources

- docs.rs: tokio, reqwest, ffmpeg-sidecar, tokio-tungstenite, tracing, tracing-subscriber, serde, serde_json, toml, anyhow, thiserror, image, tauri, image, chrono, uuid, base64, tempfile, regex, which
- `.planning/codebase/STACK.md` -- Python dependency inventory
- `.planning/codebase/INTEGRATIONS.md` -- External service integration patterns
- `.planning/codebase/ARCHITECTURE.md` -- Current architecture for mapping
- `.planning/PROJECT.md` -- Project constraints and requirements
- Python source: `app/services/voice.py` -- TTS engine routing and protocol analysis
- Python source: `config.example.toml` -- Configuration structure
