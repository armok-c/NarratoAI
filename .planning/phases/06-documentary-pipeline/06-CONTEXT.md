# Phase 6: Documentary Pipeline - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Phase Boundary

纪录片模式完整流水线——Phase 1-5 所有模块在此汇合的旗舰功能。包含两部分：

1. **6 步视频处理流水线**（加载脚本 → TTS 生成 → 视频裁剪 → 音频字幕合并 → 片段拼接 → 最终合成），按 OST 类型差异化处理音频轨道，输出最终视频文件。

2. **帧分析→文案生成预处理**（独立于流水线），从视频中提取关键帧、调用 LLM 生成解说脚本，供用户编辑后传入流水线。

覆盖需求：FFMP-02, FFMP-03, FFMP-04, FFMP-05, FFMP-06, FFMP-07, FFMP-08, DOCS-01, DOCS-02, DOCS-03

</domain>

<decisions>
## Implementation Decisions

### 流水线编排结构
- **D-01:** Step-by-Step 函数链——每步一个独立 async 函数（step_load_script, step_tts, step_clip, step_merge_audio, step_concat, step_composite），主函数 `run_documentary()` 顺序调用。不做 Pipeline trait 抽象，不做状态机驱动。
- **D-02:** 可变 PipelineState 结构体——定义 `struct PipelineState`，每步接收 `&mut PipelineState` 并更新字段。包含：script（Vec<ScriptClip>）、tts_results、video_clips、merged_audio_path、merged_subtitle_path、combined_video_path 等中间产物。与 Python 版行为最接近。
- **D-03:** DocumentaryRequest 参数结构体——入口函数接收 `DocumentaryRequest`（video_path, script_path, tts_engine, voice_name, 音量参数, 字幕配置, BGM 配置等）+ `Option<ProgressCallback>`。与 Phase 9 的 ExportRequest 模式一致。
- **D-04:** 代码放在 `src/documentary/` 模块。pipeline.rs 包含 PipelineState 和主编排函数，各 step 函数可在同一文件或拆分为 step_*.rs。
- **D-05:** 不做跨模式抽象——Phase 6 先做出能用的纪录片流水线。Phase 7/8 实现时发现重复逻辑再抽取到 `src/pipeline/common.rs`。避免过早抽象。

### 错误恢复策略
- **D-06:** 完全 fail-fast——任何步骤失败立即返回 PipelineError，不继续处理。与 Python 版行为 1:1 对齐。
- **D-07:** 单一 PipelineError 枚举——包含各步骤变体（ScriptLoad, TtsGeneration, VideoClip, AudioMerge, Concat, Composite），内嵌具体领域错误。通过 `From<DomainError>` 自动转换，调用方只需处理 PipelineError。
- **D-08:** 中间产物不清理——流水线失败时保留 task_dir 下所有中间文件（TTS 音频、视频片段等），便于调试和手动恢复。与 Python 版一致。

### 字幕生成流程
- **D-09:** TTS 后即时生成 SRT 片段——TTS 生成返回 word boundary 时间戳，即时为每段生成 SRT 片段。step_tts 返回的 TtsResult 包含音频文件路径 + 字幕片段文件路径。与 Python 版行为一致。
- **D-10:** 字幕烧录在最终合成步骤——步骤 6 的 `step_composite()` 通过 FFmpeg subtitles filter 一次性合并视频+音频+BGM+字幕烧录。不做单独烧录步骤（避免多一次视频编码的质量损失）。
- **D-11:** 对齐 Python 版字幕合并逻辑——每段字幕有时间偏移（基于前面所有片段的累计时长），合并为单个 SRT 文件。1:1 对齐 `subtitle_merger.py`。

### 帧分析→文案生成编排
- **D-12:** 独立预处理函数——`generate_documentary_script(video_path, config, callback) -> Result<Script>` 与 6 步流水线解耦。用户先调用它生成脚本，编辑后传入 `run_documentary()`。与 Python 版工作流一致（先分析→编辑→再生成视频）。
- **D-13:** 直接输出 Script——预处理函数内部两步调用（帧分析→LLM 文案生成），最终输出 `Vec<ScriptClip>`（完整脚本）。用户可直接保存或编辑后传给流水线。
- **D-14:** 两步 LLM 调用——先调 Phase 4 的 frame_analysis Prompt 分析帧内容，再调 narration_generation Prompt 根据帧分析结果生成解说文案。与 Python 版 `frame_analysis_service.py` 流程一致。
- **D-15:** LLM 输出 OST + timestamp——Prompt 要求 LLM 同时输出每段的 OST 类型和 timestamp。与 Python 版 `generate_narration_script.py` 一致。

### Claude's Discretion
- PipelineState 的具体字段定义（哪些 Option、哪些默认值）
- step 函数的拆分粒度（每个 step 独立文件 vs. 统一放在 pipeline.rs）
- DocumentaryRequest 的具体字段和默认值设计
- PipelineError 枚举的具体变体和错误消息
- SRT 生成和合并的具体实现细节
- 帧分析预处理函数的参数和中间结果管理
- OST=0/1/2 分支在视频裁剪和混音中的 FFmpeg filter chain 构建方式

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版核心流水线参考
- `app/services/task.py` — ★ 核心流水线编排（start_subclip_unified），6 步完整流程的权威参考
- `app/services/clip_video.py` — ★ 视频裁剪（clip_video_unified），OST=0/1/2 三种裁剪策略
- `app/services/merger_video.py` — 视频片段拼接（combine_clip_videos），concat demuxer
- `app/services/generate_video.py` — 最终合成（merge_materials），音频+BGM+字幕烧录一体化
- `app/services/voice.py` — TTS 路由和批量生成（tts_multiple）
- `app/services/audio_merger.py` — 音频文件合并
- `app/services/subtitle_merger.py` — 字幕文件合并（时间偏移计算）
- `app/services/update_script.py` — 管道处理后脚本字段更新

### Python 版帧分析→文案生成参考
- `app/services/documentary/frame_analysis_service.py` — 帧分析→文案生成编排流程
- `app/services/documentary/frame_analysis_models.py` — 帧分析数据模型
- `app/services/generate_narration_script.py` — 解说文案生成，LLM 输出脚本格式

### Rust 代码库依赖
- `src/script/types.rs` — ScriptClip + OstType 数据模型
- `src/script/mod.rs` — load_script/save_script/validate
- `src/tts/mod.rs` — TtsProvider trait + synthesize 路由器
- `src/ffmpeg/mod.rs` — FFmpeg spawn_blocking 封装
- `src/ffmpeg/probe.rs` — ffprobe 视频探测
- `src/ffmpeg/command.rs` — FFmpeg 命令构建
- `src/llm/mod.rs` — LlmProvider trait
- `src/llm/types.rs` — LLM 请求/响应类型
- `src/prompt/mod.rs` — Prompt 系统
- `src/prompt/manager.rs` — PromptManager（获取模板、渲染）
- `src/visual/mod.rs` — 视觉分析器模块
- `src/visual/analyzer.rs` — 批量帧分析
- `src/visual/frame_extractor.rs` — 帧提取（4 级回退）
- `src/config/types.rs` — 配置类型
- `src/lib.rs` — 库入口

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库、文件存储、不可变模式）
- `.planning/REQUIREMENTS.md` — Phase 6 需求：FFMP-02~08, DOCS-01~03
- `.planning/ROADMAP.md` — Phase 6 定义和成功标准
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（FFmpeg 模式、错误处理、spawn_blocking、ProgressCallback）
- `.planning/phases/05-script-management/05-CONTEXT.md` — Phase 5 决策（ScriptClip 模型、OstType 枚举、校验策略）
- `.planning/phases/04-prompt-system-visual-analyzer/04-CONTEXT.md` — Phase 4 决策（帧提取、视觉分析、Prompt Registry）

### 代码库映射
- `.planning/codebase/STRUCTURE.md` — Python 版目录结构
- `.planning/codebase/CONVENTIONS.md` — Provider/Registry 模式、编码约定
- `.planning/codebase/INTEGRATIONS.md` — LLM/TTS/FFmpeg 集成点

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/tts/mod.rs` — Phase 3 TtsProvider trait + synthesize 路由器，step_tts 直接调用
- `src/ffmpeg/mod.rs` — Phase 1 FFmpeg spawn_blocking 封装，视频裁剪/拼接/合成复用
- `src/ffmpeg/probe.rs` — ffprobe 探测能力，获取视频时长/音频时长
- `src/script/types.rs` — ScriptClip + OstType，流水线核心数据模型
- `src/script/mod.rs` — load_script，步骤 1 直接调用
- `src/llm/mod.rs` — LlmProvider trait，文案生成调用 analyze_images/generate_text
- `src/prompt/manager.rs` — PromptManager，获取并渲染文案生成模板
- `src/visual/analyzer.rs` — 批量帧分析，预处理函数直接调用
- `src/visual/frame_extractor.rs` — 帧提取，预处理函数直接调用
- `src/config/types.rs` — AppConfig 各配置段，读取 TTS/FFmpeg/字幕参数

### Established Patterns
- **模块级结构（Phase 1 D-02）**：Phase 6 创建 `src/documentary/` 模块，由 `src/lib.rs` 导出
- **领域错误枚举（Phase 1 D-16/D-17）**：PipelineError 使用 thiserror 派生
- **中文用户可见错误信息（Phase 1 D-18）**：PipelineError 的 Display 实现用中文
- **ProgressCallback 回调（Phase 1 D-15）**：流水线每步完成后上报进度百分比和步骤描述
- **显式初始化函数（Phase 2 D-*）**：register_all_providers/register_all_prompts 模式
- **Arc<RwLock<T>> 全局实例**：Config/Registry 模式

### Integration Points
- `src/documentary/mod.rs` — 新模块入口，由 `src/lib.rs` pub mod 导出
- `src/tts/mod.rs` — step_tts 调用 synthesize 路由器
- `src/ffmpeg/mod.rs` — step_clip/step_concat/step_composite 调用 FFmpeg 封装
- `src/script/mod.rs` — step_load_script 调用 load_script
- `src/visual/` — generate_documentary_script 调用帧提取和视觉分析
- `src/prompt/` — 文案生成调用 PromptManager 获取并渲染模板
- `src/llm/` — 文案生成调用 LlmProvider
- `src/lib.rs` — 新增 `pub mod documentary;`
- `Cargo.toml` — 可能需新增依赖（如 tempfile 用于任务目录管理）

</code_context>

<specifics>
## Specific Ideas

- Python 版 `task.py` 中步骤 6 的 FFmpeg 命令极为复杂（同时处理音频混合+BGM+字幕烧录），Rust 版需仔细对齐 filter chain 构建
- Python 版 `clip_video.py` 中 OST=0 裁剪按 TTS 音频时长动态调整视频长度（非按 timestamp），OST=1 严格按 timestamp 裁剪，OST=2 按 TTS 时长调整但保留原声——这三种策略必须在 Rust 版精确复现
- Python 版通过 `update_script.update_script_timestamps()` 将裁剪后的实际时长和文件路径回写到脚本，PipelineState 需要对齐此更新逻辑
- 帧分析预处理函数需注意 LLM 输出的脚本可能需要校验（Phase 5 的 validate 函数可复用）
- 流水线的进度百分比分配应与 Python 版对齐：步骤 1=0%, 步骤 2=20%, 步骤 3=60%, 步骤 4=60%, 步骤 5=80%, 步骤 6=80%, 完成=100%

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-documentary-pipeline*
*Context gathered: 2026-05-02*
