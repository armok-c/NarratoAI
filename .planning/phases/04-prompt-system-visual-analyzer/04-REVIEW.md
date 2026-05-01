---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-02T00:00:00Z
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
  info: 4
  total: 7
status: issues_found
---

# Phase 04: Code Review Report (Iteration 13)

**Reviewed:** 2026-05-02T00:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

对 Phase 04 (prompt-system-visual-analyzer) 进行第 13 次迭代审查（re-review #12）。上一轮修复的 3 个 WARNING（WR-12-01: FrameObservation deny_unknown_fields 移除、WR-12-02: truncate 测试字节切片改字符级别、WR-12-03: 7 个 clippy lint）均已验证正确修复，无回归。`cargo clippy -- -D warnings` 确认 Phase 04 范围内 0 个 clippy 警告（全项目 9 个警告均在 Phase 04 之外的 config/watcher.rs、tts/edge_tts.rs、llm/、jianying/ 文件中）。273 个测试全部通过。

发现 3 个新 WARNING 和 4 个 INFO 级别问题。最关键的发现是 WR-13-01：`manager.rs` 的 `render_prompt()` 从未校验 `ParameterDef.required` 字段——当 required=true 且无 default 时，如果调用方未提供该参数，模板渲染器会产生一个空字符串替换而不是报错，导致 LLM 收到不完整的 prompt。这是一个影响生产正确性的逻辑缺陷。

## Build Verification

| Check | Result | Notes |
|-------|--------|-------|
| `cargo check` | PASS | No errors |
| `cargo clippy -- -D warnings` | FAIL | 9 lints project-wide, 0 in Phase 04 scope |
| `cargo test --lib` | PASS | 273 tests passed, 0 failed, 1 ignored |

## Previous Fix Verification

| ID | Fix Commit | Status | Detail |
|----|-----------|--------|--------|
| WR-12-01 | 47b312e | PASS | `FrameObservation` 移除 `deny_unknown_fields`，测试 `test_unknown_fields_silently_ignored` 验证 camelCase 字段被静默忽略。 |
| WR-12-02 | 8ee2a54 | PASS | truncate 测试使用 `result.chars().take(97).collect::<String>()` 字符级别比较。 |
| WR-12-03 | 556106f | PASS | Phase 04 范围内 7 个 clippy lint 全部修复。`cargo clippy` 确认 Phase 04 文件 0 警告。 |

All three fixes verified correct with no regression.

## Warnings

### WR-13-01: render_prompt 忽略 ParameterDef.required 字段，required=true 的参数缺失时静默替换为空字符串

**File:** `src/prompt/manager.rs:57-66`
**Issue:** `render_prompt()` 方法（第 57-66 行）合并默认值和调用方变量时，只填充了 `param.default` 不为 None 的参数（第 59-62 行），然后覆盖调用方变量（第 64-66 行）。但从未检查 `param.required == true && default.is_none() && !vars.contains_key(&param.name)` 的情况。

这意味着当一个参数声明为 `required: true, default: None`（如 `video_title`、`frame_analysis_json`、`subtitle_content`、`plot_analysis`），且调用方未在 `vars` 中提供时，该参数不会出现在 `merged` HashMap 中。后续 `template::render()` 虽然会报告"缺少必需参数"错误（如果模板中有 `${variable}` 语法），但这依赖于模板实现细节而非参数定义——更重要的是，如果模板恰好不引用某个 required 参数（比如参数名拼写错误），就会完全绕过校验。

此外，`ParameterDef.required` 字段（types.rs 第 28 行）在代码中从未被读取过（除测试中的构造），属于死数据。

**Fix:**

```rust
// 在 manager.rs render_prompt() 中，合并 defaults 之前添加校验：
pub fn render_prompt(
    &self,
    category: &str,
    name: &str,
    version: Option<&str>,
    vars: &HashMap<&str, &str>,
) -> Result<String, PromptError> {
    let prompt = self.get_prompt(category, name, version)?;

    // Validate required parameters
    let mut missing: Vec<String> = Vec::new();
    for param in &prompt.metadata.parameters {
        if param.required && param.default.is_none() && !vars.contains_key(param.name.as_str()) {
            missing.push(param.name.clone());
        }
    }
    if !missing.is_empty() {
        return Err(PromptError::Validation(format!(
            "缺少必需参数: {}", missing.join(", ")
        )));
    }

    // Merge defaults: caller vars take precedence over parameter defaults
    let mut merged: HashMap<String, String> = HashMap::new();
    // ... rest unchanged
```

### WR-13-02: Level 3/4 回退路径中，ffmpeg 成功但文件无效时未清理 PNG/BMP 残留文件

**File:** `src/visual/frame_extractor.rs:334-345, 366-377`
**Issue:** 在 `extract_single_frame()` 的 Level 3 回退中（第 334-345 行），条件 `if level3_ok && file_is_valid(&png_path)` 只在两个条件都为真时进入。但当 `level3_ok == true && file_is_valid(&png_path) == false` 时（ffmpeg 命令返回成功退出码但生成了零字节或无效 PNG 文件），PNG 文件不会被清理。

具体场景：
- 第 320-333 行：ffmpeg 输出到 `png_path`（成功返回 true）
- 第 334 行：`file_is_valid(&png_path)` 返回 false（文件为空）
- 条件不满足，跳过整个 if 块，`png_path` 残留
- 继续到 Level 4，`output_path.with_extension("bmp")` 创建 `bmp_path`
- Level 4 同理，如果 ffmpeg 成功但 BMP 文件无效，BMP 残留

最终第 379 行 `let _ = std::fs::remove_file(output_path)` 只清理 `.jpg` 目标文件，不清理 `.png` 和 `.bmp` 中间文件。

**Fix:**

```rust
// 在第 345 行 if 块结束后添加 else 分支清理 PNG：
    if level3_ok && file_is_valid(&png_path) {
        match convert_image_to_jpeg(&png_path, output_path, quality) {
            Ok(_) => {
                let _ = std::fs::remove_file(&png_path);
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("PNG 转 JPEG 失败: {}", e);
                let _ = std::fs::remove_file(&png_path);
            }
        }
    } else if level3_ok {
        // ffmpeg 成功但 PNG 文件无效，清理残留
        let _ = std::fs::remove_file(&png_path);
    }

// 同理在第 377 行后添加 else 分支清理 BMP：
    } else if level4_ok {
        let _ = std::fs::remove_file(&bmp_path);
    }
```

### WR-13-03: cleanup_fast_path_files 仅匹配 fastframe_ 前缀但不限制扩展名

**File:** `src/visual/frame_extractor.rs:466-479`
**Issue:** `cleanup_fast_path_files()`（第 474 行）使用 `name.starts_with("fastframe_")` 匹配文件，不检查扩展名。虽然 FFmpeg 快路径总是输出 `.jpg`，但如果有其他进程在同目录下创建了以 `fastframe_` 开头的非帧文件（如 `fastframe_log.txt`），它们也会被误删。

与 `rename_fast_path_frames()`（第 409 行）和 `collect_frame_paths()`（analyzer.rs 第 255 行）均同时检查 `starts_with` 和 `ends_with(".jpg")` 的模式不一致。

**Fix:**

```rust
// 第 474 行，添加扩展名检查：
if name.starts_with("fastframe_") && name.ends_with(".jpg") {
    let _ = std::fs::remove_file(&path);
}
```

## Info

### IN-13-01: template.rs 中 missing 变量去重使用 Vec::contains 导致 O(n^2) 复杂度

**File:** `src/prompt/template.rs:89, 120`
**Issue:** 第 89 行 `!missing.contains(&name.to_string())` 和第 120 行 `!missing_filter_vars.contains(&var_name.to_string())` 对 Vec 做线性查找。当模板中变量很多时（实际场景中不太可能超过几十个），性能不是问题。但使用 `HashSet` 是更惯用的去重方式，代码也更清晰。与上一轮 IN-12-03（Regex 每次编译）类似，属于低优先级改进。

**Fix:** 建议将来统一使用 `HashSet<String>` 替代 `Vec<String>` + `contains()`。

### IN-13-02: filter_re 对同一字符串迭代 3 次（变量存在校验 + 过滤器名校验 + 实际替换）

**File:** `src/prompt/template.rs:118-139`
**Issue:** 第 118-123 行（校验变量存在）、第 132-139 行（校验过滤器名）、第 141-150 行（实际替换）分别对 `filter_re.captures_iter(&result)` 迭代一次，共 3 次。可以合并为单次迭代，在替换时同时校验。但当前逻辑清晰、模板变量少，属于维护性建议。

**Fix:** 建议在单次 `replace_all` 闭包内同时做校验和替换，减少重复迭代。

### IN-13-03: BatchResponse 的 observations 字段名与 prompt schema 中声明的 frame_observations 不一致

**File:** `src/visual/analyzer.rs:29-31, 110`
**Issue:** `BatchResponse` 使用 `#[serde(alias = "frame_observations")]` 和字段名 `observations`（第 30-31 行）。prompt schema（第 110 行）向 LLM 声明的字段名是 `frame_observations`。虽然 `serde(alias)` 保证两种 JSON 键名都能反序列化，但 Rust 结构体的字段名 `observations` 与 prompt 中声明的 `frame_observations` 不同，增加了维护时的认知负担。

这不是 bug——`#[serde(alias)]` 正确处理了两种键名——但如果未来有人只看结构体不看 `alias`，可能误以为 LLM 返回 `observations` 键名。

**Fix:** 可考虑将 Rust 字段名改为 `frame_observations`，使用 `#[serde(alias = "observations")]` 作为回退，使代码与 prompt schema 更直观对齐。低优先级。

### IN-13-04: parse_and_retry 静默吞掉 BatchResponse 解析错误

**File:** `src/visual/analyzer.rs:199-227`
**Issue:** `parse_and_retry()` 在第 203 行尝试 `BatchResponse` 解析失败时，不记录错误日志就回退到 `FrameObservation` 单对象解析。如果 LLM 返回的 JSON 结构接近 BatchResponse 但有轻微格式问题（如缺少一个逗号），错误信息会被吞掉，只报告最终 `FrameObservation` 解析失败的错误。这使得调试 LLM 响应格式问题变得困难。

上一轮 IN-12-02 已指出 JSON schema 硬编码与结构分离的问题，此问题是同一区域的补充观察。

**Fix:** 建议在 BatchResponse 解析失败时添加 `tracing::debug!` 级别日志，记录第一次解析的错误原因。

---
_Reviewed: 2026-05-02T00:00:00Z_
_Reviewer: Claude (rust-reviewer, gsd-code-reviewer)_
_Depth: standard_
_Iteration: 13_
