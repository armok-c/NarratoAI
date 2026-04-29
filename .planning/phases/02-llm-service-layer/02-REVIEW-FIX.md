---
phase: "02"
fixed_at: "2026-04-29T15:50:00Z"
review_path: ".planning/phases/02-llm-service-layer/02-REVIEW.md"
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T15:50:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### WR-01: 非 JPEG 路径下存在 TOCTOU 竞态条件

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 43bb808
**Applied fix:** 将 `image::open(path)` 替换为 `file.seek(SeekFrom::Start(0))` + `file.read_to_end()` + `image::load_from_memory()`，消除重新按路径打开文件导致的 TOCTOU 窗口。JPEG 直通路径已通过同一文件句柄读取，不受影响。

### WR-02: Vision JSON 回退路径丢失 temperature / max_tokens 参数

**Files modified:** `src/llm/provider.rs`, `src/llm/openai_compatible.rs`, `tests/llm_test.rs`
**Commit:** eb3f9fd
**Applied fix:** 为 `LlmProvider::analyze_images` trait 方法添加 `temperature: Option<f32>` 和 `max_tokens: Option<u32>` 参数；在 `OpenAiCompatibleProvider` 实现中捕获并在主请求构建器中设置；为 `create_vision_chat_with_json_fallback` 添加相同参数，在回退重试构建器中设置；更新测试调用点。

### WR-03: 请求重建失败时使用了不一致的 LLMError 变体

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** fdb86b3
**Applied fix:** 将 `create_vision_chat_with_json_fallback` 中请求 `build()` 失败的错误变体从 `LLMError::Configuration` 改为 `LLMError::APICall`，与 `generate_text_with_json_fallback` 保持一致。

### IN-01: `analyze_images` 中不必要的双重克隆

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** eb3f9fd (与 WR-02 同次提交)
**Applied fix:** 在 WR-02 修复过程中移除了冗余的 `prompt_fb` 和 `system_prompt_fb` 克隆变量，直接使用 `&prompt_owned` 和 `system_prompt_owned.as_deref()`。

### IN-02: `Registry::get()` 上冗余的 `#[must_use]`

**Files modified:** `src/llm/registry.rs`
**Commit:** ccf8eab
**Applied fix:** 移除 `#[must_use]` 标注。`Result` 类型本身已自带 `#[must_use]`。

### IN-03: `tests/llm_test.rs` 中存在未使用的 `Path` 导入

**Files modified:** `tests/llm_test.rs`
**Commit:** 53e7fa8
**Applied fix:** 将 `use std::path::{Path, PathBuf};` 改为 `use std::path::PathBuf;`，移除未使用的 `Path`。

---

_Fixed: 2026-04-29T15:50:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
