---
phase: 12-additional-tts-engines
fixed_at: 2026-05-01T23:30:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 3
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report (Iteration 3)

**Fixed at:** 2026-05-01T23:30:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 3

**Summary:**
- Findings in scope: 3 (Critical + Warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: Azure/Edge-TTS SSML 中 voice_name 和 lang 未转义直接插入 XML 属性

**Files modified:** `src/tts/common.rs`, `src/tts/azure_speech.rs`, `src/tts/edge_tts.rs`
**Commit:** de67c42
**Applied fix:** Added shared `escape_xml_attr()` function to `common.rs` that escapes `&`, `<`, `>`, `"`, `'` for XML attribute values. Applied it to `lang` and `voice_name` parameters in SSML generation in both `azure_speech.rs:277-279` and `edge_tts.rs:93-95`. Text content already had its own escaping, but attribute values were unprotected.

### WR-02: Qwen TTS audio_url 未验证，存在 SSRF 风险

**Files modified:** `src/tts/qwen_tts.rs`
**Commit:** de67c42
**Applied fix:** Added URL scheme validation after extracting `audio_url` from DashScope response. Accepts `https://` (production) and `http://127.0.0.1`/`http://localhost` (testing/loopback). Rejects any other `http://` URL to prevent SSRF via DashScope returning a malicious URL pointing to internal network resources.

### WR-03: IndexTTS2 参考音频路径来自用户输入，缺少基本的文件存在性检查

**Files modified:** `src/tts/indextts2.rs`
**Commit:** de67c42
**Applied fix:** Added `tokio::fs::metadata()` check before `tokio::fs::read()`. Verifies the path exists and is a file. Returns clear error messages: "参考音频路径不是文件" for directories, "无法访问参考音频文件" for missing/inaccessible paths. This provides actionable error messages instead of raw IO errors.

## Verification

- `cargo check`: passes (only pre-existing `get_azure_voices` dead_code warning)
- `cargo test --lib`: 366 passed, 1 failed (pre-existing `ffmpeg::hwaccel::tests::test_detect_encoders_format` unrelated)
- All 46 TTS-related tests pass (azure_speech, edge_tts, qwen_tts, indextts2, common)

---
*Fixed: 2026-05-01T23:30:00Z*
*Fixer: Claude (gsd-code-fixer)*
*Iteration: 3*
