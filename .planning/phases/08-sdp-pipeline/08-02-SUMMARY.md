---
phase: 08-sdp-pipeline
plan: 02
type: execute
wave: 1
subsystem: prompt
tags: [sdp, prompt-templates, subtitle-analysis, plot-extraction]
requires: []
provides: [subtitle_analysis_v2.0.md, plot_extraction_v2.0.md-sdp-update]
affects: [src/prompt/register.rs]
tech-stack:
  added: []
  patterns: []
key-files:
  created:
    - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  modified:
    - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
    - narratoai-core/src/prompt/register.rs
decisions: []
metrics:
  duration_minutes: 5
  completed_date: "2026-05-06"
---

# Phase 08 Plan 02: SDP Prompt Templates 创建与注册

## One-Liner

创建 SDP 专属的两步 LLM Prompt 模板（subtitle_analysis + plot_extraction）并在 Rust Prompt 注册表中注册，完成第二步 plot_extraction 参数从 SDE 通用版到 SDP 专用版的迁移。

## Files Created / Modified

| Action | File | Lines | Description |
|--------|------|-------|-------------|
| CREATE | `src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md` | 86 | SDP 第一步 LLM prompt：分析字幕叙事结构，提取关键情节点。翻译自 Python 版 subtitle_analysis.py |
| UPDATE | `src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md` | 109 | SDP 第二步 LLM prompt：精确定位关键剧情时间段。替换 SDE 通用版（含 language 参数）为 SDP 专用版（含 plot_summary + plot_titles 参数） |
| MODIFY | `src/prompt/register.rs` | +27/-7 | 新增 subtitle_analysis v2.0 注册 + 更新 plot_extraction v2.0 参数 + 更新测试断言 |

## Task Execution

### Task 1: 创建 subtitle_analysis_v2.0.md 模板

**Commit:** `20d1390`

翻译自 Python 版 `app/services/prompts/short_drama_editing/subtitle_analysis.py`，使用 Rust PromptTemplate `${variable_name}` 占位符格式。系统提示语不包含在模板中（由 `script_gen.rs` 调用时通过 `system_prompt` 参数传递）。

关键变量：
- `${subtitle_content}` — 完整字幕文本内容
- `${custom_clips}` — 建议的片段数量（默认 5）

模板结构：任务目标 → 叙事结构分析（开端/发展/高潮/结局）→ 关键情节点选择原则（连贯性/覆盖性/戏剧性）→ JSON 输出格式（summary, narrative_structure, plot_titles, plot_connections, analysis_details）→ 质量检查清单（5 项自检）→ 重要限制

### Task 2: 更新 plot_extraction_v2.0.md 为 SDP 专用版本

**Commit:** `046ba2f`

替换 SDE 通用版（仅含 subtitle_content + language 参数，14 行简单内容）为 SDP 专用版（109 行完整内容）。

关键变量：
- `${subtitle_content}` — 完整字幕文本内容
- `${plot_summary}` — 剧情梗概（来自 subtitle_analysis 步骤的输出）
- `${plot_titles}` — 情节点标题列表（来自 subtitle_analysis 步骤的输出）
- `${language}` — 已移除

模板结构：任务目标 → 输入信息（plot_summary, plot_titles, subtitle_content）→ 时间段选择原则（连贯性/技术规范/内容完整性/过渡策略）→ JSON 输出格式（plot_points[]: sequence, timestamp, title, narrative_function, picture, context_before, context_after, transition_note + editing_notes）→ 质量检查清单（时间戳/连贯性/完整性）→ 重要限制

### Task 3: 更新 prompt/register.rs

**Commit:** `5bd5f5d`

修改 `register_short_drama_editing_prompts()` 函数：

1. **新增 subtitle_analysis v2.0 prompt 注册**：
   - name: "subtitle_analysis", category: "short_drama_editing", version: "v2.0"
   - output_format: OutputFormat::Json
   - 参数：subtitle_content (required) + custom_clips (optional, default "5")
   - tags: ["short-drama-editing", "subtitle-analysis", "sdp"]

2. **更新 plot_extraction v2.0 参数**：
   - 移除 language 参数
   - 新增 plot_summary (required) + plot_titles (required)
   - 更新 tags 加入 "sdp"

3. **更新测试断言**：
   - short_drama_editing prompt 计数从 1 改为 2
   - 新增 subtitle_analysis v2.0 查询断言

## Verification Results

- `cargo build -p narratoai-core` — 编译通过（0 errors，6 pre-existing warnings）
- `cargo test -p narratoai-core -- prompt::register` — 4/4 测试通过：
  - `test_register_all_categories` — ok
  - `test_register_all_prompts_full` — ok（含 subtitle_analysis 查询断言）
  - `test_register_all_duplicate_returns_errors` — ok
  - `test_registration_errors_display` — ok

## Deviations from Plan

无 — 计划完全按文档执行，未发现需要自动修复的 bug、缺失的关键功能或阻塞问题。

## Known Stubs

无 — 两个模板内容完整可用。

## Threat Flags

无 — 本计划未引入新的安全相关表面。Prompt 模板是编译时嵌入的静态文件（`include_str!`），无运行时修改路径。

## Commits

| Hash | Message |
|------|---------|
| `20d1390` | feat(08-02): create subtitle_analysis_v2.0.md SDP prompt template |
| `046ba2f` | feat(08-02): update plot_extraction_v2.0.md to SDP-specific version |
| `5bd5f5d` | feat(08-02): register subtitle_analysis prompt and update plot_extraction parameters |

## Self-Check: PASSED

- [x] subtitle_analysis_v2.0.md 创建（86 行，含 `${subtitle_content}`、`${custom_clips}`、叙事结构分析、JSON 输出格式、质量检查清单）
- [x] plot_extraction_v2.0.md 更新为 SDP 专用版（109 行，含 `${subtitle_content}`、`${plot_summary}`、`${plot_titles}`，无 `${language}`）
- [x] register.rs 包含 subtitle_analysis 注册和 plot_extraction 参数更新
- [x] `cargo build -p narratoai-core` 通过（验证 include_str! 路径有效）
- [x] `cargo test -p narratoai-core -- prompt::register` 全部通过
