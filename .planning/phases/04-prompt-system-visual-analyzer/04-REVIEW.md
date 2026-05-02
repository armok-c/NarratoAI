---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-02T20:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 04: Code Review Report (Iteration 14)

**Reviewed:** 2026-05-02T20:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

对 Phase 04 (prompt-system-visual-analyzer) 进行第 14 次迭代审查（re-review #13）。上一轮修复的 3 个 WARNING 均已验证正确修复，无回归：

- WR-13-01（render_prompt required 参数校验）：验证通过，manager.rs:57-68 校验逻辑正确，双重防线完整
- WR-13-02（Level 3/4 回退无效文件清理）：验证通过，frame_extractor.rs:347/382 else if 分支正确清理残留
- WR-13-03（cleanup_fast_path_files 扩展名限制）：验证通过，frame_extractor.rs:480 同时检查 starts_with + ends_with(".jpg")

发现 3 个新 WARNING 和 5 个 INFO 级别问题。无 CRITICAL。Prompt 系统（14 文件）审查结果为 clean，所有问题均在 Visual 系统（5 文件）中。

## Build Verification

| Check | Result | Notes |
|-------|--------|-------|
| `cargo check` | PASS | No errors |
| `cargo clippy --lib` (prompt scope) | PASS | 0 warnings in prompt module |
| `cargo clippy -- -D warnings` | FAIL | 9 lints project-wide, 0 in Phase 04 scope |
| `cargo test --lib` | PASS | 274 tests passed, 0 failed, 1 ignored |

## Previous Fix Verification

| ID | Fix Commit | Status | Detail |
|----|-----------|--------|--------|
| WR-13-01 | c05991f | PASS | `render_prompt()` 在合并 defaults 之前正确校验 required=true + default=None 的参数。测试 `test_render_prompt_missing_required_param` 覆盖该场景。 |
| WR-13-02 | 14a3caf | PASS | `extract_single_frame()` Level 3/4 回退路径中无效 PNG/BMP 文件正确清理。 |
| WR-13-03 | 14a3caf | PASS | `cleanup_fast_path_files()` 增加 `.jpg` 扩展名检查，测试验证非帧文件不被误删。 |

## Warnings

### WR-14-01: 快路径不验证帧文件内容完整性，损坏帧可能被误报为成功

**File:** `src/visual/frame_extractor.rs:399-458`（`rename_fast_path_frames`）
**Issue:** 快路径 `extract_frames_fast_path` 完成后调用 `rename_fast_path_frames`，该函数仅检查文件名模式（`fastframe_*.jpg`），不验证帧文件的实际内容完整性。如果 ffmpeg 退出成功但产出了损坏的 JPEG 文件（例如视频流损坏导致 ffmpeg 写入无效 JPEG 头），这些损坏文件会被重命名为 `keyframe_*` 并计入 `count`。

由于 `count > 0`，`extract_frames` 返回 `Ok(count)`，后续 `collect_frame_paths` 会收集这些损坏文件，`analyze_images` 发送给 LLM 时可能产生垃圾结果或编码失败。对比回退路径：`extract_single_frame` 逐帧提取失败不影响其他帧，且快路径完全无有效帧时仍会返回 `Ok(0)` 触发回退，所以最坏情况是部分损坏帧被传递到下游。

**Fix:**

```rust
// 在 rename_fast_path_frames 的循环中，过滤掉过小的文件
for entry in dir_reader {
    let entry = entry.map_err(|e| VisualError::FrameExtraction(format!("读取目录项失败: {}", e)))?;
    let path = entry.path();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name.starts_with("fastframe_") && file_name.ends_with(".jpg") {
        if let Ok(meta) = path.metadata() {
            if meta.len() < 100 {
                let _ = std::fs::remove_file(&path);
                continue;
            }
        }
        entries.push(path);
    }
}
```

### WR-14-02: `BatchResponse` 的 serde 字段名与 prompt 声明的 JSON schema 不一致

**File:** `src/visual/analyzer.rs:28-33`
**Issue:** `BatchResponse` 结构体的主字段名为 `observations`，通过 `#[serde(alias = "frame_observations")]` 接受 LLM 响应。但 `analyze_video_frames` 中的 prompt 明确要求 LLM 返回 `"frame_observations"` 作为键名。代码可读性降低——读者查看 prompt 会期望字段名为 `frame_observations`，但实际 Rust 字段名是 `observations`。如果将来需要序列化 `BatchResponse`，输出会使用 `observations` 而非 prompt 中声明的 `frame_observations`，造成不一致。

**Fix:**

```rust
#[derive(serde::Deserialize)]
struct BatchResponse {
    #[serde(rename = "frame_observations")]
    observations: Vec<FrameObservation>,
    overall_activity_summary: Option<String>,
}
```

### WR-14-03: `extract_frames_fallback` 错误路径丢弃了详细错误信息

**File:** `src/visual/frame_extractor.rs:242-247`
**Issue:** 当回退路径中所有帧提取均失败时，错误消息仅包含错误数量（`errors.len()`），但 `errors: Vec<String>` 向量中的详细错误信息被丢弃。调用者无法得知哪些帧失败以及失败原因。对比同文件中 `analyzer.rs` 的 `VisualError::BatchPartial`（保留了完整错误列表），处理不一致。

**Fix:**

```rust
if extracted_count == 0 && !errors.is_empty() {
    let detail = errors.iter().take(5).cloned().collect::<Vec<_>>().join("; ");
    return Err(VisualError::FrameExtraction(format!(
        "所有帧提取均失败 ({} 个错误): {}",
        errors.len(),
        detail
    )));
}
```

## Info

### IN-14-01: template::render 中 filter_re 对同一字符串迭代 3 次

**File:** `src/prompt/template.rs:118, 132, 141`
**Issue:** 第 118-129 行（校验过滤器引用的变量存在性）、第 132-139 行（校验过滤器名称合法性）、第 141-150 行（实际替换）分别对 `filter_re.captures_iter(&result)` 独立迭代一次，共 3 次遍历。可合并为单次 `replace_all` 调用。当前实现在正确性和可读性上没有问题，属于维护性建议。与 IN-13-02 指出的是同一问题，本轮确认仍未修改。

**Fix:** 可合并为单次 `replace_all` 闭包内同时做校验和替换。

### IN-14-02: validate_output 方法签名接受 &self 但未使用 registry

**File:** `src/prompt/manager.rs:112-118`
**Issue:** `PromptManager::validate_output()` 接收 `&self` 参数，但方法体只调用无状态的 `validators::validate_output(output, format)`，从未访问 `self.registry`。方法可改为关联函数。当前不影响正确性或安全性，仅是 API 设计风格问题。

**Fix:** 改为关联函数或添加 `#[allow(clippy::unused_self)]`。

### IN-14-03: `seconds_to_hhmmssmmm` 标记为 `#[allow(dead_code)]` 但被广泛使用

**File:** `src/visual/frame_extractor.rs:462-463`
**Issue:** 函数标记了 `#[allow(dead_code)]`，但实际在 `rename_fast_path_frames`（第 444 行）、`extract_frames_fallback`（第 217 行）和多个测试中被使用。`dead_code` lint 抑制不再需要。

**Fix:** 移除 `#[allow(dead_code)]`。

### IN-14-04: `parse_frame_number_from_name` 标记为 `#[allow(dead_code)]` 但被测试引用

**File:** `src/visual/frame_extractor.rs:554-555`
**Issue:** 函数标记了 `#[allow(dead_code)]`，有对应测试 `test_parse_frame_number`。在 `cfg(test)` 编译时不会触发 dead_code 警告。

**Fix:** 移除 `#[allow(dead_code)]` 或删除函数和对应测试。

### IN-14-05: `strip_code_fence` 中 `text.trim()` 被冗余调用 3 次

**File:** `src/visual/types.rs:44-49`
**Issue:** `text.trim()` 在函数中被调用了多次，但结果未被复用。`trim()` 虽然廉价，但绑定到变量更清晰。

**Fix:**

```rust
pub(crate) fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim().trim_end_matches("```").trim())
        .unwrap_or(trimmed)
}
```

---
_Reviewed: 2026-05-02T20:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 14_
