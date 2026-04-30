---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T22:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/lib.rs
  - src/tts/edge_tts.rs
  - src/tts/mod.rs
  - tests/tts_test.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**Reviewed:** 2026-04-29T22:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Re-review of the Edge-TTS engine after previous fixes (CR-01, WR-01, WR-02). All three prior issues are confirmed resolved:

- **CR-01 (Message::Text type mismatch):** Line 423 uses `Message::Text(stt_message.into())` -- the `.into()` converts `String` to `Utf8Bytes` as required by `tungstenite 0.29`. Correct.
- **WR-01 (AuthenticationFailed retry short-circuit):** Line 385 checks `matches!(&e, TTSError::AuthenticationFailed(_))` before retrying. The `classify_error` function (lines 356-366) correctly tags errors containing "401", "authentication", or "unauthorized" before they reach the retry loop. Both the proxy and direct-connect paths use this classification.
- **WR-02 (connect() timeout):** Line 127 wraps the connection logic in `tokio::time::timeout(CONNECT_TIMEOUT, ...)` (30 seconds). Line 352 maps timeout errors to `TTSError::ConnectionFailed`.

No regressions were introduced by any of the three fix commits (a43d796, d7b9eb9, 34118ab).

Two new Warnings and two Info items were found. The primary concerns are: (1) word boundary data may be silently lost if Edge TTS returns float-valued JSON offsets -- the code only attempts `as_u64()` which returns `None` for floats; and (2) `voice_name_to_lang()` returns an empty string for non-standard voice names, producing invalid SSML `xml:lang=""`. These are medium-severity robustness issues.

## Warnings

### WR-01: Word boundary offsets silently dropped when Edge TTS returns float-valued JSON

**File:** `src/tts/edge_tts.rs:458-461`

**Issue:** The code uses `as_u64()` to extract `offset` and `duration` from each word boundary JSON payload. Per `serde_json::Value` semantics, `as_u64()` returns `None` for float-typed JSON numbers (e.g., `"offset": 0.0`), not just non-integers. If the Edge TTS service ever returns float-typed values for these fields -- which is a plausible server-side change given these represent fractional 100-ns time units -- every word boundary entry would be silently dropped. The only symptom is a `tracing::warn!` log at line 468-471, which may go unnoticed in production. Downstream code (e.g., subtitle generation in Phase 6) would receive empty or incomplete boundary data.

Additionally, the `turn.end` duration parsing at line 488-500 already demonstrates the correct defensive pattern: it tries `as_f64()`, `as_str()`, and nested `ticks.as_u64()` as fallbacks. The word boundary parsing should follow the same principle.

**Fix:** Use `as_f64()` as the primary extraction method and convert to `u64`. This handles both integer and float JSON representations:

```rust
if let (Some(offset_val), Some(duration_val), Some(text)) = (
    wb_json["offset"].as_f64(),
    wb_json["duration"].as_f64(),
    wb_json["text"].as_str(),
) {
    word_boundaries.push(WordBoundary {
        start_offset: offset_val as u64,
        end_offset: (offset_val + duration_val) as u64,
        text: text.to_string(),
    });
}
```

Or, for backward compatibility, try `as_u64()` first with fallback to `as_f64()`:

```rust
let offset = wb_json["offset"]
    .as_u64()
    .or_else(|| wb_json["offset"].as_f64().map(|f| f as u64));
let duration = wb_json["duration"]
    .as_u64()
    .or_else(|| wb_json["duration"].as_f64().map(|f| f as u64));
if let (Some(offset), Some(duration), Some(text)) =
    (offset, duration, wb_json["text"].as_str())
{
    word_boundaries.push(WordBoundary {
        start_offset: offset,
        end_offset: offset.saturating_add(duration),
        text: text.to_string(),
    });
}
```

### WR-02: `voice_name_to_lang` returns empty string for non-standard voice names, producing invalid SSML

**File:** `src/tts/edge_tts.rs:62-68` (function), line 91 (usage in SSML)

**Issue:** `voice_name_to_lang` finds the second hyphen in the voice name and returns the preceding substring. If the voice name does not contain a second hyphen, it returns an empty `String`. This empty string propagates to `build_ssml` at line 91, producing `<speak ... xml:lang="">` -- an empty `xml:lang` attribute, which is invalid XML per the W3C specification. The Edge TTS server may reject such requests or silently substitute a default language, leading to unpredictable behavior (e.g., wrong phoneme mapping, incorrect prosody).

The `synthesize` method (lines 614-617) only validates that `voice_name` is non-empty, not that it follows the required `lang-Region-VoiceName` format with at least two hyphens. A voice name like `"XiaoyiNeural"` would pass validation but produce invalid SSML.

**Fix:** In `voice_name_to_lang`, return a safe fallback when the second hyphen is missing, and log a warning:

```rust
fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some((idx, _)) = voice_name
        .char_indices()
        .filter(|(_, c)| *c == '-')
        .nth(1)
    {
        voice_name[..idx].to_string()
    } else {
        tracing::warn!(
            "voice name '{}' does not follow lang-Region-Name format, falling back to en-US",
            voice_name
        );
        "en-US".to_string()
    }
}
```

## Info

### IN-01: Voice name not escaped in SSML attribute value (defense-in-depth)

**File:** `src/tts/edge_tts.rs:90-92`

**Issue:** The `voice_name` parameter is interpolated directly into an XML attribute value without escaping: `format!(r#"..."<voice name="{}">..."#, ..., voice_name, ...)`. If a voice name contained a double-quote character, the XML would be malformed (e.g., `<voice name="foo"bar">`). The `text` parameter, which appears in element content, is properly escaped for `&`, `<`, and `>` at lines 76-78, but the `voice_name` -- which appears in an attribute context where `"` is also significant -- is not. In practice, Edge TTS voice names are constrained to alphabetic characters and hyphens, so the practical risk is near-zero. This is recorded as a defense-in-depth gap.

**Suggestion:** Either escape double quotes in `voice_name` before injection, or add a validation assertion that `voice_name` matches `^[a-zA-Z0-9_-]+$` at the `synthesize` entry point alongside the existing non-empty check.

### IN-02: SOCKS proxy rejection only checks SOCKS5, not SOCKS4/SOCKS4a

**File:** `src/tts/edge_tts.rs:167-171`

**Issue:** The proxy scheme check at line 168 explicitly rejects `"socks5"` but does not reject `"socks4"` or `"socks4a"`. If a user configures a SOCKS4 proxy URL, the code falls through to the HTTP CONNECT tunnel logic, which would fail with an unclear error: "连接代理失败" (connection to proxy failed) rather than the descriptive SOCKS5 error. SOCKS4 is largely obsolete, so this is unlikely in practice, but the rejection should be more general.

**Suggestion:** Check for any scheme starting with `"socks"`:

```rust
let proxy_scheme_lower = proxy_scheme.to_lowercase();
if proxy_scheme_lower.starts_with("socks") {
    return Err(TTSError::ConnectionFailed(
        "本引擎不支持 SOCKS 代理，请使用 HTTP/HTTPS 代理".to_string()
    ));
}
```

## Prior Issues Verification

| ID | Description | Status | Evidence |
|---|---|---|---|
| CR-01 | `Message::Text` takes `Utf8Bytes`, not `String` | FIXED | Line 423: `Message::Text(stt_message.into())` -- `.into()` converts `String` to `Utf8Bytes`. Also line 533: pattern match works because received `Message::Text` binds to a variable which is then printed via `Display`. |
| WR-01 | `AuthenticationFailed` errors retried unnecessarily | FIXED | Lines 385-387: `if matches!(&e, TTSError::AuthenticationFailed(_)) { return Err(e); }` short-circuits before retry. Lines 338-339 (proxy path) and 346 (direct path) both use `Self::classify_error(...)`. |
| WR-02 | `connect()` can block indefinitely during CONNECT/TLS handshake | FIXED | Line 127: `tokio::time::timeout(CONNECT_TIMEOUT, async move { ... })` wraps the entire connection logic. `CONNECT_TIMEOUT` = 30s (line 12). Line 352: timeout error mapped to `ConnectionFailed`. |

No regressions were introduced. The three fixes are minimal, targeted changes that touch only the lines described in the previous review.

---

_Reviewed: 2026-04-29T22:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
