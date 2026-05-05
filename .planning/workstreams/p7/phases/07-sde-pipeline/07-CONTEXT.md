# Phase 7: SDE Pipeline - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning

<domain>
## Phase Boundary

短剧解说模式完整流水线 — 从字幕文件（SRT/ASS）解析中文对话，经由 LLM 两步分析（剧情提炼 → 解说脚本生成），输出带 OST 标记的 JSON 脚本，最后复用 Phase 6 的视频处理步骤（TTS → 裁剪 → 合并 → 拼接 → 合成）输出最终解说视频。

与纪录片的本质差异：脚本来源于字幕文本分析而非视频帧分析。输入是字幕文件而非视频帧。输出解说视频中 OST=1（原声）占比约 70%，OST=0（解说）占比约 30%。

覆盖需求：SDE-01 (流水线编排), SDE-02 (字幕解析), SDE-03 (LLM 剧情分析)

</domain>

<decisions>
## Implementation Decisions

### 模块结构
- **D-01:** 独立 `src/sde/` 模块 — 创建 `src/sde/` 目录，视频处理步骤（TTS→裁剪→合并→拼接→合成）直接调用 `src/documentary/` 的已有函数，不重复实现。符合 Phase 6 D-05 决策（不做跨模式抽象，Phase 7/8 实现时发现重复再抽取 common.rs）。
- **D-02:** 镜像纪录片 8 文件结构 — `script_gen.rs` (LLM分析+脚本生成)、`subtitle.rs` (字幕读取/编码探测/文本提取)、`pipeline.rs` (run_sde 主编排)、`clip.rs` (视频裁剪封装)、`audio.rs` (音频合并封装)、`error.rs` (SdeError)、`types.rs` (SdeRequest/SdePipelineState/进度步骤)、`timestamp.rs` (时间戳工具) + `mod.rs` (公开 API)。
- **D-03:** 暴露 Phase 6 私有函数 — Phase 6 的 `src/documentary/pipeline.rs` 中 `step_tts`/`step_clip`/`step_merge_audio_subtitle`/`step_concat`/`step_composite` 标记为 `pub(crate)`，SDE 模块通过 `use crate::documentary::pipeline::step_*` 直接调用。
- **D-04:** 新建 `SdeRequest` — 包含 `subtitle_path: PathBuf` + `video_path: PathBuf` + `drama_name: String` + 步骤 2-6 相关的所有参数（tts_engine, voice_name, voice_rate, voice_pitch, 音量参数, 字幕配置, bgm_path 等），字段名和默认值与 `DocumentaryRequest` 完全对齐。
- **D-05:** 独立 9 步进度 — `SdeProgressStep` 枚举：ParseSubtitle → PlotAnalysis → ScriptGenerate → LoadScript → Tts → Clip → MergeAudio → Concat → Composite。每步有独立进度上报。
- **D-06:** 原子入口 `run_sde()` — 接收 `SdeRequest`，内部依次完成字幕解析→LLM剧情分析→LLM脚本生成→JSON解析修复→视频处理（步骤 2-6），返回最终视频路径。与 `run_documentary()` 模式一致。
- **D-07:** SdeRequest 含字幕路径 — `run_sde()` 接收字幕文件路径，内部自动完成脚本生成。高级用户可单独调用 `analyze_subtitle_plot()` 和 `generate_sde_script()` 两个独立公开函数。
- **D-08:** 字幕解析在 SDE 内部实现 — 编码探测和文本提取逻辑放在 `src/sde/subtitle.rs`，参考 Python 版 `app/services/subtitle_text.py` 的编码探测链（utf-8 → utf-16 → gbk → gb2312），不为 Phase 8 提前抽取公共模块。
- **D-09:** 完全对齐 DocumentaryRequest — SdeRequest 的视频处理参数（TTS引擎、音量、字幕配置等）字段名、类型和默认值与 `DocumentaryRequest` 一一对应，仅新增 SDE 特有字段（subtitle_path, drama_name, temperature）。
- **D-10:** 文件职责按纪录片镜像 — `script_gen.rs` 包含两步 LLM 调用（`analyze_subtitle_plot` + `generate_sde_script`）和 JSON 修复逻辑；`subtitle.rs` 包含字幕文件读取/编码探测/文本提取；`pipeline.rs` 包含 `SdePipelineState` + `run_sde()` 主编排 + step 函数；`clip.rs`/`audio.rs` 封装对纪录片对应步骤的调用；`timestamp.rs` 提供 SRT 时间戳解析工具。
- **D-11:** 新 `SdeError` 枚举 + 嵌入 `PipelineError` — 包含变体：`ParseSubtitle` / `PlotAnalysis` / `ScriptGeneration` / `JsonRepair` / `VideoProcess(PipelineError)` / `Validation`。视频处理错误直接嵌入 Phase 6 的 `PipelineError`，通过 `From<PipelineError>` 自动转换。
- **D-12:** JSON 修复在 SDE 内部 — LLM 输出 JSON 的修复逻辑（双大括号→单大括号、提取 code block、尾逗号修复、大括号提取）作为 `script_gen.rs` 的私有函数 `repair_json()`，与 Python 版 `parse_and_fix_json()` 对齐。
- **D-13:** 独立 `SdePipelineState` — 包含 `script: Script` + `subtitle_text: String` + `plot_analysis: String` + `narration_raw: String` + `tts_results/task_dir/video_clips` 等步骤 2-6 所需字段。字段集与 `DocumentaryPipelineState` 不同但结构类似。

### LLM 编排
- **D-14:** 统一使用 text 模型 — 两步 LLM 调用都使用配置中的 text_llm 提供商和模型，不引入 SDE 专属模型配置。与 Python 版行为一致。
- **D-15:** 纯字符串传递 — `analyze_subtitle_plot()` 返回 LLM 原始文本输出（剧情分析摘要），`generate_sde_script()` 接收此字符串作为 `${plot_analysis}` 参数渲染 Prompt 模板。中间不做结构化解析。
- **D-16:** 返回原始字符串 — `generate_sde_script()` 返回 `Result<String>`（LLM 原始 JSON 输出），调用方（`run_sde()` 或外部用户）自行调用 JSON 解析/修复逻辑。
- **D-17:** 保存所有中间产物 — 剧情分析摘要、LLM 原始响应（JSON）、修复后的最终脚本均写入 task_dir 下的文件。便于调试和手动恢复。与 Python 版 `save_result=True` 行为一致。
- **D-18:** 通过 PromptManager 加载 — 使用 Phase 4 的 `PromptManager` 获取 `short_drama_narration/plot_analysis` (v1.0) 和 `short_drama_narration/script_generation` (v2.0) 模板。SDE 不直接实例化或硬编码 Prompt。
- **D-19:** 统一 temperature 控制 — SdeRequest 包含单个 `temperature: f64` 字段（默认 1.0），两步 LLM 调用共用。与 Python 版 `SubtitleAnalyzer.__init__(temperature=1.0)` 一致。
- **D-20:** LLM 调用完全 fail-fast — 任一步 LLM 调用失败立即返回 `SdeError`，不重试（由 Phase 2 的 retry 机制处理重试逻辑），中间产物保留在 task_dir 供调试。与 Phase 6 D-06 决策一致。
- **D-21:** `run_sde()` 内部串联 — 原子入口内部依次调用：`step_parse_subtitle()` → `step_analyze_plot()` → `step_generate_script()` → JSON 解析修复 → 步骤 2-6 视频处理。外部只需调一个函数。

### 数据模型
- **D-22:** Python 版 1:1 字段映射 — LLM JSON 输出的 `_id` → `ScriptClip._id`、`timestamp` → `timestamp`、`picture` → `picture`、`narration` → `narration`、`OST` → `ost`（0→NarrationOnly, 1→OriginalSound）。映射在 `script_gen.rs` 的 JSON 解析函数中完成。
- **D-23:** 字符串前缀检测 OST=1 — 视频处理步骤检测 `narration` 字段是否以 `"播放原片"` 开头，若是则跳过该片段的 TTS 生成。不做模型字段扩展，纯运行时逻辑判断。
- **D-24:** 复用 Phase 5 脚本校验 — SDE 生成的脚本通过 Phase 5 的 `validate_script()` 校验必要字段和 timestamp 合理性，不新增 SDE 专属校验逻辑。
- **D-25:** 时间戳格式直接映射 — SDE LLM 输出的 `"HH:MM:SS,mmm-HH:MM:SS,mmm"` 格式与 ScriptClip 的 `timestamp` 字段格式一致，直接映射，无需转换。
- **D-26:** SDE 仅使用 OST=0/1 — SDE Prompt 不要求 LLM 输出 OST=2（混合模式）。SDE 流水线只处理 NarrationOnly 和 OriginalSound 两种类型。混剪模式在 Phase 8 SDP 中支持。
- **D-27:** 运行时字段由视频处理回写 — `ScriptClip` 的 `duration`/`source_time_range`/`edited_time_range`/`video`/`audio` 由视频处理步骤（step_clip 等）计算并回写，与纪录片行为完全一致。LLM 生成阶段这些字段为默认值。
- **D-28:** 字幕时间戳存入辅助结构 — 从原始字幕解析时间戳和对话文本，存入 `SdePipelineState` 的辅助字段（如 `subtitle_segments: Vec<SubtitleSegment>`），供 OST=1 片段精确裁剪使用。不扩展 `ScriptClip` 模型。
- **D-29:** OST=1 按字幕精确裁剪 — 原声片段裁剪时，从辅助结构中查找对应时间戳范围内的原始字幕时间，按精确时间戳裁剪视频、保留原声。与纪录片 OST=1 行为一致。

### Claude's Discretion
- SdePipelineState 的具体字段定义（哪些 Option、哪些默认值）
- step 函数的具体拆分和内部实现细节
- SdeError 枚举的完整变体列表和错误消息（中文）
- JSON 修复函数 `repair_json()` 的具体实现（6 步回退策略）
- SRT 时间戳解析工具函数的具体实现
- 字幕编码探测的具体实现（编码链、回退逻辑）
- 进度百分比在各步骤之间的分配
- `src/sde/mod.rs` 的公开 API 设计
- `src/lib.rs` 新增 `pub mod sde;`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 SDE 流水线参考
- `app/services/SDE/short_drama_explanation.py` — ★ SDE 核心：SubtitleAnalyzer 类（analyze_subtitle / generate_narration_script）
- `app/services/subtitle_text.py` — ★ 字幕文本工具：编码探测（decode_subtitle_bytes）、文本标准化（normalize_subtitle_text）、SRT 时间码正则
- `webui/tools/generate_short_summary.py` — ★ WebUI SDE 编排：完整流程（字幕读取→LLM分析→脚本生成→JSON修复）
- `app/services/prompts/short_drama_narration/plot_analysis.py` — Prompt 模板：剧情分析（v1.0）
- `app/services/prompts/short_drama_narration/script_generation.py` — Prompt 模板：解说脚本生成（v2.0，含 "播放原片+N" OST=1 格式规范）

### Rust 代码库依赖（Phase 6 视频处理）
- `src/documentary/pipeline.rs` — ★ 步骤 2-6 实现：step_tts/step_clip/step_merge_audio_subtitle/step_concat/step_composite（需标记为 pub(crate)）
- `src/documentary/types.rs` — DocumentaryRequest、ProgressStep、TtsResult（SdeRequest 参数对齐的参考）
- `src/documentary/audio.rs` — calculate_clip_duration、merge_audio_files、merge_subtitle_files
- `src/documentary/clip.rs` — clip_all_videos（视频裁剪主函数）
- `src/documentary/subtitle.rs` — generate_srt_from_word_boundaries、write_srt_file
- `src/documentary/timestamp.rs` — parse_time_to_secs、secs_to_ffmpeg_time
- `src/documentary/error.rs` — PipelineError 枚举（SdeError 嵌入此类型）

### Rust 代码库依赖（基础模块）
- `src/script/types.rs` — ScriptClip + OstType 数据模型（SDE 脚本的目标格式）
- `src/script/mod.rs` — load_script/save_script/validate_script
- `src/tts/mod.rs` — TtsProvider trait + synthesize 路由器
- `src/ffmpeg/mod.rs` — FFmpeg spawn_blocking 封装
- `src/llm/mod.rs` — LlmProvider trait（文本 LLM 调用）
- `src/llm/types.rs` — LLM 请求/响应类型
- `src/prompt/manager.rs` — PromptManager（获取 short_drama_narration 分类模板、渲染）
- `src/prompt/mod.rs` — Prompt 系统模块入口
- `src/config/types.rs` — AppConfig 配置类型
- `src/error.rs` — 顶层错误类型
- `src/lib.rs` — 库入口（需新增 `pub mod sde;`）

### 项目规划文档
- `.planning/ROADMAP.md` — Phase 7 定义：SDE Pipeline，需求 SDE-01/02/03，依赖 Phase 6
- `.planning/REQUIREMENTS.md` — SDE-01 (流水线编排), SDE-02 (字幕解析), SDE-03 (LLM 剧情分析)
- `.planning/PROJECT.md` — 项目约束（Rust 库、文件存储、不可变模式）
- `.planning/phases/06-documentary-pipeline/06-CONTEXT.md` — Phase 6 决策（Step-by-Step 函数链、PipelineState、fail-fast、字幕生成流程）
- `.planning/phases/05-script-management/05-CONTEXT.md` — Phase 5 决策（ScriptClip 模型、OstType 枚举、校验策略）
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（FFmpeg 模式、spawn_blocking、ProgressCallback）

### 代码库映射
- `.planning/codebase/ARCHITECTURE.md` — Python 版架构（Layered monolith、Prompt/LLM 结构）
- `.planning/codebase/INTEGRATIONS.md` — LLM/TTS/FFmpeg 集成点
- `.planning/codebase/STACK.md` — 技术栈和外部依赖

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/documentary/pipeline.rs` — 步骤 2-6 的 step_* 函数（TTS 生成/视频裁剪/音频合并/拼接/合成），标记 pub(crate) 后 SDE 直接调用
- `src/documentary/audio.rs` — calculate_clip_duration / merge_audio_files / merge_subtitle_files，OST=0/1 的时长计算和音频合并逻辑完全复用
- `src/documentary/clip.rs` — clip_all_videos，视频裁剪主函数，支持 OST=0/1/2 三种策略
- `src/documentary/timestamp.rs` — parse_time_to_secs / secs_to_ffmpeg_time，SRT 时间戳解析工具
- `src/documentary/subtitle.rs` — SRT 生成和写入，TTS 词边界→字幕文件
- `src/script/types.rs` — ScriptClip + OstType，SDE 脚本的目标数据模型
- `src/script/mod.rs` — load_script/validate_script，脚本校验复用
- `src/tts/mod.rs` — synthesize 路由器，TTS 生成复用
- `src/llm/mod.rs` — LlmProvider trait，文本 LLM 调用（generate_text）
- `src/prompt/manager.rs` — PromptManager，获取/渲染 short_drama_narration 分类模板
- `src/config/types.rs` — AppConfig 各配置段，读取 TTS/LLM/字幕参数

### Established Patterns
- **模块级结构（Phase 1 D-02）**：Phase 7 创建 `src/sde/` 模块，`src/lib.rs` 新增 `pub mod sde;`
- **Step-by-Step 函数链（Phase 6 D-01）**：每步独立 async 函数，主编排函数顺序调用
- **可变 PipelineState（Phase 6 D-02）**：定义 SdePipelineState，每步接收 &mut state
- **Request 参数模式（Phase 6 D-03 / Phase 9）**：入口函数接收 SdeRequest + &AppConfig + Option<&ProxySection>
- **领域错误枚举（Phase 1 D-16/D-17）**：SdeError 使用 thiserror 派生，嵌入 PipelineError
- **中文用户可见错误信息（Phase 1 D-18）**：SdeError 的 Display 实现用中文
- **ProgressCallback 回调（Phase 1 D-15）**：SDE 流水线每步完成后上报进度
- **fail-fast 错误恢复（Phase 6 D-06）**：任何步骤失败立即返回错误，中间产物保留
- **PromptManager 模板加载（Phase 4）**：通过 PromptManager 获取 Prompt、渲染变量

### Integration Points
- `src/sde/mod.rs` — 新模块入口，由 `src/lib.rs` pub mod 导出
- `src/sde/pipeline.rs` — 主编排函数，调用纪录片 step_* 函数
- `src/sde/subtitle.rs` — 字幕文件读取和编码探测
- `src/sde/script_gen.rs` — LLM 调用（通过 crate::llm）+ Prompt 获取（通过 crate::prompt）
- `src/documentary/pipeline.rs` — 需将步骤 2-6 的 step_* 函数标记为 pub(crate)
- `src/llm/mod.rs` — 文本 LLM 调用（generate_text）
- `src/prompt/manager.rs` — 获取 short_drama_narration 模板
- `src/script/mod.rs` — load_script / validate_script 调用
- `src/lib.rs` — 新增 `pub mod sde;`
- `Cargo.toml` — 无需新增依赖（uuid 已在纪录片使用，其他依赖均已就绪）

</code_context>

<specifics>
## Specific Ideas

- SDE 与纪录片的核心差异在预处理阶段：纪录片用帧分析+视觉 LLM 生成脚本，SDE 用字幕解析+文本 LLM 生成脚本。视频处理步骤（2-6）完全共享，差异仅在于脚本来源和 OST 比例。
- Python 版 SDE Prompt（script_generation v2.0）有详细的创作要素规范（黄金开场、爽点放大、个性吐槽、悬念预埋等），Rust 版 Prompt 模板应 1:1 对齐。
- "播放原片+N" 格式是 Python 版 SDE 的核心约定：OST=1 片段 narration 字段存此字符串，视频处理流水线检测此前缀跳过 TTS、保留原声。此约定在 Rust 版必须精确复现。
- Python 版 `parse_and_fix_json` 有 7 步回退策略（直接解析→双大括号修复→代码块提取→大括号提取→注释移除→尾逗号修复→综合修复→默认兜底），Rust 版 repair_json() 需对齐此鲁棒性。
- 字幕编码探测链（utf-8 → utf-16 → gbk → gb2312）对中文短剧场景至关重要，大多数中文字幕文件是 GBK 或 UTF-16 编码。
- SDE 的进度百分比分配参考：步骤 ParseSubtitle=0%, PlotAnalysis=15%, ScriptGenerate=30%, LoadScript=40%, Tts=50%, Clip=70%, MergeAudio=80%, Concat=90%, Composite=100%。
- SDE 生成的脚本中 OST=1 原声片段的时间戳必须与原始字幕精确对齐——LLM 输出的时间戳可能不够精确，视频裁剪时应从辅助结构（subtitle_segments）中查找精确时间戳。

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-sde-pipeline*
*Context gathered: 2026-05-05*
