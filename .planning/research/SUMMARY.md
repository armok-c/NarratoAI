# Research Summary: NarratoAI Rust Rewrite

**Domain:** AI video narration + automated editing (desktop application)
**Researched:** 2026-04-27
**Overall confidence:** HIGH

## Executive Summary

NarratoAI is a mature Python (v0.7.8) application for AI-powered video narration with three business modes: Documentary (vision-based frame analysis + narration), SDE (subtitle analysis + script editing), and SDP (subtitle analysis + multi-clip mixing). The Rust rewrite targets a library crate architecture with a Tauri 2.0 command layer, producing a native desktop application. The rewrite replaces Python's Streamlit UI, moviepy/pydub dependencies, and dual LLM interface with a clean, performant Rust stack.

The core pipeline follows a 6-step flow: load script, TTS generation, video clipping, audio/subtitle merge, clip concatenation, and final composition. Each step branches on OST type (narration-only, original-audio-only, or mixed), which drives fundamentally different processing strategies. This OST branching is the architectural spine of the system and the single most important concept to get right in the rewrite.

The technology stack is well-established in the Rust ecosystem: tokio for async runtime, reqwest for HTTP (LLM + TTS APIs), ffmpeg-sidecar for video processing (CLI wrapper matching Python's subprocess pattern), tokio-tungstenite for Edge-TTS WebSocket, serde + toml for configuration, and tracing for diagnostics. No exotic or immature dependencies are required.

The primary technical risk is Edge-TTS implementation. It is the default TTS engine (free, used by 80% of users) but requires implementing Microsoft's proprietary WebSocket protocol. The smartEdit project (a previous failed Rust rewrite attempt in the same domain) has a working Edge-TTS implementation that can be referenced directly, significantly reducing this risk.

## Key Findings

**Stack:** tokio + reqwest + ffmpeg-sidecar + serde/toml + tracing + tokio-tungstenite + tauri 2.0. All libraries are mature, well-documented, and verified via docs.rs. No build-complexity dependencies (no C FFI, no native SDKs).

**Architecture:** Layered library crate with trait-based abstractions for TTS engines and LLM providers. Pipeline steps as separate structs with typed input/output. Tauri command layer is a thin facade, no business logic. The OST type enum is the central branching mechanism.

**Critical pitfall:** Edge-TTS WebSocket protocol requires precise implementation of authentication tokens, clock skew correction, and Chromium version strings. Without it, the default (free) TTS engine is non-functional and the MVP is blocked.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Foundation (Config + FFmpeg + Error Handling)** - Everything depends on configuration loading and FFmpeg operations. These must be first.
   - Addresses: TOML config system, FFmpeg pipeline core operations, error type architecture
   - Avoids: Pitfall 1 (FFmpeg binding mistake), Pitfall 6 (error serialization), Pitfall 10 (config field misalignment)

2. **LLM Service + Prompt System** - Script generation depends on LLM API calls. Prompt versioning enables mode-specific templates.
   - Addresses: OpenAI-compatible LLM client, vision + text model support, prompt registry
   - Avoids: Pitfall 5 (LLM response parsing fragility), Pitfall 7 (no streaming extensibility)

3. **TTS Layer (Edge-TTS First)** - Voice generation is required before any pipeline can produce output. Edge-TTS is the highest priority engine.
   - Addresses: Edge-TTS WebSocket implementation, TtsProvider trait, engine routing
   - Avoids: Pitfall 4 (Edge-TTS protocol misalignment), Pitfall 5 (TTS trait design)

4. **Documentary Pipeline (Flagship)** - The complete documentary mode validates the entire architecture end-to-end. It exercises every subsystem: vision analysis, LLM, TTS, FFmpeg, and state management.
   - Addresses: Frame extraction, batch vision analysis, narration script generation, 6-step pipeline with OST branching
   - Avoids: Pitfall 3 (OST branch logic incomplete), Pitfall 2 (blocking async runtime)

5. **Script Management + SDE/SDP Modes** - With the pipeline infrastructure proven, the remaining two business modes are incremental extensions.
   - Addresses: JSON script editing, subtitle parsing, SDE pipeline, SDP pipeline
   - Avoids: Pitfall 7 (Chinese encoding in subtitles)

6. **Tauri Command Layer + Integration** - Expose all Rust functionality through Tauri commands for the Vue 3 frontend (next milestone).
   - Addresses: Tauri command definitions, State management, progress streaming via Channel API
   - Avoids: Pitfall 6 (error context loss at FFI boundary)

7. **Extended Features** - Additional TTS engines, JianYing export, YouTube download, Pexels integration, audio normalization.
   - Addresses: Feature parity with Python version
   - Standard patterns, low risk

**Phase ordering rationale:**
- Config and FFmpeg are dependencies for everything else -- must be Phase 1.
- LLM service is needed by all three business modes -- Phase 2.
- TTS is needed before any pipeline can produce output -- Phase 3.
- Documentary mode validates the architecture end-to-end -- Phase 4.
- SDE and SDP build on proven infrastructure -- Phase 5.
- Tauri commands are a thin layer over working library code -- Phase 6.
- Extended features are low-risk incremental work -- Phase 7.

**Research flags for phases:**
- Phase 3 (Edge-TTS): HIGH research need -- WebSocket protocol details, clock skew correction, Chromium version management. Reference smartEdit's existing implementation.
- Phase 6 (Tauri): MEDIUM research need -- Channel API for progress streaming, State management patterns. Tauri 2.0 API may have minor changes.
- Phase 7 (JianYing): MEDIUM research need -- Draft format reverse engineering from pyJianYingDraft Python library.
- All other phases: Standard patterns, unlikely to need significant additional research.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All libraries verified via docs.rs. Standard Rust ecosystem choices. No exotic dependencies. |
| Features | HIGH | Feature landscape mapped directly from Python codebase. Every feature has a clear Rust implementation path. |
| Architecture | HIGH | Architecture derived from Python's proven patterns with Rust-specific improvements (trait-based providers, typed OST, state machine). |
| Pitfalls | HIGH | Pitfalls identified from: (1) Python codebase's documented concerns, (2) smartEdit's failed Rust rewrite experience, (3) Rust-specific async/FFI risks. |

## Gaps to Address

- **Edge-TTS protocol version stability:** Microsoft may change the Edge-TTS WebSocket protocol at any time. The Chromium version constant needs a maintenance strategy. smartEdit hardcodes it -- consider a mechanism to detect and update it.
- **Azure Speech REST API endpoint verification:** The plan is to use Azure's REST API instead of the native SDK. The exact endpoint URL and authentication flow need verification against Azure's current documentation.
- **JianYing draft format stability:** The JSON format is not publicly documented. Reverse engineering from pyJianYingDraft is viable but the format may change with JianYing updates.
- **SQLite vs file storage decision:** PROJECT.md specifies file storage (JSON/TOML), but the existing pitfall research from smartEdit suggests SQLite is more appropriate for desktop apps. This is a product decision that should be revisited before Phase 1.
- **FFmpeg hardware acceleration:** Python has `ffmpeg_config.py` for detecting GPU encoding support. The Rust equivalent needs platform-specific detection (NVENC on Windows, VA-API on Linux, VideoToolbox on macOS). This can be deferred but should not be forgotten.

## Research Files

| File | Purpose |
|------|---------|
| `.planning/research/STACK.md` | Complete technology recommendations with rationale and confidence |
| `.planning/research/FEATURES.md` | Feature landscape: table stakes, differentiators, anti-features, priority matrix |
| `.planning/research/ARCHITECTURE.md` | Recommended architecture, patterns, anti-patterns, module organization |
| `.planning/research/PITFALLS.md` | 15 domain pitfalls with prevention strategies and phase-specific warnings |

## Sources

- docs.rs: tokio, reqwest, ffmpeg-sidecar, tokio-tungstenite, tracing, serde, toml, tauri, image, anyhow, thiserror
- `.planning/codebase/` -- Full Python codebase analysis (ARCHITECTURE, CONCERNS, STACK, STRUCTURE, INTEGRATIONS)
- `.planning/PROJECT.md` -- Project scope and constraints
- Python source code analysis -- `voice.py`, `task.py`, `config.example.toml`, LLM module, prompt system
- smartEdit project (`E:\GitLib\smartEdit\`) -- Previous Rust rewrite attempt with validated implementations
