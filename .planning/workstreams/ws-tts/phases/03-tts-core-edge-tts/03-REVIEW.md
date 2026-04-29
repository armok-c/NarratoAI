---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T10:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - src/error.rs
  - src/lib.rs
  - Cargo.toml
  - tests/tts_test.rs
findings:
  critical: 2
  warning: 2
  info: 3
  total: 7
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine — Code Review Report

**Reviewed:** 2026-04-29T10:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Reviewed the TTS core abstractions (`src/tts/mod.rs`), Edge-TTS WebSocket engine (`src/tts/edge_tts.rs`), error types (`src/error.rs`), module registrations (`src/lib.rs`), Cargo dependencies (`Cargo.toml`), and integration tests (`tests/tts_test.rs`).

The architecture is well-structured: `TtsProvider` trait with clean async boundaries, a string-dispatched router in `mod.rs`, and a focused WebSocket implementation in `edge_tts.rs`. Error types use `thiserror` idiomatically.

**However, two blocker issues exist.** The Cargo.toml references `tokio-tungstenite` feature `rustls-tls` which does not exist in v0.29.0, and the proxy code path requires the `native-tls` feature which is not enabled -- confirmed by `cargo metadata` failing resolution. Additionally, the HTTP CONNECT proxy tunnel has a `BufReader` read-ahead data loss risk, and the proxy URL parser breaks on IPv6 addresses. Three info-level findings cover a tautological test, a redundant import, and a documentation error.

## Critical Issues

### CR-01: tokio-tungstenite feature `rustls-tls` does not exist in v0.29.0; proxy code path requires `native-tls`

**File:** `Cargo.toml:17`
**File:** `src/tts/edge_tts.rs:254`
**Issue:** Two interdependent compilation errors confirmed by `cargo metadata`:

1. **Feature name mismatch.** The `Cargo.toml` specifies `features = ["rustls-tls"]` for `tokio-tungstenite = "0.29.0"`, but v0.29.0 exposes `rustls-tls-native-roots` and `rustls-tls-webpki-roots` (not bare `rustls-tls`). The dependency resolver output confirms this:
   ```
   package `narratoai-core` depends on `tokio-tungstenite` with feature `rustls-tls`
   but `tokio-tungstenite` does not have that feature.
   package `tokio-tungstenite` does have feature `__rustls-tls`
   ```

2. **Missing `native-tls` feature.** The HTTP CONNECT proxy tunnel (edge_tts.rs:254) uses `tokio_tungstenite::Connector::NativeTls(tls_connector)`, which is gated behind `#[cfg(feature = "native-tls")]` in tokio-tungstenite v0.29.0. Since `native-tls` is not enabled for tokio-tungstenite in `Cargo.toml`, the `Connector::NativeTls` variant is unavailable. The library cannot compile.

**Fix:**
```toml
# Replace Cargo.toml line 17 with:
tokio-tungstenite = { version = "0.29.0", features = ["native-tls", "rustls-tls-native-roots"] }
```

This enables both TLS backends: `native-tls` (for the CONNECT tunnel's explicit `Connector::NativeTls` usage) and `rustls-tls-native-roots` (for direct connections via `connect_async`). They coexist without conflict because `MaybeTlsStream` is a sum enum supporting both variants.

### CR-02: HTTP CONNECT proxy tunnel -- `BufReader::into_inner()` discards read-ahead buffer, risk of TLS data corruption

**File:** `src/tts/edge_tts.rs:252`
**Issue:** The proxy tunnel wraps the `TcpStream` in `tokio::io::BufReader` (default 8 KB buffer) to read the CONNECT response line-by-line. After consuming the complete response (status line + headers + terminating `\r\n`), the code calls `buf_reader.into_inner()` to extract the raw `TcpStream` for TLS upgrade. However, `into_inner()` discards the internal buffer. If the `BufReader` read ahead and consumed bytes from the TCP stream beyond the CONNECT response -- specifically, bytes from the server's subsequent TLS handshake response -- those bytes are permanently lost.

This is a **correctness bug that causes intermittent, hard-to-reproduce TLS handshake failures**. It depends on network timing: if the proxy sends the CONNECT 200 response and the TLS ServerHello coalesce into the same TCP segment (or arrive close enough that a single `read` syscall pulls in both), the `BufReader` will buffer both, and the TLS library will receive a truncated stream.

**Fix:** Avoid `BufReader` for the CONNECT response. Use a small fixed-size buffer with manual parsing, preserving the raw `TcpStream`:

```rust
use tokio::io::AsyncReadExt;

let mut tcp = tcp_stream;
let mut response_buf = vec![0u8; 4096]; // 4 KB is sufficient for proxy responses
let mut total_read = 0usize;

loop {
    let n = tcp.read(&mut response_buf[total_read..]).await
        .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;
    if n == 0 {
        return Err(TTSError::ConnectionFailed("代理连接提前关闭".to_string()));
    }
    total_read += n;
    // Look for end-of-headers marker
    if response_buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
        break;
    }
    if total_read >= response_buf.len() {
        return Err(TTSError::ConnectionFailed("代理响应头过大".to_string()));
    }
}

// Parse response from response_buf[..total_read]
// Pass raw `tcp` (unconsumed by any buffer) to TLS+WebSocket upgrade
```

If `BufReader` must be kept, verify the buffer is empty before `into_inner()`:
```rust
let remaining = buf_reader.buffer().len();
if remaining > 0 {
    tracing::warn!("BufReader has {} unconsumed bytes; potential data loss on into_inner()", remaining);
}
// Note: this still loses data, but at least surfaces the condition.
```

## Warnings

### WR-01: Proxy URL parser breaks on IPv6 addresses

**File:** `src/tts/edge_tts.rs:139-162`
**Issue:** The proxy host/port parser uses `host_port.split_once(':')` to split the address. For an IPv6 proxy address like `http://[::1]:7890`, after scheme stripping the string becomes `[::1]:7890`. Then `split_once(':')` finds the first colon after `[`, returning `("[" , ":1]:7890")`. The port parser tries to parse `:1]:7890` as `u16`, which fails. The error message ("代理端口格式错误，代理 URL 不支持认证凭据") is misleading.

IPv6 proxy addresses are a real use case (e.g., local proxy on `[::1]:7890` for testing, or IPv6-only network environments).

**Fix:** Detect IPv6 bracket notation before splitting:
```rust
let (proxy_host, proxy_port) = if host_port.starts_with('[') {
    host_port.rsplit_once("]:")
        .map(|(addr_inner, port_str)| {
            let port: u16 = port_str.parse()
                .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
            Ok::<_, TTSError>((&addr_inner[1..], port))
        })
        .unwrap_or(Err(TTSError::ConnectionFailed("代理 IPv6 地址格式错误".to_string())))?
} else {
    match host_port.split_once(':') {
        Some((host, port_str)) => {
            let port: u16 = port_str.parse()
                .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
            (host, port)
        }
        None => (host_port, default_proxy_port),
    }
};
```

### WR-02: EdgeTtsEngine struct missing `Debug` derive, impairs diagnostics

**File:** `src/tts/edge_tts.rs:86-90`
**Issue:** The `EdgeTtsEngine` struct does not derive `Debug`. All its fields (`bool`, `String`, `String`) implement `Debug`, so adding the derive costs nothing. Without it, the engine cannot be used in `tracing::debug!("{:?}", engine)` calls, test failure messages, or error reports -- all of which are valuable for a network-connected component with configurable proxy parameters.

**Fix:**
```rust
#[derive(Debug)]
pub(super) struct EdgeTtsEngine {
    pub(super) proxy_enabled: bool,
    pub(super) proxy_http: String,
    pub(super) proxy_https: String,
}
```

## Info

### IN-01: Version test is a tautology

**File:** `src/lib.rs:18`
**Issue:** The test `test_version_matches_cargo_toml` asserts:
```rust
assert_eq!(version(), env!("CARGO_PKG_VERSION"));
```
But `version()` returns `env!("CARGO_PKG_VERSION")` (line 9). Both sides of the assertion expand to the exact same compile-time constant. The test passes by construction and cannot fail for any meaningful reason. It tests nothing.

**Fix:** Either remove the test or make it assert something non-trivial (e.g., `assert!(!version().is_empty())` or `assert!(version().contains('.'))`).

### IN-02: Redundant `use async_trait::async_trait` in test module

**File:** `src/tts/mod.rs:99`
**Issue:** Inside `#[cfg(test)] mod tests`, line 98 imports `use super::*;` which brings all parent-module items into scope -- including the parent's `use async_trait::async_trait;` on line 3. Line 99's explicit `use async_trait::async_trait;` is therefore redundant. Clippy flags this under `clippy::redundant_import`.

**Fix:** Remove line 99.

### IN-03: Test comment mislabels microseconds vs nanoseconds

**File:** `tests/tts_test.rs:78`
**Issue:** The comment reads:
```rust
/// 5 秒 = 5_000_000_000 微秒 = 50_000_000 单位（100ns 单位）
```
5 seconds = 5,000,000 microseconds, not 5,000,000,000. The value `5_000_000_000` is the count of **nanoseconds**, not microseconds. The computed value of 50,000,000 hundred-nanosecond units is correct. The intermediate unit label is wrong.

**Fix:** Change `微秒` to `纳秒` in the comment.

---

## Additional Observations

- **`Cargo.toml:28`** -- `wiremock` is listed in `[dev-dependencies]` but is not used in any of the reviewed test files. If unused project-wide, it should be removed.
- **`src/tts/mod.rs:83-86`** -- The `synthesize` router unconditionally destructures the `proxy` parameter before the match dispatch. For the `_ => TTSError::UnknownEngine` path, this destructuring is wasted work. Consider deferring it inside the `"edge_tts"` arm.
- **`src/tts/edge_tts.rs:11`** -- The WebSocket receive timeout (`WS_RECEIVE_TIMEOUT` = 120 s) applies per-message, not per-synthesis. For very long texts, the Edge TTS server may take longer than 120 s to produce word-boundary messages between audio chunks. Consider documenting that this is a per-message timeout.
- **`src/tts/edge_tts.rs:196-243`** -- The proxy response reader has no line-length limit. A misconfigured or malicious proxy could send an extremely long header line, causing unbounded memory allocation. Consider adding a maximum line length check.

---

_Reviewed: 2026-04-29T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
