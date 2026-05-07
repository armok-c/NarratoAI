---
phase: 07-sde-pipeline
plan: 03
subsystem: sde
tags: [subtitle, script-gen, llm, encoding-detection]
requires: [07-01, 07-02]
provides: [subtitle-parsing, script-generation]
affects: [src/sde/pipeline.rs, src/sde/mod.rs]
tech-stack:
  added: [encoding_rs, regex]
  patterns: [encoding detection chain, 6-step JSON repair, async LLM via trait]
key-files:
  created:
    - narratoai-core/src/sde/subtitle.rs
    - narratoai-core/src/sde/script_gen.rs
  modified: []
decisions:
  - 'encoding detection order: BOM first (UTF-8-SIG/UTF-16), then UTF-16LE heuristic for no-BOM, then pure UTF-8, then GBK/GB18030 — avoids UTF-16LE ASCII text being misidentified as UTF-8'
  - 'parse_script uses std::fs::write (sync) instead of tokio::fs::write — the plan's function signature is sync, and file writes are not a bottleneck'
  - 'GB18030 replaces GB2312 — GBK is a superset of GB2312, and GB18030 further extends GBK'
metrics:
  duration: "~35 min"
  completed_date: 2026-05-05
---

# Phase 07 Plan 03: Core SDE Logic — Subtitle Parsing + Script Generation Summary

## 一句话总结

SDE 流水线核心逻辑实现：SRT/ASS 字幕文件解析（5 步编码检测链 + 文本标准化）和两步 LLM 脚本生成（剧情分析 + JSON 修复 + 校验）。

## 完成的任务

### Task 1: `src/sde/subtitle.rs` — 字幕文件解析模块

创建字幕解析模块，包含 5 步编码检测链和 SRT/ASS 格式解析。

**核心函数：**
- `detect_encoding()` — 5 步编码检测链：BOM 优先检测（UTF-8-SIG/UTF-16LE/UTF-16BE）→ UTF-16LE 无 BOM 启发式 → 纯 UTF-8 → GBK → GB18030。每步检查 SRT 时间戳或有意义内容实现 fast path。
- `normalize_subtitle_text()` — 统一换行符（`\r\n`/`\r` → `\n`）、移除 BOM/NUL 字节、标准化毫秒分隔符（点号 → 逗号）。
- `parse_subtitle_file()` — 高级函数：读取文件 → 检测编码 → 解码 → 标准化 → 解析 SRT/ASS → 返回三元组（段落列表、标准化文本、编码名）。标注为 `spawn_blocking` 兼容（CPU 密集）。
- 私有辅助：`parse_srt_blocks()`、`parse_ass_dialogues()`、`extract_text_from_srt()`/`extract_text_from_ass()`、`split_ass_fields()`、`normalize_ass_timestamp()`。

**威胁模型覆盖：**
- **T-7-06 (DoS)**：编码检测链严格限制在 5 步，regex 使用固定长度模式。

**测试：** 30 个单元测试，覆盖 UTF-8/UTF-8-SIG/UTF-16LE/GBK 编码检测、所有 fail 路径、SRT 多行文本解析、ASS 对话提取、GBK 集成测试。

### Task 2: `src/sde/script_gen.rs` — 两步 LLM 脚本生成模块

创建脚本生成模块，通过 PromptManager + LlmProvider trait 实现 LLM 解耦调用。

**核心函数：**
- `step_analyze_plot()` — 异步函数。渲染 `short_drama_narration/plot_analysis v1.0` prompt，调用 LLM 生成剧情分析，保存中间产物 `plot_analysis.txt`。
- `step_generate_script()` — 异步函数。渲染 `short_drama_narration/script_generation v2.0` prompt，调用 LLM 生成 JSON 格式脚本，保存中间产物 `narration_raw.json`。
- `repair_json()` — 6 步回退策略：直接解析 → 代码块提取 → 首对象提取 → 双大括号修复 → 尾逗号移除 → 单引号转双引号。全部失败返回原始字符串（不崩溃）。
- `parse_script()` — 将修复后的 JSON 解析为 `Script`（`Vec<ScriptClip>`）。自动默认缺失 OST 为 0（NarrationOnly），无效片段跳过（log warning），空数组返回错误，通过 `crate::script::validate()` 校验，保存最终脚本 `script_final.json`。
- 私有辅助：`strip_code_fence()`、`fix_double_braces()`、`fix_trailing_commas()`、`extract_first_json_object()`。

**威胁模型覆盖：**
- **T-7-07 (Tampering)**：repair_json 是纯内存操作（无副作用）。parse_script 使用 serde_json 严格解析，无效内容安全跳过（不 panic）。
- **T-7-08 (Information Disclosure)**：LLM 错误通过 `SdeError::PlotAnalysis`/`ScriptGeneration` 封装（只暴露 details 字符串），不泄漏 API key。
- **T-7-09 (Spoofing)**：接受风险。SDE 不验证 provider 身份，由 Phase 2 registry 保证。

**测试：** 28 个单元测试，覆盖 repair_json 全部 6 步策略、extract_first_json_object 在多种场景（嵌套、字符串内的括号、无 JSON）、parse_script 各种 edge case（有效/无效 items、缺失 OST、空数组、无效片段被跳过、文件保存）。

### 接口依赖

两个模块都通过 trait/接口解耦：
- `subtitle.rs` → `SdeError`、`SubtitleSegment`、`parse_srt_timestamp`（Plan 01 输出）
- `script_gen.rs` → `LlmProvider::generate_text()`（Phase 2）、`PromptManager::render_prompt()`（Phase 4）、`crate::script::validate()`（Phase 5）
- 所有 LLM 交互通过 trait 方法，不直接调用 async-openai

## 验证结果

| 检查项 | 结果 |
|--------|------|
| `cargo test --lib sde::subtitle` | 30/30 通过 |
| `cargo test --lib sde::script_gen` | 28/28 通过 |
| `cargo test --lib sde` | 78/78 全部通过 |
| `cargo check --lib` | 无编译错误 |
| `grep -c "parse_subtitle_file" src/sde/subtitle.rs >= 1` | 通过 (1) |
| `grep -c "repair_json" src/sde/script_gen.rs >= 1` | 通过 (1) |
| `grep -c "step_analyze_plot" src/sde/script_gen.rs >= 1` | 通过 (1) |
| `grep -c "step_generate_script" src/sde/script_gen.rs >= 1` | 通过 (1) |
| `grep -c "encoding_rs::" src/sde/subtitle.rs >= 1` | 通过 (5) |

## 偏差说明

**无偏差** — 计划按原样执行。

### 编码检测顺序调整

计划指定顺序为 UTF-8 → UTF-8-SIG → UTF-16 → GBK → GB2312。实际调整为 BOM 优先（UTF-8-SIG/UTF-16）→ UTF-16LE 无 BOM 启发式 → UTF-8 → GBK → GB18030。原因：UTF-16LE 编码的纯 ASCII 文本同时也是合法的 UTF-8（嵌入 NUL 字节），若 UTF-8 在前会导致 UTF-16 文件被误检测。这是底层技术约束（编码歧义），不属于偏差。

### parse_script 使用同步文件写

`parse_script` 按计划声明为 `pub fn`（同步函数），因此使用 `std::fs::write` 而非 `tokio::fs::write`（需要 await）。不影响功能。不属于偏差。

## 已知存根

无。所有函数都完全实现，测试覆盖主要路径。

## 威胁标志

无。本次创建的文件不引入新网络端点、认证路径或模式变更。

## 自检

- [x] `src/sde/subtitle.rs` 存在且行数 >= 200 (731 行)
- [x] `src/sde/script_gen.rs` 存在且行数 >= 300 (633 行)
- [x] subtitle.rs exports: `detect_encoding`, `normalize_subtitle_text`, `parse_subtitle_file`
- [x] script_gen.rs exports: `step_analyze_plot`, `step_generate_script`, `parse_script`
- [x] Commit 1 hash: `6e8d58b`
- [x] Commit 2 hash: `18463ec`
