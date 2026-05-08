---
phase: 12-additional-tts-engines
fixed_at: 2026-05-08
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW-12.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report (Iteration 1)

**Fixed at:** 2026-05-08
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW-12.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (2 Critical + 6 Warning)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: Azure Speech route unreachable for -Female/-Male voice name suffixes

**Files modified:** `narratoai-core/src/tts/azure_speech.rs`
**Commit:** 4cb6eb2
**Applied fix:** 修改 `should_use_azure_services()` 函数，在正则匹配前用 `trim_end_matches("-Female").trim_end_matches("-Male")` 剥离性别后缀，使默认语音名 `zh-CN-XiaoyiNeural-Female` 能正确路由到 Azure Speech Services。更新测试断言从 `assert!(!...)` 改为 `assert!(...)`，新增 `-Male` 后缀测试。

### CR-02: DoubaoTTS ak/sk config fields never used

**Files modified:** `narratoai-core/src/tts/doubaotts.rs`, `narratoai-core/src/config/types.rs`, `narratoai-core/src/config/defaults.rs`
**Commit:** 7af149c
**Applied fix:** 在 `synthesize_once()` 中，当 `token` 为空但 `ak`/`sk` 非空时返回明确的 `AuthenticationFailed` 错误，告知用户 "ak/sk 认证方式暂未实现，请使用 appid + token 认证"。新增测试 `test_doubaotts_ak_sk_not_supported` 验证此行为。

### WR-01: Azure SSML text escaping inconsistency

**Files modified:** `narratoai-core/src/tts/common.rs`, `narratoai-core/src/tts/azure_speech.rs`
**Commit:** e43ff32
**Applied fix:** 在 `common.rs` 新增 `escape_xml_text()` 函数（仅转义 `&`, `<`, `>`，不转义引号），替代 `azure_speech.rs` 中的内联 `.replace()` 链。函数注释明确说明与 `escape_xml_attr` 的区别。

### WR-02: DoubaoTTS hardcoded API URL

**Files modified:** `narratoai-core/src/tts/doubaotts.rs`, `narratoai-core/src/config/types.rs`, `narratoai-core/src/config/defaults.rs`
**Commit:** 7af149c (与 CR-02 合并提交)
**Applied fix:** 在 `DoubaoTTSSection` 添加 `api_url: String` 字段，默认为空字符串。引擎使用空字符串回退模式：`api_url.is_empty()` 时使用默认 URL，否则使用配置值。与其他引擎（SoulVoice、Qwen、IndexTTS2）模式一致。

### WR-03: IndexTTS2 repetition_penalty default mismatch

**Files modified:** `narratoai-core/src/tts/indextts2.rs`
**Commit:** de0fcdb
**Applied fix:** 新增 `test_default_config` 测试，使用 `IndexTTS2Section::default()` 并断言 `repetition_penalty == 1.5`，确保默认值被测试覆盖。默认值本身保持 1.5 不变（与 Python 版一致）。

### WR-04+WR-06: Tencent voice_type silent default and no range check

**Files modified:** `narratoai-core/src/tts/tencent_tts.rs`
**Commit:** d5bd7e7
**Applied fix:** 将 `voice_type_str.parse().unwrap_or(101001)` 替换为 `.parse().map_err(...)` 返回明确的 `SynthesisFailed` 错误消息。添加正整数范围校验 `voice_type <= 0` 时返回错误。

### WR-05: Qwen TTS SSRF check too permissive

**Files modified:** `narratoai-core/src/tts/qwen_tts.rs`
**Commit:** 44ebf30
**Applied fix:** 在现有 SSRF allowlist 检查之前添加显式拒绝块：阻止 `http://0.0.0.0` 和 `http://[::1]`。添加详细注释说明当前检查的局限性（localhost 无端口限制、https scheme 不检查内部 IP、IPv6 长格式未覆盖）。

## Verification

- `cargo test --lib tts`: 84 passed, 0 failed, 1 ignored
- 所有新增测试通过：
  - `test_default_config` (WR-03)
  - `test_doubaotts_ak_sk_not_supported` (CR-02)
  - `test_should_use_azure_services_female_suffix` (CR-01)
  - `test_should_use_azure_services_male_suffix` (CR-01)

## Not in Scope (Info — skipped)

| ID | Description | Action |
|----|-------------|--------|
| IN-01 | `zh-CN-YunzeNeural` absent from get_azure_voices list | 记录信息，不影响正确性 |
| IN-02 | Azure voice regex does not match hyphens in name | 假设性风险，当前无影响 |
| IN-03 | Multiple engines init HTTP client per call | 优化项，不影响正确性 |

## Additional Fix

- `narratoai-core/src/visual/frame_extractor.rs` (commit 270743b): 修复预存的 non-exhaustive pattern match 编译错误，移除 match guard 使模式穷尽。

---
*Fixed: 2026-05-08*
*Fixer: Claude (gsd-code-fixer)*
*Iteration: 1*
