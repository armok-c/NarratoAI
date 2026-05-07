---
phase: 06-documentary-pipeline
reviewed: 2026-05-08T00:45:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - narratoai-core/src/documentary/mod.rs
  - narratoai-core/src/documentary/error.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/documentary/timestamp.rs
  - narratoai-core/src/documentary/subtitle.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/documentary/clip.rs
  - narratoai-core/src/documentary/audio.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/lib.rs
findings:
  critical: 1
  warning: 1
  info: 4
  total: 6
status: issues_found
previous_review:
  date: 2026-05-07
  iteration: 6
  findings: 4 (0 CR + 2 WR + 2 IN)
  fixes_applied: 2 fixed (WR-01 negative timestamp continue, WR-02 empty stubs replaced)
---

# Phase 06: Code Review Report (Iteration 7)

**Reviewed:** 2026-05-08T00:45:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

第 7 次审查基于最新代码重新分析纪录片流水线模块。前次（iteration 6）的 2 个 WARNING 均已修复：
- WR-01（负时间戳 continue）：`subtitle.rs:30` 已有 `continue`，跳过负时间戳块 ✓
- WR-02（空测试桩）：`documentary_integration_test.rs` 已替换为实际测试（类型校验、错误消息验证）✓

本轮新发现 **1 个 CRITICAL**（Windows 字幕路径转义）、**1 个 WARNING**（amix 音量衰减）和 **4 个 INFO**。

## Critical

### CR-01: FFmpeg 字幕滤镜路径在 Windows 上双重转义 (pipeline.rs)

**File:** `narratoai-core/src/documentary/pipeline.rs:385-396`
**Impact:** Windows 默认配置下合成步骤必定失败

**Issue:** `step_composite` 中字幕路径同时使用单引号包裹和冒号转义，产生冲突：

```rust
// 当前代码（有问题）
let escaped_srt = srt_str
    .replace('\\', "/")      // C:\Users\... → C:/Users/...
    .replace(':', "\\:")     // C:/Users/... → C\:/Users/...  ← 冒号被转义
    .replace("'", "\\'")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace(';', "\\;")
    .replace('\n', "")
    .replace('\r', "");
// 生成: [0:v]subtitles='C\:/Users/.../merged_subtitle.srt':force_style='...'[vout]
```

FFmpeg 滤镜解析中，单引号内的 `\:` 是两个字面量字符（反斜杠 + 冒号），不是转义冒号。因此 FFmpeg 查找的文件路径是 `C\:/Users/.../merged_subtitle.srt`，该路径不存在。

**触发条件（默认配置即触发）：**
1. Windows 系统：`std::env::temp_dir()` → `C:\Users\...\AppData\Local\Temp`
2. `subtitle_enabled` 默认 `true`
3. `output_dir` 默认 `None`，走 `temp_dir()`

**Fix:** 单引号已保护冒号等特殊字符，移除冒号转义：

```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace('\'', "'\\''")  // 转义单引号本身
    .replace('\n', "")
    .replace('\r', "");
```

## Warnings

### WR-01: amix 滤镜按 1/N 归一化导致多片段音频衰减 (audio.rs)

**File:** `narratoai-core/src/documentary/audio.rs:69-73`
**Impact:** 长视频（多旁白片段）时解说音量显著降低

**Issue:** `merge_audio_files` 将静音基底和所有 TTS 片段一起送入 `amix=inputs=N:duration=longest`。FFmpeg 的 `amix` 默认将每个输入音量除以 N（总输入数），即使同一时刻仅一个片段在播放。

例如 20 个旁白片段 + 1 静音基底 = 21 个输入，每个片段有效音量 = 原始音量 / 21。

```rust
filter_parts.push(format!(
    "{}amix=inputs={}:duration=longest[aout]",
    amix_inputs, input_count
));
```

**Fix（推荐）：** 在 amix 后补偿增益：

```rust
filter_parts.push(format!(
    "{}amix=inputs={}:duration=longest,volume={}[aout]",
    amix_inputs, input_count, input_count as f64
));
```

或使用 FFmpeg 4.4+ 的 `normalize=0` 选项。

## Info

### IR-01: collect_keyframe_paths 为死代码 (script_gen.rs)

**File:** `narratoai-core/src/documentary/script_gen.rs:416-432`
**Issue:** 该函数从未被调用。`analyze_video` 直接使用 `extract_frames` 返回的路径列表。函数位于 `#[cfg(test)]` 之外的生产代码中。

**Fix:** 移除或移入 `#[cfg(test)] mod tests`。

### IR-02: ProgressStep 枚举在生产代码中未使用 (types.rs)

**File:** `narratoai-core/src/documentary/types.rs:101-109`
**Issue:** `ProgressStep` 枚举（6 变体）已定义并导出，但 `PipelineState.emit_progress` 使用 `&str` 参数（如 `"load_script"`, `"tts"`）。该枚举仅在集成测试中使用，且测试中的回调签名 `(ProgressStep, f32, &str)` 与 `ProgressCallback`（即 `Fn(&str, f32, &str)`）不匹配。

**Note:** SDE 和 SDP 模块有各自对应的 `SdeProgressStep` 和 `SdpProgressStep` 并在管道中实际使用。纪录片模块是唯一未使用其 ProgressStep 枚举的模块。

### IR-03: strip_and_repair_json 尾逗号修复可能误改字符串内容 (script_gen.rs)

**File:** `narratoai-core/src/documentary/script_gen.rs:348-349`
**Issue:** `.replace(",}", "}").replace(",]", "]")` 是全局替换，会匹配 JSON 字符串值内部的 `,}` 和 `,]`。例如 `{"text": "hello,} world"}` → `{"text": "hello} world"}`。

**Mitigation:** 仅在直接 JSON 解析失败后尝试，且 LLM 输出极少在字符串中包含 `,}`。

### IR-04: generate_srt_from_word_boundaries 跳过块时序号不连续 (subtitle.rs)

**File:** `narratoai-core/src/documentary/subtitle.rs:22-42`
**Issue:** 使用 `enumerate` 的 `i + 1` 作为 SRT 序号。若因负时间戳跳过块，序号间断（如 1, 3, 4）。大多数 SRT 播放器可处理，但严格解析器可能拒绝。`merge_srt_files` 会重新编号，因此影响仅限于独立使用的 SRT 文件。

## Carried Over (from Iteration 6)

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| IN-01 (iter 6) | INFO | `strip_and_repair_json` 单引号替换可损坏自然语言 | 仍适用，与 IR-03 互补 |

## Cross-File Analysis

### 错误处理链完整性

- `PipelineError` 12 变体完整覆盖：ScriptLoad / TtsGeneration / VideoClip / AudioMerge / Concat / Composite / Io / Timestamp / SrtGeneration / Validation / FrameExtraction / FFmpeg / Llm
- 5 个 `From` 实现支持 `?` 自动转换
- 所有 `Display` 消息使用中文

### OST 分发策略一致性

| OST | clip.rs | audio.rs | calculate_clip_duration 优先级 |
|-----|---------|----------|-------------------------------|
| 0 (NarrationOnly) | `-an` 移除原声 | adelay + amix | TTS → duration → range |
| 1 (OriginalSound) | 保留原声 | 跳过（无 TTS） | duration → range |
| 2 (Mixed) | 保留原声 | adelay + amix | TTS → duration → range |

`calculate_clip_duration` 在 `audio.rs` 和 `pipeline.rs` 中使用相同的优先级链，逻辑一致。

### 安全性

- **FFmpeg 命令注入**：全部通过 `ffmpeg-sidecar` 的 Rust API 构建，`cmd.arg()` 逐参数传递，无 shell 拼接
- **路径注入**：`step_concat` 检查视频路径中的 `\n`/`\r` 并拒绝
- **字体名清理**：`alphanumeric + space + dash + underscore` 白名单过滤
- **字幕颜色**：`validate()` 校验 `#RRGGBB`，composite 中 ASS 转换有 `&H00FFFFFF` 兜底
- **Concat 路径引号**：单引号包裹 + 内部单引号转义

### 资源管理

- `CleanupOnDrop`（script_gen.rs:35-55）：RAII 守卫确保分析失败时自动清理 keyframe 目录，成功时 `cancel()` 保留
- `PipelineState` 无 Drop 清理——临时文件保留在 task_dir 供调试（设计选择）

## Iteration History

| Iteration | Critical | Warning | Info | Fixed | Status |
|-----------|----------|---------|------|-------|--------|
| 1–5 | (see iter 6) | | | | |
| 6 | 0 | 2 | 2 | 2 | partial |
| 7 (this) | 1 | 1 | 4 | 0 | issues_found |

---
_Reviewed: 2026-05-08T00:45:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 7_
