---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T12:00:00Z
depth: standard
files_reviewed: 20
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
  info: 3
  total: 6
status: issues_found
---

# Phase 04: Code Review Report (Iteration 10)

**Reviewed:** 2026-04-30T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

对 Phase 04 (prompt-system-visual-analyzer) 进行第 10 次迭代审查。上次审查（迭代 9）发现的 3 个 WARNING（WR-01: 未校验 quality 范围、WR-02: extract_frames_fast_path 公开可见性、WR-03: analyzed_count 始终为 0）已在 commits b14a6a7、96e788c、7256b6a 中修复。本次审查确认所有历史修复均保持有效，并发现 3 个新 WARNING 和 3 个 INFO 级别问题。

整体代码质量良好：类型系统完整、错误处理规范、测试覆盖充分。新发现的问题集中在临时文件清理、边界验证和 LLM 响应兼容性三个方面。

## Previous Fix Verification

以下验证上次审查（迭代 9）的 3 个 WARNING 修复是否保持有效：

| 旧编号 | 问题 | 修复 Commit | 状态 |
|--------|------|-------------|------|
| WR-01 | 未校验 JPEG quality 范围 | b14a6a7 | 已验证：`frame_extractor.rs:42-46` 添加了 `quality_val < 2 \|\| quality_val > 31` 校验 |
| WR-02 | `extract_frames_fast_path` 公开可见性 | 96e788c | 已验证：`frame_extractor.rs:102` 标记为 `pub(crate)` |
| WR-03 | `BatchPartial` 的 `analyzed_count` 始终为 0 | 7256b6a | 已验证：`analyzer.rs:162` 使用 `raw_results.len() - errors.len()` 计算成功数 |

所有三个历史修复均已验证有效。

## Warnings

### WR-01: Level 3/4 回退路径失败时临时文件未清理

**File:** `src/visual/frame_extractor.rs:316-375`
**Issue:** `extract_single_frame` 的 Level 3 和 Level 4 回退会创建 PNG/BMP 临时文件。当 FFmpeg 生成文件成功但后续 `convert_image_to_jpeg` 失败时（第 340-343 行和 369-373 行），临时 PNG/BMP 文件不会被删除。虽然最终 `extract_frames_fallback` 会报告错误并继续处理下一帧，但这些残留文件会留在 `output_dir` 中。

更关键的场景：当 Level 3 PNG 写入成功但转换失败，然后进入 Level 4 时，Level 3 的 PNG 文件仍然残留。同理 Level 4 BMP 转换失败时 BMP 也残留。此外，Level 1/2 写入的无效 `output_path` 文件在最终所有级别均失败时也不会清理。

**Fix:**

```rust
// Level 3: PNG -> JPEG conversion
// ... (existing code) ...
if level3_ok && file_is_valid(&png_path) {
    match convert_image_to_jpeg(&png_path, output_path, quality) {
        Ok(_) => {
            let _ = std::fs::remove_file(&png_path);
            return Ok(());
        }
        Err(e) => {
            tracing::warn!("PNG 转 JPEG 失败: {}", e);
            let _ = std::fs::remove_file(&png_path); // 清理失败的 PNG
        }
    }
}

// Level 4: BMP -> JPEG conversion
// ... (existing code) ...
if level4_ok && file_is_valid(&bmp_path) {
    match convert_image_to_jpeg(&bmp_path, output_path, quality) {
        Ok(_) => {
            let _ = std::fs::remove_file(&bmp_path);
            return Ok(());
        }
        Err(e) => {
            tracing::warn!("BMP 转 JPEG 失败: {}", e);
            let _ = std::fs::remove_file(&bmp_path); // 清理失败的 BMP
        }
    }
}

// 最终所有级别均失败时清理残留的 output_path
let _ = std::fs::remove_file(output_path);
Err(VisualError::FrameExtraction(...))
```

### WR-02: `validate_narration_script` 按 `\n\n` 分割不验证空段落

**File:** `src/prompt/validators.rs:64-70`
**Issue:** 验证解说文案时按 `"\n\n"` 分割并检查段落数 >= 3。但分割不排除空段落——例如输入以 `"\n\n"` 开头会产生一个空字符串作为第一个段落。虽然长度检查 `>= 50` 会阻止极端情况，但 `"a\n\nb\n\nc"` 这样总长超过 50 字符、每段仅含 1 字符的输入也能通过验证，显然不满足"解说文案"的语义要求。空段落不构成有效内容，应在计数前过滤。

**Fix:**

```rust
let paragraphs: Vec<&str> = trimmed
    .split("\n\n")
    .filter(|p| !p.trim().is_empty())
    .collect();
if paragraphs.len() < 3 {
    return Err(PromptError::Validation(format!(
        "解说文案段落数不足: {} 段（需要 >= 3）",
        paragraphs.len()
    )));
}
```

### WR-03: `BatchResponse` 的 `deny_unknown_fields` 与 LLM 响应兼容性风险

**File:** `src/visual/analyzer.rs:28-34`
**Issue:** `BatchResponse` 结构体使用了 `#[serde(deny_unknown_fields)]`。当 LLM 返回的 JSON 中包含除 `observations`/`frame_observations` 和 `overall_activity_summary` 之外的任何额外字段时，整个批次解析会失败。某些 LLM 可能在响应中额外返回 `frame_number`、`total_frames` 等元数据字段，导致整个批次的解析结果被丢弃。

虽然 `parse_and_retry` 存在回退到单帧 `FrameObservation` 解析的逻辑，但该回退只能处理单帧响应。如果 LLM 返回多帧数组附带额外字段，整个批次将丢失。

`FrameObservation`（`types.rs`）同样使用了 `deny_unknown_fields`，但那是对已知 schema 的严格校验。`BatchResponse` 作为 LLM 输出的外层包装，应更宽松。

**Fix:** 移除 `BatchResponse` 上的 `deny_unknown_fields`：

```rust
#[derive(serde::Deserialize)]
struct BatchResponse {
    #[serde(alias = "frame_observations")]
    observations: Vec<FrameObservation>,
    overall_activity_summary: Option<String>,
}
```

## Info

### IN-01: `template.rs` json 过滤器使用 `expect` 代替错误传播

**File:** `src/prompt/template.rs:54`
**Issue:** `serde_json::to_string(s).expect("serde_json cannot fail serializing &str")` 使用 `expect` 而非 `Result`。虽然对于 `&str` 输入 `serde_json::to_string` 不会失败，但这是一个硬编码的不可恢复假设。代码中已有注释说明，无需修改。

**Fix:** 无需修改。现有注释已足够说明。

### IN-02: `file_is_valid` 存在 TOCTOU 竞态窗口

**File:** `src/visual/frame_extractor.rs:510-512`
**Issue:** `file_is_valid` 先检查 `path.exists()` 再读取 `path.metadata()`，两步之间存在理论上的 TOCTOU（Time-of-check to time-of-use）窗口。在此代码的上下文中（单线程帧提取）不存在实际风险，但可简化为单步操作。

**Fix:**

```rust
fn file_is_valid(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}
```

### IN-03: `interval_seconds` 缺少 `is_infinite()` 检查

**File:** `src/visual/frame_extractor.rs:48`
**Issue:** 验证检查了 `is_nan() || <= 0.0` 但未检查 `is_infinite()`。如果 `interval_seconds` 为 `f64::INFINITY`，会通过验证。快速路径将生成 `fps=1/inf`，FFmpeg 会拒绝，静默回退到逐帧路径。在回退路径中 `(duration / f64::INFINITY).ceil()` 产生 0.0，函数返回 `Ok(0)` 静默无操作。不会崩溃，但行为令人困惑。

**Fix:**

```rust
if interval_seconds.is_nan() || interval_seconds.is_infinite() || interval_seconds <= 0.0 {
    return Err(VisualError::FrameExtraction("帧提取间隔必须为有限正数".into()));
}
```

---

_Reviewed: 2026-04-30T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 10_
