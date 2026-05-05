# Phase 8: SDP Pipeline - Research

**Researched:** 2026-05-05
**Domain:** 短剧混剪流水线（Rust 实现）
**Confidence:** HIGH

## Summary

Phase 8 实现短剧混剪（SDP）完整流水线，覆盖需求 SDP-01（流水线编排）和 SDP-02（多片段混剪）。该流水线与 Phase 7 SDE 共享大量基础设施：字幕解析模块（提取为公共 `src/subtitle/`）、LLM 两步分析与 JSON 修复、FFmpeg 裁剪与合成模式。关键区别在于 SDP 全部 OST=1（仅原声），无 TTS、无字幕烧录，流水线仅 4 步：clip → concat → composite。脚本生成流程使用 `short_drama_editing` 分类的 `subtitle_analysis` + `plot_extraction` 两个 Prompt 模板，后一个在 Rust 中已有注册但需更新为 SDP 专属版本。

**Primary recommendation:** 采用与 SDE 一致的模块结构（`src/sdp/{pipeline,types,error,clip,script_gen,mod}.rs`），复用 SDE 的 `sde_step_clip()`（需提升可见性为 `pub(crate)`），复用 documentary 的 `clip_all_videos()`、concat 和 composite 模式。先提取 `src/sde/subtitle.rs` 为公共 `src/subtitle/` 模块，再创建 SDP 模块。

<user_constraints>
## User Constraints (from CONTEXT.md)

### Implementation Decisions

#### 依赖关系与前置工作
- **D-01:** Phase 8 依赖 Phase 6 + Phase 7。Phase 7 已全部完成（4/4 plans），`run_sde()` 可用，`pub(crate)` 接口已暴露（step 函数 + subtitle 模块）。
- **D-02:** 前置步骤——将 `src/sde/subtitle.rs` 提取为独立的 `src/subtitle/` 公共模块。Phase 7 改为引用新路径（`crate::subtitle`），Phase 8 同样引用。在 Phase 8 计划内完成，不创建独立 phase。

#### 多视频源策略
- **D-03:** 先只支持单视频文件输入。所有 plot_point 片段从同一个视频按时间戳裁剪。

#### 流水线结构
- **D-04:** 沿用 Phase 6 的 step-by-step 模式。创建 `src/sdp/` 模块，定义 `SdpPipelineState` 可变结构体 + 独立 step 函数 + `run_sdp()` 主编排函数。
- **D-05:** 不做跨模式抽象——Phase 6/7/8 各自独立实现，等三个流水线都完成后再考虑抽取公共。

#### 流水线步骤设计
- **D-06:** 4 步流水线：step_clip（视频裁剪）→ step_concat（片段拼接）→ step_composite（最终合成 + BGM 混音）→ 输出。
- **D-07:** 进度百分比分配：step_clip = 30%、step_concat = 60%、step_composite = 90%、完成 = 100%。

#### 视频处理复用
- **D-08:** 视频裁剪直接复用 SDE 的 `sde_step_clip()`（OST=1 精确时间戳校正 → 委托 documentary `clip_all_videos()`）。
- **D-09:** 视频拼接和最终合成复用 documentary 的 FFmpeg concat/composite 模式（filter_complex + amix BGM）。

#### 脚本生成流程
- **D-10:** 1:1 对齐 Python 版两步 LLM 调用。第 1 步调 `short_drama_editing/subtitle_analysis` prompt，第 2 步调 `short_drama_editing/plot_extraction` prompt。temperature=0.1，max_tokens=4000。
- **D-11:** 复用 SDE 的 LLM 调用模式——PromptManager 渲染模板 → `LlmProvider::generate_text()` 调用 → JSON 修复（repair_json 6 步策略）→ 校验保存。
- **D-12:** `merge_script()` 将 plot_points 合并为 `Vec<ScriptClip>`，所有片段 OST=1，narration 字段设为占位值（如 `"播放原生_xxxxx"`）。
- **D-13:** SDP 不使用视觉分析（不提取帧），纯依赖字幕文本分析。

#### 流程拆分
- **D-14:** 拆分为两个独立函数 —— `generate_sdp_script(subtitle_input, llm_config) -> Result<Vec<ScriptClip>>` 和 `run_sdp(video_path, script_path, config) -> Result<PathBuf>`。
- **D-15:** `generate_sdp_script()` 使用 `src/subtitle/` 公共模块 + Phase 2 LLM + Phase 4 Prompt 系统，输出标准 ScriptClip[]。

#### 模块结构
- **D-16:** `src/sdp/` 对齐 SDE 文件结构：`error.rs`、`types.rs`、`pipeline.rs`、`clip.rs`、`script_gen.rs`、`mod.rs`。
- **D-17:** 独立 `SdpError` 枚举 —— 使用 thiserror 派生，中文用户可见错误信息。
- **D-18:** 独立 `SdpPipelineState` 结构体 —— 精简为 4 步所需字段（无 TTS/字幕相关字段）。
- **D-19:** `SdpRequest` 参数结构体设计由 planner 裁量。

#### BGM 支持
- **D-20:** 最终合成步骤支持 BGM 混音（可选），通过 FFmpeg amix filter。

#### 输入验证
- **D-21:** 两阶段验证 —— `generate_sdp_script()` 验证字幕文件存在/可读/非空；`run_sdp()` 验证视频文件存在、脚本非空且有效、OST 全为 1。

#### 测试策略
- **D-22:** 对齐 SDE 测试密度 —— 字幕解析（如涉及）、LLM 脚本生成、JSON 修复、流水线编排、视频裁剪各模块都有单元测试。

#### Python 对齐策略
- **D-23:** 1:1 对齐 Python 版核心逻辑 —— LLM 调用、JSON 修复、脚本合并。

### Claude's Discretion
- `SdpPipelineState` 的具体字段定义
- `SdpRequest` 参数结构体和默认值
- `SdpError` 枚举的具体变体
- step 函数的拆分粒度（几个 step 文件）
- JSON 修复逻辑的实现方式（可直接参考 Phase 7 `repair_json`）
- 是否需要新增 Cargo.toml 依赖
- Prompt 模板的 Rust 版本具体内容
- `custom_clips` 参数的默认值（Python 版默认 5）

### Deferred Ideas (OUT OF SCOPE)
- 多视频源支持（SDP-02 完整含义）——可按视频-片段映射规则扩展，当前 Phase 8 只做单视频
- 跨模式公共流水线抽象——等 Phase 6/7/8 都完成后评估抽取 common.rs
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SDP-01 | 短剧混剪完整流水线编排（字幕解析 → LLM 分析 → 多片段脚本 → 视频处理） | 两步 LLM 调用模式 + 4 步流水线编排，复用 `src/sde/script_gen.rs` 的 `repair_json` 和 `parse_script` 模式 |
| SDP-02 | 多片段混剪——跨多个视频源裁剪和拼接 | D-03 限定单视频源；复用 `sde_step_clip()` → `clip_all_videos()` 裁剪 + FFmpeg concat/composite 拼接合成 |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| 字幕解析 | API / Backend | — | 文件 I/O + 编码检测 + 结构化解析，全在 Rust 服务端完成 |
| LLM 脚本生成 | API / Backend | — | Prompt 渲染 + LLM API 调用 + JSON 修复，纯服务端操作 |
| 视频裁剪 | API / Backend | FFmpeg 子进程 | 通过 ffmpeg-sidecar 异步裁剪，spawn_blocking 不阻塞 tokio |
| 视频拼接 | API / Backend | FFmpeg 子进程 | concat demuxer 操作，纯 FFmpeg 子进程 |
| 最终合成 + BGM | API / Backend | FFmpeg 子进程 | filter_complex + amix + 编码输出 |
| 进度回调 | API / Backend | — | 每步完成后通过回调上报，供 Tauri/CLI 使用 |
| BGM 音频处理 | API / Backend | FFmpeg 子进程 | aloop/atrim/volume filter 链，全在 composite 步骤内 |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ffmpeg-sidecar` | 2.5.1 | FFmpeg CLI 封装 | 已在 Phase 1/6/7 中使用，裁剪/拼接/合成全依赖它 [VERIFIED: Cargo.toml] |
| `serde` / `serde_json` | 1.0.228 / 1.0.140 | JSON 序列化 | ScriptClip 读写、LLM JSON 解析统一使用 [VERIFIED: Cargo.toml] |
| `thiserror` | 2.0.18 | 错误类型派生 | SdpError 枚举使用，与 SdeError 模式一致 [VERIFIED: Cargo.toml] |
| `tokio` | 1.52.1 | 异步运行时 | 所有异步操作的运行时 [VERIFIED: Cargo.toml] |
| `tracing` | 0.1.44 | 日志 | 与全项目统一的 logging 方案 [VERIFIED: Cargo.toml] |
| `regex` | 1.11 | 正则匹配 | 字幕时间戳检测、文本模式匹配 [VERIFIED: Cargo.toml] |
| `encoding_rs` | 0.8 | 字幕编码检测 | 多编码字幕读取（UTF-8/16, GBK）[VERIFIED: Cargo.toml] |
| `uuid` | 1.23.1 | 任务 ID 生成 | 流水线 task_id 生成 [VERIFIED: Cargo.toml] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3.27.0 | 测试临时目录 | 单元测试中 `tempfile::TempDir` [VERIFIED: Cargo.toml dev-dependencies] |
| `serde_repr` | 0.1.20 | OST 类型枚举序列化 | ScriptClip 的 OST 字段序列化为数字 [VERIFIED: Cargo.toml] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| 复用 `sde_step_clip()` | 自建 SDP 裁剪逻辑 | 核心逻辑完全相同（OST=1 精确时间戳 → clip_all_videos），复用减少重复代码 |
| 复用 documentary concat/composite | 自建 concat/composite | FFmpeg 命令完全相同（concat demuxer + filter_complex composite），复用已验证的代码 |

**Installation:**
```bash
# No new dependencies needed — everything is already in Cargo.toml
```

**Version verification:** All core dependencies verified against `Cargo.toml` at commit `139773c`. No new dependencies required for this phase.

## Architecture Patterns

### System Architecture Diagram

```
字幕文件 (.srt/.ass)
       │
       ▼
┌─────────────────────────────────────────────┐
│          generate_sdp_script()              │
│                                             │
│  1. parse_subtitle_file() (公共 subtitle/)   │
│     → Vec<SubtitleSegment> + clean text     │
│                                             │
│  2. LLM Step 1: subtitle_analysis prompt     │
│     → summary + plot_titles + structure     │
│                                             │
│  3. LLM Step 2: plot_extraction prompt       │
│     → Vec<PlotPoint> (timestamp + picture)  │
│                                             │
│  4. merge_script()                           │
│     → Vec<ScriptClip> (all OST=1)           │
│     → save as merged_subtitle.json          │
└──────────────┬──────────────────────────────┘
               │ ScriptClip[] JSON file
               ▼ (用户可编辑后传入)
┌─────────────────────────────────────────────┐
│              run_sdp()                      │
│                                             │
│  1. load_script() → validate OST all = 1    │
│                                             │
│  2. step_clip()        [30%]               │
│     └─ sde_step_clip() → clip_all_videos() │
│        (FFmpeg async spawn_blocking)       │
│                                             │
│  3. step_concat()       [60%]              │
│     └─ FFmpeg concat demuxer (-c copy)      │
│                                             │
│  4. step_composite()    [90% → 100%]       │
│     └─ FFmpeg filter_complex + amix BGM    │
│     └─ ├─ video: pass-through (no subtitle) │
│        ├─ audio: orig_vol + bgm (optional)  │
│        └─ libx264 + aac output             │
│                                             │
│  return final_video_path                    │
└─────────────────────────────────────────────┘
```

### Recommended Project Structure
```
src/
├── subtitle/                  # [COMMON] 从 src/sde/subtitle.rs 提取
│   ├── mod.rs                 #   公共模块入口
│   └── ...                    #   parse_subtitle_file, detect_encoding, SubtitleSegment
├── sde/                       # [EXISTING] Phase 7 修改
│   ├── subtitle.rs (REMOVED)  #   删除，改为引用 crate::subtitle
│   ├── clip.rs                #   保持 pub(crate)，SDP 复用
│   └── ...
├── sdp/                       # [NEW] Phase 8
│   ├── mod.rs                 #   模块入口 + pub use
│   ├── error.rs               #   SdpError 枚举
│   ├── types.rs               #   SdpRequest, SdpPipelineState, SdpProgressStep
│   ├── pipeline.rs            #   run_sdp() 主编排 + step_clip / step_concat / step_composite
│   ├── clip.rs                #   sdp_step_clip() 包装（复用 sde_step_clip）
│   └── script_gen.rs          #   generate_sdp_script() + LLM 调用 + merge_script()
├── prompt/
│   ├── register.rs            # [MODIFY] 新增 subtitle_analysis prompt + 更新 plot_extraction 参数
│   └── templates/
│       └── short_drama_editing/
│           ├── subtitle_analysis_v2.0.md    # [NEW] 翻译自 Python 版
│           └── plot_extraction_v2.0.md      # [UPDATE] 替换为 SDP 专用版
└── lib.rs                     # [MODIFY] 新增 pub mod sdp; + pub mod subtitle;
```

### Pattern 1: Step-by-Step Pipeline 函数链
**What:** 与 Phase 6/7 一致的 `PipelineState` + 独立 step 函数 + `run_sdp()` 主编排模式。状态通过 `&mut state` 传递，每步修改状态字段。进度通过 `emit_progress()` 回调上报。

**When to use:** SDP 是第三个实现此模式的流水线，需保持一致。不做 trait 抽象（D-05）。

**Example (from Phase 7 SDE):**
```rust
// Source: src/sde/pipeline.rs L55-L57
let mut state = SdePipelineState::new(task_id, task_dir);
// ... step 0: subtitle parsing ...
// ... step 1: plot analysis ...
// ... step 2: script generation ...
// ... step 3: script load + validation ...
// ... step 4: TTS (SKIPPED for SDP) ...
// ... step 5: clip ...
// ... step 6: merge audio (SKIPPED for SDP) ...
// ... step 7: concat ...
// ... step 8: composite ...
state.output_video_path.ok_or_else(|| ...)
```

### Pattern 2: OST=1 精确时间戳校正
**What:** LLM 生成的时间戳可能有 ±1s 偏差。使用 `find_precise_range()` 从原始字幕段落获取精确时间戳，仅对 OST=1 片段执行。找不到精确匹配时保留原始时间戳（降级不崩溃）。

**When to use:** SDP 所有片段都是 OST=1，每个片段都需要时间戳校正。

**Example (from Phase 7 SDE clip.rs L20-L31):**
```rust
// Source: src/sde/clip.rs
for clip in &mut state.script {
    if clip.ost == OstType::OriginalSound {
        if let Some((precise_start, precise_end)) =
            find_precise_range(&clip.timestamp, &state.subtitle_segments)
        {
            let start_str = secs_to_timestamp_str(precise_start);
            let end_str = secs_to_timestamp_str(precise_end);
            clip.timestamp = format!("{}-{}", start_str, end_str);
        }
    }
}
```

### Pattern 3: 两步 LLM 调用 + JSON 修复
**What:** 第 1 步 `subtitle_analysis` 分析整体剧情结构，输出 JSON（summary + plot_titles）。第 2 步 `plot_extraction` 基于分析结果定位具体时间段，输出 JSON（plot_points[]）。两步 temperature=0.1，max_tokens=4000。输出使用 `repair_json` 6 步策略修复。

**When to use:** 每次生成 SDP 脚本时执行。

**Example (adapted from Phase 7 SDE script_gen.rs L22-L63):**
```rust
// Step 1: subtitle_analysis
let mut vars = HashMap::new();
vars.insert("subtitle_content", &state.subtitle_text);
vars.insert("custom_clips", &custom_clips_str);
let prompt = prompt_manager.render_prompt(
    "short_drama_editing", "subtitle_analysis", Some("v2.0"), &vars)?;
let result = provider.generate_text(
    &prompt, Some(system_prompt), Some(0.1_f32), Some(4000), None).await?;
let analysis = repair_json(&result); // + parse into summary_data

// Step 2: plot_extraction (depends on step 1 output)
let mut vars = HashMap::new();
vars.insert("subtitle_content", &state.subtitle_text);
vars.insert("plot_summary", &summary_data.summary);
vars.insert("plot_titles", &plot_titles_text);
let prompt = prompt_manager.render_prompt(
    "short_drama_editing", "plot_extraction", Some("v2.0"), &vars)?;
let result = provider.generate_text(
    &prompt, Some(system_prompt), Some(0.1_f32), Some(4000), None).await?;
let plot_data = repair_json(&result); // + parse into plot_points[]
```

### Pattern 4: SDP Step Composite（无 TTS/字幕的简化版）
**What:** SDP 的 composite 步骤不需要 TTS 音频输入（只有原始视频音频 + 可选 BGM），不需要字幕烧录。filter_complex 只需要 video pass-through + `[orig]volume[orig]` + `[bgm]amix`。

**When to use:** SDP 的 step_composite 是 documentary composite 的简化版。

**Key differences from documentary composite:**
- 无 `[tts]` 音频输入（SDP 全部 OST=1，原始音频直接取自 video clip）
- 无字幕烧录（no `subtitles` filter, no `force_style`）
- 仍支持 BGM 混音（`aloop + atrim + volume + afade + amix`）
- 仍需要原始音频音量控制（`volume` filter on `[0:a]`）

### Anti-Patterns to Avoid
- **不要创建独立的 BGM 步骤：** BGM 混音合并到 step_composite 内部，不做单独的 BGM 处理流水线步。与 Phase 6 D-06 一致。
- **不要复用 SDE 的 merge_audio/subtitle 步骤：** SDP 无 TTS、无字幕烧录，不需要 audio 合并和 SRT 生成步骤。
- **不要跳过时间戳校正：** 即使 100% OST=1，LLM 输出的时间戳仍可能有偏差，必须执行精确时间戳校正。

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FFmpeg CLI 调用 | 手动 `std::process::Command` | `ffmpeg-sidecar` 2.5.1 + `run_ffmpeg()` 封装 | spawn_blocking + 错误处理 + 跨平台路径已在 Phase 1 封装好 [VERIFIED: Cargo.toml + src/ffmpeg/] |
| JSON 序列化/反序列化 | 手写解析器 | `serde_json` 1.0.140 | Script 读写、LLM JSON 解析一致使用 [VERIFIED: src/script/types.rs] |
| 视频裁剪逻辑 | 自建裁剪 | 复用 `sde_step_clip()` → `clip_all_videos()` | OST=1 裁剪逻辑完全相同，复用已验证代码 [VERIFIED: src/sde/clip.rs L15-L44] |
| JSON 修复策略 | 自建修复 | 复用 Phase 7 `repair_json()` 6 步策略 | `repair_json` 已在 Phase 7 实现并充分测试，可直接引用 [VERIFIED: src/sde/script_gen.rs L127-L221] |
| 字幕解析 | 自建解析 | 提取 `src/subtitle/` 公共模块 | `parse_subtitle_file()` 已实现多编码检测，SDE+SDP 共用 [VERIFIED: src/sde/subtitle.rs] |

**Key insight:** SDP 几乎所有核心技术都已在前序 phase 实现。本 phase 的核心工作是：组装（复用已有 step 函数）、简化（去掉 TTS/字幕步骤）、适配（Prompt 模板翻译 + 注册）。真正的"新代码"只有 `script_gen.rs`（generate_sdp_script + merge_script）和 `pipeline.rs`（4 步编排）。

## Common Pitfalls

### Pitfall 1: `sde_step_clip()` 可见性不足
**What goes wrong:** `src/sde/clip.rs` 的 `sde_step_clip()` 当前标记为 `pub(crate)`（仅在 crate 内可见），SDP 模块需要能调用它。但如果 SDP 模块和 SDE 模块是同层同级模块，`pub(crate)` 应该已经可见。需要验证。

**Why it happens:** `pub(crate)` 在整个 crate 内可见，但如果有跨模块导入路径问题则可能报错。

**How to avoid:** 确认在 `sde/mod.rs` 中有 `pub mod clip;` 声明，且在 `src/sdp/clip.rs` 中通过 `crate::sde::clip::sde_step_clip` 引用。

**Warning signs:** `cargo build` 报 "function `sde_step_clip` is not accessible"。

### Pitfall 2: JSON 修复全失败场景处理
**What goes wrong:** LLM 输出严重畸形，6 步 `repair_json` 全部失败，最终传给 `parse_script()` 时仍然是无效 JSON。

**Why it happens:** LLM 在某些情况下可能输出带大量非 JSON 前缀/后缀的内容，或使用非标准格式。

**How to avoid:** `repair_json` 返回原始字符串（最后一步不做硬修复），`parse_script()` 收到无效 JSON 时通过 `SdeError::JsonRepair` 清晰报错。SDP 的 `parse_script` 也应采用同样的 degrade-not-panic 策略。

**Warning signs:** 用户报告"脚本生成失败"但没有明确的 JSON 错误上下文。

### Pitfall 3: 字幕编码检测失败
**What goes wrong:** 用户字幕文件编码非 UTF-8（如 GBK、UTF-16LE），`parse_subtitle_file()` 编码检测失败或错误检测。

**Why it happens:** `encoding_rs` 检测启发式算法对短文件准确率降低 [VERIFIED: src/sde/subtitle.rs]。

**How to avoid:** `generate_sdp_script()` 接收输入前先验证文件可读、非空、包含时间码。编码检测失败时返回清晰中文错误信息，提示用户建议使用 UTF-8 编码。

**Warning signs:** 解析后字幕内容为空或乱码。

### Pitfall 4: FFmpeg concat 路径转义
**What goes wrong:** 包含空格或特殊字符的 Windows 路径在 concat demuxer 中失败。

**Why it happens:** concat demuxer 读取的文件列表需要 `file 'path'` 格式，Linux 风格正斜杠，对 Windows 反斜杠和特殊字符敏感。

**How to avoid:** 在生成 concat_list.txt 时执行 `path.replace('\\', "/")`（已在 documentary pipeline 中实现，直接复用）。

**Warning signs:** FFmpeg concat 阶段报 "No such file or directory" 但文件实际存在。

### Pitfall 5: OST 校验遗漏
**What goes wrong:** `run_sdp()` 接收的脚本中包含 OST=0 或 OST=2 的片段，但 SDP 流水线假设全部 OST=1（不做 TTS、不做字幕烧录）。

**Why it happens:** 用户手动编辑了 `merged_subtitle.json` 改变了 OST 值，或 merge_script 实现有 bug。

**How to avoid:** `run_sdp()` 严格执行 D-21 校验：验证 `script.iter().all(|c| c.ost == OstType::OriginalSound)`，否则返回错误。

**Warning signs:** 运行时出现 "missing TTS audio" 或 "missing soundtrack" 错误。

## Code Examples

### SDP Pipeline 主编排
```rust
// Reference: src/sde/pipeline.rs run_sde() pattern
// SDP 4-step simplified version
pub async fn run_sdp(
    request: SdpRequest,
    config: &AppConfig,
    proxy: Option<&ProxySection>,
) -> Result<PathBuf, SdpError> {
    request.validate().map_err(|e| SdpError::Validation { details: e })?;

    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request.output_dir.clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    std::fs::create_dir_all(&task_dir)?;

    let mut state = SdpPipelineState::new(task_id, task_dir);
    state.script = crate::script::load_script(&request.script_path)?;

    // G4: All clips must be OST=1
    if state.script.iter().any(|c| c.ost != OstType::OriginalSound) {
        return Err(SdpError::Validation {
            details: "SDP 仅支持 OST=1（原声）片段".into(),
        });
    }

    // Step 1: Clip  [30%]
    state.emit_progress(SdpProgressStep::Clip, 0.0, "正在裁剪视频...");
    sdp_step_clip(&mut state, &request.video_path).await?;
    state.emit_progress(SdpProgressStep::Clip, 30.0, "视频裁剪完成");

    // Step 2: Concat  [60%]
    state.emit_progress(SdpProgressStep::Concat, 30.0, "正在拼接视频...");
    sdp_step_concat(&mut state).await?;
    state.emit_progress(SdpProgressStep::Concat, 60.0, "视频拼接完成");

    // Step 3: Composite + BGM  [90% → 100%]
    state.emit_progress(SdpProgressStep::Composite, 60.0, "正在最终合成...");
    sdp_step_composite(&mut state, &request).await?;
    state.emit_progress(SdpProgressStep::Composite, 100.0, "最终合成完成");

    state.output_video_path.ok_or_else(|| SdpError::Validation {
        details: "流水线完成但未生成输出视频".into(),
    })
}
```

### Merge Script (plot_points → ScriptClip[])
```rust
// Reference: Python app/services/SDP/utils/step5_merge_script.py
// Reference: ScriptClip type at src/script/types.rs
fn merge_script(plot_points: &[PlotPoint], output_path: &Path) -> Result<Script, SdpError> {
    let mut clips = Vec::new();
    for (i, point) in plot_points.iter().enumerate() {
        let clip = ScriptClip {
            _id: (i + 1) as i64,
            timestamp: point.timestamp.clone(),
            picture: point.picture.clone(),
            narration: format!("播放原生_{}", hex::encode(&rand::random::<[u8; 4]>)),
            ost: OstType::OriginalSound,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };
        clips.push(clip);
    }

    let json_str = serde_json::to_string_pretty(&clips)?;
    std::fs::write(output_path, &json_str)?;
    Ok(clips)
}
```

### SDP Step Composite（简化版）
```rust
// Simplified composite with only original audio + optional BGM
async fn sdp_step_composite(
    state: &mut SdpPipelineState,
    request: &SdpRequest,
) -> Result<(), SdpError> {
    let merged_video = state.combined_video_path.as_ref().ok_or_else(|| ...)?;
    let output_path = state.task_dir.join("combined_sdp.mp4");

    // No TTS audio, no subtitle burning — only orig_vol + bgm
    let has_bgm = request.bgm_path.is_some();
    let mut filter_complex_parts = Vec::new();
    let mut amix_count = 0usize;

    // Original audio with volume control (from video clip, which already has original audio)
    filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", request.original_volume));
    amix_count += 1;

    if let Some(ref bgm_path) = request.bgm_path {
        // BGM loop + trim + fadeout
        let fade_start = (state.total_duration - 3.0).max(0.0);
        filter_complex_parts.push(format!(
            "[1:a]aloop=loop=-1:size=2e+09,atrim=0:{:.3},asetpts=PTS-STARTPTS,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
            state.total_duration, request.bgm_volume, fade_start
        ));
        amix_count += 1;
    }

    filter_complex_parts.push(format!(
        "[orig]{}amix=inputs={}:duration=longest[aout]",
        if has_bgm { "[bgm]" } else { "" },
        amix_count
    ));

    run_ffmpeg(move || { /* ... FFmpeg cmd with filter_complex ... */ }).await?;
    state.output_video_path = Some(output_path);
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `src/sde/subtitle.rs` 专属模块 | `src/subtitle/` 公共模块 | Phase 8 | Phase 7 引用路径变更为 `crate::subtitle`；Phase 8 同样引用 |
| `short_drama_editing/plot_extraction` 通用版 | 更新为 SDP 专用参数+模板 | Phase 8 | 参数从 `["subtitle_content", "language"]` 变为 `["subtitle_content", "plot_summary", "plot_titles"]` |
| `repair_json` 在 SDE 模块内 | 提取为公共函数（或保持原位引用） | Phase 8 可选 | 可直接 `use crate::sde::script_gen::repair_json` 复用，不需重复实现 |

**Deprecated/outdated:**
- Python `generate_script_short.py` —— 核心逻辑对齐，UI 部分不跟进（Rust 无 Streamlit）

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `sde_step_clip()` 的 `pub(crate)` 可见性对 `src/sdp/` 模块足够 | Architecture Patterns | `pub(crate)` 在整个 crate 内可见，但如果 SDP 模块不能通过 `crate::sde::clip::sde_step_clip` 访问，需要提升可见性 |
| A2 | `repair_json()` 在 Phase 7 的 6 步策略对 SDP 的 plot_extraction JSON 输出同样有效 | Code Examples | SDP plot_extraction JSON 结构（plot_points[]）比 SDE 的 scripts[] 更简单，修复策略应该足够 |
| A3 | SDP 不需要新增 Cargo.toml 依赖 | Standard Stack | 所有需要的库已在前面 phase 引入 |
| A4 | `src/sde/subtitle.rs` 提取为公共模块时不需要修改内部逻辑 | Architecture Patterns | 只有 `SdeError` 引用路径需要改为新模块的错误类型 |

## Open Questions (RESOLVED)

1. **`sde_step_clip()` 的可见性是否在 `sde/mod.rs` 和 `sde/clip.rs` 层面都已正确暴露？** — RESOLVED
   - What we know: `clip.rs` 使用 `pub(crate) async fn sde_step_clip()`，`mod.rs` 声明 `pub mod clip;`
   - What's unclear: SDP 模块引用时是否需要额外调整路径
   - Recommendation: 在实现前先 `cargo build` 验证；如有问题，将 `sde_step_clip` 提升为 `pub` 或调整引用方式
   - **Resolution:** Plan 04 的 `sdp_step_clip()` 未直接调用 `sde_step_clip()`，而是自行执行 `find_precise_range` + 调用 `clip_all_videos()`，避免了可见性问题。详见 Plan 04 实现。

2. **`subtitle_analysis` 和 `plot_extraction` 的参数是否需要 `custom_clips`（Python 版默认 5）？** — RESOLVED
   - What we know: Python 版 `subtitle_analysis` 接受 `custom_clips` 参数控制片段数量
   - What's unclear: Rust PromptTemplate 是否需要 `${custom_clips}` 占位符
   - Recommendation: 翻译 Prompt 时保留 `custom_clips` 参数，默认值 5，由 planner 决定是否暴露在 SdpRequest 中
   - **Resolution:** `custom_clips` 保留在 SdpRequest 中（默认 5），subtitle_analysis prompt 模板包含 `${custom_clips}` 占位符。详见 Plan 02-03。

3. **BGM 路径来源——从 `SdpRequest.bgm_path` 还是从 config 全局读取？** — RESOLVED
   - What we know: Python 版 UI 提供 BGM 选择，SDE 的 `DocumentaryRequest` 和 `SdeRequest` 都有 `bgm_path: Option<PathBuf>`
   - What's unclear: SdpRequest 是否跟随相同模式
   - Recommendation: 跟随 SdeRequest 模式，`bgm_path: Option<PathBuf>` 在 Request 级控制
   - **Resolution:** SdpRequest 包含 `bgm_path: Option<PathBuf>` 字段，跟随 SdeRequest 模式。详见 Plan 03 types.rs。

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | `cargo build` | ✓ | 待确认 | — |
| FFmpeg | ffmpeg-sidecar 内部调用 | ✓（需系统安装） | 待确认 | ffmpeg-sidecar download_ffmpeg feature |

**Missing dependencies with no fallback:**
- 无（所有外部依赖已在前面 phase 验证可用）

**Missing dependencies with fallback:**
- 无

## Validation Architecture

> Skipped — workflow.nyquist_validation not set in .planning/config.json (default: disabled).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | built-in `#[test]` |
| Config file | none — Cargo.toml dev-dependencies |
| Quick run command | `cargo test -p narratoai-core -- sdp` |
| Full suite command | `cargo test -p narratoai-core` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SDP-01 | 字幕解析（公共模块） | unit | `cargo test -p narratoai-core -- subtitle` | ❌ (new module) |
| SDP-01 | LLM JSON 修复 | unit | `cargo test -p narratoai-core -- sdp::script_gen` | ❌ (new) |
| SDP-01 | merge_script | unit | `cargo test -p narratoai-core -- sdp::script_gen` | ❌ (new) |
| SDP-01 | generate_sdp_script 编排 | unit (mocked LLM) | `cargo test -p narratoai-core -- sdp::script_gen` | ❌ (new) |
| SDP-02 | sdp_step_clip (复用 test) | unit | `cargo test -p narratoai-core -- sde::clip` | ✅ (existing) |
| SDP-02 | sdp_step_concat | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ (new) |
| SDP-02 | sdp_step_composite (no TTS/subtitle) | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ (new) |
| SDP-02 | OST 校验（全为 1） | unit | `cargo test -p narratoai-core -- sdp::pipeline` | ❌ (new) |
| SDP-02 | 参数 validate | unit | `cargo test -p narratoai-core -- sdp::types` | ❌ (new) |

### Wave 0 Gaps
- [ ] `src/sdp/script_gen.rs` — 单元测试覆盖 repair_json 复用 + merge_script + parse_script
- [ ] `src/sdp/pipeline.rs` — 单元测试覆盖 run_sdp 编排 + step_concat + step_composite
- [ ] `src/sdp/types.rs` — 单元测试覆盖 validate + default + ProgressStep order
- [ ] `src/sdp/error.rs` — 单元测试覆盖中文错误消息
- [ ] `src/subtitle/mod.rs` — 单元测试覆盖 parse_subtitle_file (从 SDE 迁移)

## Security Domain

> This phase has no new security exposure. All data inputs (subtitle files, video files) are local files specified by the user. LLM calls use the existing Phase 2 provider infrastructure. No network endpoints, no authentication, no user data storage.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | partial | 字幕文件路径存在性校验，非空校验，包含时间码校验（D-21） |

### Known Threat Patterns for Rust + FFmpeg

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious subtitle file with path traversal in content | Tampering | SDP only reads subtitle text content, does not execute or eval any part of it |
| Malformed JSON from LLM causing panic | DoS | `repair_json` is panic-free; `parse_script` uses safe deserialization with skips on invalid items |
| FFmpeg command injection via file paths with special chars | Tampering | Paths are escaped with `replace('\\', "/")` and passed as arguments (not shell), `run_ffmpeg` closure pattern prevents injection |

## Sources

### Primary (HIGH confidence)
- `E:\GitLib\NarratoAI\src\sde\pipeline.rs` — SDE 流水线编排模式参考
- `E:\GitLib\NarratoAI\src\sde\script_gen.rs` — `repair_json` 6 步策略 + LLM 调用模式
- `E:\GitLib\NarratoAI\src\sde\clip.rs` — `sde_step_clip()` OST=1 裁剪逻辑
- `E:\GitLib\NarratoAI\src\sde\subtitle.rs` — 待提取的字幕解析模块
- `E:\GitLib\NarratoAI\src\sde\error.rs` — `SdeError` 模板
- `E:\GitLib\NarratoAI\src\sde\types.rs` — `SdePipelineState` / `SdeRequest` 模板
- `E:\GitLib\NarratoAI\src\documentary\pipeline.rs` — documentary concat/composite 模式
- `E:\GitLib\NarratoAI\src\prompt\register.rs` — 现有 prompt 注册（须修改添加 SDP 模板）
- `E:\GitLib\NarratoAI\src\prompt\templates\short_drama_editing\plot_extraction_v2.0.md` — 现有的 plot_extraction 模板（须更新）
- `E:\GitLib\NarratoAI\Cargo.toml` — 依赖版本验证
- `E:\GitLib\NarratoAI\app\services\SDP\generate_script_short.py` — Python 版 SDP 脚本生成主入口
- `E:\GitLib\NarratoAI\app\services\SDP\utils\step1_subtitle_analyzer_openai.py` — Python 版两步 LLM 分析
- `E:\GitLib\NarratoAI\app\services\SDP\utils\step5_merge_script.py` — Python 版 merge_script
- `E:\GitLib\NarratoAI\app\services\SDP\utils\short_schema.py` — Python 版 SDP 数据模型
- `E:\GitLib\NarratoAI\app\services\prompts\short_drama_editing\subtitle_analysis.py` — Python 版 subtitle_analysis prompt
- `E:\GitLib\NarratoAI\app\services\prompts\short_drama_editing\plot_extraction.py` — Python 版 plot_extraction prompt

### Secondary (MEDIUM confidence)
- `E:\GitLib\NarratoAI\.planning\phases\08-sdp-pipeline\08-CONTEXT.md` — User decisions and constraints
- `E:\GitLib\NarratoAI\.planning\phases\08-sdp-pipeline\08-DISCUSSION-LOG.md` — Discussion log

### Tertiary (LOW confidence)
- None — all claims verified against source code or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified against Cargo.toml at known commit
- Architecture: HIGH — patterns directly copied from Phase 6/7 verified working code
- Pitfalls: HIGH — based on real Phase 7 implementation experience and documentary pipeline known issues

**Research date:** 2026-05-05
**Valid until:** 2026-06-05 (stable dependencies, no fast-moving libraries)
