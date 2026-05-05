---
status: all_fixed
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
previous_review: 2026-05-04 (Re-Fix #6 — WR-31 fixed)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review #6)

## Summary

第七次审查 12 个 Rust 源文件。上次 1 个 WARNING（WR-31）已验证修复。本次新发现 1 个 WARNING（WR-32：`extract_json_object` 字节/字符索引混用导致 panic 风险）。7 项 INFO 均为遗留。

WR-32 是 `script_gen.rs` 中 JSON 修复链的关键函数：当 LLM 返回包含中文字符的非法 JSON（带前后文本）时，该函数因字节索引与字符索引混用触发 Rust 运行时 panic。由于 LLM 在 JSON mode 下通常会返回干净 JSON，实际触发概率中等，但一旦触发即导致流水线崩溃（不可恢复）。

---

## WARNING Findings

### WR-32: `extract_json_object` 字节/字符索引混用导致 panic
- **文件**: `src/documentary/script_gen.rs:342-376`
- **描述**: 函数中存在三处索引类型混用：
  1. `text.find('{')?` 返回**字节索引**，赋值给 `start`
  2. `for i in start..chars.len()` 将 `start`（字节索引）用作字符迭代起点，`i` 成为**字符索引**
  3. `text[start..=i]` 将 `i`（字符索引）用作字节切片上界

  当 JSON 响应包含中文内容时（如 `Based on analysis:\n{"items": [{"narration": "这是一个测试"}]}`），字符索引与字节索引的偏移量会累积。外层 `}` 的字符索引用作字节切片时，大概率落在多字节 UTF-8 字符中间（3 字节汉字的边界：字节 0%3=0, 1%3=1, 2%3=2——中文字符的中间字节是非法的切片边界），触发 Rust 运行时 panic。

  **触发条件**：LLM 返回非法 JSON（包含前后文本）且 JSON 值中包含中文字符。当 `serde_json::from_str` 直接解析成功时不会进入此函数，因此实际触发概率取决于 LLM JSON mode 的可靠性。
- **复现路径**: LLM 返回 `"根据分析结果：\n{\"items\": [{\"narration\": \"你好世界\"}]}\n以上是脚本。"` → `strip_code_fence` 无变化 → `serde_json::from_str` 失败 → `extract_json_object` 被调用 → `text[start..=i]` 在中文字符中间切片 → **panic**
- **修复**: 统一使用字符索引，避免字节切片：

  ```rust
  fn extract_json_object(text: &str) -> Option<String> {
      let chars: Vec<char> = text.chars().collect();
      let start = chars.iter().position(|&c| c == '{')?;
      let mut depth = 0i32;
      let mut in_string = false;
      let mut escape_next = false;
      for i in start..chars.len() {
          let c = chars[i];
          if escape_next {
              escape_next = false;
              continue;
          }
          if c == '\\' && in_string {
              escape_next = true;
              continue;
          }
          if c == '"' {
              in_string = !in_string;
              continue;
          }
          if !in_string {
              match c {
                  '{' => depth += 1,
                  '}' => {
                      depth -= 1;
                      if depth == 0 {
                          return Some(chars[start..=i].iter().collect());
                      }
                  }
                  _ => {}
              }
          }
      }
      None
  }
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

### IN-32: `parse_script_clips` 根级数组回退为不可达死代码（遗留）
- **文件**: `src/documentary/script_gen.rs:267`
- **状态**: 无变化。`parsed.get("items")` 在根为数组时返回 `None`，`ok_or_else` 提前返回错误，`or_else(|| parsed.as_array())` 不可达。

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-31 | `parse_batch_response` 字节截断 panic | ✅ 已修复 | script_gen.rs:236 使用 `cleaned.chars().take(200).collect::<String>()` |
| WR-29 | `validate()` 未校验 `voice_pitch` | ✅ 已修复 | types.rs:58-60 添加 `[-10.0, 10.0]` 范围检查 |
| WR-30 | 集成测试进度值与实现不同步 | ✅ 已修复 | documentary_integration_test.rs:135 值为 `70.0` |
| IN-02~IN-32 | Re-Review #5 全部 7 项 INFO | 无变化 | 保持不变 |

---

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error（1 warning，`get_azure_voices` 不在审查范围） |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| WR-31 修复: 字符迭代截断 | ✅ script_gen.rs:236 |

---

## Priority Fix Order

1. **WR-32** — 修复 `extract_json_object` 字节/字符索引混用（约 15 行重写）
2. **IN-32** — 移除 `parse_script_clips` 不可达的根级数组回退（1 行）
3. **IN-31** — 文档化或实现未使用的字幕/宽高字段（低优先级，设计决策）
