---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 16
  info: 13
  total: 29
reviewer: gsd-code-reviewer
date: 2026-05-03
---

# Code Review: Phase 06 — Documentary Pipeline

## Summary

审查了 12 个 Rust 源文件（9 个核心模块 + 1 个 lib.rs + 2 个测试文件）。
发现 0 个 CRITICAL、16 个 WARNING、13 个 INFO 级别问题。

主要风险集中在三个方面：
1. **时间戳负数溢出** — `secs_to_srt_time` 等函数对负数输入产生静默 u64 溢出
2. **FFmpeg 命令构建缺陷** — `-map` 冲突、`-vf`/`-filter_complex` 混用、路径转义不完整
3. **JSON 修复逻辑脆弱** — 花括号/引号替换会破坏合法 JSON

---

## WARNING Findings

### WR-01: `secs_to_srt_time` 对负数 secs 产生静默 u64 溢出
- **文件**: `src/documentary/timestamp.rs:73`
- `secs` 为负时，`(secs * 1000.0).round() as u64` 会 wrapping 到 `u64::MAX`，产生无意义时间戳。同样影响 `secs_to_ffmpeg_time`。
- **修复**: cast 前检查 `total_ms >= 0.0`，或对 `secs < 0.0` 返回错误。

### WR-02: `parse_time_to_secs` 未验证 h/m/s 各分量范围
- **文件**: `src/documentary/timestamp.rs:41-49`
- 允许 `00:61:00`（61 分钟）作为有效时间戳（= 3660s），负数分量也只在总和层面被拒绝。
- **修复**: 对 h(0-23)、m(0-59)、s(0-59) 分别验证。

### WR-03: `parse_srt_blocks` 纯数字文本行被误判为序号
- **文件**: `src/documentary/subtitle.rs:79`
- `line.chars().all(|c| c.is_ascii_digit())` 无法区分文本行 "2000" 和序号行。
- **修复**: 改为按空行分割块，块内按行解析（序号→时间戳→文本）。

### WR-04: `merge_srt_files` 偏移后不检查负时间戳
- **文件**: `src/documentary/subtitle.rs:137-151`
- `apply_offset_to_block` 直接计算 `start + offset_secs`，负数结果传给 `secs_to_srt_time` 触发 WR-01。
- **修复**: 检查 `new_start >= 0.0`，否则返回错误或 clamp。

### WR-05: `DocumentaryRequest` 音量/比率字段缺少取值验证
- **文件**: `src/documentary/types.rs:12-15`
- `voice_rate`、`tts_volume` 等字段可接受任意 f64（含负数、NaN、Infinity），会传播到 FFmpeg 命令。
- **修复**: 添加 `validate(&self) -> Result<(), PipelineError>` 方法。

### WR-06: `step_composite` 死代码 — `input_idx`、`filter_parts` 未使用
- **文件**: `src/documentary/pipeline.rs:261-288`
- 第 261-265 行声明 `input_idx` 后立即丢弃；第 268-288 行构建的 `filter_parts`/`audio_mix_inputs` 在闭包内被完全重建。
- **修复**: 删除这些死代码行。

### WR-07: `-map 0:v` 未在字幕滤镜路径中移除
- **文件**: `src/documentary/pipeline.rs:369-380`
- 当 `audio_inputs_count > 0` 时添加了 `-map 0:v`，但后续字幕滤镜路径添加 `-map [vout]` 时没有移除前者，导致 FFmpeg 收到两个视频 map。
- **修复**: 添加 `-map [vout]` 前移除之前的 `-map 0:v`，或重构为条件分支。

### WR-08: `-vf` 与 `-filter_complex` 潜在冲突
- **文件**: `src/documentary/pipeline.rs:389-395`
- 同时使用 `-vf`（视频滤镜）和 `-filter_complex`（音频混流）时，FFmpeg 不允许对同一流同时使用两者。
- **修复**: 统一使用 `filter_complex` 处理所有滤镜。

### WR-09: SRT 路径转义不完整
- **文件**: `src/documentary/pipeline.rs:350`
- 只转义了 `:`，未转义 `[` 和 `]`。FFmpeg `subtitles` 滤镜对这些字符敏感。
- **修复**: 添加 `.replace('[', "\\[").replace(']', "\\]")`。

### WR-10: 两个不同的 clip duration 计算函数
- **文件**: `src/documentary/pipeline.rs:439-462` vs `src/documentary/audio.rs:164-196`
- `compute_clip_duration`（pipeline.rs）优先 TTS 时长；`calculate_clip_duration`（audio.rs）优先显式 duration。同一片段在 pipeline 不同步骤可能得到不同时长，导致音画不同步。
- **修复**: 统一为一个共享函数。

### WR-11: FFmpeg 错误处理不一致
- **文件**: `clip.rs`（严格）/ `audio.rs`、`pipeline.rs`（宽松）
- `clip.rs` 检测 `FfmpegEvent::Error` 设 `had_errors=true`；`audio.rs` 和 `pipeline.rs` 的 concat/composite 只记录日志不报错。FFmpeg 可能 stderr 报错但退出码为 0。
- **修复**: 统一为严格模式，或提取公共 FFmpeg 事件处理函数。

### WR-12: `strip_and_repair_json` 的 `{{`→`{` 替换破坏合法 JSON
- **文件**: `src/documentary/script_gen.rs:309`
- `{"a": {"b": 1}}` 中 `}}` 是合法的右花括号对，替换后变成 `{"a": {"b": 1}`，结构损坏。
- **修复**: 只在首次 `serde_json::from_str` 失败后尝试修复，并使用更精确的模式匹配。

### WR-13: `extract_json_object` 不考虑字符串内的花括号
- **文件**: `src/documentary/script_gen.rs:321-337`
- 简单深度计数在字符串包含 `{`/`}` 时失效（如 `{"text": "use {brackets}"}`）。
- **修复**: 在计数花括号时跟踪是否在字符串内部。

### WR-14: `amix` 滤镜 inputs=1 不合法
- **文件**: `src/documentary/audio.rs:63-66`
- 当没有 TTS 音频且只有静音源时，`amix=inputs=1` 会导致 FFmpeg 报错。
- **修复**: 只有一个输入时跳过 `amix`，直接使用原始音频流。

### WR-15: `audio.rs` FFmpeg Error 事件只记录不传播
- **文件**: `src/documentary/audio.rs:97-101`
- 与 WR-11 相关但独立：`audio.rs` 甚至没有 `had_errors` 变量，Error 事件被静默吞掉。
- **修复**: 添加 `had_errors` 标志，与 `clip.rs` 保持一致。

### WR-16: `clip.rs` 未使用的 `probe_video` 导入
- **文件**: `src/documentary/clip.rs:10`
- `use crate::ffmpeg::probe::probe_video;` 被导入但从未使用。
- **修复**: 删除此行。

---

## INFO Findings

### IN-01: `secs_to_srt_time` / `secs_to_ffmpeg_time` 代码重复
- **文件**: `src/documentary/timestamp.rs:72-93`
- 两个函数仅毫秒分隔符不同（`,` vs `.`），其余逻辑完全相同。
- **修复**: 提取 `secs_to_time_str(secs: f64, millis_sep: char)` 内部函数。

### IN-02: `parse_timestamp_range` 应使用 `splitn(2, '-')`
- **文件**: `src/documentary/timestamp.rs:7`
- `split('-')` 在输入含多个 `-` 时可能意外分割。`splitn(2, '-')` 更安全。
- **修复**: 改为 `input.splitn(2, '-')`。

### IN-03: `clip_id: i64` 与 `index: u32` 类型不一致
- **文件**: `src/documentary/types.rs:70` vs `src/documentary/subtitle.rs`
- TtsResult.clip_id 用 i64，SRT 序号用 u32，语义可能混淆。
- **修复**: 统一类型或确认语义差异并文档化。

### IN-04: `ProgressStep` 缺少 `Hash` trait
- **文件**: `src/documentary/types.rs:54`
- 未来如需在 HashMap 中使用需手动添加。
- **修复**: 添加 `Hash` 到 derive 列表。

### IN-05: 空 segments 输入返回空字符串而非错误
- **文件**: `src/documentary/subtitle.rs:43`
- `merge_srt_files(&[])` 返回 `Ok("")`，可能掩盖上游逻辑错误。
- **修复**: 考虑对空输入返回错误。

### IN-06: `write_srt_file` 路径未做安全验证
- **文件**: `src/documentary/subtitle.rs:60-66`
- `create_dir_all` 会递归创建目录。路径来自不可信输入时存在路径遍历风险。
- **修复**: 验证路径在预期工作目录内。

### IN-07: Concat 列表路径单引号未转义
- **文件**: `src/documentary/pipeline.rs:183`
- `file '{}'\n` 中路径包含单引号会导致 concat demuxer 解析失败。
- **修复**: 对单引号进行转义。

### IN-08: OST=1 片段 end 时间未直接从 timestamp 范围获取
- **文件**: `src/documentary/pipeline.rs:108-133`
- `end_secs` 由 `start_secs + clip_duration` 计算，而非使用 timestamp 范围的结束值。
- **修复**: OST=1 时直接使用范围结束值。

### IN-09: 关键帧缓存回退到当前工作目录
- **文件**: `src/documentary/script_gen.rs:38-41`
- `video_path.parent()` 为 None 时回退到 `"."`，缓存可能写入意外位置。
- **修复**: 使用任务专用临时目录。

### IN-10: 字幕偏移双重应用风险
- **文件**: `src/documentary/audio.rs:137-144` + `src/documentary/subtitle.rs:41-56`
- `generate_srt_from_word_boundaries` 已内嵌偏移，`merge_srt_files` 的 `offset_secs` 设为 0.0 避免了双重偏移，但设计脆弱。
- **修复**: 选择一种偏移策略并保持一致。

### IN-11: 公共 API 暴露所有内部模块
- **文件**: `src/lib.rs:1-13`
- 13 个子模块全部 public，内部结构成为公共 API 的一部分。
- **修复**: 使用 `pub(crate)` 限制内部模块，选择性 `pub use` 导出核心类型。

### IN-12: 忽略的 FFmpeg 测试体为空
- **文件**: `tests/documentary_integration_test.rs:167-203`
- 4 个 `#[ignore]` 测试只有空体和冗余的 `ffmpeg_available()` 检查。
- **修复**: 为测试添加实际主体或删除。

### IN-13: 测试 WordBoundary 数值使用非标准分组
- **文件**: `tests/documentary_integration_test.rs:50-72` + `src/documentary/subtitle.rs:162`
- `5_000_000_0`、`500_000_0` 等非标准分组降低可读性。
- **修复**: 改为标准千位分组 `50_000_000`、`5_000_000`。

---

## Priority Fix Order

1. **WR-01** + **WR-04** — 时间戳负数溢出链（最可能在实际运行中触发）
2. **WR-07** + **WR-08** — FFmpeg 命令冲突（可能导致合成失败）
3. **WR-10** — 时长计算不一致（可能导致音画不同步）
4. **WR-12** + **WR-13** — JSON 修复破坏合法结构
5. **WR-11** + **WR-15** — FFmpeg 错误处理不一致
6. **WR-14** — `amix` inputs=1 边界情况
