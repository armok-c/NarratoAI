---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-28T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/tts/mod.rs
  - src/error.rs
  - src/lib.rs
  - Cargo.toml
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
findings:
  critical: 1
  warning: 3
  info: 4
  total: 8
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Report

**Reviewed:** 2026-04-28T00:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Six files implementing the TTS core abstraction (Phase 03-01: `TtsProvider` trait, router, data types), Edge-TTS engine via WebSocket (Phase 03-02: tokio-tungstenite, SSML, retry loop, proxy tunnel), and integration tests (Phase 03-03: 8 tests) were reviewed at standard depth.

The overall architecture is sound: clean `TtsProvider` trait with `Send + Sync`, proper thiserror error types, correct async patterns throughout, and a working retry loop. The proxy tunnel implementation via HTTP CONNECT is present and functionally correct in the non-proxy cases.

One critical bug was found in the `turn.end` duration parsing where the numeric branch (likely dead code today) omits the required tick-to-seconds division. Three warnings were identified: the CONNECT tunnel target incorrectly includes the URL path and query string (which may break strict proxies), word boundary parse failures silently drop data, and one test performs no meaningful verification.

## Critical Issues

### CR-01: Duration parsing in `turn.end` omits tick-to-seconds division for numeric `audio_duration`

**File:** `src/tts/edge_tts.rs:294-296`
**Issue:** In `synthesize_once()`, when parsing the `turn.end` binary message's JSON payload, three branches attempt to extract `audio_duration`:

1. `as_f64()` (lines 294-295): If `audio_duration` is a JSON number, the value is used directly as seconds.
2. `as_str()` (lines 296-298): If `audio_duration` is a JSON string, the value is parsed as `f64` and divided by 10,000,000.
3. `["ticks"].as_u64()` (lines 299-300): If `audio_duration` is an object with a `ticks` field, it is divided by 10,000,000.

Branches 2 and 3 correctly convert from 100-ns tick units to seconds (division by 10,000,000). Branch 1 omits this division. Edge TTS's `audio_duration` is consistently documented as being in 100-ns tick units (matching the same unit used by `WordBoundary.start_offset`/`end_offset`). If the server returns a numeric JSON value instead of a string, the duration would be wrong by a factor of 10 million (e.g., 12,345,678 seconds instead of ~1.23 seconds).

While Edge TTS currently appears to return `audio_duration` as a string (so branch 2 fires), the numeric branch is a latent bug that would produce catastrophically wrong results if ever triggered by a server schema change.

**Fix:** Add the `/ 10_000_000.0` division to the `as_f64()` branch:
```rust
duration = if let Some(d) = metadata["audio_duration"].as_f64() {
    d / 10_000_000.0   // Add division here
} else if let Some(s) = metadata["audio_duration"].as_str() {
    s.parse::<f64>().unwrap_or(0.0) / 10_000_000.0
} else if let Some(ticks) = metadata["audio_duration"]["ticks"].as_u64() {
    ticks as f64 / 10_000_000.0
} else {
    0.0
};
```

## Warnings

### WR-01: CONNECT tunnel target includes URL path and query string, violating RFC 7231

**File:** `src/tts/edge_tts.rs:123-129`
**Issue:** When building the HTTP CONNECT request for the proxy tunneling path, the code parses the target URL by trimming the `wss://` prefix and checking for a colon. The constant `EDGE_TTS_WSS_URL` has no explicit port (no colon after the host), so `split_once(':')` returns `None`. The `unwrap_or` fallback then uses the **entire remaining URL** (including path `/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=...`) as `target_host`.

This produces an invalid CONNECT request line:
```
CONNECT speech.platform.bing.com/consumer/speech/.../v1?TrustedClientToken=...:443 HTTP/1.1
```

Per RFC 7231 Section 4.3.6, the CONNECT method MUST contain only `host:port`. Many proxy servers are lenient and extract the hostname anyway, but strict proxies (e.g., corporate proxies with URL filtering) may reject this. The `Host` header in the CONNECT request also suffers from the same issue.

The `and_then` branch (when the URL has an explicit port) correctly strips the path with `p.split('/').next()`, but the `unwrap_or` fallback does not.

**Fix:** Strip the path in both code paths:
```rust
let target_host_only = target_addr.split('/').next().unwrap_or(target_addr);
let (target_host, target_port_str) = target_addr
    .split_once(':')
    .and_then(|(h, p)| Some((h, p.split('/').next().unwrap_or("443"))))
    .unwrap_or((target_host_only, "443"));
```

### WR-02: Word boundary parse failures silently drop data without diagnostic log

**File:** `src/tts/edge_tts.rs:272-286`
**Issue:** When parsing a `wordboundary` binary message's JSON payload, if any of `offset`, `duration`, or `text` fields are missing (or have unexpected types), the entire word boundary is silently discarded. No `tracing::warn!` or `tracing::error!` is emitted.

If the Edge TTS server changes its response format even slightly, all word boundaries would silently disappear. Downstream consumers (Phase 6 subtitle generation) would receive an empty `word_boundaries` vector with no indication that data was lost. Debugging such an issue without the diagnostic log would require deep protocol inspection.

**Fix:** Add a `tracing::warn!` call when word boundary parsing fails:
```rust
if let (Some(offset), Some(duration_100ns), Some(text)) = (
    wb_json["offset"].as_u64(),
    wb_json["duration"].as_u64(),
    wb_json["text"].as_str(),
) {
    word_boundaries.push(WordBoundary {
        start_offset: offset,
        end_offset: offset + duration_100ns,
        text: text.to_string(),
    });
} else {
    tracing::warn!(
        "Failed to parse word boundary fields from JSON: {:?}",
        wb_json
    );
}
```

### WR-03: `test_synthesize_function_signature` performs no meaningful runtime verification

**File:** `tests/tts_test.rs:141-150`
**Issue:** The test defines an inner function `_check_type` that references `tts::synthesize`, but this inner function is **never called**. The outer function body is:
```rust
fn _check_type() {
    let _ = tts::synthesize;  // compile-time check only
}
let _ = _check_type;  // assigns function pointer to _, discarded
```

There are zero assertions and zero side effects. The test provides no runtime coverage and can never fail. The compile-time check it attempts (verifying `tts::synthesize` exists) is already guaranteed by the `use narratoai_core::tts::{self, ...};` import at line 2 of the same file.

**Fix:** Either remove the test entirely, or replace it with a meaningful assertion:
```rust
// Removal:
// Delete lines 141-150.

// OR replacement:
#[test]
fn test_synthesize_returns_result_type() {
    let _: fn(&str, &str, &str, f64, f64, &Path, Option<&ProxySection>) -> _ = tts::synthesize;
}
```

## Info

### IN-01: Hardcoded `TrustedClientToken` in WebSocket URL

**File:** `src/tts/edge_tts.rs:11`
**Issue:** The Microsoft Edge TTS authentication token (`6A5AA1D4EAFF4E9FB37E23D68491D6F4`) is hardcoded in the source. This is the same publicly-known token used by the Python edge-tts library and many other open-source implementations, so it is not a secret in the traditional sense. However, there is no mechanism to override or update it via configuration. If Microsoft rotates or revokes this token, the engine stops working and requires a source code change. Consider making this configurable or documenting the dependency.

### IN-02: `unwrap()` used on hardcoded header values

**File:** `src/tts/edge_tts.rs:92,97`
**Issue:** `HeaderValue::parse()` is called on hardcoded string literals with `.unwrap()`. While these strings are valid HTTP header values that will never trigger a parse error, using `.expect("...")` would be more self-documenting and consistent with the explicit error handling patterns used elsewhere in the file (where every fallible operation is mapped to a `TTSError` variant).

### IN-03: CONNECT response checked with loose `contains("200")`

**File:** `src/tts/edge_tts.rs:157`
**Issue:** The proxy CONNECT response is validated via `response_line.contains("200")`. This substring check would match `HTTP/1.1 1200 Something` or any line that happens to contain the substring "200". The intended check is `HTTP/1.1 2xx` on the status line. While extremely unlikely to false-positive in practice, a more precise check would be:
```rust
if !response_line.starts_with("HTTP/") || !response_line.contains("200 Connection") {
```

### IN-04: `voice_name` not XML-escaped for SSML attribute context

**File:** `src/tts/edge_tts.rs:51-53`
**Issue:** In `build_ssml()`, `voice_name` is interpolated directly into an XML attribute value (`<voice name="{}">`) without escaping `"` to `&quot;`. While voice names in Edge TTS are controlled values (e.g., `zh-CN-XiaoyiNeural`), the `TtsProvider::synthesize()` trait accepts any `&str`. A caller passing a voice name containing `"` would produce malformed SSML. For defense-in-depth, consider validating or escaping attribute values.

---

_Reviewed: 2026-04-28T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
