---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T15:10:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/error.rs
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report (Iteration 2)

**Reviewed:** 2026-04-29T15:10:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

This is the second review iteration for phase 03. The first review (03-REVIEW.md, previous version) found 13 issues: 0 critical, 5 warnings, 8 info. **Those 13 issues remain unaddressed** -- none of the phase 03 fix commits have been applied to source files (only phase 02 fixes appear in `git log`). `from_utf8_lossy`, `socks5 => 1080u16`, `wiremock`, and all other previously reported issues still exist in the current code.

This second review identifies **5 new findings** (3 warnings, 2 info) that were not caught in the first review. There are **0 critical** issues.

New findings summary:
1. **WR-06**: CONNECT response parsing data loss when 1xx interim and 2xx final responses coalesce in the same TCP segment, causing a permanent hang (no timeout applies during CONNECT).
2. **WR-07**: `parse_edge_tts_binary` matches the `Path:` header case-sensitively -- a future Edge TTS server change to lowercase `path:` would silently break all binary message parsing.
3. **WR-08**: `synthesize` validates `text`, `rate`, and `pitch` but not `voice_name` -- an empty voice name produces invalid SSML (`<voice name="">`).
4. **IN-09**: `expect()` calls for hardcoded `HeaderValue` parsing are a latent panic risk.
5. **IN-10**: Signature test uses `Path::new("")` -- fragile if the test is ever actually awaited.

## Warnings

### WR-06: CONNECT 1xx+2xx responses coalesced in same TCP segment cause data loss and hang

**File:** `src/tts/edge_tts.rs:227-272`
**Severity:** Warning

**Issue:** The HTTP CONNECT response parsing loop uses a three-level nested structure (outer loop for 1xx retry, inner loop for reading until `\r\n\r\n`). When a proxy sends both a 1xx interim response (e.g., `HTTP/1.1 100 Continue`) and the 2xx final response in the same TCP segment, a single `tcp.read()` call consumes both responses:

1. The inner loop reads all available bytes into `response_buf`, finds the first `\r\n\r\n` (end of the 1xx response), and breaks.
2. `response_buf[..total_read]` now contains both responses contiguously: `"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 Connection established\r\n\r\n"`.
3. The parser extracts the status line from `response_str.lines().next()`, which is `"HTTP/1.1 100 Continue"`. Status code 100 -- `continue` goes to the outer loop.
4. The outer loop **creates a new `response_buf`** (destroying the old one), resetting `total_read = 0`.
5. The 2xx response bytes that were already consumed from the kernel TCP receive buffer are now lost -- they were in the destroyed `response_buf`.
6. `tcp.read()` on the new buffer blocks waiting for data that will never arrive. **The CONNECT handshake hangs forever.**

The code at lines 210-211 has a detailed comment about avoiding BufReader because `into_inner()` discards buffered data and corrupts the TLS handshake. The same class of data-loss bug exists in the response parsing loop itself.

This is uncommon because:
- Most proxies respond to CONNECT with a single 200 OK (no 1xx interim).
- The CONNECT request does not include `Expect: 100-continue`, so 100 Continue is unusual.
- TCP segments may not coalesce on every platform.

However, when triggered this is a permanent hang -- the `WS_RECEIVE_TIMEOUT` of 120 seconds (line 384) applies to WebSocket message receiving, NOT to the CONNECT handshake stage. The `connect()` function has no timeout of its own.

**Fix:** Before `continue`ing on a 1xx response, check whether the current `response_buf` already contains additional data after the `\r\n\r\n` separator. If so, process it as the next response instead of creating a fresh buffer:

```rust
} else if (100..200).contains(&status_code) {
    // 1xx interim -- check for leftover 2xx response in current buffer
    let sep_end = response_buf[..total_read]
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    if sep_end < total_read {
        // Leftover data exists -- shift it to buffer start and re-parse
        let extra = response_buf[sep_end..total_read].to_vec();
        response_buf[..extra.len()].copy_from_slice(&extra);
        total_read = extra.len();
        // Re-enter the parsing logic instead of continue
        // (needs restructuring to avoid goto-like control flow)
    } else {
        continue;
    }
}
```

A simpler alternative: add a timeout to the CONNECT stage itself so that a hang is not permanent:

```rust
use tokio::time::timeout;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

// wrap the outer loop with timeout
timeout(CONNECT_TIMEOUT, async {
    // ... existing CONNECT logic ...
}).await
.map_err(|_| TTSError::ConnectionFailed("代理 CONNECT 超时".to_string()))?
```

---

### WR-07: `parse_edge_tts_binary` Path header matching is case-sensitive

**File:** `src/tts/edge_tts.rs:523`
**Severity:** Warning

**Issue:** The binary message parser uses case-sensitive matching:

```rust
.find(|line| line.starts_with("Path:"))
```

If Edge TTS ever sends the header in lowercase (`path:`) or mixed case (`PATH:`), `parse_edge_tts_binary` returns `None` for all binary messages. The consequence:

- All audio frames are discarded -- `audio_data` remains empty.
- All `wordboundary` and `turn.end` messages are discarded.
- The function eventually errors with `"未收到音频数据"`.
- Every binary message triggers a `tracing::warn!("无法解析 Edge-TTS 二进制消息: {}")`.

RFC 7230 Section 3.2 states that HTTP header field names are case-insensitive. While Microsoft's Edge TTS currently sends `Path:` (capital P), this is an implementation detail that could change. The Rust ecosystem standard practice for HTTP header matching is `eq_ignore_ascii_case()` or `to_ascii_lowercase()`.

**Fix:**

```rust
             .find(|line| line.to_ascii_lowercase().starts_with("path:"))
```

This adds negligible runtime cost (ASCII lowercasing a short string once per message) and eliminates a latent failure mode.

---

### WR-08: Missing validation for empty `voice_name` in `synthesize`

**File:** `src/tts/edge_tts.rs:540-542`
**Severity:** Warning

**Issue:** The `TtsProvider::synthesize` implementation validates three input parameters:

```rust
if text.is_empty() { ... }     // line 540
if !rate.is_finite() || rate < 0.0 { ... }  // line 543
if !pitch.is_finite() { ... }  // line 548
```

But `voice_name` is not validated. An empty `voice_name` string propagates into `build_ssml()` and produces:

```xml
<voice name="">...</voice>
```

This is invalid SSML. Edge TTS will reject it with a server-side error that arrives as a generic WebSocket close or error frame, making diagnosis difficult (the error message will mention connection or protocol issues, not the root cause of an empty voice name).

**Fix:** Add validation before the SSML construction:

```rust
if text.is_empty() {
    return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
}
if voice_name.is_empty() {
    return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
}
if !rate.is_finite() || rate < 0.0 {
    return Err(TTSError::SynthesisFailed(format!("rate 必须为有限正数: {}", rate)));
}
if !pitch.is_finite() {
    return Err(TTSError::SynthesisFailed(format!("pitch 必须为有限数值: {}", pitch)));
}
```

---

## Info

### IN-09: Hardcoded header `expect()` is a latent panic risk

**File:** `src/tts/edge_tts.rs:132, 137`
**Severity:** Info

**Issue:** The `Origin` and `User-Agent` header values are parsed with `.expect()`:

```rust
"chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
    .parse()
    .expect("Origin 值是硬编码字面量，解析 HeaderValue 不应失败"),
```

While the current hardcoded strings are valid `HeaderValue`s, if a future modification introduces an invalid character (e.g., non-ASCII, control character, embedded NUL), `parse()` returns `Err` and `expect()` causes a panic at runtime. This is a common pattern in Rust for hardcoded values and the risk is low, but the `expect()` message is misleading -- it says "不应失败" (should not fail) but provides no recourse if it does.

**Suggestion:** Replace with `unwrap()` (shorter, same semantics) and add an explanatory comment. Or, extract the headers into `const` values using `HeaderValue::from_static()` when the values are known to be valid static strings, eliminating the runtime parse entirely:

```rust
use http::HeaderValue;
const EDGE_ORIGIN: HeaderValue = HeaderValue::from_static(
    "chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
);
```

(Note: `http` crate is not currently a dependency. This would need to be added, or kept as-is with a clearer comment.)

---

### IN-10: Signature test uses `Path::new("")` empty path

**File:** `tests/tts_test.rs:108`
**Severity:** Info

**Issue:** The compile-time signature test `test_synthesize_function_signature` passes `Path::new("")` as the output path:

```rust
#[allow(clippy::let_underscore_future)]
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize(engine, text, voice_name, 1.0, 0.0, Path::new(""), None);
}
```

The `let _ =` immediately drops the future without polling it, so the empty path is never used. But this is fragile:
- If a future refactoring adds `.await` to this test (perhaps to verify more than just the signature), the empty path would cause an I/O error: `tokio::fs::write(output_path, ...)` with an empty string path fails with a platform-dependent error (e.g., "No such file or directory" on Linux, "The filename, directory name, or volume label syntax is incorrect" on Windows).
- The `#[allow]` suppresses a Clippy lint that exists for a reason.

**Suggestion:** Replace with a non-empty path that is clearly a sentinel:

```rust
let _ = tts::synthesize(
    engine, text, voice_name, 1.0, 0.0,
    Path::new("/tmp/_narratoai_sig_check.mp3"),
    None,
);
```

Or, if the test should remain strictly compile-time and never poll, add a comment explicitly documenting this constraint:

```rust
// WARNING: This future is NEVER polled. It is a compile-time signature check only.
// Do NOT add .await or block_on without also providing a valid output_path.
```

---

## Previously Reported Issues (Still Present -- Not Re-Listed)

The following 13 issues from the first review are still present in the current code (no fix commits applied):

| ID | Severity | Description |
|----|----------|-------------|
| WR-01 | Warning | IPv6 proxy without explicit port fails |
| WR-02 | Warning | Proxy credentials silently dropped |
| WR-03 | Warning | `from_utf8_lossy` masks data corruption |
| WR-04 | Warning | turn.end parse failure silently sets duration 0 |
| WR-05 | Warning | Partial output file not cleaned up on retry exhaustion |
| IN-01 | Info | XML 1.0 restricted chars 0x7F-0x9F not filtered |
| IN-02 | Info | `connect` function too long (~180 lines) |
| IN-03 | Info | Duplicate error classification functions |
| IN-04 | Info | Unused `wiremock` dev-dependency |
| IN-05 | Info | Missing boundary value tests for `convert_rate_to_percent` |
| IN-06 | Info | SOCKS5 handling misleading |
| IN-07 | Info | `voice_name_to_lang` defaults to "zh-CN" for unknown voices |
| IN-08 | Info | No unit tests for `parse_edge_tts_binary` |

---

_Reviewed: 2026-04-29T15:10:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 2_
