---
phase: 12-additional-tts-engines
fixed_at: 2026-05-08T18:00:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW-12.md
iteration: 1
findings_in_scope: 6
fixed: 5
skipped: 1
status: partial
---

# Phase 12: Code Review Fix Report (Pass 2)

**Fixed at:** 2026-05-08T18:00:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW-12.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (1 Critical, 5 Warning)
- Fixed: 5
- Skipped: 1

## Fixed Issues

### CR-01: SSRF protection in Qwen TTS can be bypassed with private IP addresses over HTTPS

**Files modified:** `narratoai-core/src/tts/qwen_tts.rs`
**Commit:** c374913
**Applied fix:** Replaced simplistic scheme-only SSRF check with comprehensive `is_ssrf_safe_url()` function that extracts the host from the URL and rejects private IP ranges (10.x.x.x, 192.168.x.x, 172.16-31.x.x), link-local (169.254.x.x), 0.0.0.0, and IPv6 loopback [::1]. The 172.x range uses numeric parsing of the second octet to correctly match only 172.16-31.

### WR-01: SoulVoice and Doubao accept user-configurable api_url without HTTPS enforcement

**Files modified:** `narratoai-core/src/tts/soulvoice.rs`, `narratoai-core/src/tts/doubaotts.rs`
**Commit:** 3c3280c
**Applied fix:** Added HTTPS enforcement check in both engines' `synthesize_once()` after resolving the API URL. Rejects non-HTTPS URLs except for localhost (127.0.0.1 / localhost), preventing API credentials from being sent over cleartext HTTP.

### WR-03: Azure Speech strips -Female/-Male and -V2 suffixes independently, producing invalid voice names for compound suffixes

**Files modified:** `narratoai-core/src/tts/azure_speech.rs`
**Commit:** 90d690a
**Applied fix:** Changed `synthesize_once()` to strip all three suffixes in sequence (-V2, then -Female, then -Male) before building the SSML. This handles compound voice names like `zh-CN-YunzeNeural-Female-V2` correctly.

### WR-04: Tencent TTS word boundary silently swallows negative timestamps

**Files modified:** `narratoai-core/src/tts/tencent_tts.rs`
**Commit:** 17f740f
**Applied fix:** Added `tracing::warn!` log when Tencent API returns subtitles with negative BeginTime or EndTime values, making timing data quality issues diagnosable in production without changing the clamping behavior.

### WR-05: Azure voice regex does not match Multilingual voices with additional capital segments

**Files modified:** `narratoai-core/src/tts/azure_speech.rs`
**Commit:** 1ca55ba
**Applied fix:** Loosened the Azure voice regex from `\w+` to `.+` between the region code and `Neural` suffix (`r"^[a-z]{2}-[A-Z]{2}-.+Neural$"`), providing forward compatibility for future Azure voices with hyphens or special characters in the name portion.

## Skipped Issues

### WR-02: Doubao TTS sends appid and token in both Authorization header and JSON body

**File:** `narratoai-core/src/tts/doubaotts.rs:52-56,84`
**Reason:** Cannot verify against Doubao/Volcano Engine TTS API docs whether both are required. The Python version (voice.py:1149-1176) sends credentials in both locations, suggesting the API may genuinely require both. Removing either without confirmation could break authentication. Documented as-is for human verification.
**Original issue:** The `appid` and `token` are sent in the JSON body payload AND the token is also sent in the Authorization header. This is redundant and increases the attack surface for credential leakage.

---

_Fixed: 2026-05-08T18:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
