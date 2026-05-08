---
phase: 06-documentary-pipeline
reviewed: 2026-05-08T14:30:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - narratoai-core/src/documentary/audio.rs
  - narratoai-core/src/documentary/clip.rs
  - narratoai-core/src/documentary/error.rs
  - narratoai-core/src/documentary/mod.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/documentary/subtitle.rs
  - narratoai-core/src/documentary/timestamp.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/lib.rs
  - tests/common/mod.rs
  - tests/documentary_integration_test.rs
findings:
  critical: 0
  warning: 0
  info: 5
  total: 5
status: clean
previous_review:
  date: 2026-05-08
  iteration: 9
  findings: "1 critical, 1 warning, 5 info"
---

# Phase 6: Code Review Report (Iteration 10)

**Reviewed:** 2026-05-08T14:30:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** clean

## Summary

本次为第 10 轮迭代审查，重点验证 iter 9 的两个发现是否已修复。

**iter 9 CR-01 验证结果：已修复。** `script_gen.rs:117` 已添加 `None` 作为 `cancel: Option<CancellationToken>` 参数，`analyze_images()` 调用签名与 `LlmProvider` trait 方法签名完全匹配。

**iter 9 WR-01 验证结果：已修复。** `pipeline.rs:375-376` 的 composite 步骤 amix volume 补偿已改为 `volume=N`，与 `audio.rs:72` 保持一致。两处归一化补偿策略现已统一。

**整体评估：** 全部 12 个文件审查完毕，未发现新的 BLOCKER 或 WARNING 级别问题。剩余 5 个 INFO 级别发现均为低优先级的代码质量改进建议，不影响功能正确性或安全性。代码质量良好，可以合并。

**注意：** 项目整体存在一个审查范围外的编译错误（`visual/frame_extractor.rs:78` 非 exhaustive match），不在本次审查文件范围内。

## 前次修复验证

| ID (iter) | 描述 | 修复提交 | 验证结果 |
|-----------|------|----------|----------|
| CR-01 (9) | analyze_images() 缺少 CancellationToken 参数 | 9238390 | Pass - 第 117 行已添加 `None` 参数 |
| WR-01 (9) | composite amix volume 补偿系数不一致 | fa1aa81 | Pass - 已改为 `volume=N`，与 audio.rs 一致 |

## Info

### IN-01: collect_keyframe_paths 函数为死代码

**File:** `narratoai-core/src/documentary/script_gen.rs:417-433`
**Issue:** `collect_keyframe_paths()` 函数在模块内定义但从未被调用。`analyze_video()` 直接使用 `extract_frames()` 返回的路径列表（第 76-87 行），无需重新扫描目录。`visual/frame_extractor.rs` 中有功能类似的 `collect_keyframe_paths_from_dir()` 负责实际使用场景。
**Fix:** 移除该函数。

### IN-02: ProgressStep 枚举在生产代码中未使用，测试中存在类型不匹配

**File:** `narratoai-core/src/documentary/types.rs:101-109`
**File:** `tests/documentary_integration_test.rs:121`
**Issue:** `ProgressStep` 枚举定义了 6 个变体（LoadScript、Tts、Clip 等），但生产代码的 `ProgressCallback` 类型为 `Box<dyn Fn(&str, f32, &str) + Send + Sync>`（`types.rs:112`），使用 `&str` 传递步骤名而非枚举。`documentary_integration_test.rs:121` 中测试回调签名 `|step: ProgressStep, pct: f32, msg: &str|` 与 `ProgressCallback` 的 `Fn(&str, f32, &str)` 不匹配。该测试无法作为 `ProgressCallback` 使用，仅验证了枚举自身的 Debug 格式化，实际类型契约未被测试覆盖。
**Fix:** (1) 移除 `ProgressStep` 枚举，统一使用 `&str`；或 (2) 将 `ProgressCallback` 改为使用枚举。同时修正测试以匹配实际的回调签名。

### IN-03: strip_and_repair_json trailing comma 修复可能破坏字符串内容

**File:** `narratoai-core/src/documentary/script_gen.rs:349`
**Issue:** `text.replace(",}", "}").replace(",]", "]")` 是全局字符串替换，无法区分 JSON 结构字符和字符串值内的内容。例如 `{"text": "hello,}"}` 中 `,}` 出现在字符串值中，替换后会破坏数据。实际风险较低（LLM 输出极少包含此模式），且仅在直接解析失败时触发，属于最后一道防线。
**Fix:** 可考虑基于 JSON tokenizer 的更精确替换，但优先级低。

### IN-04: SRT 序列号在跳过负时间戳块时不连续

**File:** `narratoai-core/src/documentary/subtitle.rs:36`
**Issue:** `generate_srt_from_word_boundaries()` 使用 `enumerate()` 的 `i + 1` 作为 SRT 序号，但当跳过负时间戳的 word boundary 时（第 30 行 `continue`），序号会出现间隔（如 1, 2, 4, 5）。SRT 规范不要求连续序号（播放器按时间戳排序），`merge_srt_files` 会重新编号，因此仅影响独立使用 `generate_srt_from_word_boundaries` 的场景。
**Fix:** 使用独立计数器 `seq` 替代 `i + 1`。

### IN-05: 单引号替换可能破坏自然语言内容

**File:** `narratoai-core/src/documentary/script_gen.rs:363-367`
**Issue:** 在 `!text.contains('"') && text.contains('\'')` 条件下执行 `text.replace('\'', "\"")`。虽然前置条件降低了误替换风险，但如果 LLM 返回的纯单引号 JSON 文本包含自然语言单引号（如英文 "it's"），这些引号会被全部替换为双引号，可能破坏字符串内容。中文场景下风险极低。
**Fix:** 优先级低。可考虑更精确的引号对匹配策略。

## Cross-File Analysis

### 安全性

- **FFmpeg 命令注入**：全部通过 `ffmpeg-sidecar` Rust API 的 `cmd.arg()` 逐参数传递，无 shell 拼接风险
- **路径注入**：`step_concat`（pipeline.rs:193）检查 `\n`/`\r` 并拒绝非法路径
- **字体名清理**：pipeline.rs:317-319 使用字符白名单过滤（字母数字、空格、连字符、下划线）
- **字幕颜色**：`validate()` 校验 `#RRGGBB` 格式（types.rs:92-95），composite 中 ASS 转换有兜底值（pipeline.rs:308）
- **字幕路径转义**：pipeline.rs:390-394 正确转义单引号和反斜杠

### 资源管理

- **CleanupOnDrop**（script_gen.rs:35-55）：RAII 守卫确保分析失败时自动清理 keyframe 目录；成功时调用 `cancel()` 保留文件
- **PipelineState**：无 Drop 清理，临时文件保留在 task_dir 供用户调试（合理的设计选择）

### 错误处理链完整性

- `PipelineError` 12 个变体完整覆盖所有流水线步骤
- 5 个 `From` 实现支持 `?` 自动转换（ScriptError、TTSError、io::Error、FFmpegError、LLMError）
- 所有 `Display` 消息使用中文
- 所有 FFmpeg 操作统一处理 spawn 失败、事件错误、退出码异常三种错误路径

### 数值安全性

- `secs_to_srt_time`/`secs_to_ffmpeg_time`：输入限制在 `[0, 86399.999]` 范围，`(secs * 1000.0).round() as u64` 不会溢出
- `audio.rs:54`：`adelay` 的毫秒值基于视频时长计算，远在 u64 范围内
- `pipeline.rs:375-376`：amix volume 补偿使用 `amix_input_count`（usize），值域为 1-3（原声 + TTS + BGM），不会溢出

## Iteration History

| Iter | Date       | Critical | Warning | Info | Fixed | Status   | Key Changes                            |
|------|------------|----------|---------|------|-------|----------|----------------------------------------|
| 1-5  | 2026-05-07 | (see iter 6) |     |      |       |          | 初始审查与修复迭代                      |
| 6    | 2026-05-07 | 0        | 2       | 2    | 2     | partial  |                                        |
| 7    | 2026-05-07 | 1        | 1       | 4    | 2     | fixed    | subtitle 路径转义 + amix 补偿          |
| 8    | 2026-05-08 | 0        | 1       | 5    | 0     | found    | composite amix 补偿验证                |
| 9    | 2026-05-08 | 1        | 1       | 5    | 2     | found    | 编译错误 + 补偿系数不一致              |
| 10   | 2026-05-08 | 0        | 0       | 5    | 2     | clean    | 全部修复验证通过，无新问题             |

---
_Reviewed: 2026-05-08T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 10_
