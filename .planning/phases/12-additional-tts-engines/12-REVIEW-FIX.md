---
phase: 12-additional-tts-engines
fixed_at: 2026-05-01T00:00:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-05-01T00:00:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (1 critical, 7 warnings)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: `reqwest::Client::new()` returns `Result` -- compilation error

**Files modified:** `src/tts/common.rs`
**Commit:** `90c51ab`
**Applied fix:** Changed `reqwest::Client::new()` to `reqwest::Client::new().expect("Failed to create default HTTP client")` to resolve the `Result<Client, Error>` type mismatch introduced by reqwest 0.12+.

### WR-01: Blocking `std::fs::remove_file` called in async runtime context

**Files modified:** `src/tts/azure_speech.rs`, `src/tts/doubaotts.rs`, `src/tts/indextts2.rs`, `src/tts/qwen_tts.rs`, `src/tts/soulvoice.rs`, `src/tts/tencent_tts.rs`
**Commit:** `594d363`
**Applied fix:** Restructured `synthesize()` in all 6 engines to use async `tokio::fs::remove_file(output_path).await` instead of blocking `std::fs::remove_file(output_path)` inside `.map_err()`. The pattern was changed from `retry_loop(...).await.map_err(|e| { std::fs::remove_file(output_path); e })` to `let result = retry_loop(...).await; if result.is_err() { tokio::fs::remove_file(output_path).await; } result`.

### WR-02: TtsProvider trait contract violated -- `rate` silently ignored by 4 engines

**Files modified:** `src/tts/soulvoice.rs`, `src/tts/doubaotts.rs`
**Commit:** `2632b01`
**Applied fix:** Implemented rate support in two engines whose APIs support it:
- **SoulVoice**: Threaded `rate` through `synthesize_once()` and used it as the `"speed"` JSON field value (was hardcoded to `1.0`).
- **Doubao**: Threaded `rate` through `synthesize_once()` and used it as `speed_ratio` (was hardcoded to `1.0`).
- **IndexTTS2** and **Qwen**: Left as `_rate` (underscore-prefixed to document intentional disuse) since their upstream APIs lack speed control parameters.

### WR-03: TtsProvider trait contract violated -- `pitch` silently ignored by 5 engines

**Files modified:** `src/tts/doubaotts.rs`
**Commit:** `0460226`
**Applied fix:** Implemented pitch support in Doubao (only engine whose API supports per-request pitch):
- Changed `"pitch_ratio": self.config.pitch` to `"pitch_ratio": pitch` to use the caller's pitch parameter instead of engine config default.
- Threaded `pitch` through `synthesize_once()` signature and closure.
- Other engines (IndexTTS2, Qwen, SoulVoice, Tencent) left as `_pitch` due to genuine API limitations.

### WR-04: Response body discarded in HTTP error responses (4 engines)

**Files modified:** `src/tts/azure_speech.rs`, `src/tts/doubaotts.rs`, `src/tts/qwen_tts.rs`, `src/tts/indextts2.rs`
**Commit:** `f5f44cb`
**Applied fix:** Added response body capture in all 4 engines' HTTP error handling paths: `let body = response.text().await.unwrap_or_default();` and included the body text in the error message format `"{}: {}"`, matching the pattern already used by soulvoice and tencent.

### WR-05: Missing empty `voice_name` validation in 5 engines

**Files modified:** `src/tts/doubaotts.rs`, `src/tts/indextts2.rs`, `src/tts/qwen_tts.rs`, `src/tts/soulvoice.rs`, `src/tts/tencent_tts.rs`
**Commit:** `cd029c4`
**Applied fix:** Added `if voice_name.trim().is_empty()` validation at the start of each engine's `synthesize()` method, matching the pattern already used by Azure Speech and Edge TTS. Returns `TTSError::SynthesisFailed("voice_name 不能为空")`.

### WR-06: Tencent TC3 signature uses hardcoded host -- test does not verify production signing path

**Files modified:** `src/tts/tencent_tts.rs`
**Commit:** `cb58fac`
**Applied fix:** Changed the TC3 signing to derive the canonical host dynamically from `api_url` instead of hardcoding `tts.tencentcloudapi.com`. Both the canonical request's `host:` header line and the HTTP `Host` request header now use this dynamically-extracted host, ensuring the signature covers the actual target server. This allows wiremock tests to validate the full signing path.

### WR-07: Edge-TTS `synthesize_with_retry` duplicates `common::retry_loop` logic

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `cb88623`
**Applied fix:** Replaced the duplicated retry logic in `synthesize_with_retry()` with a delegation to `common::retry_loop()`. Added the missing `use crate::tts::common;` import. Cleanup on retry exhaustion handled by `tokio::fs::remove_file`.

---

_Fixed: 2026-05-01T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
