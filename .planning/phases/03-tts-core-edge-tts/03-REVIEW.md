---
phase: 03-tts-core-edge-tts
reviewed: 2026-05-06T17:30:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/lib.rs
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
findings:
  critical: 0
  warning: 0
  info: 1
  total: 1
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**Reviewed:** 2026-05-06T17:30:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

第三次审查（iteration 3），验证上一轮 WR-01 死锁修复后无回归。所有 prior issues 已确认修复：

- **WR-01 (iter 1, word boundary as_u64 → as_f64 fallback):** `edge_tts.rs:444-449` 使用 `as_u64().or_else(|| ... as_f64())` 防御模式。✓
- **WR-02 (iter 1, voice_name_to_lang 空字符串):** `edge_tts.rs:66-71` 返回 `"en-US"` 并输出 `tracing::warn!` 日志。✓
- **IN-01 (iter 1, voice_name 未转义):** `edge_tts.rs:95-96` 使用 `common::escape_xml_attr()` 转义 `&`、`<`、`>`、`"`、`'`。✓
- **WR-01 (iter 2, CONNECT 死锁):** `edge_tts.rs:273-288` 内循环在 `tcp.read()` 之前检查 `\r\n\r\n`，1xx 压缩后不阻塞。✓

唯一剩余发现为 Info 级别的 SOCKS 代理检查覆盖范围不足（沿用），无新 Critical/Warning。

## Info

### IN-01: SOCKS 代理拒绝仅检查 `socks5`，未覆盖 `socks4`/`socks4a`

**File:** `src/tts/edge_tts.rs:174`

**Issue:** 代理协议检查使用精确匹配 `proxy_scheme_lower == "socks5"`，未拒绝 `"socks4"` 或 `"socks4a"`。若用户配置 SOCKS4 代理 URL，代码会落入 HTTP CONNECT 隧道逻辑，产生通用 "连接代理失败" 错误而非描述性的 SOCKS 拒绝消息。SOCKS4 已基本淘汰，实际触发概率极低。

**Suggestion:** 匹配所有 `socks` 前缀：

```rust
if proxy_scheme_lower.starts_with("socks") {
    return Err(TTSError::ConnectionFailed(
        "本引擎不支持 SOCKS 代理，请使用 HTTP/HTTPS 代理".to_string(),
    ));
}
```

## Prior Issues Verification

| ID | Iteration | Description | Status | Evidence |
|---|---|---|---|---|
| CR-01 | 1 | `Message::Text` takes `Utf8Bytes`, not `String` | FIXED | Line 407: `Message::Text(stt_message.into())` |
| WR-01 | 1 | Word boundary offsets silently dropped for float JSON | FIXED | Lines 444-449: `as_u64().or_else(…as_f64()…)` |
| WR-02 | 1 | `voice_name_to_lang` returns empty string | FIXED | Lines 66-71: Returns `"en-US"` with `tracing::warn!` |
| IN-01 | 1 | Voice name not escaped in SSML attribute | FIXED | Lines 95-96: Uses `common::escape_xml_attr()` |
| IN-02 | 1 | SOCKS proxy only rejects `socks5` | UNCHANGED | Line 174: `== "socks5"` exact match. Carried as IN-01. |
| WR-01 | 2 | CONNECT response parsing deadlock (1xx+2xx coalesce) | FIXED | Lines 273-288: `\r\n\r\n` check before `tcp.read()` (commit 2f00d43) |

No regressions introduced by fix commits. All fixes are minimal, targeted changes.

---

_Reviewed: 2026-05-06T17:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
