---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-28T12:15:00Z
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
  critical: 0
  warning: 2
  info: 1
  total: 3
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Report (Re-review)

**Reviewed:** 2026-04-28T12:15:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Re-review of 6 files after 4 fixes were applied (CR-01: duration division, WR-01: CONNECT RFC 7231 compliance, WR-02: word boundary parse logging, WR-03: dead test replacement). All 4 fixes are verified correct and complete.

Two new warnings were identified: (1) proxy port defaults to 80 even for HTTPS proxies without an explicit port, and (2) duration silently becomes 0.0 when the WebSocket connection closes before the `turn.end` message. One info-level issue was found: the version test is tied to a specific version string.

The 4 previously known info items (IN-01 through IN-04: hardcoded TrustedClientToken, unwrap on header values, loose CONNECT response check, voice_name not XML-escaped) remain intentionally unfixed and are not re-reported here.

---

## Verdict on Previously Fixed Issues

### CR-01: Duration tick-to-seconds division -- FIX VERIFIED CORRECT

**File:** `src/tts/edge_tts.rs:301`

The `as_f64()` branch at line 301 now correctly divides by `10_000_000.0`:
```rust
d / 10_000_000.0
```

All three branches (f64, str, ticks) are now consistent in dividing by 10,000,000. Previously this branch used the raw value `d` (the original code at commit `f7be7a9`), which would have produced a duration 10 million times too large if this branch had ever been exercised. The fix at commit `accc735` correctly adds the division.

### WR-01: CONNECT tunnel target strips path/query for RFC 7231 -- FIX VERIFIED CORRECT

**File:** `src/tts/edge_tts.rs:124-128`

A `target_host_only` variable was added to strip the URL path and query string before the first `/`. The `unwrap_or` fallback at line 128 now uses `target_host_only` instead of the unsanitized `target_addr`:

```rust
.unwrap_or((target_host_only, "443"));
```

The `and_then` branch already stripped the path correctly. Both paths are now consistent. Previously, the CONNECT request line would have included the full URL path and query string (e.g., `speech.platform.bing.com/consumer/speech/.../v1?TrustedClientToken=...`), violating RFC 7231 Section 4.3.6 which requires only `host:port`. The fix at commit `790355f` resolves this.

### WR-02: Word boundary parse failure logging -- FIX VERIFIED CORRECT

**File:** `src/tts/edge_tts.rs:287-291`

A `tracing::warn!` call was added in the `else` branch when `offset`, `duration`, or `text` fields are missing from the word boundary JSON:

```rust
tracing::warn!(
    "Failed to parse word boundary fields from JSON: {:?}",
    wb_json
);
```

Previously these failures were silently dropped, making debugging server format changes difficult. The fix at commit `daa8b2f` adds diagnostic logging.

Note: there is still a silent failure path -- if the JSON parse itself fails at line 273 (`serde_json::from_str` returns `Err`), the `if let Ok(...)` silently drops it. This was pre-existing and not in scope of WR-02.

### WR-03: Dead test replacement -- FIX VERIFIED CORRECT

**File:** `tests/tts_test.rs:145-148`

The original test defined an inner function that was never called, producing zero runtime coverage. The replacement at commit `1892514` now calls `tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None)` and assigns the returned future to `_`:

```rust
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None);
}
```

This provides a genuine compile-time check of the full function signature (all 7 parameter types and the return type) while having no runtime side effects because the future is dropped without being polled. This is a correct improvement over the previous version.

---

## Warnings

### WR-04: Proxy port defaults to 80 for HTTPS proxies without explicit port

**File:** `src/tts/edge_tts.rs:115-119`

**Issue:** When parsing the proxy URL, the code strips the scheme prefix, then splits on `:`. If there is no explicit port, the `unwrap_or` fallback hardcodes port 80:

```rust
let (proxy_host, proxy_port_str) = proxy_addr
    .split_once(':')
    .unwrap_or((proxy_addr, "80"));   // line 117: always 80
```

When `proxy_https` is set to a URL like `https://proxy.example.com` (no explicit port), the stripped URL `proxy.example.com` has no colon, triggering the port 80 fallback. However, the default port for HTTPS is 443, not 80.

This is significant because the proxy selection logic (lines 103-106) prefers `proxy_https` over `proxy_http`. A user configuring `proxy_https = "https://proxy.example.com"` would silently fail to connect:

```rust
let proxy_url = if !self.proxy_https.is_empty() {
    &self.proxy_https      // selected
} else if !self.proxy_http.is_empty() {
    &self.proxy_http
};
```

**Fix:** Determine the default port from the proxy URL scheme before stripping it:

```rust
let default_port = if proxy_url.starts_with("https://") {
    "443"
} else {
    "80"
};
let (proxy_host, proxy_port_str) = proxy_addr
    .split_once(':')
    .unwrap_or((proxy_addr, default_port));
```

### WR-05: Duration silently zero when WebSocket closes before turn.end

**File:** `src/tts/edge_tts.rs:256-328`

**Issue:** The `duration` variable is initialized to `0.0` at line 257 and only updated when the `turn.end` message is processed (lines 293-311). If the WebSocket connection terminates with a `Close` frame (line 320-323) before `turn.end` arrives, the loop breaks and the function proceeds to write whatever audio data was received and returns `Ok(TtsOutput { duration: 0.0, ... })`.

The caller cannot distinguish between:
- A valid zero-length synthesis (impossible in practice but represented as `0.0`)
- A connection that dropped mid-stream with partial audio but no duration metadata

This is a real scenario: if the Edge TTS server encounters an error after sending some audio data but before the final `turn.end` metadata, the `Close` frame causes the loop to exit (line 322), leaving `duration` at `0.0`. Downstream consumers using `duration` for timing (subtitle alignment, audio trimming) would silently operate on incorrect values.

**Fix:** After the message loop (after line 328), check if duration is still `0.0` despite having received audio data. At minimum add a warning:

```rust
if duration == 0.0 && !audio_data.is_empty() {
    tracing::warn!(
        "Edge-TTS connection closed before turn.end; duration is 0.0, audio data size: {} bytes",
        audio_data.len()
    );
}
```

For stronger guarantees, consider returning `Err(TTSError::SynthesisFailed(...))` when the connection drops mid-stream without duration metadata.

---

## Info

### IN-05: Version test tied to hardcoded version string

**File:** `src/lib.rs:17-19`

**Issue:** The test `test_version_returns_0_1_0` asserts against the literal string `"0.1.0"`:

```rust
#[test]
fn test_version_returns_0_1_0() {
    assert_eq!(version(), "0.1.0");
}
```

This test will fail whenever the version in `Cargo.toml` is bumped, requiring a coordinated update to both files. Consider comparing against the compile-time constant directly:

```rust
#[test]
fn test_version_matches_cargo_toml() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
```

---

## Pre-existing Info Items (Intentionally Deferred)

The following 4 info-level items from the original review remain intentionally unfixed:

- **IN-01:** Hardcoded `TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4` in WebSocket URL (not a secret, but no override mechanism)
- **IN-02:** `unwrap()` on hardcoded header values at lines 92,97 (safe but not self-documenting)
- **IN-03:** CONNECT response checked with loose `contains("200")` at line 158 (may match unexpected status lines)
- **IN-04:** `voice_name` not XML-escaped in SSML attribute context at line 51 (controlled inputs mitigate risk)

---

_Reviewed: 2026-04-28T12:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: re-review (post-fix)_
