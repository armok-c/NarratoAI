---
phase: 12-additional-tts-engines
reviewed: 2026-05-01T12:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - Cargo.lock
  - Cargo.toml
  - src/config/defaults.rs
  - src/config/types.rs
  - src/tts/azure_speech.rs
  - src/tts/common.rs
  - src/tts/doubaotts.rs
  - src/tts/indextts2.rs
  - src/tts/mod.rs
  - src/tts/qwen_tts.rs
  - src/tts/soulvoice.rs
  - src/tts/tencent_tts.rs
findings:
  critical: 1
  warning: 3
  info: 1
  total: 5
status: issues_found
---

# Phase 12: Code Review Report -- Additional TTS Engines (Rust, Re-review)

**Reviewed:** 2026-05-01T12:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Re-review of 12 Phase 12 files after previous review fixes. The previous review identified 11 findings (1 critical, 7 warnings, 3 info). Most have been correctly addressed in the current codebase:

- **CR-01 (previous, `reqwest::Client::new()` type mismatch):** False positive. `reqwest::Client::new()` returns `Client` directly, not `Result`. The fallback code in `common.rs:58` compiles correctly.
- **WR-01 (previous, blocking fs in async):** Fixed. All engines now use `tokio::fs::remove_file`.
- **WR-02 (previous, rate ignored):** Fixed. Doubao and SoulVoice now use rate; IndexTTS2 and Qwen use `_rate` (documented API limitation).
- **WR-03 (previous, pitch ignored):** Partially fixed. However, the fix for Doubao introduced a **new critical bug** (see CR-01 below) by passing the pitch value as `pitch_ratio` without semantic conversion.
- **WR-04 (previous, response body discarded):** Fixed. All engines now include response body text in error messages.
- **WR-05 (previous, missing voice_name validation):** Fixed. All engines now validate non-empty voice_name.
- **WR-06 (previous, Tencent hardcoded host):** Fixed. Host is now dynamically extracted from `api_url` via `trim_start_matches` + `split`.
- **WR-07 (previous, edge_tts duplicate retry):** Fixed. Now delegates to `common::retry_loop`.
- **IN-01/IN-02/IN-03 (previous):** Noted but not actionable.

**5 remaining issues found**, including 1 critical bug in the Doubao engine introduced by the previous fix for WR-03.

## Critical Issues

### CR-01: Doubao TTS passes `pitch` value directly as `pitch_ratio` without semantic conversion

**File:** `src/tts/doubaotts.rs:55`

**Issue:** The `TtsProvider` trait documents `pitch: f64` as "0 = standard tone" (注释: 音调（0 为标准音调）). The Edge-TTS and Azure engines correctly interpret 0 as standard (meaning "+0Hz offset"). However, the Doubao engine passes this value directly as `pitch_ratio` in the API JSON body without any conversion.

The Doubao/火山引擎 TTS API expects `pitch_ratio` as a multiplicative ratio, where:
- `1.0` = standard pitch (no modification)
- Valid range documented as `[0.5, 2.0]`
- `0.0` = zero pitch (effectively muted or API error)

When a caller (or UI default) passes `pitch=0` -- the standard value per the trait contract -- the Doubao API receives `pitch_ratio=0`, which is **outside the valid range**. This will either cause the API to return an error or produce garbled/muted audio.

By contrast:
- `edge_tts.rs` interprets 0 as "+0Hz" (correct for its API)
- `azure_speech.rs` interprets 0 as "+0Hz" (correct for its API)
- `soulvoice.rs` uses `_pitch` (correctly ignores it -- that API has no pitch control)
- `indextts2.rs` uses `_pitch` (correctly ignores)
- `qwen_tts.rs` uses `_pitch` (correctly ignores)
- `tencent_tts.rs` uses `_pitch` (correctly ignores -- Tencent Cloud TTS has no per-request pitch parameter)

Only Doubao consumes `pitch` but uses it incorrectly. This was introduced when fixing the previous WR-03 ("pitch was ignored") -- the fix added pitch usage but without the required conversion.

**Fix:** Choose one of the following:

**Option A (recommended -- simplest, consistent with all other non-Hz engines):** Ignore pitch like SoulVoice/Qwen/IndexTTS2/Tencent. Change the signature to `_pitch: f64` and hardcode `"pitch_ratio": 1.0` in the request body. Pitch control would then be configurable through a future config field (like `volume` is for Doubao).

```rust
// In TtsProvider impl for DoubaoTtsEngine, line ~120:
async fn synthesize(
    &self,
    text: &str,
    voice_name: &str,
    rate: f64,
    _pitch: f64,       // ignored; Doubao uses pitch_ratio from config
    output_path: &Path,
) -> Result<TtsOutput, TTSError> {
```

```rust
// In synthesize_once, line ~55:
"pitch_ratio": 1.0,   // hardcoded standard; configurable later if needed
```

**Option B:** Convert the pitch value from "0=standard, +50=+50Hz" format to Doubao's ratio format:
```rust
// Convert trait pitch (Hz offset where 0=standard) to Doubao ratio where 1.0=standard
let pitch_ratio = if pitch == 0.0 {
    1.0
} else {
    1.0 + pitch / 50.0
};
// Then clamp to [0.5, 2.0] per Doubao API spec
let pitch_ratio = pitch_ratio.clamp(0.5, 2.0);
```

However, Option B is fragile because the mapping from "Hz offset" to "ratio" is not standardized and may produce unexpected results.

## Warnings

### WR-01: Azure Speech SSML does not filter XML 1.0 disallowed control characters

**File:** `src/tts/azure_speech.rs:267-270`

**Issue:** The `synthesize_once` method escapes XML entities (`&`, `<`, `>`) in text content before embedding in SSML, but does **not** filter XML 1.0 disallowed control characters (U+0000-U+0008, U+000B-U+000C, U+000E-U+001F, U+007F-U+009F). By contrast, `edge_tts.rs:83-91` applies a `.chars().filter(...)` that correctly removes these characters after entity replacement.

If input text contains control characters (from untrusted user input, clipboard content, or data processing artifacts), the Azure Speech REST API will receive malformed SSML and return a 400 Bad Request error. The user would get a confusing "Azure API returned 400" error with no indication that control characters are the cause.

**Fix:** Apply the same character filter used in `edge_tts.rs`:

```rust
// In azure_speech.rs, after the XML entity replacements (line 270), replace:
//     let escaped_text: String = text
//         .replace('&', "&amp;")
//         .replace('<', "&lt;")
//         .replace('>', "&gt;");

// With:
let escaped_text: String = text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .chars()
    .filter(|&c| {
        c == '\t' || c == '\n' || c == '\r'
            || (c as u32 >= 0x20 && !(0x7F..=0x9F).contains(&(c as u32)))
    })
    .collect();
```

### WR-02: Tencent TTS sends empty `X-TC-Token` header instead of omitting it

**File:** `src/tts/tencent_tts.rs:168`

**Issue:** The TC3-HMAC-SHA256 signed request always sets the header `X-TC-Token: ""` (empty string):

```rust
.header("X-TC-Token", "")
```

Per the Tencent Cloud API specification, this header is required only when using STS temporary credentials. For permanent credentials (SecretId + SecretKey, which is the current credential model), the header should be **omitted entirely**, not sent as an empty string.

Some Tencent Cloud API gateway versions may reject requests with an empty `X-TC-Token` header, treating it as an invalid token rather than "no token." This could cause permanent-credential users to receive authentication errors.

**Fix:** Remove the `X-TC-Token` header entirely. If temporary STS credential support is added in the future, add a dedicated config field (e.g., `tencent.session_token`) and conditionally set the header only when that field is non-empty.

```rust
// Remove line 168:
//     .header("X-TC-Token", "")
// No replacement needed for the permanent credential path.
```

### WR-03: `build_client` silently falls back to proxy-less client on builder failure

**File:** `src/tts/common.rs:53-60`

**Issue:** When `reqwest::Client::builder().build()` fails -- which is rare but can happen with TLS backend initialization issues or certain platform configurations -- the code falls back to `reqwest::Client::new()` which has **no proxy configuration**:

```rust
pub fn build_client(proxy_config: &ProxyConfig) -> reqwest::Client {
    let builder = reqwest::Client::builder();
    let builder = proxy_config.apply_to_client(builder);
    builder.build().unwrap_or_else(|e| {
        tracing::warn!("Failed to build HTTP client with proxy config: {}. Falling back to default.", e);
        reqwest::Client::new()
    })
}
```

The proxy applied via `apply_to_client` is silently discarded. The only indication is a `warn!`-level log message that users may never see in production. Users who depend on a proxy for network access (corporate environments, restricted networks) would have their TTS requests sent directly -- leaking both traffic and the fact that they are bypassing corporate policy.

**Note on the previous CR-01:** The previous review incorrectly claimed that `reqwest::Client::new()` returns `Result`. This is false -- `Client::new()` has always returned `Client` directly in reqwest 0.11.x, 0.12.x, and 0.13.x. The fallback code compiles correctly. The actual issue is the silent loss of proxy configuration, not the return type of `new()`.

**Fix:** Change the function to return `Result` so callers can handle the failure:

```rust
pub fn build_client(proxy_config: &ProxyConfig) -> Result<reqwest::Client, TTSError> {
    let builder = reqwest::Client::builder();
    let builder = proxy_config.apply_to_client(builder);
    builder.build().map_err(|e| {
        TTSError::ConnectionFailed(format!(
            "Failed to build HTTP client with proxy config: {}", e
        ))
    })
}
```

If the current infallible signature must be preserved, upgrade the log to `error!` level:

```rust
tracing::error!("HTTP client builder failed with proxy config: {}. Falling back to default client WITHOUT proxy. {}", e, if proxy_config.enabled { "Proxy configuration has been discarded." } else { "" });
```

## Informational

### IN-01: Hardcoded `HeaderValue` strings use `unwrap()` instead of `expect()` in `edge_tts.rs`

**File:** `src/tts/edge_tts.rs:142,146,149,152`

**Issue:** Four calls to `.parse::<HeaderValue>()` on hardcoded string literals (the `Origin` and `User-Agent` headers) use `.unwrap()` instead of `.expect()`:

```rust
request.headers_mut().insert(
    "Origin",
    "chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
        .parse()
        .unwrap(),   // line 146
);
```

While these strings are ASCII-only and will never fail to parse, `unwrap()` provides no context about which assertion failed. If these strings are ever modified to contain non-ASCII characters (the `User-Agent` string is particularly long and could easily accumulate non-ASCII during maintenance), a panic would occur with no indication of the cause or location.

**Fix:** Replace each `.unwrap()` with `.expect("...")` providing a short description:

```rust
"chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
    .parse()
    .expect("hardcoded Origin header value is valid ASCII"),
```

---

_Reviewed: 2026-05-01T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
