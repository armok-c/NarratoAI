---
phase: 12-additional-tts-engines
reviewed: 2026-05-08T12:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - narratoai-core/src/tts/common.rs
  - narratoai-core/src/tts/soulvoice.rs
  - narratoai-core/src/tts/doubaotts.rs
  - narratoai-core/src/tts/qwen_tts.rs
  - narratoai-core/src/tts/indextts2.rs
  - narratoai-core/src/tts/azure_speech.rs
  - narratoai-core/src/tts/tencent_tts.rs
  - narratoai-core/src/tts/mod.rs
  - Cargo.toml
  - narratoai-core/src/config/types.rs
  - narratoai-core/src/config/defaults.rs
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 12: Code Review Report -- Additional TTS Engines (Re-review)

**Reviewed:** 2026-05-08T12:00:00Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Re-review of Phase 12 TTS engine codebase after recent fix commits (1ca55ba, 17f740f, 90d690a, 3c3280c, c374913). Verified all prior fixes are correctly applied. Found 0 critical issues, 1 warning, and 3 informational items.

**Prior fix verification (all confirmed correct):**
- CR-01 (SSRF protection): `qwen_tts.rs:14-56` `is_ssrf_safe_url()` blocks private IPs (10.x, 192.168.x, 172.16-31.x), link-local (169.254.x), 0.0.0.0, [::1]. Scheme allowlist: https, http://127.0.0.1, http://localhost. Verified via runtime testing that octal, hex, and decimal IP bypasses are all blocked by the prefix-based scheme check.
- WR-01 (HTTPS enforcement): `soulvoice.rs:46-53` and `doubaotts.rs:49-56` both enforce HTTPS for cloud APIs while allowing http://127.0.0.1 and http://localhost for local testing.
- WR-03 (suffix stripping): `azure_speech.rs:261-265` chains `trim_end_matches("-V2").trim_end_matches("-Female").trim_end_matches("-Male")`. Verified via runtime testing that Rust's `trim_end_matches(&str)` correctly treats the string as a suffix pattern, not a char set. Composite suffixes like "YunzeNeural-Female-V2" are correctly reduced to "YunzeNeural".
- WR-04 (negative timestamps): `tencent_tts.rs:222-227` adds `tracing::warn!` for negative subtitle timestamps before clamping to 0.
- WR-05 (regex loosen): `azure_speech.rs:14` uses `.+` instead of `\w+` before "Neural", accepting multilingual voice names like "AvaMultilingualNeural".

## Warnings

### WR-01: Qwen SSRF host extraction does not block localhost on non-standard ports

**File:** `narratoai-core/src/tts/qwen_tts.rs:23-32`
**Issue:** The `is_ssrf_safe_url` function allows `http://127.0.0.1` and `http://localhost` without port restriction (intentional for testing). However, the host extraction logic splits on `:` to strip the port, meaning `http://127.0.0.1:6379/` extracts host `127.0.0.1` and passes the private IP blocklist check. Since the scheme check explicitly allows `http://127.0.0.1`, this is by design for local testing, but it means a compromised DashScope API could redirect to any local service port (e.g., Redis, internal admin panels). This is an accepted trade-off documented in the code comments, but worth noting: the SSRF protection here is defense-in-depth, not a security boundary. The real trust anchor is that DashScope is a legitimate Alibaba Cloud service.

**Fix:** If stricter protection is desired, restrict localhost URLs to specific allowed ports (e.g., 443, 8080) or remove the localhost allowance entirely and only allow HTTPS. Current behavior is acceptable given the threat model.

## Info

### IN-01: DoubaoTTSSection.ak and .sk fields declared but never read by Rust engine

**File:** `narratoai-core/src/config/types.rs:171-174`
**Issue:** `DoubaoTTSSection` contains `ak` and `sk` fields that are never used by the Rust engine. The engine uses `appid`/`token` authentication. These fields exist for Python version compatibility and possible future OAuth flows. The engine correctly returns a clear error message if ak/sk are provided without appid/token (line 34-37).
**Fix:** No change needed. Fields are properly handled.

### IN-02: Azure Speech convert_rate_to_percent saturates on extreme inputs

**File:** `narratoai-core/src/tts/azure_speech.rs:220`
**Issue:** `((rate - 1.0) * 100.0).round() as i32` saturates to `i32::MAX` or `i32::MIN` for extreme rate values (e.g., rate=1e18 produces "+2147483647%"). The `is_finite()` guard catches NaN/Infinity but not large finite values. In practice, rate values come from the UI and are always in the 0.5-2.0 range, so this is not a real-world concern. The Azure API would reject absurd percentage values with a clear error.
**Fix:** Could add `let percent = ((rate - 1.0) * 100.0).round().clamp(-200.0, 200.0) as i32;` for robustness. Not required given the controlled input source.

### IN-03: get_azure_voices() function declared but unreferenced (dead_code warning)

**File:** `narratoai-core/src/tts/azure_speech.rs:46`
**Issue:** `get_azure_voices()` produces a `dead_code` compiler warning. The function provides a hardcoded list of Azure Neural voice names for UI/debug purposes. Retained as a useful reference for future UI voice selection features.
**Fix:** No change needed. Could annotate with `#[allow(dead_code)]` to suppress the warning.

---
_Reviewed: 2026-05-08T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
