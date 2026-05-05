# Phase 8: SDP Pipeline - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning (blocked on Phase 7 completion)

<domain>
## Phase Boundary

短剧混剪模式完整流水线——从字幕文件输入到最终混剪视频输出的端到端流程。核心流程分两部分：

1. **脚本生成预处理**（`generate_sdp_script()`）：字幕文本 → LLM 两步分析（subtitle_analysis → plot_extraction）→ 合并为 ScriptClip[]（OST=1 仅原声）→ 保存 JSON。用户可编辑后传入流水线。

2. **视频处理流水线**（`run_sdp()`）：加载脚本 → 视频裁剪 → 片段拼接 → 最终合成（+ 可选 BGM）。无需 TTS、无需字幕烧录。

覆盖需求：SDP-01, SDP-02。依赖 Phase 6（纪录片流水线模式）+ Phase 7（字幕解析模块）。

</domain>

<decisions>
## Implementation Decisions

### 依赖关系
- **D-01:** Phase 8 依赖更新为 Phase 6 + Phase 7。Phase 7 的 `src/subtitle/` 公共模块提供 SRT/ASS 解析能力（编码检测、文本规范化、时间码提取），Phase 8 直接引用。ROADMAP.md 需同步更新 Depends on 字段。
- **D-02:** 先讨论并写定 CONTEXT.md，等 Phase 7 完成后再执行 Phase 8 计划。

### 多视频源策略
- **D-03:** 先只支持单视频文件输入。所有 plot_point 片段从同一个视频按时间戳裁剪。与 Python 版当前行为对齐。多视频源映射留待后续阶段。

### 流水线结构
- **D-04:** 沿用 Phase 6 的 step-by-step 模式。创建 `src/sdp/` 模块，定义 `SdpPipelineState` 可变结构体 + 独立 step 函数 + `run_sdp()` 主编排函数。与 Phase 6 的 `PipelineState` 模式架构一致。
- **D-05:** 不做跨模式抽象——Phase 6/7/8 各自独立实现，等三个流水线都完成后再考虑抽取公共。遵循 Phase 6 D-05 原则。

### 脚本生成流程
- **D-06:** 1:1 对齐 Python 版两步 LLM 调用。第 1 步调 `short_drama_editing/subtitle_analysis` prompt（提取 plot_titles + summary），第 2 步调 `short_drama_editing/plot_extraction` prompt（定位 plot_points 含 timestamp + picture 描述）。temperature=0.1，max_tokens=4000。
- **D-07:** `merge_script()` 将 plot_points 合并为 `Vec<ScriptClip>`，所有片段 OST=1（仅原声），narration 字段设为占位值。

### 流程拆分
- **D-08:** 拆分为两个独立函数——`generate_sdp_script(subtitle_input, llm_config) -> Result<Vec<ScriptClip>>`（预处理，生成可编辑的脚本）和 `run_sdp(video_path, script_path, config) -> Result<PathBuf>`（纯视频处理流水线）。与 Phase 6 的 D-12/D-13 模式一致（先分析→编辑→再生成视频）。
- **D-09:** `generate_sdp_script()` 内部调用 Phase 7 的字幕解析 + Phase 2 的 LLM + Phase 4 的 Prompt 系统，输出标准 ScriptClip[] 可保存为 JSON。用户可编辑后再传给 `run_sdp()`。

### BGM 支持
- **D-10:** 最终合成步骤支持 BGM 混音（可选）。`SdpRequest` 包含 BGM 路径和音量参数，合成时通过 FFmpeg amix filter 混合。与 Phase 6 的 step_composite 模式一致。

### Claude's Discretion
- SdpPipelineState 的具体字段定义
- step 函数的拆分粒度（几个 step 文件）
- SdpRequest 参数结构体和默认值
- SdpPipelineError 枚举的具体变体
- JSON 修复逻辑（对应 Python `parse_and_fix_json`）的实现方式
- `custom_clips` 参数的默认值（Python 版默认 5）
- Prompt 模板的 Rust 版本内容对齐方式（直接翻译还是从 Python 导入）

</decisions>

<specifics>
## Specific Ideas

- SDP 混剪的核心价值在于"剧情连贯性"——LLM 选出的片段必须能让观众看懂故事线，不是随机爆点拼凑。Python 版 Prompt 强调 narrative_structure（开端→发展→高潮→结局）+ plot_connections（情节间逻辑关联），Rust 版需保留这种约束强度
- Python 版 `merge_script` 中 OST 全部为 1，narration 用 `"播放原生_xxxxx"` 占位。这个占位值不影响功能但便于调试查看
- SDP 不使用视觉分析（不提取帧），纯依赖字幕文本分析。这是与纪录片模式（帧分析驱动）的本质区别
- Python 版 SDP 脚本输出放在 `merged_subtitle.json`，与纪录片模式共用同一个 ScriptClip 格式

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 SDP 核心参考
- `app/services/SDP/generate_script_short.py` — ★ SDP 脚本生成主入口（generate_script_result / generate_script），编排 subtitle_analysis → plot_extraction → merge_script
- `app/services/SDP/utils/step1_subtitle_analyzer_openai.py` — ★ 两步 LLM 分析核心（analyze_subtitle），第1步 subtitle_analysis prompt + 第2步 plot_extraction prompt，含 JSON 解析修复
- `app/services/SDP/utils/step5_merge_script.py` — ★ plot_points → ScriptClip[] 合并逻辑，OST=1 + 占位 narration
- `app/services/SDP/utils/short_schema.py` — SDP 数据模型（PlotPoint, Commentary, ScriptItem 等）
- `app/services/SDP/utils/utils.py` — 字幕加载工具（pysrt 多编码解析）
- `webui/tools/generate_script_short.py` — SDP 脚本生成 UI 入口，输入验证 + 进度更新

### Python 版 SDP Prompts
- `app/services/prompts/short_drama_editing/subtitle_analysis.py` — 第1步 Prompt：字幕剧情分析（叙事结构、关键情节点、连贯性约束、JSON 输出格式）
- `app/services/prompts/short_drama_editing/plot_extraction.py` — 第2步 Prompt：时间段精确定位（时间戳匹配、片段时长、过渡策略、质量检查清单）

### Python 版公共依赖
- `app/services/subtitle_text.py` — 字幕文本解码/规范化（多编码检测、时间码规范化、has_timecodes 检测）
- `app/services/upload_validation.py` — 输入验证（ensure_existing_file, InputValidationError）

### Rust 代码库依赖（Phase 6 模式参考）
- `src/documentary/pipeline.rs` — Phase 6 流水线编排模式（PipelineState + step 函数链 + run_documentary）
- `src/documentary/types.rs` — Phase 6 请求/状态类型设计模式
- `src/documentary/error.rs` — Phase 6 PipelineError 枚举模式
- `src/documentary/clip.rs` — OST 分支视频裁剪逻辑（OST=1 严格按 timestamp 裁剪）
- `src/ffmpeg/mod.rs` — FFmpeg spawn_blocking 封装
- `src/ffmpeg/probe.rs` — ffprobe 视频探测
- `src/ffmpeg/command.rs` — FFmpeg 命令构建
- `src/script/mod.rs` — ScriptClip 加载/保存/校验
- `src/script/types.rs` — ScriptClip + OstType 数据模型
- `src/llm/mod.rs` — LlmProvider trait
- `src/prompt/manager.rs` — PromptManager（获取模板、渲染）

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库、文件存储）
- `.planning/REQUIREMENTS.md` — Phase 8 需求：SDP-01, SDP-02
- `.planning/ROADMAP.md` — Phase 8 定义和成功标准（需更新 Depends on 为 Phase 6 + Phase 7）
- `.planning/phases/06-documentary-pipeline/06-CONTEXT.md` — Phase 6 决策（流水线模式参考）
- `.planning/phases/07-sde-pipeline/07-CONTEXT.md` — Phase 7 决策（字幕解析模块位置，待 Phase 7 讨论后更新）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/documentary/pipeline.rs` — Phase 6 的 PipelineState + step 函数链 + ProgressCallback 模式，SDP 流水线直接复用此模式
- `src/documentary/clip.rs` — OST=1 视频裁剪逻辑，SDP 只用 OST=1，可直接复用或参数化调用
- `src/ffmpeg/mod.rs` — FFmpeg 封装，视频裁剪/拼接/合成全部复用
- `src/script/mod.rs` + `src/script/types.rs` — ScriptClip 模型（含 OstType::Original），SDP 脚本数据模型直接复用
- `src/llm/mod.rs` — LlmProvider trait，脚本生成调 generate_text
- `src/prompt/manager.rs` — PromptManager，获取并渲染 SDP 的 subtitle_analysis / plot_extraction 模板
- `src/config/types.rs` — AppConfig 配置段

### Established Patterns
- **模块级结构（Phase 1 D-02）**：Phase 8 创建 `src/sdp/` 模块，由 `src/lib.rs` pub mod 导出
- **领域错误枚举（Phase 1 D-16/D-17）**：SdpPipelineError 使用 thiserror 派生
- **中文用户可见错误信息（Phase 1 D-18）**：SdpPipelineError 的 Display 实现用中文
- **ProgressCallback 回调（Phase 1 D-15）**：流水线每步完成后上报进度
- **不可变数据更新（Phase 5）**：所有编辑方法返回新实例
- **Step-by-Step 函数链（Phase 6 D-01）**：主函数顺序调用各 step，不做 trait 抽象

### Integration Points
- `src/sdp/mod.rs` — 新模块入口，由 `src/lib.rs` 导出
- `src/subtitle/mod.rs` — Phase 7 的字幕解析模块（待实现），generate_sdp_script 调用
- `src/llm/mod.rs` — generate_sdp_script 调用 LLM
- `src/prompt/manager.rs` — generate_sdp_script 获取 SDP prompt 模板
- `src/script/mod.rs` — run_sdp 加载脚本，generate_sdp_script 保存脚本
- `src/ffmpeg/mod.rs` — run_sdp 调用视频裁剪/拼接/合成
- `src/lib.rs` — 新增 `pub mod sdp;`
- `.planning/ROADMAP.md` — 需更新 Phase 8 的 Depends on 字段

</code_context>

<deferred>
## Deferred Ideas

- 多视频源支持（SDP-02 完整含义）——可按视频-片段映射规则扩展，当前 Phase 8 只做单视频。用户选择先与 Python 版对齐
- 跨模式公共流水线抽象——等 Phase 6/7/8 都完成后评估抽取 common.rs（Phase 6 D-05 原则）

</deferred>

---

*Phase: 08-sdp-pipeline*
*Context gathered: 2026-05-05*
