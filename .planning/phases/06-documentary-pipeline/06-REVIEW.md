---
phase: 06-documentary-pipeline
reviewed: 2026-05-08T12:00:00Z
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
  critical: 0
  warning: 1
  info: 5
  total: 6
status: issues_found
previous_review:
  date: 2026-05-08
  iteration: 7
  findings: 6 (1 CR + 1 WR + 4 IN)
  fixes_applied: 2 fixed (CR-01 subtitle path escape, WR-01 amix volume in audio.rs)
---

# Phase 06: Code Review Report (Iteration 8)

**Reviewed:** 2026-05-08T12:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

第 8 次审查基于最新代码（含 iter 7 的 2 项修复）重新分析纪录片流水线模块。

前次修复验证：
- **CR-01**（字幕路径双重转义）：pipeline.rs:385-389 已简化为 `\`→`/` + `'`→`'\''` + 去换行，单引号保护特殊字符 ✓
- **WR-01**（amix 归一化）：audio.rs:70-73 已追加 `volume=N` 补偿 ✓

本轮新发现 **1 个 WARNING**：composite 步骤的 amix 滤镜与 audio.rs 存在相同根因但未修复。5 个 INFO 沿用自前次。

## Warnings

### WR-01: composite 步骤 amix 滤镜缺少音量补偿 (pipeline.rs)

**File:** `narratoai-core/src/documentary/pipeline.rs:375-378`
**Impact:** 最终合成时解说/原声/BGM 音量被 amix 按 1/N 衰减，输出显著低于预期

**Issue:** `step_composite` 将 orig、TTS、BGM 三路音频通过 `amix=inputs=N` 混合，amix 默认将每路音量除以 N。iter 7 的 WR-01 修复了 `audio.rs` 中 merge_audio_files 的同一问题，但 pipeline.rs 的 composite 步骤未同步修复。

```rust
// 当前代码（pipeline.rs:375-378）——无音量补偿
filter_complex_parts.push(format!(
    "{}amix=inputs={}:duration=longest[aout]",
    mix_inputs, amix_input_count
));
```

**典型影响（默认配置，3 路输入）：**
- orig (0.70) → 0.70/3 ≈ 0.23
- TTS  (1.00) → 1.00/3 ≈ 0.33
- BGM  (0.30) → 0.30/3 ≈ 0.10

**注意：** 此处与 audio.rs 的修复场景有本质差异。audio.rs 混合多个 TTS 片段（同一时刻仅一路播放），`volume=N` 补偿完全安全。composite 同时混合 2-3 路音频，`volume=N` 可能导致削波（0.7+1.0+0.3=2.0 > 1.0）。推荐使用更温和的补偿系数或 `normalize=0`（FFmpeg 4.4+）。

**Fix（推荐方案一，温和补偿）：**
```rust
// 方案一：amix 后补偿为原始音量的一半，保留削波余量
let compensation = if amix_input_count > 1 {
    format!(",volume={}", amix_input_count as f64 * 0.5)
} else {
    String::new()
};
filter_complex_parts.push(format!(
    "{}amix=inputs={}:duration=longest{}[aout]",
    mix_inputs, amix_input_count, compensation
));
```

**Fix（推荐方案二，FFmpeg 4.4+）：**
```rust
// 方案二：禁用归一化，依赖用户自行调整各路音量
filter_complex_parts.push(format!(
    "{}amix=inputs={}:duration=longest:normalize=0[aout]",
    mix_inputs, amix_input_count
));
```

## Info

### IR-01: collect_keyframe_paths 为死代码 (script_gen.rs)

**File:** `narratoai-core/src/documentary/script_gen.rs:416-432`
**Issue:** 该函数从未被调用。`analyze_video` 直接使用 `extract_frames` 返回的路径列表。

### IR-02: ProgressStep 枚举在生产代码中未使用 (types.rs)

**File:** `narratoai-core/src/documentary/types.rs:101-109`
**Issue:** `ProgressStep` 枚举已定义并导出，但 `PipelineState.emit_progress` 使用 `&str` 参数。

### IR-03: strip_and_repair_json 尾逗号修复可能误改字符串内容 (script_gen.rs)

**File:** `narratoai-core/src/documentary/script_gen.rs:348-349`
**Issue:** `.replace(",}", "}").replace(",]", "]")` 是全局替换，可能匹配 JSON 字符串值内部的 `,}` 和 `,]`。

### IR-04: generate_srt_from_word_boundaries 跳过块时序号不连续 (subtitle.rs)

**File:** `narratoai-core/src/documentary/subtitle.rs:22-42`
**Issue:** 使用 `enumerate` 的 `i + 1` 作为 SRT 序号，跳过负时间戳块后序号间断。`merge_srt_files` 会重新编号，影响仅限独立使用。

### IN-01 (iter 6): 单引号替换可损坏自然语言 (script_gen.rs)

**File:** `narratoai-core/src/documentary/script_gen.rs:362-367`
**Issue:** `text.replace('\'', "\"")` 将所有单引号替换为双引号，若 LLM 输出含英文缩写（don't, it's）或自然语言引用，内容会被损坏。

## Cross-File Analysis

### 前次修复验证

| ID | 描述 | 修复位置 | 验证结果 |
|----|------|----------|----------|
| CR-01 (iter 7) | 字幕路径双重转义 | pipeline.rs:385-389 | ✓ 单引号内仅转义 `'` 自身 |
| WR-01 (iter 7) | amix 音量衰减 | audio.rs:70-73 | ✓ volume=N 补偿已生效 |

### 安全性

- **FFmpeg 命令注入**：全部通过 `ffmpeg-sidecar` Rust API 构建，`cmd.arg()` 逐参数传递，无 shell 拼接 ✓
- **路径注入**：step_concat 检查 `\n`/`\r` 并拒绝 ✓
- **字体名清理**：字符白名单过滤 ✓
- **字幕颜色**：`validate()` 校验 `#RRGGBB`，composite 中 ASS 转换有兜底 ✓
- **字幕路径转义**：CR-01 修复后，单引号正确保护特殊字符 ✓

### 资源管理

- **CleanupOnDrop**（script_gen.rs:35-55）：RAII 守卫确保分析失败时自动清理 keyframe 目录 ✓
- **PipelineState**：无 Drop 清理——临时文件保留在 task_dir 供调试（设计选择）✓

### 错误处理链完整性

- `PipelineError` 12 变体完整覆盖
- 5 个 `From` 实现支持 `?` 自动转换
- 所有 `Display` 消息使用中文

## Iteration History

| Iteration | Critical | Warning | Info | Fixed | Status |
|-----------|----------|---------|------|-------|--------|
| 1–5 | (see iter 6) | | | | |
| 6 | 0 | 2 | 2 | 2 | partial |
| 7 | 1 | 1 | 4 | 2 | all_fixed |
| 8 (this) | 0 | 1 | 5 | 0 | issues_found |

---
_Reviewed: 2026-05-08T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 8_
