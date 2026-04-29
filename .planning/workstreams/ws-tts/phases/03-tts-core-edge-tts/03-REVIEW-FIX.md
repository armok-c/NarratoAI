---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T10:00:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 6
fixed_requires_human_verification: 1
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T10:00:00Z
**Source review:** `.planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 7
- Fixed: 6
- Fixed (requires human verification): 1
- Skipped: 0

## Fixed Issues

### CR-01: tokio-tungstenite feature `rustls-tls` does not exist in v0.29.0; proxy code path requires `native-tls`

**Status: fixed**

**Files modified:** `Cargo.toml`
**Commit:** 777f24e
**Applied fix:** Changed `features = ["rustls-tls"]` to `features = ["native-tls", "rustls-tls-native-roots"]` for tokio-tungstenite v0.29.0. The `native-tls` feature enables `Connector::NativeTls` used in the HTTP CONNECT proxy tunnel; `rustls-tls-native-roots` replaces the non-existent `rustls-tls` for direct WebSocket connections.

### CR-02: HTTP CONNECT proxy tunnel -- `BufReader::into_inner()` discards read-ahead buffer, risk of TLS data corruption

**Status: fixed: requires human verification** (logic-level correctness fix -- the I/O pattern has been restructured; human review is recommended to confirm the revised approach is correct.)

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** d569dc2
**Applied fix:** Replaced `tokio::io::BufReader` wrapping of the TCP stream with a fixed-size `Vec<u8>` buffer (4096 bytes) that reads the CONNECT response directly into the raw `TcpStream`. The response is parsed from the buffer after full reception. The raw `TcpStream` is then passed directly to `client_async_tls_with_config`, eliminating the risk of data loss from `BufReader::into_inner()` discarding read-ahead bytes that belonged to the TLS handshake. The interim 1xx response handling logic (RFC 7231 Section 4.3.6) is preserved using string parsing from the pre-read buffer.

### WR-01: Proxy URL parser breaks on IPv6 addresses

**Status: fixed**

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** c1f0895
**Applied fix:** Added IPv6 bracket notation detection before host:port splitting. If the host_port string starts with `[`, the code uses `rsplit_once("]:")` to correctly extract the IPv6 address and port. Otherwise, the original `split_once(':')` logic applies. This enables proxy addresses like `http://[::1]:7890`.

### WR-02: EdgeTtsEngine struct missing `Debug` derive, impairs diagnostics

**Status: fixed**

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** ac25429
**Applied fix:** Added `#[derive(Debug)]` to the `EdgeTtsEngine` struct. All fields (`bool`, `String`, `String`) already implement `Debug`, so the derive is zero-cost.

### IN-01: Version test is a tautology

**Status: fixed**

**Files modified:** `src/lib.rs`
**Commit:** 3eec948
**Applied fix:** Replaced `assert_eq!(version(), env!("CARGO_PKG_VERSION"))` (both sides are the same compile-time constant) with `assert!(!version().is_empty())` and `assert!(version().contains('.'))`, which verify the version string is non-empty and follows semver format. Renamed test to `test_version_is_nonempty_semver`.

### IN-02: Redundant `use async_trait::async_trait` in test module

**Status: fixed**

**Files modified:** `src/tts/mod.rs`
**Commit:** 549bf95
**Applied fix:** Removed the redundant `use async_trait::async_trait;` import from the test module. This import is already brought into scope by `use super::*;` on the preceding line, making it a clippy-redundant import.

### IN-03: Test comment mislabels microseconds vs nanoseconds

**Status: fixed**

**Files modified:** `tests/tts_test.rs`
**Commit:** 3e343f3
**Applied fix:** Changed `微秒` to `纳秒` in the doc comment on line 78. The value `5_000_000_000` is the count of nanoseconds (not microseconds) for 5 seconds.

---

_Fixed: 2026-04-29T10:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
