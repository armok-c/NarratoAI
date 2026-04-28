---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-28T10:00:00Z
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
  warning: 8
  info: 5
  total: 13
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Report

**Reviewed:** 2026-04-28T10:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Fresh review of the TTS core abstraction (TTSError, WordBoundary, TtsOutput, TtsProvider trait, synthesize() router) and the Edge-TTS engine implementation (native WebSocket via tokio-tungstenite, SSML generation, binary message parsing, 3-retry loop). Also reviewed Cargo.toml dependencies, module structure, and integration tests.

Previous review findings (CR-01, WR-01/02/03/04/05 from the prior round) have been verified as fixed in the current codebase. This report covers issues still present after those fixes.

The architecture is sound overall. SSML escaping is correct for XML text content. Binary message parsing handles the Edge TTS protocol correctly including audio, wordboundary, and turn.end events. Word boundary collection and audio accumulation flow are properly sequenced.

Eight warnings remain, primarily around HTTP CONNECT compliance, retry semantics, misleading identifiers/log messages, and missing timeout protection on WebSocket operations. No Critical issues were identified.

---

## Warnings

### WR-01: Proxy CONNECT handler does not support interim 1xx responses

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:163`

**Issue:** The proxy CONNECT handler reads only a single response line and checks `response_line.contains("200")`. Per RFC 7231 Section 4.3.6, HTTP/1.1 proxies MAY send interim 1xx responses (e.g., `HTTP/1.1 100 Continue`) before the final `HTTP/1.1 200 Connection Established`. If a proxy sends an interim response, the code reads the 1xx line, `"200"` is not present, and it returns `Err(TTSError::ConnectionFailed(...))` -- even though the connection would have succeeded on the next line.

This is a real issue: some transparent proxies, load balancers, and TLS-terminating proxies emit interim 1xx responses during CONNECT tunnel setup.

**Fix:** Add a loop that continues reading response lines (skipping 1xx interim responses and their trailing headers) until either a 2xx final response (success) or a non-1xx, non-2xx response (failure) is received:

```rust
// Replace the single read_line + contains("200") check with:
loop {
    response_line.clear();
    buf_reader.read_line(&mut response_line).await
        .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;

    let trimmed = response_line.trim();
    if trimmed.is_empty() {
        return Err(TTSError::ConnectionFailed("代理响应为空".to_string()));
    }

    // Extract HTTP status code
    let status_code: u16 = trimmed
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if (200..300).contains(&status_code) {
        break; // success
    }
    if status_code < 200 || status_code >= 300 {
        // Not an interim 1xx -- permanent failure
        return Err(TTSError::ConnectionFailed(format!(
            "代理 CONNECT 失败: {}",
            trimmed
        )));
    }
    // Interim 1xx: skip its trailing headers, then loop for the next response
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line).await
            .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应头失败: {}", e)))?;
        if line.trim().is_empty() {
            break;
        }
    }
}
```

---

### WR-02: Retry semantics do not match documentation

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:210-233`

**Issue:** The comment documents `3 次重试` (3 retries) and the error message at line 229 says `重试 3 次后仍失败` (failed after 3 retries). However, the loop `for attempt in 1..=max_retries` with `max_retries = 3` yields attempts 1, 2, and 3 -- which is only 2 retries (the first attempt is not a retry). The requirement "D-03: 3 次重试，间隔 1 秒" is not met.

| Attempt | Type |
|---------|------|
| 1       | Initial |
| 2       | Retry 1 |
| 3       | Retry 2 |

Total retries: 2, not 3.

**Fix:** Set `max_retries = 4` to get 1 initial attempt + 3 retries:

```rust
let max_retries = 4;
for attempt in 1..max_retries {
    tracing::info!("Edge-TTS 合成尝试 {}/{}", attempt, max_retries - 1);
    // ...
}
// Error message remains: "Edge-TTS 重试 3 次后仍失败: {}"
```

Alternatively, if 2 retries is the intended behavior, update the comment to `2 次重试` and the error message to `重试 2 次后仍失败`.

---

### WR-03: `convert_pitch_to_percent` function name is misleading

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:29`

**Issue:** The function is named `convert_pitch_to_percent` but outputs `+XHz` / `-XHz` format (hertz), not percent. The adjacent function `convert_rate_to_percent` correctly outputs `+X%` / `-X%`, making the inconsistency confusing for maintainers.

**Line 28 comment** correctly says "Hz format" but the function name contradicts it.

**Fix:** Rename to `convert_pitch_to_hz`:

```rust
fn convert_pitch_to_hz(pitch: f64) -> String {
    // body unchanged
}
// Update call at line 44:
let pitch_str = convert_pitch_to_hz(pitch);
```

---

### WR-04: Warning log misleading when turn.end received with missing audio_duration

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:340-345`

**Issue:** The warning message at line 341 says `Edge-TTS connection closed before turn.end` whenever `duration == 0.0`. But this also fires when `turn.end` WAS received and parsed, but the `audio_duration` field was missing, `null`, or in an unexpected format (lines 309-314 fall through to `else { 0.0 }`). The log message is actively misleading -- it blames a missing `turn.end` event when the actual cause is a missing or unparseable duration field within a successfully received `turn.end`.

**Fix:** Track whether `turn.end` was received separately from the duration value:

```rust
let mut received_turn_end = false;

// In the turn.end handler (around line 298):
} else if content.path == "turn.end" {
    received_turn_end = true;
    // ... existing duration parsing ...
}

// In the post-loop check (around line 340):
if duration == 0.0 {
    if received_turn_end {
        tracing::warn!(
            "Edge-TTS turn.end received but audio_duration missing/unparseable, \
             audio data size: {} bytes",
            audio_data.len()
        );
    } else {
        tracing::warn!(
            "Edge-TTS connection closed before turn.end; duration is 0.0, \
             audio data size: {} bytes",
            audio_data.len()
        );
    }
}
```

---

### WR-05: No timeout on WebSocket message receive

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:263`

**Issue:** The `ws_stream.next().await` call at line 263 can block indefinitely if the Edge TTS server stalls, the network connection becomes unresponsive but does not close (half-open TCP state), or the server encounters an internal deadlock. There is no configurable timeout on any network operation within `synthesize_once`. A hung connection consumes the task/thread without recovery.

This is especially risky because `synthesize_with_retry` calls `synthesize_once` up to 3 times -- a hung first attempt delays the entire retry sequence until an external watchdog (if any) intervenes.

**Fix:** Wrap the receive loop with `tokio::time::timeout`. Use a generous but finite timeout:

```rust
use tokio::time::{timeout, Duration};

const WS_RECEIVE_TIMEOUT: Duration = Duration::from_secs(120);

while let Some(msg_result) = timeout(WS_RECEIVE_TIMEOUT, ws_stream.next()).await
    .map_err(|_| TTSError::SynthesisFailed("接收消息超时: 服务器无响应".to_string()))?
{
    // ... existing message handling ...
}
```

---

### WR-06: Proxy URL with authentication credentials is not supported

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:120-122`

**Issue:** The proxy URL parsing splits on the first colon to separate host and port. If the proxy URL contains authentication credentials in standard format `http://user:pass@proxy.example.com:8080`, the first colon belongs to `user:pass`, not the host:port separator. The result is:

- `proxy_host` = `"user"`
- `proxy_port_str` = `"pass@proxy.example.com:8080"`
- `"pass@proxy.example.com:8080".parse::<u16>()` fails
- Error returned: `TTS 连接失败: 代理端口格式错误`

This error message does not hint at the actual problem (unsupported authentication format), making debugging difficult.

**Fix:** Use a proper URL parser (the `url` crate is already a transitive dependency via `reqwest`) or manually strip credentials before parsing:

```rust
fn pretty_proxy_host_port(proxy_url: &str) -> Result<(&str, u16), TTSError> {
    let without_scheme = proxy_url
        .strip_prefix("http://")
        .or_else(|| proxy_url.strip_prefix("https://"))
        .unwrap_or(proxy_url);
    // Strip any user:pass@ prefix
    let host_part = without_scheme
        .rsplit('@')
        .next()
        .unwrap_or(without_scheme);
    // Now parse host:port from host_part
    match host_part.split_once(':') {
        Some((host, port_str)) => {
            let port: u16 = port_str.parse()
                .map_err(|_| TTSError::ConnectionFailed("代理端口格式错误".to_string()))?;
            Ok((host, port))
        }
        None => {
            let default_port = if proxy_url.starts_with("https://") { 443 } else { 80 };
            Ok((host_part, default_port))
        }
    }
}
```

---

### WR-07: `xml:lang="zh-CN"` hardcoded in SSML template

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:50-53`

**Issue:** The SSML template hardcodes `xml:lang="zh-CN"`. When using non-Chinese voices (e.g., `en-US-JennyNeural`, `ja-JP-NanamiNeural`), the language attribute does not match the voice. While the Edge TTS server primarily uses the voice name to determine language synthesis, an incorrect `xml:lang` can cause:
- Suboptimal prosody and pronunciation for non-Chinese text
- Potential rejection by some TTS-compatible engines
- Confusion when the SSML is logged or inspected for debugging

**Fix:** Derive the language tag from the voice name, or add it as a parameter. Voice names follow the pattern `{lang}-{region}-{Name}{Attributes}`:

```rust
fn voice_name_to_lang(voice_name: &str) -> &str {
    // "zh-CN-XiaoyiNeural" -> "zh-CN"
    // "en-US-JennyNeural"   -> "en-US"
    let parts: Vec<&str> = voice_name.splitn(3, '-').collect();
    if parts.len() >= 2 {
        // Rejoin first two parts: "zh-CN"
        // Actually voice names have format "zh-CN-XiaoyiNeural" where
        // lang = "zh-CN" (first two hyphen-separated tokens)
        let lang_parts: Vec<&str> = voice_name.splitn(3, '-').collect();
        // But "zh-CN" has - in it; the first split gives "zh" and "CN" and "XiaoyiNeural"
        // So lang = "zh-CN" = parts[0] + "-" + parts[1]
        return &voice_name[..parts[0].len() + 1 + parts[1].len()];
    }
    "zh-CN" // fallback
}
```

Or simpler: use the first two characters before the second hyphen as the language prefix:
```rust
fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some(idx) = voice_name.chars()
        .enumerate()
        .filter(|(_, c)| *c == '-')
        .nth(1)
        .map(|(i, _)| i)
    {
        voice_name[..idx].to_string()
    } else {
        "zh-CN".to_string()
    }
}
```

---

### WR-08: Word boundary offset addition can overflow `u64`

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:288`

**Issue:** Line 288 computes `end_offset: offset + duration_100ns`. Both `offset` and `duration_100ns` are `u64` values parsed from an untrusted server response. If their sum exceeds `u64::MAX` (theoretically possible with corrupted or extreme server data representing ~584,000 years of audio), the addition:

- **Panics** in debug builds (Rust enforces overflow checking)
- **Wraps silently** in release builds, producing `end_offset < start_offset`, which violates the type's implied invariant and could cause downstream bugs in subtitle generation or audio trimming logic

**Fix:** Use `saturating_add` for defense-in-depth without changing the control flow:

```rust
word_boundaries.push(WordBoundary {
    start_offset: offset,
    end_offset: offset.saturating_add(duration_100ns),
    text: text.to_string(),
});
```

Or for strict validation with error propagation (preferred for security-critical contexts):

```rust
let end_offset = offset.checked_add(duration_100ns).ok_or_else(|| {
    TTSError::SynthesisFailed("word boundary offset overflow".to_string())
})?;
word_boundaries.push(WordBoundary {
    start_offset: offset,
    end_offset,
    text: text.to_string(),
});
```

---

## Info

### IN-01: Redundant `.into()` on `Message::Text`

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:255`

**Issue:** `stt_message` is already `String`. `Message::Text` takes `String`. `stt_message.into()` produces `String` from `String` -- a no-op.

**Fix:** Remove `.into()`:
```rust
ws_stream.send(Message::Text(stt_message)).await
```

---

### IN-02: `and_then(|(h, p)| Some(...))` should be `map`

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:132`

**Issue:** `Option::and_then` with a closure that returns `Some(...)` is semantically identical to `Option::map`. Using `map` is clearer about the intent (transforming the inner value, not chaining to another Option).

**Fix:**
```rust
let (target_host, target_port_str) = target_addr
    .split_once(':')
    .map(|(h, p)| (h, p.split('/').next().unwrap_or("443")))
    .unwrap_or((target_host_only, "443"));
```

---

### IN-03: Hardcoded `TrustedClientToken` in WebSocket URL

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:11`

**Issue:** The WebSocket URL contains a hardcoded `TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4`. While this is a public token used by the browser extension (same across all `edge-tts` implementations), hardcoding it:
- Ties the code to a specific client identity that Microsoft could deprecate or rotate
- Provides no configuration path to update the token without a code change
- Makes the URL non-configurable even if Microsoft changes the endpoint

**Suggestion:** Extract the base URL and token into named constants, or better, make them configurable via environment variables or config.toml.

---

### IN-04: `unwrap()` on hardcoded header value parse

**File:** `E:\GitLib\NarratoAI\src\tts\edge_tts.rs:92,98`

**Issue:** Two `"string".parse().unwrap()` calls convert hardcoded string literals to HTTP header values. While these specific strings are guaranteed valid (they are hardcoded literals), using `unwrap()` without `expect()` provides no documentation of why the unwrap is safe, making it look like a lazy shortcut.

**Fix:** Replace `.unwrap()` with `.expect("hardcoded header value is valid")`:
```rust
"chrome-extension://jdiccldimpdaibmpcddlniojbpldgahh"
    .parse()
    .expect("hardcoded Origin header is valid"),
```

---

### IN-05: Version test tied to hardcoded version string

**File:** `E:\GitLib\NarratoAI\src\lib.rs:17-19`

**Issue:** The test `test_version_returns_0_1_0` asserts `version() == "0.1.0"` by literal string. This will break on any version bump, requiring a coordinated change in two places (`Cargo.toml` and `lib.rs`). The same value is available at compile time via `env!("CARGO_PKG_VERSION")`.

**Fix:** Compare against the compile-time constant:
```rust
#[test]
fn test_version_matches_cargo_toml() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
```

---

## Scope Notes

- **No Critical issues** found. The codebase is well-structured with proper error types, async trait patterns, and defensive parsing logic.
- **Cargo dependencies** are properly specified. `native-tls` 0.2.18 is used consistently by both the explicit dependency and the `tokio-tungstenite` dependency chain (verified via Cargo.lock lines 1608-1622 and 2890-2901). No version conflict.
- **Previous round fixes verified in current code:** CR-01 (duration division), WR-01 target path stripping, WR-02 word boundary logging, WR-03 dead test replacement, WR-04 proxy port default, WR-05 duration-zero warning. All are confirmed present.
- **Test coverage** is adequate for Phase 3 but the `parse_edge_tts_binary` function has no unit tests for edge cases (empty data, malformed header, missing Path field). Consider adding these.
- **Tests and test infrastructure** (mock engine, router tests, error display tests) are correct and reliable. No issues.

---

_Reviewed: 2026-04-28T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
