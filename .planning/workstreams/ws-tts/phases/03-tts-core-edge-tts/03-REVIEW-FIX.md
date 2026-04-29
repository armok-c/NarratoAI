---
phase: "03"
fixed_at: 2026-04-29T11:30:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 3
findings_in_scope: 5
fixed: 4
skipped: 1
status: partial
---

# Phase 03: TTS Core + Edge-TTS Engine — Code Review Fix Report (Iteration 3)

**Fixed at:** 2026-04-29T11:30:00Z
**Source review:** 03-REVIEW.md (Iteration 3, standard depth)
**Fix scope:** all (Info included)

**Summary:**
- Findings in scope: 5 (Info)
- Fixed: 4
- Skipped: 1

## Fixed Issues

### IN-02: and_then 应改为 map（闭包始终返回 Some）

**Files modified:** `src/tts/edge_tts.rs:166`
**Commit:** `5d9ee9d`
**Applied fix:** 将 `.and_then(|(h, p)| Some((h, p.split('/').next().unwrap_or("443"))))` 替换为 `.map(|(h, p)| (h, p.split('/').next().unwrap_or("443")))`。闭包始终返回 `Some`，使用 `map` 语义更准确。

### IN-03: 硬编码公开令牌的设计风险

**Files modified:** `src/tts/edge_tts.rs:13-16`
**Commit:** `5d9ee9d`
**Applied fix:** 在 `EDGE_TTS_WSS_URL` 常量上方添加安全文档注释，说明 `TrustedClientToken` 是公开令牌（广泛见于开源工具如 edge-tts Python 库），不是秘密凭据，以及令牌轮换时的影响。不修改代码逻辑，仅添加文档。

### IN-04: .unwrap() 缺少 .expect() 说明

**Files modified:** `src/tts/edge_tts.rs:119,125`
**Commit:** `5d9ee9d`
**Applied fix:** 将两处 `.parse().unwrap()` 替换为 `.parse().expect("...")`：
- Line 119: `.expect("Origin 值是硬编码字面量，解析 HeaderValue 不应失败")`
- Line 125: `.expect("User-Agent 值是硬编码字面量，解析 HeaderValue 不应失败")`

### IN-05: 版本测试绑定硬编码字符串

**Files modified:** `src/lib.rs:17-19`
**Commit:** `dc51713`
**Applied fix:** 将 `test_version_returns_0_1_0` 重命名为 `test_version_matches_cargo_toml`，使用 `env!("CARGO_PKG_VERSION")` 替代硬编码版本号 `"0.1.0"`。`env!` 是编译时从 `Cargo.toml` 注入的值，与 `version()` 函数使用同一来源，两者必然相等。

## Skipped Issues

### IN-01: 多余的 .into() 转换

**位置:** `src/tts/edge_tts.rs:336`
**原因:** `Message::Text` 接受 `Utf8Bytes` 类型，而非 `String`。`.into()` 是必要的类型转换（`String` → `Utf8Bytes`），不可移除。最初错误移除了 `.into()`，导致编译错误 `E0308: mismatched types`，已回退。

---

_Fixed: 2026-04-29T11:30:00Z_
_Fixer: Claude (gsd-code-fixer + manual correction)_
_Iteration: 3_
