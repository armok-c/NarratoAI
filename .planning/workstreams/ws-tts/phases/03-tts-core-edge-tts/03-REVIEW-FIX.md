---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-28T12:35:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 2
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-04-28T12:35:00Z
**Source review:** `.planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md`
**Iteration:** 2

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-04: Proxy port defaults to 80 for HTTPS proxies without explicit port

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 95adfd0
**Applied fix:** Before stripping the URL scheme, check if the proxy URL starts with `https://`. If so, the default port is `"443"`; otherwise `"80"`. The `unwrap_or` fallback now uses this scheme-aware default instead of the hardcoded `"80"`. This ensures that a proxy configuration like `proxy_https = "https://proxy.example.com"` correctly connects to port 443 instead of silently using port 80.

### WR-05: Duration silently zero when WebSocket closes before turn.end

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 84ecb72
**Applied fix:** After the message loop and the `audio_data.is_empty()` guard, added a warning log when `duration == 0.0` but audio data was received. This informs downstream consumers that the connection dropped mid-stream without duration metadata, providing visibility into what would otherwise be a silent incorrect duration value of 0.0.

---

_Fixed: 2026-04-28T12:35:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
