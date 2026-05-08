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
  - Cargo.toml
  - narratoai-core/src/tts/mod.rs
  - narratoai-core/src/config/types.rs
  - narratoai-core/src/config/defaults.rs
findings:
  critical: 1
  warning: 5
  info: 2
  total: 8
status: issues_found
note: CR-02 标记为 FALSE POSITIVE（TencentSection 无 api_url 字段）
---

# Phase 12: Code Review Report (Pass 2)

**Reviewed:** 2026-05-08T12:00:00Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Reviewed 11 source files implementing 6 additional TTS engines (SoulVoice, Doubao, Qwen, IndexTTS2, Azure Speech, Tencent TTS) plus shared utilities, configuration types, and routing. Previous review passes (12-REVIEW.md, 12-REVIEW-FIX.md) already addressed: Azure voice -Female/-Male suffix handling in `should_use_azure_services()`, Doubao ak/sk error messaging, XML text escaping extraction into `escape_xml_text()`, DoubaoTTS configurable `api_url`, Tencent voice_type parse error (no silent default), and IndexTTS2 default `repetition_penalty` validation test.

This pass identified 2 critical and 5 warning-level issues remaining in the codebase, none of which are duplicates of previously reported findings.

Key concerns: SSRF bypass in Qwen TTS audio URL download, Tencent TTS host header injection from user-controlled URL, multiple engines accepting user-configurable URLs without HTTPS enforcement, and Azure Speech incomplete suffix stripping for compound voice names.

## Critical Issues

### CR-01: SSRF protection in Qwen TTS can be bypassed with private IP addresses over HTTPS

**File:** `narratoai-core/src/tts/qwen_tts.rs:100-112`
**Issue:** The SSRF check on the audio download URL only validates the URL scheme (https://) or localhost addresses. An attacker who can manipulate the DashScope API response (e.g., via a compromised or malicious proxy, DNS rebinding, or a returned URL like `https://10.0.0.1/internal-endpoint`) can cause the engine to make GET requests to arbitrary internal hosts over HTTPS. The code's own comment acknowledges this limitation at line 98 ("does not prevent private IP targets"), but this is a real attack surface for a desktop application that may run on private networks.

Additionally, `http://localhost` is allowed without port restriction. An attacker could return `http://localhost:9200/_cluster/health` to probe internal services running on the host.

**Fix:**
```rust
// Add a private IP / link-local check. Parse the host from the URL
// and reject known private IP patterns:
fn is_ssrf_safe_url(url: &str) -> bool {
    if !url.starts_with("https://") {
        if !url.starts_with("http://127.0.0.1") && !url.starts_with("http://localhost") {
            return false;
        }
    }
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/').next().unwrap_or("")
        .split(':').next().unwrap_or("");

    if host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("172.16.")
        || host.starts_with("172.17.")
        || host.starts_with("172.18.")
        || host.starts_with("172.19.")
        || host.starts_with("172.2")
        || host.starts_with("172.3")
        || host == "0.0.0.0"
        || host == "[::1]"
    {
        return false;
    }
    true
}
```

### ~~CR-02: Tencent TTS host extraction sends secret_id in Authorization header to user-controlled URL~~ — FALSE POSITIVE

**Status:** FALSE POSITIVE — 事实性错误，无需修复
**原因:** 审查时假设 `TencentSection` 含有用户可配置的 `api_url` 字段，但实际代码中 `TencentSection` 只有 `secret_id`、`secret_key`、`region` 三个字段（`config/types.rs:108-115`），无 `api_url`。生产 URL 硬编码为 `const PRODUCTION_URL: &str = "https://tts.tencentcloudapi.com"`（`tencent_tts.rs:247`），`endpoint_override` 为 `pub(super)` 且仅在测试中赋值。不存在用户可控 URL 的攻击面。

## Warnings

### WR-01: SoulVoice and Doubao accept user-configurable api_url without HTTPS enforcement

**File:** `narratoai-core/src/tts/soulvoice.rs:39-43`, `narratoai-core/src/tts/doubaotts.rs:42-46`
**Issue:** Two cloud API engines allow the user to configure `api_url` via TOML config and will use `http://` URLs without warning. This means API keys and tokens are sent in cleartext over the network. For SoulVoice (line 63: `Authorization: Bearer {api_key}`) and Doubao (line 84: `Bearer;{token}`), this exposes credentials to network sniffing.

**Note:** Tencent TTS was originally included in this finding but has been removed — `TencentSection` has no `api_url` field; the production URL is hardcoded.

IndexTTS2 uses HTTP by default (`http://127.0.0.1:8081`) which is acceptable for a local service, so it is excluded from this finding.

**Fix:** Add HTTPS enforcement for cloud APIs at the point of URL resolution:
```rust
// In each engine's synthesize_once, after resolving api_url:
if !api_url.starts_with("https://")
    && !api_url.starts_with("http://127.0.0.1")
    && !api_url.starts_with("http://localhost")
{
    return Err(TTSError::SynthesisFailed(
        "API URL must use HTTPS for cloud services".to_string()
    ));
}
```

### WR-02: Doubao TTS sends appid and token in both Authorization header and JSON body

**File:** `narratoai-core/src/tts/doubaotts.rs:52-56,84`
**Issue:** The `appid` and `token` are sent in the JSON body payload (lines 52-56: `appid: self.config.appid, token: self.config.token`) AND the token is also sent in the Authorization header (line 84: `Bearer;{token}`). This is redundant and increases the attack surface for credential leakage. If the API truly requires both, this is fine, but the Python version (referenced as `voice.py:1149-1176`) should be verified to confirm this is intentional. If the body credentials are sufficient, the header is unnecessary; if the header is the real auth, the body fields leak credentials into server logs that typically log request bodies less securely than headers.

**Fix:** Verify against the Doubao/Volcano Engine TTS API documentation whether both are needed. If only one is required, remove the other to minimize credential exposure.

### WR-03: Azure Speech strips -Female/-Male and -V2 suffixes independently, producing invalid voice names for compound suffixes

**File:** `narratoai-core/src/tts/azure_speech.rs:33-35,262`
**Issue:** In `should_use_azure_services()` at lines 33-35, the voice name has `-Female` and `-Male` stripped before regex matching. Separately, in `synthesize_once()` at line 262, only `-V2` is stripped. If a voice name is `zh-CN-YunzeNeural-Female-V2`, the routing check at line 29 (`name.ends_with("-V2")`) returns true, routing to Azure. Then `synthesize_once` strips `-V2` producing `zh-CN-YunzeNeural-Female`, which is sent to the Azure API. The Azure API may reject the `-Female` suffix. The Python version likely handles this, but the Rust version's independent stripping logic could produce invalid voice names for compound suffixes.

**Fix:** Strip all three suffixes in `synthesize_once()` before building the SSML:
```rust
let processed_voice_name = voice_name
    .trim()
    .trim_end_matches("-V2")
    .trim_end_matches("-Female")
    .trim_end_matches("-Male");
```

### WR-04: Tencent TTS word boundary silently swallows negative timestamps

**File:** `narratoai-core/src/tts/tencent_tts.rs:222-223`
**Issue:** The guard `if start_ms >= 0 { (start_ms as u64) * 10000 } else { 0 }` silently replaces negative timestamps with 0 instead of logging or warning. If the Tencent API returns a subtitle with a negative BeginTime, the resulting word boundary will have `start_offset: 0`, which could produce incorrect subtitle timing without any indication to the caller. This is a data quality issue that would be hard to diagnose in production.

**Fix:** Log a warning for negative timestamps:
```rust
if start_ms < 0 || end_ms < 0 {
    log::warn!(
        "Tencent subtitle has negative timestamp: start={}, end={}, text={}",
        start_ms, end_ms, text_val
    );
}
```

### WR-05: Azure voice regex does not match Multilingual voices with additional capital segments

**File:** `narratoai-core/src/tts/azure_speech.rs:14`
**Issue:** The regex `r"^[a-z]{2}-[A-Z]{2}-\w+Neural$"` uses `\w+` which matches `[a-zA-Z0-9_]+`. This works for current Azure voices. However, the voice list returned by `get_azure_voices()` includes voices like `en-US-ChristopherNeural` (no suffix) and `en-US-AvaMultilingualNeural` (works with `\w+`). The regex is fragile because any future Azure voice with a hyphen in the name portion (e.g., `en-US-Some-NameNeural`) would silently fail and fall back to Edge-TTS without warning. This is a maintenance risk.

**Fix:** Consider loosening the regex to `r"^[a-z]{2}-[A-Z]{2}-.+Neural$"` which accepts any characters between the region code and the `Neural` suffix, providing forward compatibility.

## Info

### IN-01: zh-CN-YunzeNeural used in tests but absent from get_azure_voices() list

**File:** `narratoai-core/src/tts/azure_speech.rs:185-186,396,445,462,481,512`
**Issue:** The hardcoded voice list returned by `get_azure_voices()` includes `zh-CN-YunxiNeural` and `zh-CN-YunyangNeural` but not `zh-CN-YunzeNeural`. Meanwhile, `zh-CN-YunzeNeural` is used extensively in tests and comments throughout the file. While `YunzeNeural` is a valid Azure voice, its absence from the list means `get_azure_voices().contains(&"zh-CN-YunzeNeural")` would return false. The list is only used for UI/debugging, so this is not a correctness issue, but it suggests the list may be outdated.

**Fix:** Add `"zh-CN-YunzeNeural"` to the `get_azure_voices()` list, or update tests to use a voice that is in the list.

### IN-02: Multiple engines initialize reqwest::Client on every synthesize() call

**File:** `narratoai-core/src/tts/mod.rs:89-155`
**Issue:** The TTS router function `synthesize()` creates a new engine instance (and therefore a new `reqwest::Client`) on every call. For example, the SoulVoice branch (lines 98-103) calls `SoulVoiceEngine::new()` which calls `common::build_client()` every time. The `reqwest::Client` is designed to be reused (it maintains an internal connection pool). Creating a new one per call discards the connection pool and incurs unnecessary overhead. All engines except Edge-TTS follow this pattern.

**Fix:** Consider caching engine instances (e.g., via `OnceCell` or a factory map) so the `reqwest::Client` is reused across calls. This is a performance optimization and does not affect correctness.

---

_Reviewed: 2026-05-08T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
