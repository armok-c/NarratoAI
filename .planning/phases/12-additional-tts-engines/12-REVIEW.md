---
phase: 12-additional-tts-engines
reviewed: 2026-04-30T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - Cargo.toml
  - Cargo.lock
  - src/config/types.rs
  - src/config/defaults.rs
  - src/tts/common.rs
  - src/tts/soulvoice.rs
  - src/tts/doubaotts.rs
  - src/tts/qwen_tts.rs
  - src/tts/indextts2.rs
  - src/tts/azure_speech.rs
  - src/tts/tencent_tts.rs
  - src/tts/mod.rs
findings:
  critical: 0
  warning: 9
  info: 3
  total: 12
status: issues_found
---

# Phase 12: Code Review Report — Additional TTS Engines (Rust)

**Reviewed:** 2026-04-30T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Reviewed 12 files implementing 6 new TTS engine backends in Rust (SoulVoice, Doubao, Qwen, IndexTTS2, Azure Speech, Tencent Cloud) plus shared infrastructure (config types, defaults, common utilities). The overall code organization and error handling approach is solid, but several significant issues were found:

1. **Logic errors** — Tencent TTS speed computation uses a constant `0.0_f64` instead of the input `rate` parameter; Azure Speech SSML omits prosody tags making rate/pitch have no effect.
2. **Validation gaps** — Whitespace-only text bypasses empty-text validation in 4 of 6 engines.
3. **Dead configuration** — Three configuration fields (`voice_uri`, `ak`/`sk`, `doubaotts_voice_type`/`doubaotts_rate`) are defined in config structs but never read by engine code.
4. **Silent fallback risks** — Proxy configuration errors are silently swallowed.

No critical (security) issues were found — API keys are properly segregated in configuration, no hardcoded secrets in source, no command injection vectors, and HTTP client libraries handle TLS correctly.

## Warnings

### WR-01: Tencent TTS `speed_value` always computed as 0.0 (rate parameter ignored)

**File:** `src/tts/tencent_tts.rs:51`

**Issue:**
Line 51 computes `speed_value` as `0.0_f64.clamp(-2.0, 2.0)` which always evaluates to `0.0`. The `_rate: f64` parameter passed to the public `synthesize()` method is entirely ignored. The intended computation (matching the Python version) should derive speed from the rate input.

The comment `// 使用默认 rate=1.0` confirms this was known but left unimplemented. However, unlike other engines that intentionally have no speed control (e.g., Qwen API has no speed parameter), Tencent Cloud TTS explicitly supports a `Speed` field that should reflect the caller's rate setting.

```rust
// Current (wrong):
let speed_value = 0.0_f64.clamp(-2.0, 2.0);

// Should be:
// Pass rate parameter through synthesize_once → synthesize_at_url, then:
let speed_value = ((rate - 1.0) * 2.0).clamp(-2.0, 2.0);
```

**Fix:** Pass the `rate` parameter from `synthesize()` through `synthesize_once()` into `synthesize_at_url()` and use it in the speed computation.

---

### WR-02: Azure Speech SSML omits prosody rate and pitch

**File:** `src/tts/azure_speech.rs:237-239`

**Issue:**
The Azure Speech SSML template omits the `<prosody>` element entirely:
```rust
r#"<speak version="1.0" ... xml:lang="{}"><voice name="{}">{}</voice></speak>"#
```

All `_rate` and `_pitch` parameters passed to `synthesize()` are silently ignored. By contrast, the Edge-TTS engine (in `edge_tts.rs:94`) wraps the voice block in `<prosody rate="..." pitch="...">`. The Azure Speech REST API fully supports prosody tags for rate/pitch control, so this is a functionality gap — not an API limitation.

**Fix:**
```rust
r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="{}"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody></voice></speak>"#
```
And convert `rate`/`pitch` to the appropriate percentage/Hz format (similar to edge_tts.rs `convert_rate_to_percent` and `convert_pitch_to_hz`).

---

### WR-03: Whitespace-only text bypasses empty-text validation

**File (all occurences):**
- `src/tts/soulvoice.rs:55` — `text.trim()` inside `synthesize_once`
- `src/tts/doubaotts.rs:59` — `text.trim()` inside `synthesize_once`
- `src/tts/qwen_tts.rs:60` — `text.trim()` inside `synthesize_once`
- `src/tts/indextts2.rs:62` — `text.trim().to_string()` inside `synthesize_once`

**Issue:**
Each engine's `synthesize()` method validates `if text.is_empty()` before calling `retry_loop`. Inside `synthesize_once()`, the text is trimmed before being sent to the API. If `text` contains only whitespace (e.g., `"  "`), `is_empty()` returns `false`, the validation passes, but `trim()` produces an empty string which is sent to the upstream API. The API will likely return an error, but the resulting error message will be confusing (protocol-level error rather than "text cannot be empty").

Azure Speech and Tencent TTS engines have a related but different variant of this issue — they do not trim the text before sending, so whitespace-only text is sent as-is to the API, which may also reject it.

**Fix:** Change all empty-text checks from:
```rust
if text.is_empty() { return Err(...) }
```
to:
```rust
if text.trim().is_empty() { return Err(...) }
```

---

### WR-04: `should_use_azure_services` recompiles regex on each invocation

**File:** `src/tts/azure_speech.rs:28`

**Issue:**
`Regex::new(r"^[a-z]{2}-[A-Z]{2}-\w+Neural$")` is called every time `should_use_azure_services()` is invoked. If the regex compilation fails (e.g., under memory pressure), `.unwrap_or(false)` silently returns `false`, meaning the Azure engine is never selected and all voices silently fall through to Edge-TTS. This makes the Azure routing heuristic fragile.

The regex should be compiled once (e.g., via `once_cell::sync::Lazy` or `std::sync::OnceLock`) and reused.

**Fix:**
```rust
use std::sync::OnceLock;

fn azure_voice_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z]{2}-[A-Z]{2}-\w+Neural$").expect("Azure voice regex is valid"))
}
```

---

### WR-05: `build_client` silently falls back to unproxied client on error

**File:** `src/tts/common.rs:56`

**Issue:**
```rust
builder.build().unwrap_or_else(|_| reqwest::Client::new())
```
If `reqwest::Client::builder().build()` fails (e.g., due to TLS backend initialization), the error is silently swallowed and a new unconfigured client is returned. This means:
- Proxy configuration is lost without any warning
- If a corporate network requires the proxy, all subsequent TTS requests will fail with connection errors
- The user sees obscure connection-failure errors that don't mention the proxy

**Fix:** Log a warning when the builder fails:
```rust
builder.build().unwrap_or_else(|e| {
    tracing::warn!("Failed to build HTTP client with proxy config: {}. Falling back to default.", e);
    reqwest::Client::new()
})
```

---

### WR-06: Dead config field `SoulVoiceSection::voice_uri`

**File:** `src/config/types.rs:118`

**Issue:**
The `voice_uri` field is defined in `SoulVoiceSection` with a default value and even has a test value (`"speech:test"` in the test TOML at line 254), but it is **never read** by the `soulvoice` engine or any other code path. The voice identifier is passed exclusively through the `voice_name` parameter from the caller, parsed via `parse_engine_prefix`.

This field creates false expectations: users who set `voice_uri` in their config.toml will believe it affects behavior, but it has no effect.

**Fix:** Remove the `voice_uri` field from `SoulVoiceSection`, or document that it is reserved for future use.

---

### WR-07: Dead config fields `DoubaoTTSSection::ak` and `sk`

**File:** `src/config/types.rs:166,168`

**Issue:**
The `ak` (access key) and `sk` (secret key) fields in `DoubaoTTSSection` are never read by the Doubao TTS engine. Authentication uses only the `token` field, sent as `Authorization: Bearer;{token}`. The `ak` and `sk` fields appear to be leftover from a different authentication scheme (possibly CAM/STS temporary credentials that were never integrated).

These fields add confusion and surface area for misconfiguration.

**Fix:** Remove `ak` and `sk` from `DoubaoTTSSection`.

---

### WR-08: Dead config fields `UiSection::doubaotts_voice_type` and `doubaotts_rate`

**File:** `src/config/types.rs:84,86`

**Issue:**
The `UiSection` contains `doubaotts_voice_type: String` and `doubaotts_rate: f64` with both a default value and a test value. However, when the `"doubaotts"` engine is instantiated in `src/tts/mod.rs:104-109`, only `cfg.doubaotts` is passed — the `ui` section is never consulted. The voice type comes from the `voice_name` parameter (which originates from `ui.edge_voice_name` or `ui.azure_voice_name` in the caller), and the rate is hardcoded to 1.0.

This means users who configure `doubaotts_voice_type` or `doubaotts_rate` in their `[ui]` section will find their settings have no effect. The `doubaotts_rate` comment on line 37 of `doubaotts.rs` even acknowledges this: `// rate 暂不使用（Python 版实际使用固定值或 ui.doubaotts_rate）`.

**Fix (option A):** Wire the values through: pass `cfg.ui.doubaotts_voice_type` and `cfg.ui.doubaotts_rate` to the engine, or at minimum pass the `ui` section alongside `doubaotts` config.

**Fix (option B):** Remove the fields from `UiSection` and document that Doubao voice type comes from the standard `voice_name` flow (same as other engines).

---

### WR-09: `retry_loop` callers do not clean up corrupted output files after retry exhaustion

**File:** `src/tts/common.rs:89` (comment) and all 6 engine `synthesize()` implementations

**Issue:**
The comment on `retry_loop` states: `调用方负责在重试耗尽后清理可能残留的损坏输出文件`. However, none of the 6 engine `synthesize()` implementations that call `retry_loop` perform any cleanup on `TTSError::RetryExhausted`. If a previous retry attempt wrote partial data to the output file before failing, the corrupted file remains. Downstream consumers (video processing pipeline) may read this partial file and fail with cryptic errors.

By contrast, the `edge_tts` engine's internal `synthesize_with_retry` method (in `edge_tts.rs:402`) correctly cleans up:
```rust
let _ = tokio::fs::remove_file(output_path).await;
```

**Fix:** Either (a) add cleanup inside `retry_loop` after exhausting retries (before returning `RetryExhausted`), or (b) update all callers to clean up the output file on retry exhaustion:
```rust
common::retry_loop(|| self.synthesize_once(text, voice_name, output_path)).await.map_err(|e| {
    let _ = std::fs::remove_file(output_path);
    e
})
```

---

## Info

### IN-01: Tencent TTS sends empty `X-TC-Token` header

**File:** `src/tts/tencent_tts.rs:160`

**Issue:**
The header `X-TC-Token: ""` is sent with an empty value. For CAM temporary token authentication, this header should be omitted entirely when not using temporary credentials — some API gateways or proxies may reject empty header values per RFC 7230 Section 3.2.6.

**Fix:** Conditionally include the header only when a non-empty token is configured:
```rust
if !self.config.token.is_empty() {
    builder = builder.header("X-TC-Token", &self.config.token);
}
```
(Note: the `token` field in `TencentSection` does not exist currently, so this path would only activate if CAM support is added later.)

---

### IN-02: Tests bypass `retry_loop` by calling `synthesize_once` directly

**Files:** `soulvoice.rs:155`, `doubaotts.rs:201`, `qwen_tts.rs:194`, `indextts2.rs:182`, `azure_speech.rs:397`, `tencent_tts.rs:303`

**Issue:**
All 6 engine test suites call `synthesize_once()` directly instead of going through the public `synthesize()` method, which means (a) the retry logic in `retry_loop` is never tested, and (b) the empty-text validation in `synthesize()` is not exercised by success-path tests (only by explicit empty-text test cases).

The test for `soulvoice.rs:155` explicitly notes this: `// Override retry_loop by calling synthesize_once directly to avoid retry complexity`.

**Fix:** Add integration tests that exercise the full `synthesize()` path (with mock servers returning transient errors to exercise retry logic), even if they exist as separate test functions.

---

### IN-03: Proxy extraction code duplication in `mod.rs`

**File:** `src/tts/mod.rs:91-94` and `132-135`

**Issue:**
The same 3-line proxy extraction block (`(proxy_enabled, proxy_http, proxy_https) = match proxy {...}`) is duplicated verbatim in both the `"edge_tts"` and `"azure_speech"` fallback branches. This creates a maintenance hazard — if the proxy extraction logic changes, both sites must be updated.

```rust
let (proxy_enabled, proxy_http, proxy_https) = match proxy {
    Some(p) => (p.enabled, p.http.clone(), p.https.clone()),
    None => (false, String::new(), String::new()),
};
```

**Fix:** Extract to a helper function or use `common::ProxyConfig::from_proxy(proxy)` which already encapsulates this logic, and construct the `EdgeTtsEngine` from a `ProxyConfig` instead of three separate parameters. This would require adding a `From<&ProxyConfig>` or constructor overload on `EdgeTtsEngine`.

---

_Reviewed: 2026-04-30T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
