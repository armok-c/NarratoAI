---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 1
  info: 7
  total: 8
reviewer: gsd-code-reviewer
date: 2026-05-04
previous_review: 2026-05-04 (Re-Fix #5 — WR-29/WR-30 fixed)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review #5)

## Summary

第六次审查 12 个 Rust 源文件。上次 2 个 WARNING（WR-29、WR-30）全部验证已修复。本次新发现 0 个 CRITICAL、1 个 WARNING（WR-31：多字节字符切片 panic 风险）、新增 1 个 INFO（IN-32）。

代码质量持续改善：`voice_pitch` 范围校验和测试进度值同步均已到位，`validate()` 覆盖所有数值参数。剩余唯一 WARNING 是 `script_gen.rs` 中错误路径的字符串切片可能在中文 JSON 响应边界处 panic。

---

## WARNING Findings

### WR-31: `parse_batch_response` 错误路径字符串切片可能 panic
- **文件**: `src/documentary/script_gen.rs:236`
- **描述**: 错误回退分支中 `&cleaned[..cleaned.len().min(200)]` 按**字节**截断字符串。当 LLM 返回包含中文字符的 JSON（每个汉字 3 字节 UTF-8），200 字节边界大概率落在多字节字符中间（200 % 3 = 2），触发 Rust 运行时 panic。此代码路径在 LLM 返回非法 JSON 且 `frame_observations` 字段缺失时触发，属于错误处理路径中的二次故障。
- **修复**: 用字符迭代替代字节截断：
  ```rust
  // 替换第 236 行
  let truncated: String = cleaned.chars().take(200).collect();
  errors: vec![format!("无法解析批次响应: {}", truncated)],
  ```

---

## INFO Findings

### IN-02: `parse_timestamp_range` 使用 `split('-')` 而非 `splitn(2, '-')`（遗留）
- **文件**: `src/documentary/timestamp.rs:7`
- **状态**: 无变化，影响极低。时间戳格式不含内部短横线，`split` 与 `splitn(2, '-')` 结果等价。

### IN-05: `merge_srt_files(&[])` 返回 `Ok("")` 而非错误（遗留）
- **文件**: `src/documentary/subtitle.rs:43`
- **状态**: 无变化，调用方已确保传入非空。

### IN-27: 关键帧缓存回退到当前工作目录（遗留）
- **文件**: `src/documentary/script_gen.rs:39`
- **状态**: 无变化。

### IN-28: 忽略的 FFmpeg 测试体为空（遗留）
- **文件**: `tests/documentary_integration_test.rs:166-203`
- **状态**: 无变化。

### IN-29: 测试 WordBoundary 数值使用非标准分组（遗留）
- **文件**: `tests/documentary_integration_test.rs:50-72`
- **状态**: 无变化。数值正确但分组写法不直观（如 `5_000_000_0` 应为 `50_000_000`）。

### IN-31: `subtitle_font`、`subtitle_color`、`subtitle_position`、`video_aspect` 字段声明但未使用（遗留）
- **文件**: `src/documentary/types.rs:19,21-23`、`src/documentary/pipeline.rs`（step_composite）
- **状态**: 无变化。`step_composite` 的 FFmpeg `force_style` 只使用 `FontSize`，未使用 `FontName`、`PrimaryColour`、`MarginV`。

### IN-32: `parse_script_clips` 根级数组回退为不可达死代码
- **文件**: `src/documentary/script_gen.rs:267`
- **描述**: `items.as_array().or_else(|| parsed.as_array())` 中，`parsed.as_array()` 回退路径不可达。若 JSON 根为数组 `[...]`，`parsed.get("items")` 返回 `None`，在 `ok_or_else`（第 263 行）处已提前返回错误，永远不会到达第 267 行的回退逻辑。此回退代码无实际效果。
- **修复**: 移除 `or_else` 分支，直接使用 `items.as_array()`：
  ```rust
  let arr = items.as_array().ok_or_else(|| PipelineError::Llm {
      source: crate::error::LLMError::Validation("items 不是数组".to_string()),
  })?;
  ```

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-29 | `validate()` 未校验 `voice_pitch` 范围 | ✅ 已修复 | types.rs:58-60 添加 `[-10.0, 10.0]` 范围检查，与其他参数校验风格一致 |
| WR-30 | 集成测试进度值与实现不同步 | ✅ 已修复 | documentary_integration_test.rs:135 值为 `70.0`，与 pipeline.rs:174 一致 |
| IN-02~IN-29 | Re-Review #4 全部 6 项 INFO | 无变化 | 保持不变 |
| IN-31 | 四个字幕/宽高字段未使用 | 无变化 | 设计决策，需后续版本规划 |

---

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error（1 warning，`get_azure_voices` 不在审查范围） |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| WR-29 修复: `voice_pitch` 校验存在 | ✅ types.rs:58-60 |
| WR-30 修复: 测试进度值 70.0 | ✅ documentary_integration_test.rs:135 |

---

## Priority Fix Order

1. **WR-31** — 修复 `parse_batch_response` 字符串切片 panic 风险（1-2 行）
2. **IN-32** — 移除 `parse_script_clips` 不可达的根级数组回退（1 行）
3. **IN-31** — 文档化或实现未使用的字幕/宽高字段（低优先级，设计决策）
