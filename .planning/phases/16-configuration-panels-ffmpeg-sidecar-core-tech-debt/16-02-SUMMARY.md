---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
plan: 02
subsystem: backend-tts-visual
tags: [edge-tts, proxy, visual-analyzer, tauri-config]
requires:
  - phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
    plan: 01
    provides: ConfigManager-backed Tauri config commands
provides:
  - ProxyConfig environment proxy detection and config-over-env precedence
  - Edge-TTS proxy-aware HTTP client for clock skew correction and voice list fetch
  - Core get_edge_tts_voices API reused by the Tauri config command
  - Documentary visual analysis routed through the analyze_video_frames facade
  - VisualError to PipelineError conversion for facade call integration
  - TDET-06 call-path review note for current Rust pipeline behavior
affects: [phase-16, edge-tts, proxy, visual-analyzer, documentary-pipeline]
tech-stack:
  added: []
  patterns:
    - Manual proxy config wins when http/https fields are non-empty; otherwise env proxy variables are detected.
    - Tauri config command delegates Edge-TTS voice loading to narratoai-core instead of rebuilding the request locally.
    - Documentary script analysis renders prompts before entering the VisualAnalyzer facade.
key-files:
  created:
    - .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-02-SUMMARY.md
  modified:
    - narratoai-core/src/tts/common.rs
    - narratoai-core/src/tts/edge_tts.rs
    - narratoai-core/src/tts/mod.rs
    - src-tauri/src/commands/config.rs
    - narratoai-core/src/documentary/error.rs
    - narratoai-core/src/documentary/script_gen.rs
    - narratoai-core/src/visual/analyzer.rs
key-decisions:
  - "ProxyConfig::from_proxy() now falls back to ProxyConfig::from_env() only when no manual proxy endpoint is configured."
  - "Edge-TTS voice list fetching lives in narratoai-core so synthesis and config-panel voice loading share proxy behavior."
  - "The current v2.0 Rust visual-analysis path is Documentary only; SDE/SDP do not perform visual analysis."
patterns-established:
  - "Environment proxy tests isolate and restore proxy variables under a shared mutex."
  - "VisualAnalyzer facade outputs remain serialized as a Vec<BatchAnalysisResult> to preserve analysis.json shape."
requirements-completed: [TDET-01, TDET-06]
duration: resumed session
completed: 2026-05-14
---

# Phase 16 Plan 02: Edge-TTS Proxy + VisualAnalyzer Facade Summary

**Edge-TTS proxy support now covers env detection, clock skew correction, voice list loading, and current Rust visual-analysis callers route through the VisualAnalyzer facade.**

## Accomplishments

- Added `ProxyConfig::from_env()` for `HTTPS_PROXY` / `HTTP_PROXY` / `ALL_PROXY` detection and made `from_proxy()` fall back to env proxy values when no manual endpoint is configured.
- Added a proxy-aware Edge-TTS HTTP client path and used it for clock skew correction and the new `get_voices()` voice list request.
- Added `narratoai_core::tts::get_edge_tts_voices()` and updated the Tauri `get_edge_tts_voices` command to reuse core Edge-TTS request construction.
- Updated `get_system_proxy` to report the effective proxy source (`config`, `env`, or `none`) plus config/env details for frontend display.
- Routed Documentary visual analysis through `crate::visual::analyzer::analyze_video_frames()` while preserving the existing `analysis.json` array shape.
- Added `VisualError` to `PipelineError` conversion so facade failures map cleanly into the documentary pipeline.
- Documented the TDET-06 call-path review in `visual/analyzer.rs`: current Rust visual analysis enters through the facade; SDE/SDP have no visual-analysis step; old Python paths are outside the v2.0 Tauri/Rust runtime path.

## Verification

- `cargo check -p narratoai-core` passed.
- `cd src-tauri; cargo check` passed.
- `cargo test -p narratoai-core -- tts::common::tests` passed: 8 passed.
- `cargo test -p narratoai-core -- tts::edge_tts::tests` passed: 32 passed, 1 ignored.
- `cargo test -p narratoai-core -- visual::analyzer::tests` passed: 10 passed.

## Known Non-Blocking Warnings

- `ts-rs` still warns that one serde `deserialize_with` attribute cannot be parsed.
- `src-tauri/src/commands/pipeline.rs` still has two pre-existing unnecessary-parentheses warnings.
- Some test targets still report pre-existing unused test helpers/imports.

## Deviations From Plan

- The plan referenced old Python `app/services/*` pipeline files for review. The effective v2.0 path is Rust/Tauri, so the implementation updated and documented the current Rust documentary path instead.
- TDET-06 required confirming all visual-analysis callers use the facade. Current Rust SDE/SDP pipelines have no visual-analysis step, so there was no SDE/SDP runtime caller to update.

## Next Plan Readiness

Wave 0 is complete after 16-02. Resume with `16-03-PLAN.md` for Pinia stores, `useConfig`, and ts-rs type synchronization.

---
*Phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt*
*Completed: 2026-05-14*
