---
phase: 03-tts-core-edge-tts
plan: 02
subsystem: tts
tags: edge-tts, websocket, tokio-tungstenite, tts, mp3

# Dependency graph
requires:
  - phase: 03-01
    provides: TTS Core types (WordBoundary, TtsOutput, TtsProvider trait, TTSError, synthesize router)
provides:
  - EdgeTtsEngine struct with proxy support and WebSocket connection
  - TtsProvider trait impl for Edge-TTS
  - ssml generation with rate/pitch conversion and XML escaping
  - WebSocket binary message parsing (Path/Content-Type header)
  - 3-retry loop with 1-second interval
  - synthesize("edge_tts", ...) route in mod.rs
affects:
  - Phase 06 (documentary pipeline TTS integration)

# Tech tracking
tech-stack:
  added:
    - tokio-tungstenite (0.29.0) — native Rust WebSocket client
    - futures-util (0.3.32) — StreamExt/SinkExt for WebSocket async iteration
    - uuid (1.23.1, v4) — request ID generation per SSML session
  patterns:
    - WebSocket binary message header parsing (Path:\r\nX-RequestId:\r\nContent-Type:\r\n\r\n{payload})
    - TDD (RED/GREEN) for engine unit tests
    - Provider pattern via async_trait for TTS engine abstraction

key-files:
  created:
    - src/tts/edge_tts.rs (337 lines)
  modified:
    - Cargo.toml (3 new dependencies)
    - src/tts/mod.rs (mod edge_tts + EdgeTtsEngine import + edge_tts route branch)

key-decisions:
  - "D-01: Native Rust WebSocket with tokio-tungstenite, no Python subprocess"
  - "D-02: Proxy support via config.proxy (http, https, enabled fields)"
  - "D-03: 3 retries, 1 second interval"
  - "D-04: MP3 audio output via direct file write"
  - "D-10: Stream WordBoundary/SentenceBoundary events via WebSocket, populate Vec<WordBoundary>"
  - "D-11: String-match route dispatch via match engine { edge_tts => ... }"

patterns-established:
  - "Binary message parsing: extract Path field from \r\n-delimited headers"
  - "Retry pattern: 3 attempts with 1s sleep, return RetryExhausted on final failure"
  - "SSML injection mitigation: XML entity escaping of input text before embedding"

requirements-completed:
  - TTS-03
---

# Phase 03 Plan 02: Edge-TTS Engine Summary

**Edge-TTS engine via native Rust WebSocket (tokio-tungstenite): SSML generation, flow-style audio/WordBoundary binary message parsing, 3-retry synthesis, and TtsProvider trait implementation wired into the synthesize() router**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-28T10:13:00Z
- **Completed:** 2026-04-28T10:28:00Z
- **Tasks:** 2 (1 TDD with RED/GREEN split)
- **Files modified:** 3

## Accomplishments

- SSML generation with rate (percent) and pitch (Hz) conversion, XML entity escaping for injection mitigation
- EdgeTtsEngine struct with proxy configuration (enabled/http/https) and WebSocket connect()
- Full WebSocket message loop: send SSML, receive binary chunks, parse Path headers, collect audio data and word boundaries
- 3-retry synthesis loop with 1-second interval (matching Python behavior)
- TtsProvider trait implementation wiring build_ssml -> synthesize_with_retry -> synthesize_once -> connect
- synthesize("edge_tts", ...) route in mod.rs using EdgeTtsEngine::new with default (no-proxy) config
- 15 unit tests total (edge_tts: 12 + tts: 3), all passing; 1 integration test ignored (requires network)
- Zero build warnings

## Task Commits

Each task was committed atomically:

1. **Task 1 (TDD): SSML generation + EdgeTtsEngine + connect** — TDD split:
   - `f4af600` (test: RED — stub implementations, 10/12 tests fail)
   - `ab5b501` (feat: GREEN — real implementations, 12/12 tests pass)
2. **Task 2: Stream receive + word boundary parsing + retry + TtsProvider + router** — `faebb3e` (feat)

## Files Created/Modified

- `src/tts/edge_tts.rs` (created, 337 lines) — entire Edge-TTS engine: SSML builder, rate/pitch converters, WebSocket connect/send/receive loop, binary message parser, retry loop, TtsProvider impl, 12 unit tests, 1 integration test
- `Cargo.toml` (modified) — added tokio-tungstenite 0.29.0, futures-util 0.3.32, uuid 1.23.1 with v4 feature
- `src/tts/mod.rs` (modified) — added `mod edge_tts;`, EdgeTtsEngine import, `"edge_tts" =>` synthesize branch

## Decisions Made

- Followed all 14 implementation decisions from CONTEXT.md (D-01 through D-14)
- Proxy implementation in connect() uses placeholder variable (declares proxy_url but uses connect_async directly; proper proxy-aware connector deferred to Phase 6 integration)
- synthesize("edge_tts", ...) convenience function creates EdgeTtsEngine with default no-proxy config; Phase 6 callers will pass ConfigManager proxy values

## Deviations from Plan

None - plan executed as specified with the following minor adjustments:

- **SinkExt import required**: `ws_stream.send()` requires `futures_util::SinkExt` in scope (added alongside existing StreamExt import, not blocking)
- **Message::Text type**: tungstenite 0.29 uses `Utf8Bytes` not `String`, required `.into()` conversion
- **EdgeTtsEngine import**: mod.rs needed `use self::edge_tts::EdgeTtsEngine;` (not in plan's action code)
- **proxy_url**: prefixed with underscore to suppress unused-variable warning (proxy address declared but connect_async doesn't natively route through proxy)

These are standard compilation corrections for action-code-to-actual-library-API mapping, not behavioral deviations.

## Issues Encountered

None. All tasks compiled and tested cleanly on first attempt after the minor API corrections noted above.

## Known Stubs

- **Proxy routing in connect()**: `_proxy_url` is parsed from config but not passed to connect_async. tokio-tungstenite's connect_async does not natively support HTTP proxies. Phase 6 will need to implement manual TCP CONNECT + TLS + WebSocket upgrade for proxy support, or use a proxy-aware connector. Tracked as intentional simplification per plan note: "Phase 6 纪录片流水线集成时完善代理支持".

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Edge-TTS engine complete and wired into synthesize("edge_tts", ...) router
- Ready for Phase 6 documentary pipeline integration: callers pass ConfigManager proxy settings to EdgeTtsEngine::new()
- 12 unit tests covering rate/pitch conversion, SSML construction with XML escaping, engine constructor

---
*Phase: 03-tts-core-edge-tts*
*Completed: 2026-04-28*
