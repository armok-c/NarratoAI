---
status: issues_found
phase: 12
phase_name: additional-tts-engines
depth: standard
files_reviewed: 12
reviewer: gsd-code-reviewer
date: 2026-05-01
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
---

# Phase 12: Code Review Report -- Additional TTS Engines (Rust, Post-fix Re-review)

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Re-review of 12 Phase 12 files after all previous review fixes (CR-01, WR-01 through WR-07) have been applied. All previously identified issues have been correctly resolved:

- **CR-01 (Doubao pitch_ratio):** Fixed. synthesize_once now uses _pitch: f64 (ignored) and hardcodes pitch_ratio: 1.0 in the request body.
- **WR-01 (Azure SSML control characters):** Fixed. azure_speech.rs now applies .chars().filter(...) after XML entity escaping, matching the edge_tts.rs pattern.
- **WR-02 (Tencent empty X-TC-Token):** Fixed. The header is no longer sent.
- **WR-03 (build_client silent fallback):** Fixed. build_client now returns Result<reqwest::Client, TTSError>.
- **WR-04 (response body in errors):** Fixed in prior iteration.
- **WR-05 (empty voice_name validation):** Fixed in prior iteration.
- **WR-06 (Tencent hardcoded host):** Fixed. Host dynamically derived from api_url.
- **WR-07 (Edge-TTS duplicate retry):** Fixed. Delegates to common::retry_loop.
- **IN-01 (edge_tts unwrap):** Partially addressed with a SAFETY comment, but still uses unwrap(). Downgraded to informational.

**3 warnings and 3 info-level findings remain.** No critical issues found.

## Previous Findings -- Verification

All 5 findings from the previous review have been correctly fixed. The codebase is in good shape overall. The new findings below are either residual issues from previous rounds or new observations.

## Warnings

### WR-01: build_client returns Result but all callers use .expect() -- panics in production on TLS failure

**Severity:** Warning
**File:** src/tts/common.rs:53 (and callers in all 6 engine new() constructors)
**Summary:** Every engine constructor unwraps build_client with .expect(), which panics if the HTTP client builder fails.
**Details:** The previous fix (WR-03) correctly changed build_client to return Result<reqwest::Client, TTSError>. However, every engine constructor calls it with .expect():

    // soulvoice.rs:24-25
    let client = common::build_client(proxy_config)
        .expect(“构建 SoulVoice HTTP 客户端失败”);

The same pattern appears in all 6 engines: soulvoice.rs:25, doubaotts.rs:26, qwen_tts.rs:27, indextts2.rs:27, azure_speech.rs:207, tencent_tts.rs:35.

If the TLS backend fails to initialize (e.g., missing native-tls certs on a minimal Docker image, or a system-level TLS configuration issue), the application will panic with an expect message rather than returning a graceful error to the caller. Since TtsProvider::synthesize returns Result<TtsOutput, TTSError>, the user would see an unrecoverable crash instead of an actionable error message.

**Fix:** Propagate the Result through the new() constructors and handle the error at the call site in mod.rs::synthesize(). For example:

    // In each engine:
    pub(super) fn new(config: SoulVoiceSection, proxy_config: &common::ProxyConfig) -> Result<Self, TTSError> {
        let client = common::build_client(proxy_config)?;
        Ok(Self { client, config })
    }

    // In mod.rs router:
    let engine = soulvoice::SoulVoiceEngine::new(cfg.soulvoice.clone(), &proxy_config)
        .map_err(|e| TTSError::ConnectionFailed(format!(“引擎初始化失败: {}”, e)))?;

### WR-02: Tencent TTS subtitle timestamp cast from i64 to u64 silently wraps negative values

**Severity:** Warning
**File:** src/tts/tencent_tts.rs:214-216
**Summary:** Negative subtitle timestamps from the Tencent API would silently wrap to very large u64 values.
**Details:** The subtitle parsing code casts i64 millisecond timestamps to u64:

    start_offset: (start_ms as u64) * 10000,
    end_offset: (end_ms as u64) * 10000,

If the Tencent API ever returns a negative BeginTime or EndTime (e.g., due to a server bug, clock skew, or unexpected data), the as u64 cast would produce a wrapping value like 18446744073709551615 instead of the intended negative number. This would corrupt word boundary data used for subtitle generation, potentially producing garbled or missing subtitles downstream.

While the Tencent API should never return negative timestamps in practice, the cast is not defensive.

**Fix:** Add a guard clause or use saturating_abs():

    let start_offset = if start_ms >= 0 { (start_ms as u64) * 10000 } else { 0 };
    let end_offset = if end_ms >= 0 { (end_ms as u64) * 10000 } else { 0 };

### WR-03: Doubao TTS test test_doubaotts_payload_structure asserts pitch_ratio == 1.2 but production code hardcodes 1.0

**Severity:** Warning
**File:** src/tts/doubaotts.rs:300,323
**Summary:** The payload structure test is out of sync with the production code after the CR-01 fix.
**Details:** The test at line 270 builds a local payload JSON object that uses config.pitch (1.2) as pitch_ratio:

    // Test code (line 300):
    "pitch_ratio": config.pitch,
    // ...
    assert_eq!(payload["audio"]["pitch_ratio"], 1.2);

However, the actual production code in synthesize_once (line 56) now hardcodes "pitch_ratio": 1.0 (after the CR-01 fix). The test is testing a standalone JSON construction that no longer matches production behavior. This means the test would still pass even if someone inadvertently changed the production pitch_ratio back to use the pitch parameter incorrectly.

**Fix:** Update the test payload to match production behavior:

    // In test_doubaotts_payload_structure:
    "pitch_ratio": 1.0,  // hardcoded standard, matching production code
    // ...
    assert_eq!(payload["audio"]["pitch_ratio"], 1.0);

## Informational

### IR-01: edge_tts.rs hardcoded HeaderValue parse uses .unwrap() with deliberate SAFETY comment

**Severity:** Info
**File:** src/tts/edge_tts.rs:142-152
**Summary:** The .unwrap() calls on hardcoded ASCII header strings now have a SAFETY comment explaining the rationale.
**Details:** A SAFETY comment was added (line 140-141) explaining that the hardcoded strings are ASCII-only and parse() will never fail. This is a reasonable engineering choice -- the strings are compile-time constants that will never change in a way that would introduce non-ASCII characters.

While .expect("hardcoded ASCII header") would be marginally more informative in a panic trace, the current code is acceptable given the safety argument in the comment. No action required.

### IR-02: parse_edge_tts_binary allocates Vec<u8> for every binary message including large audio chunks

**Severity:** Info
**File:** src/tts/edge_tts.rs:577
**Summary:** Every Edge-TTS binary message (including audio data chunks) allocates a new Vec<u8> via data[sep_pos + 4..].to_vec().
**Details:** The parse_edge_tts_binary function always copies the payload into an owned Vec<u8>:

    let payload = data[sep_pos + 4..].to_vec();

For audio messages (which are the majority of binary messages), the payload is then copied again into audio_data via extend_from_slice(&content.payload). This means each audio chunk is allocated twice. For a typical TTS response generating several hundred kilobytes of audio, this creates unnecessary memory pressure and allocation overhead.

**Fix:** Consider returning a (&str, &[u8]) tuple (borrowing from the input) or using a lifetime-parameterized return type. For audio messages specifically, the header parsing could be separated from payload handling to avoid the intermediate allocation.

### IR-03: extract_azure_lang duplicates voice_name_to_lang logic from edge_tts.rs

**Severity:** Info
**File:** src/tts/azure_speech.rs:322-331 and src/tts/edge_tts.rs:62-72
**Summary:** Both modules independently implement extract language from voice name with slightly different approaches.
**Details:** azure_speech.rs uses splitn(3, '-') to extract the first two components, while edge_tts.rs uses char_indices().filter(|(_, c)| *c == '-').nth(1) to find the byte offset of the second hyphen. Both produce the same result for well-formed voice names like zh-CN-XiaoyiNeural, but the duplicated logic creates a maintenance risk if the voice name format changes.

**Fix:** Consider extracting this into a shared utility function in common.rs:

    pub fn extract_lang_from_voice_name(voice_name: &str) -> String {
        let parts: Vec<&str> = voice_name.splitn(3, '-').collect();
        if parts.len() >= 2 {
            format!("{}-{}", parts[0], parts[1])
        } else {
            "en-US".to_string()
        }
    }

---

_Reviewed: 2026-05-01_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
