---
phase: 12-additional-tts-engines
reviewed: 2026-05-01T00:00:00Z
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
  critical: 1
  warning: 7
  info: 3
  total: 11
status: issues_found
---

# Phase 12: Code Review Report — Additional TTS Engines (Rust, Iteration 2)

**Reviewed:** 2026-05-01T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Reviewed 12 files implementing 6 TTS engine backends in Rust. The previous review (2026-04-30) identified 9 warnings and 3 info items, all of which were confirmed fixed in the current codebase (WR-01 through WR-09 applied).

However, this second pass identified **11 additional findings** not caught in the initial review:

1. **1 CRITICAL (blocker):** `reqwest::Client::new()` returns `Result<Client, Error>` in reqwest 0.13.3, but `common.rs:59` treats it as returning `Client` directly -- the code will not compile.
2. **6 warnings:** Blocking I/O calls in async context, `rate`/`pitch` contract violations in multiple engines, missing API response body in error messages, missing `voice_name` validation across five engines, test validity gap in Tencent TC3 signing, and duplicate retry-logic in edge_tts.
3. **3 info items:** Zero-duration placeholder in 4 engines, hardcoded sample rate, and Cargo.lock review scope.

The compilation error in `common.rs` must be resolved before any other finding -- the project cannot build in its current state.

## Critical Issues

### CR-01: `reqwest::Client::new()` returns `Result` -- compilation error

**File:** `src/tts/common.rs:59`

**Issue:**
In reqwest 0.12+, `Client::new()` was changed to return `Result<Client, reqwest::Error>` instead of `Client` (breaking change). The Cargo.lock resolves to reqwest 0.13.3. However, `build_client()` at line 59 calls `reqwest::Client::new()` inside an `unwrap_or_else` closure that must return `Client`:

```rust
builder.build().unwrap_or_else(|e| {
    tracing::warn!("Failed to build HTTP client with proxy config: {}. Falling back to default.", e);
    reqwest::Client::new()   // Returns Result<Client, Error>, expected Client
})
```

The closure's return type is `Result<Client, reqwest::Error>` but `unwrap_or_else` expects `Client`. This is a type mismatch -- the code **will not compile**.

**Fix:**
```rust
builder.build().unwrap_or_else(|e| {
    tracing::warn!("Failed to build HTTP client with proxy config: {}. Falling back to default.", e);
    reqwest::Client::new().expect("Failed to create default HTTP client")
})
```

Or alternatively, to avoid a panic:
```rust
builder.build().unwrap_or_else(|e| {
    tracing::warn!("Failed to build HTTP client with proxy config: {}. Falling back to default.", e);
    reqwest::Client::builder().build().unwrap_or_else(|e| {
        panic!("Failed to create HTTP client (with and without proxy): {}", e)
    })
})
```

Note: This finding was NOT present in the previous review. The previous WR-05 only addressed the silent-fallback issue (missing `tracing::warn!`), not the type-mismatch compilation error on the fallback path itself.

---

## Warnings

### WR-01: Blocking `std::fs::remove_file` called in async runtime context

**Files:**
- `src/tts/azure_speech.rs:342`
- `src/tts/doubaotts.rs:131`
- `src/tts/indextts2.rs:122`
- `src/tts/qwen_tts.rs:134`
- `src/tts/soulvoice.rs:107`
- `src/tts/tencent_tts.rs:249`

**Issue:**
All 6 new engine `synthesize()` methods call `std::fs::remove_file(output_path)` inside a `.map_err()` closure that executes on the tokio async runtime after `.await`:

```rust
common::retry_loop(|| self.synthesize_once(...)).await.map_err(|e| {
    let _ = std::fs::remove_file(output_path);  // Blocking call on async runtime
    e
})
```

`std::fs::remove_file` is a blocking system call. When called on a tokio worker thread, it blocks that thread from processing other async tasks. By contrast, the `edge_tts` engine (line 402 of `edge_tts.rs`) correctly uses the async equivalent:

```rust
let _ = tokio::fs::remove_file(output_path).await;
```

While the file deletion is fast in most cases, on network filesystems or under high I/O load this can stall the tokio runtime.

**Fix:** Change all 6 occurrences from `std::fs::remove_file(output_path)` to `tokio::fs::remove_file(output_path).await`:

```rust
common::retry_loop(|| self.synthesize_once(...)).await.map_err(|e| {
    let _ = tokio::fs::remove_file(output_path);
    e
})
```

Note: Since the cleanup runs synchronously inside `.map_err` (not inside a future), the correct approach would be to move the cleanup before `.map_err`:

```rust
let result = common::retry_loop(|| self.synthesize_once(...)).await;
if result.is_err() {
    let _ = tokio::fs::remove_file(output_path).await;
}
result
```

---

### WR-02: TtsProvider trait contract violated -- `rate` silently ignored by 4 engines

**Files:**
- `src/tts/doubaotts.rs:122` -- `_rate` ignored, speed_ratio hardcoded to 1.0 (line 37)
- `src/tts/indextts2.rs:107` -- `_rate` ignored, no speed parameter in API call
- `src/tts/qwen_tts.rs:126` -- `_rate` ignored, Qwen API has no rate parameter
- `src/tts/soulvoice.rs:98` -- `_rate` ignored, speed hardcoded to 1.0 (line 58)

**Issue:**
The `TtsProvider` trait defines `rate: f64` as the speed control parameter (1.0 = standard). However, 4 of 6 new engines discard this parameter without using it. The parameter is underscore-prefixed (documenting the intentional disuse), but this violates the caller's expectation that setting rate will affect speech speed.

The impact: if the application calls `synthesize()` with `rate: 0.8` expecting slower speech, Doubao, IndexTTS2, Qwen, and SoulVoice will all produce output at normal speed (1.0) without warning.

Only `azure_speech`, `tencent_tts`, and `edge_tts` correctly use the rate parameter.

**Fix:** Either implement rate support in each engine (e.g., passing as speed parameter if the API supports it), or document in the `TtsProvider` trait docstring that rate/pitch support is engine-dependent. For engines whose upstream API genuinely lacks speed control (e.g., Qwen), the trait documentation should note the limitation.

---

### WR-03: TtsProvider trait contract violated -- `pitch` silently ignored by 5 engines

**Files:**
- `src/tts/doubaotts.rs:123` -- `_pitch` ignored (uses config pitch_ratio instead)
- `src/tts/indextts2.rs:108` -- `_pitch` ignored
- `src/tts/qwen_tts.rs:127` -- `_pitch` ignored
- `src/tts/soulvoice.rs:99` -- `_pitch` ignored
- `src/tts/tencent_tts.rs:242` -- `_pitch` ignored

**Issue:**
Same pattern as WR-02, but for pitch. Only `azure_speech` and `edge_tts` use the pitch parameter. The remaining 5 engines silently discard caller-supplied pitch values, returning default-pitch audio.

Note: This affects one more engine than rate (WR-02) because `tencent_tts` uses rate but not pitch.

**Fix:** Same as WR-02 -- implement pitch support or document the limitation in the trait. For Tencent Cloud TTS specifically, the API supports a `VoiceSpeed` parameter but has no per-request pitch control, so this is a genuine API limitation that should be documented.

---

### WR-04: Response body discarded in HTTP error responses (4 engines)

**Files:**
- `src/tts/azure_speech.rs:291-295` -- only status code, no body
- `src/tts/doubaotts.rs:84-86` -- only status code, no body
- `src/tts/qwen_tts.rs:76-79` -- only status code, no body
- `src/tts/indextts2.rs:81-85` -- only status code, no body

**Issue:**
When the upstream API returns a non-success HTTP status, 4 of 6 new engines only include the status code in the error message:

```rust
if !status.is_success() {
    return Err(TTSError::SynthesisFailed(format!(
        "Azure TTS 返回 {} (期望 200)", status.as_u16()
    )));
}
```

The response body typically contains critical debugging information (e.g., "Access denied", "Invalid subscription key", "Quota exceeded for current month", "Region not supported"). Without the body text, users and developers cannot diagnose API configuration issues.

By contrast, `soulvoice.rs:73-76` and `tencent_tts.rs:169-172` correctly read and include the body text:

```rust
// soulvoice.rs (correct pattern)
let body_text = response.text().await.unwrap_or_default();
return Err(TTSError::SynthesisFailed(format!(
    "SoulVoice API 返回 {}: {}", status.as_u16(), body_text
)));
```

**Fix:** Replace the status-code-only error with body-inclusive error handling in all 4 engines:
```rust
if !status.is_success() {
    let body = response.text().await.unwrap_or_default();
    return Err(TTSError::SynthesisFailed(format!(
        "Azure TTS 返回 {}: {}", status.as_u16(), body
    )));
}
```

---

### WR-05: Missing empty `voice_name` validation in 5 engines

**Files:**
- `src/tts/doubaotts.rs:120-134` -- `synthesize()` only checks empty text, not voice_name
- `src/tts/indextts2.rs:102-126` -- same
- `src/tts/qwen_tts.rs:121-138` -- same
- `src/tts/soulvoice.rs:94-111` -- same
- `src/tts/tencent_tts.rs:236-253` -- same

**Issue:**
Only `azure_speech.rs:338-340` and `edge_tts.rs:626` validate that `voice_name` is non-empty before proceeding with TTS synthesis. The other 5 engines pass an empty (or whitespace-only) voice name string to upstream APIs, which will respond with protocol-level errors rather than clear validation errors.

Example: If `voice_name` is `""`:
- `soulvoice::synthesize_once` sends `"voice": ""` in JSON body -- API returns confusing error
- `tencent_tts::synthesize_at_url` parses `""` as voice_type via `unwrap_or(101001)` -- silently uses wrong voice
- `indextts2::synthesize_once` tries to construct path from `""` -- confusing file-not-found error

**Fix:** Add validation at the start of each engine's `synthesize()` method:
```rust
if voice_name.trim().is_empty() {
    return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
}
```

---

### WR-06: Tencent TC3 signature uses hardcoded host -- test does not verify production signing path

**File:** `src/tts/tencent_tts.rs:93`

**Issue:**
The TC3-HMAC-SHA256 signature canonical request hardcodes `host:tts.tencentcloudapi.com` at line 92:
```rust
let canonical_headers = format!(
    "content-type:application/json; charset=utf-8\nhost:tts.tencentcloudapi.com\n"
);
```

When `endpoint_override` is set (for wiremock tests, line 301), the request is sent to a different URL than what the signature covers. The wiremock test passes because it only checks `method("POST")` and ignores the `Authorization` header. This means the test does NOT verify that the production signing path would work -- if the production host changes, or if there's a signing bug, the test would still pass.

Additionally, the canonical `Host` header line (92-93) and the actual HTTP `Host` header (line 155) must match. If `endpoint_override` disagrees with the hardcoded host, the actual request has a different Host header than what was signed, which would cause a 401 from a real Tencent Cloud server. The test never catches this mismatch.

This is a **test validity** concern, not a production bug (production uses the correct host). However, it means the TC3 signing implementation has zero automated coverage.

**Fix:** Either:
- (a) Make the canonical host configurable alongside `endpoint_override` so the test validates the full signing path with a correct signature + host match; or
- (b) Add a dedicated unit test that verifies the signing computation against a known test vector (Tencent Cloud publishes TC3 signing examples in their SDK documentation).

---

### WR-07: Edge-TTS `synthesize_with_retry` duplicates `common::retry_loop` logic

**File:** `src/tts/edge_tts.rs:375-408`

**Issue:**
The `synthesize_with_retry` method in `edge_tts.rs` implements the exact same retry pattern as `common::retry_loop`: 4 total attempts (1 initial + 3 retries), immediate bail on `AuthenticationFailed`, 1-second delay between retries, and a `RetryExhausted` error on exhaustion.

This duplication means any change to retry behavior (e.g., exponential backoff, configurable retry count, jitter) must be applied in two separate places. The edge_tts version was written before `common::retry_loop` existed, and was never refactored to use it.

The edge_tts version differs from `common::retry_loop` in one beneficial way: it correctly uses `tokio::fs::remove_file` (async) for cleanup at line 402, while `common::retry_loop` delegates cleanup to callers who (as noted in WR-01) use blocking `std::fs::remove_file`.

**Fix:** Refactor `EdgeTtsEngine::synthesize` to use `common::retry_loop` instead of its own `synthesize_with_retry`. Since edge_tts `synthesize_once` takes different parameters (pre-built SSML string vs raw text/voice_name/rate/pitch), either:
- (a) Add a wrapper closure that matches the `retry_loop` signature; or
- (b) Keep the `synthesize_with_retry` but have it delegate to `retry_loop` for the retry logic, extracting only the sleep/schedule decisions.

---

## Info

### IN-01: `duration: 0.0` returned as placeholder by 4 engines

**Files:**
- `src/tts/azure_speech.rs:307`
- `src/tts/doubaotts.rs:112`
- `src/tts/indextts2.rs:96`
- `src/tts/soulvoice.rs:88`

**Issue:**
Four engines return `duration: 0.0` in `TtsOutput` because their respective REST APIs do not provide audio duration in the response. By contrast, `qwen_tts` reads duration from the API response (`result["output"]["audio"]["duration"]`), and `tencent_tts` could potentially compute it from subtitle timestamps.

Downstream code that consumes `duration` (e.g., for progress tracking, segment timing, or concatenation) will get `0.0` from these engines. This is documented in comments (e.g., `// REST API 不返回词边界`) but client code may not handle `0.0` as "unknown" rather than "zero-length."

The `TencentTtsEngine` at line 220 also returns `0.0`, even though it has subtitle data (`word_boundaries`) from which the total duration could be derived as `max(end_offset) / 10000` ms.

**Suggestion:** Consider computing duration from the last word boundary's `end_offset` when available (Tencent engine), and document in `TtsOutput::duration` docstring that `0.0` means "not provided by API."

---

### IN-02: Doubao TTS hardcodes audio sample rate at 24000 Hz

**File:** `src/tts/doubaotts.rs:52`

**Issue:**
The audio sample rate is hardcoded to 24000 Hz:
```rust
"rate": 24000,
```

This is consistent with the Python version behavior. However, unlike `azure_speech` or `tencent_tts` which allow the caller to determine output format, Doubao always uses 24kHz mp3. If downstream processing expects a different sample rate (e.g., 44100 Hz to match other engines), resampling would be required. This is a known limitation tracked in the Python codebase.

---

### IN-03: `Cargo.lock` included in review scope

**File:** `Cargo.lock`

**Issue:**
`Cargo.lock` is an auto-generated file (resolved dependency tree). While it is a first-class artifact in Rust projects that must be committed, its inclusion in the code review file list adds noise without providing reviewable value. Changes to `Cargo.lock` are meaningful only as side effects of `Cargo.toml` changes or `cargo update` commands -- the file itself should not be manually reviewed for bugs.

**Suggestion:** Exclude `Cargo.lock` from future review scopes unless the explicit goal is to audit dependency versions for known CVEs.

---

_Reviewed: 2026-05-01T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
