---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T11:42:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
  - src/error.rs
  - src/lib.rs
  - Cargo.toml
findings:
  critical: 2
  warning: 3
  info: 3
  total: 8
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**Reviewed:** 2026-04-29T11:42:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Reviewed the TTS core abstractions (`src/tts/mod.rs`), Edge-TTS WebSocket engine (`src/tts/edge_tts.rs`), error types (`src/error.rs`), module registrations (`src/lib.rs`), dependency declarations (`Cargo.toml`), and integration tests (`tests/tts_test.rs`).

The previous iteration's 7 findings have been fixed (verified by cross-referencing against `03-REVIEW-FIX.md`). The `BufReader::into_inner()` data-loss issue (former CR-02) was restructured to use a fixed-size raw buffer -- a sound pattern that eliminates the root cause.

**However, the restructuring introduced a new compile error in the proxy tunnel, and two pre-existing UB-class bugs remain unaddressed.** Two critical issues are present: an undefined variable in the proxy code path (blocking compilation of the proxy feature), and undefined behavior on NaN/infinite floating-point inputs in the rate/pitch conversion functions. Three warning-level findings cover missing input validation, silent protocol error swallowing, and fragile proxy URL parsing. Three info items cover unused variables and minor dependency concerns.

## Critical Issues

### CR-01: Undefined variable `connect_req` in HTTP CONNECT proxy tunnel -- compile error

**File:** `src/tts/edge_tts.rs:194`
**Issue:** The variable `connect_req` is referenced on line 194 (`tcp.write_all(connect_req.as_bytes()).await`) but is never defined anywhere in the function. The previous CR-02 fix (BufReader restructure, commit d569dc2) replaced the I/O buffering pattern but omitted the construction of the HTTP CONNECT request string. Additionally, `target_host` (line 178) and `target_port` (line 182) are parsed and computed but never consumed, confirming that the intended CONNECT request construction was never completed.

This blocks compilation of the entire proxy code path. Confirmed by `cargo check`:

```
error[E0425]: cannot find value `connect_req` in this scope
   --> src\tts\edge_tts.rs:194:27
    |
194 |             tcp.write_all(connect_req.as_bytes()).await
    |                           ^^^^^^^^^^^ not found in this scope
```

**Fix:** Construct the CONNECT request string before sending it. The request must include the target host and port extracted above, and follow HTTP/1.1 CONNECT format:

```rust
// After line 184 (let target_port ...)
let connect_req = format!(
    "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
    target_host, target_port, target_host, target_port
);
```

### CR-02: Undefined behavior on NaN / infinity rate or pitch in conversion functions

**File:** `src/tts/edge_tts.rs:28,42`
**Issue:** Both `convert_rate_to_percent` and `convert_pitch_to_hz` cast a computed `f64` value to `i32` without checking for NaN or infinity:

```rust
// Line 28
let diff = ((rate - 1.0) * 100.0).round() as i32;  // UB if rate is NaN or infinity

// Line 42
let p = pitch.round() as i32;  // UB if pitch is NaN or infinity
```

In Rust, casting `f64::NAN` or `f64::INFINITY` to an integer type is **undefined behavior**. The Rust Reference explicitly states that "casting a NaN or infinity value to an integer type results in undefined behavior." Since `synthesize()` is a public async function accepting `f64` without validation, any caller (including unit tests, integration tests, or future Python bindings via PyO3) can trigger UB.

These functions are called from `build_ssml()` (line 69-70), which is called from the public `synthesize()` (line 519). No input validation gate protects the conversion.

**Fix:** Add a finite check at the top of both functions. Alternatively, validate rate/pitch at the `synthesize()` entry point and return `Err(TTSError::SynthesisFailed(...))` for invalid values.

Option A -- validate at the conversion functions (defense-in-depth):

```rust
fn convert_rate_to_percent(rate: f64) -> String {
    if !rate.is_finite() || rate < 0.0 {
        return "+0%".to_string();  // fallback safe value
    }
    if rate == 1.0 {
        return "+0%".to_string();
    }
    let diff = ((rate - 1.0) * 100.0).round() as i32;
    // ... rest unchanged
}

fn convert_pitch_to_hz(pitch: f64) -> String {
    if !pitch.is_finite() {
        return "+0Hz".to_string();  // fallback safe value
    }
    if pitch == 0.0 {
        return "+0Hz".to_string();
    }
    let p = pitch.round() as i32;
    // ... rest unchanged
}
```

Option B -- validate at the synthesize entry point (fail-fast):

```rust
// In TtsProvider::synthesize for EdgeTtsEngine or the router function:
if !rate.is_finite() || !pitch.is_finite() {
    return Err(TTSError::SynthesisFailed(
        "rate/pitch must be finite numbers".to_string(),
    ));
}
```

## Warnings

### WR-01: No input validation before establishing WebSocket connection

**File:** `src/tts/edge_tts.rs:519`
**Issue:** The public `synthesize()` method (both the trait impl in edge_tts.rs and the router in mod.rs) accepts arbitrary `text`, `rate`, and `pitch` values without validation. Empty `text` results in an SSML request with an empty `<speak>...</speak>` body, which the Edge TTS server may reject or handle unpredictably. NaN/infinite `rate`/`pitch` are forwarded to `build_ssml` which invokes UB (see CR-02). Negative `rate` (e.g., -1.0) produces nonsensical percentage strings like "-200%". Exceeding reasonable bounds on `rate` (e.g., 100.0, producing "+9900%") wastes a WebSocket connection that will almost certainly be rejected.

While most callers within this codebase pass reasonable values, the API surface has no contracts enforcing this, making it fragile when reused.

**Fix:** Add validation at the top of `EdgeTtsEngine::synthesize()` (or the router `synthesize()`) before building SSML or connecting:

```rust
async fn synthesize(
    &self,
    text: &str,
    voice_name: &str,
    rate: f64,
    pitch: f64,
    output_path: &Path,
) -> Result<TtsOutput, TTSError> {
    if text.is_empty() {
        return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
    }
    if !rate.is_finite() || rate < 0.0 {
        return Err(TTSError::SynthesisFailed(
            format!("rate 必须为有限正数: {}", rate),
        ));
    }
    if !pitch.is_finite() {
        return Err(TTSError::SynthesisFailed(
            format!("pitch 必须为有限数值: {}", pitch),
        ));
    }
    let ssml = build_ssml(text, voice_name, rate, pitch);
    self.synthesize_with_retry(&ssml, output_path).await
}
```

### WR-02: Binary message parse failure silently discarded -- no diagnostic

**File:** `src/tts/edge_tts.rs:387`
**Issue:** In the receive loop of `synthesize_once`, the result of `parse_edge_tts_binary(&data)` is consumed with `if let Some(content) = ...`. When parsing fails (returns `None`), the binary message is silently skipped with zero diagnostic output:

```rust
if let Some(content) = parse_edge_tts_binary(&data) {
    if content.path == "audio" { ... }
    else if content.path == "wordboundary" { ... }
    // etc.
}
// No `else` branch -- parse failure is invisible
```

During development or against future server protocol changes, unrecognized binary messages will be dropped without trace, making debugging extremely difficult. The same problem applies at the inner field-parsing level (line 394-413): when word boundary JSON parsing fails, a `tracing::warn!` is emitted, but the raw payload that failed to parse is not included in the log, making diagnosis harder.

**Fix:** Add a `tracing::warn!` for parse failures, including the raw bytes truncated to a reasonable limit:

```rust
if let Some(content) = parse_edge_tts_binary(&data) {
    // ... existing processing ...
} else {
    // Log truncated raw data for diagnostics
    let preview = if data.len() > 256 {
        format!("{} bytes (first 256 shown: {:?})", data.len(), &data[..256])
    } else {
        format!("{:?}", data)
    };
    tracing::warn!("无法解析 Edge-TTS 二进制消息: {}", preview);
}
```

### WR-03: Fragile proxy URL parsing -- uppercase scheme, missing scheme, unknown scheme

**File:** `src/tts/edge_tts.rs:140-146`
**Issue:** The proxy URL parsing uses `trim_start_matches` which is case-sensitive. A proxy URL with uppercase scheme (`HTTP://proxy:8080`) or mixed case (`Http://proxy:8080`) will not be stripped correctly:

```rust
let default_proxy_port = if proxy_url.starts_with("https://") { 443 } else { 80 };
let proxy_addr = proxy_url
    .trim_start_matches("http://")     // won't match "HTTP://..."
    .trim_start_matches("https://");   // won't match "HTTPS://..."
```

For example, `HTTP://proxy:8080` becomes `HTTP://proxy:8080` (no stripping), and `host_port.split_once(':')` splits on the first colon, yielding `("HTTP", "//proxy:8080")` which parses as host=`HTTP` and port=`//proxy:8080` -- the port parse fails.

Additionally, unsupported scheme prefixes like `socks5://` are not stripped, and the `default_proxy_port` logic only recognizes `https://`, defaulting to port 80 for all other schemes (including `socks5://`).

**Fix:** Use case-insensitive matching or lower-case the scheme portion before checking. Also handle explicit port defaults by scheme:

```rust
let (proxy_scheme, proxy_rest) = proxy_url.split_once("://")
    .unwrap_or(("", proxy_url));  // no scheme → assume http
let proxy_addr = proxy_rest;
let default_proxy_port = match proxy_scheme.to_lowercase().as_str() {
    "https" | "wss" => 443u16,
    "socks5" => 1080u16,
    _ => 80u16,
};
```

## Info

### IN-01: Unused variables `target_host` and `target_port` -- dead code from incomplete CONNECT request

**File:** `src/tts/edge_tts.rs:178,182`
**Issue:** The variables `target_host` (line 178) and `target_port` (line 182) are computed to support the HTTP CONNECT request construction, but since `connect_req` was never defined (CR-01), these variables are dead code. They will produce Rust compiler warnings (`unused_variables`). While removing them would hide the CR-01 bug, they should be kept once CR-01 is fixed since the CONNECT request needs them.

**Fix:** Resolve CR-01 first. These variables become active code once `connect_req` is defined and used. Until then, prefix with underscore to suppress warnings:

```rust
let (target_host, target_port_str) = target_addr...  // or use let (_target_host, ...) 
```

### IN-02: Pre-release dependency `notify = "9.0.0-rc.3"`

**File:** `Cargo.toml:14`
**Issue:** The `notify` crate is pinned to a release candidate version (`9.0.0-rc.3`). Release candidates are not guaranteed API-stable -- a semver-incompatible change in the final 9.0.0 release could break the `From<notify::Error>` impl in `src/error.rs`. The project should either migrate to a stable version or acknowledge the dependency risk.

**Fix:** Pin to the latest stable `notify` release if available (e.g., `notify = "8.0.0"` or wait for `9.0.0` stable), or add a comment documenting why the RC is required.

### IN-03: Signature test uses opaque empty-string parameters

**File:** `tests/tts_test.rs:105`
**Issue:** The compile-time signature test `test_synthesize_function_signature` calls `tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None)` with all string parameters passed as `""`. This makes the parameter order opaque -- a reader cannot tell which `""` maps to `engine`, `text`, or `voice_name`. A future refactor that reorders parameters would still compile but behave incorrectly, and this test would not catch it because it discards the future.

**Fix:** Use descriptive variable bindings:

```rust
#[test]
fn test_synthesize_function_signature() {
    let engine = "edge_tts";
    let text = "";
    let voice_name = "";
    let _ = tts::synthesize(engine, text, voice_name, 1.0, 0.0, Path::new(""), None);
}
```

This also makes it clearer that passing an empty `engine` to the signature test is acceptable (the future is never polled, so `UnknownEngine` is never returned).

---

_Reviewed: 2026-04-29T11:42:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
