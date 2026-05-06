---
phase: 03-tts-core-edge-tts
reviewed: 2026-05-06T14:30:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/lib.rs
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - src/tts/common.rs
  - tests/tts_test.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**Reviewed:** 2026-05-06T14:30:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Re-review of the TTS Core + Edge-TTS engine after previous fixes (WR-01, WR-02, IN-01). All three prior issues confirmed resolved:

- **WR-01 (word boundary as_u64 → as_f64 fallback):** Lines 442-447 use `as_u64().or_else(|| ... as_f64())` pattern. Correct.
- **WR-02 (voice_name_to_lang empty string):** Lines 66-71 return `"en-US"` with `tracing::warn!` log. Correct.
- **IN-01 (voice name not escaped in SSML attribute):** Lines 95-96 now use `common::escape_xml_attr()` which escapes `&`, `<`, `>`, `"`, `'`. Correct.

One new Warning found: the HTTP CONNECT proxy tunnel can deadlock for 30 seconds when a proxy coalesces a 1xx interim response with the final 2xx response in a single TCP segment. The response parsing loop re-enters `tcp.read()` without first checking whether the compacted buffer already contains a complete 2xx response. One Info carried forward (SOCKS proxy check).

## Warnings

### WR-01: CONNECT response parsing deadlock when 1xx + 2xx coalesce in same TCP segment

**File:** `src/tts/edge_tts.rs:270-328`

**Issue:** After compacting leftover bytes from a 1xx interim response (lines 307-318), the outer loop `continue`s back to the inner read loop (line 272). The inner loop immediately calls `tcp.read(&mut response_buf[total_read..])` without first checking whether the compacted data already contains a complete 2xx response (i.e., `\r\n\r\n`). If the proxy sent both 1xx and 2xx in a single TCP segment — which is the exact scenario the 1xx compaction code was written to handle — the `read()` call blocks because no more data is pending. The 30-second `CONNECT_TIMEOUT` (line 133) prevents an infinite hang, but the user experiences an unnecessary 30-second delay and a misleading timeout error instead of immediate success.

The code explicitly handles 1xx interim responses with the comment "handles proxies that send interim 1xx and final 2xx in separate TCP segments", so this path is intended to work correctly. The deadlock is a logic bug in the implementation: the compaction step (copy leftover to buffer front) is correct, but the subsequent re-entry into the inner loop skips the "check if we already have a complete response" step.

**Fix:** After compacting leftover data and before re-entering the inner read loop, check if the buffer already contains `\r\n\r\n`:

```rust
// After 1xx compaction, check if compacted data already has a complete response
if extra_len >= 4
    && response_buf[..extra_len].windows(4).any(|w| w == b"\r\n\r\n")
{
    // Re-parse the compacted data without reading more
    // (move parsing to a helper or restructure the loop)
}
```

Alternatively, restructure the inner/outer loop so that the `\r\n\r\n` check happens before `tcp.read()` on each iteration:

```rust
loop {
    // Check if buffer already has a complete response (e.g., after 1xx compaction)
    if total_read >= 4
        && response_buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n")
    {
        break;
    }
    let n = tcp.read(&mut response_buf[total_read..]).await
        .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;
    if n == 0 {
        return Err(TTSError::ConnectionFailed("代理连接提前关闭".to_string()));
    }
    total_read += n;
    if total_read >= response_buf.len() {
        return Err(TTSError::ConnectionFailed("代理响应头过大".to_string()));
    }
}
```

## Info

### IN-01: SOCKS proxy rejection only checks `socks5`, not `socks4`/`socks4a`

**File:** `src/tts/edge_tts.rs:174`

**Issue:** The proxy scheme check at line 174 uses exact match `proxy_scheme_lower == "socks5"`, which does not reject `"socks4"` or `"socks4a"`. If a user configures a SOCKS4 proxy URL, the code falls through to the HTTP CONNECT tunnel logic, which would fail with a generic "连接代理失败" error instead of the descriptive SOCKS rejection message. SOCKS4 is largely obsolete, so this is unlikely in practice.

**Suggestion:** Match any scheme starting with `"socks"`:

```rust
if proxy_scheme_lower.starts_with("socks") {
    return Err(TTSError::ConnectionFailed(
        "本引擎不支持 SOCKS 代理，请使用 HTTP/HTTPS 代理".to_string(),
    ));
}
```

## Prior Issues Verification

| ID | Description | Status | Evidence |
|---|---|---|---|
| CR-01 | `Message::Text` takes `Utf8Bytes`, not `String` | FIXED | Line 405: `Message::Text(stt_message.into())` — `.into()` converts `String` to `Utf8Bytes`. |
| WR-01 | Word boundary offsets silently dropped for float JSON | FIXED | Lines 442-447: `as_u64().or_else(|| ... as_f64().map(\|f\| f as u64))` defensive fallback. |
| WR-02 | `voice_name_to_lang` returns empty string | FIXED | Lines 66-71: Returns `"en-US"` with `tracing::warn!` log for non-standard names. |
| IN-01 | Voice name not escaped in SSML attribute | FIXED | Lines 95-96: Uses `common::escape_xml_attr()` which escapes `&`, `<`, `>`, `"`, `'`. |
| IN-02 | SOCKS proxy only rejects `socks5` | UNCHANGED | Line 174: Still uses `== "socks5"` exact match. Carried forward as IN-01 in this review. |

No regressions were introduced by the three prior fix commits. All fixes are minimal, targeted changes.

---

_Reviewed: 2026-05-06T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
