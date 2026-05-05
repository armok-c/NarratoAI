# Phase 7: SDE Pipeline - Research

**Researched:** 2026-05-05
**Domain:** Short Drama Narration (SDE) — subtitle parsing, two-step LLM plot analysis + script generation, OST-marked JSON script output, video processing integration
**Confidence:** HIGH

## Summary

Phase 7 implements the Short Drama Explanation (SDE) pipeline in Rust: from subtitle file (SRT/ASS) to final narrated video output. The core difference from the Documentary pipeline (Phase 6) is the preprocessing stage — SDE uses subtitle text analysis via text LLM, while documentary uses video frame analysis via vision LLM. The video processing steps (TTS -> clip -> merge -> concat -> composite) are fully shared with Phase 6.

The pipeline consists of 9 steps: ParseSubtitle -> PlotAnalysis -> ScriptGenerate -> LoadScript -> Tts -> Clip -> MergeAudio -> Concat -> Composite. Steps 0-3 (subtitle parsing + two LLM calls + JSON repair) are SDE-specific; steps 4-8 (TTS onward) directly call Phase 6's `pub(crate)` step functions.

**Primary recommendation:** Create `src/sde/` module (8 files + mod.rs) mirroring documentary structure. Expose Phase 6 step functions as `pub(crate)`. Create `plot_analysis_v1.0.md` and upgrade `script_generation` to v2.0 prompt templates. No new crate dependencies needed beyond possibly `encoding_rs` for GBK subtitle detection.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### 模块结构
- **D-01:** 独立 `src/sde/` 模块，视频处理步骤直接调用 `src/documentary/` 已有函数
- **D-02:** 镜像纪录片 8 文件结构：`script_gen.rs`, `subtitle.rs`, `pipeline.rs`, `clip.rs`, `audio.rs`, `error.rs`, `types.rs`, `timestamp.rs` + `mod.rs`
- **D-03:** 暴露 Phase 6 私有函数为 `pub(crate)`：`step_tts`/`step_clip`/`step_merge_audio_subtitle`/`step_concat`/`step_composite`
- **D-04:** 新建 `SdeRequest`，包含 `subtitle_path`, `video_path`, `drama_name` + 步骤 2-6 所有参数，字段与 `DocumentaryRequest` 对齐
- **D-05:** 独立 9 步进度枚举 `SdeProgressStep`
- **D-06:** 原子入口 `run_sde()`，接收 `SdeRequest`
- **D-07:** `run_sde()` 接收字幕文件路径，内部自动完成脚本生成。提供独立 `analyze_subtitle_plot()` 和 `generate_sde_script()` API
- **D-08:** 字幕解析在 SDE 内部实现，编码探测放在 `src/sde/subtitle.rs`
- **D-09:** SdeRequest 视频处理参数与 DocumentaryRequest 一一对应
- **D-10:** 文件职责按纪录片镜像
- **D-11:** 新 `SdeError` 枚举 + 嵌入 `PipelineError`，通过 `From<PipelineError>` 自动转换
- **D-12:** JSON 修复在 SDE 内部，`repair_json()` 作为 `script_gen.rs` 私有函数
- **D-13:** 独立 `SdePipelineState`

#### LLM 编排
- **D-14:** 两步 LLM 调用统一使用 text 模型
- **D-15:** 纯字符串传递——`analyze_subtitle_plot()` 返回原始 LLM 输出
- **D-16:** `generate_sde_script()` 返回 `Result<String>`（原始 JSON），调用方负责解析
- **D-17:** 保存所有中间产物到 task_dir
- **D-18:** 通过 PromptManager 加载模板
- **D-19:** 统一 temperature 控制，默认 1.0
- **D-20:** LLM 调用 fail-fast，不重试
- **D-21:** `run_sde()` 内部串联所有步骤

#### 数据模型
- **D-22:** Python 版 1:1 字段映射
- **D-23:** 字符串前缀检测 OST=1——narration 字段以 "播放原片" 开头则跳过 TTS
- **D-24:** 复用 Phase 5 `validate_script()` 校验
- **D-25:** 时间戳格式直接映射
- **D-26:** SDE 仅使用 OST=0/1
- **D-27:** 运行时字段由视频处理回写
- **D-28:** 字幕时间戳存入辅助结构 `SubtitleSegment`
- **D-29:** OST=1 按字幕精确裁剪

### Claude's Discretion
- SdePipelineState 具体字段定义
- step 函数具体拆分和内部实现
- SdeError 枚举的完整变体列表和错误消息（中文）
- JSON 修复函数 `repair_json()` 具体实现（6 步回退策略）
- SRT 时间戳解析工具函数
- 字幕编码探测具体实现
- 进度百分比在各步骤分配
- `src/sde/mod.rs` 公开 API 设计
- `src/lib.rs` 新增 `pub mod sde;`

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SDE-01 | 短剧解说完整流水线编排（字幕解析 -> LLM 分析 -> 带OST脚本 -> 视频处理） | Verified: Phase 6 `pipeline.rs` exposes step functions; `run_sde()` orchestrates 9 steps sequentially; SdeRequest carries all params |
| SDE-02 | 字幕文本解析——从 SRT/ASS 文件提取对话内容 | Verified: Python `subtitle_text.py` encoding chain (utf8 -> utf16 -> gbk -> gb2312); `src/sde/subtitle.rs` will implement equivalent in Rust |
| SDE-03 | LLM 剧情分析——生成带 OST 类型标记的解说脚本 | Verified: Python `SubtitleAnalyzer` two-step pattern; `LlmProvider::generate_text()` with `LlmResponseFormat::Json` for script generation; PromptManager renders short_drama_narration templates |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Subtitle file reading + encoding detection | SDE (src/sde/subtitle.rs) | — | CPU-bound, isolated in spawn_blocking. No shared processing with documentary. |
| LLM plot analysis (Step 2a) | SDE (src/sde/script_gen.rs) | LLM Service (src/llm/) | SDE owns prompt composition and result parsing; LLM Service provides generic generate_text() |
| LLM script generation (Step 2b) | SDE (src/sde/script_gen.rs) | LLM Service (src/llm/) | Same pattern as Step 2a. SDE owns JSON repair + parsing into Script type. |
| TTS generation | Phase 6 (src/documentary/pipeline.rs) | TTS Core (src/tts/) | Called as pub(crate) step_tts(). SDE wraps but does not reimplement. |
| Video clipping | Phase 6 (src/documentary/clip.rs) | — | Called directly. OST=1 precise clipping uses SDE's subtitle_segments for precision. |
| Audio/subtitle merging | Phase 6 (src/documentary/audio.rs) | — | Called as pub(crate) step_merge_audio_subtitle(). Logic identical to documentary. |
| Video concatenation | Phase 6 (src/documentary/pipeline.rs) | — | Called as pub(crate) step_concat(). Identical to documentary. |
| Final composite | Phase 6 (src/documentary/pipeline.rs) | — | Called as pub(crate) step_composite(). Identical to documentary. |
| Prompt template management | Prompt System (src/prompt/) | — | PromptManager holds registry; templates stored in src/prompt/templates/short_drama_narration/ |
| Script validation | Phase 5 (src/script/mod.rs) | — | Reuse validate_script() per D-24. SDE calls after JSON parse. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.52.1 | Async runtime for all async pipeline steps | Already in Cargo.toml, used by all phases |
| serde | 1.0.228 | JSON deserialization for LLM output parsing | Already in Cargo.toml, ScriptClip uses serde derive |
| serde_json | 1.0.140 | JSON value manipulation for repair_json() | Already in Cargo.toml |
| thiserror | 2.0.18 | Error enum derivation for SdeError | Already in Cargo.toml, used by PipelineError |
| tracing | 0.1.44 | Structured logging at pipeline step boundaries | Already in Cargo.toml |
| uuid | 1.23.1 | Task ID generation | Already in Cargo.toml |
| regex | 1.11 | SRT timestamp parsing patterns | Already in Cargo.toml |
| sha2 | 0.11 | Content-addressed caching for plot analysis dedup | Already in Cargo.toml |

### Potential New Dependency
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| encoding_rs | 0.8.x | GBK/GB2312 subtitle file decoding in Rust | Only if std::str/from_utf16 cannot handle Chinese subtitle encodings. Alternative: manual byte-level detection heuristics. |

**Version verification:**
```bash
# All core deps are already in Cargo.toml at known versions
# encoding_rs is not yet in Cargo.toml — needs evaluation during planning
```

## Architecture Patterns

### System Architecture Diagram

```
[SdeRequest]
  subtitle_path: PathBuf
  video_path: PathBuf
  drama_name: String
  temperature: f64
  + video processing params (aligned with DocumentaryRequest)
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ run_sde() orchestrator (src/sde/pipeline.rs)            │
│                                                         │
│  Step 0: parse_subtitle()                               │
│    │  reads raw bytes → encoding detection chain        │
│    │  → normalize text → SubtitleSegments                    │
│    ▼                                                    │
│  Step 1: analyze_plot()                                 │
│    │  PromptManager::render("plot_analysis", v1.0)      │
│    │  → LlmProvider::generate_text() → plot_analysis    │
│    ▼                                                    │
│  Step 2: generate_script()                              │
│    │  PromptManager::render("script_generation", v2.0)   │
│    │  → LlmProvider::generate_text(Json) → raw JSON     │
│    ▼                                                    │
│  Step 3: repair_json() + parse_script()                 │
│    │  6-strategy repair → serde per-item parse          │
│    │  → validate_script() → Script (Vec<ScriptClip>)         │
│    ▼                                                    │
│  Step 4: step_tts() [pub(crate) from Phase 6]           │
│    │  Skip OST=1 clips ("播放原片" prefix detection)     │
│    │  → synthesize() per OST=0 clip → TtsResult         │
│    ▼                                                    │
│  Step 5: step_clip() [pub(crate) from Phase 6]          │
│    │  OST=0: clip from video start + TTS duration       │
│    │  OST=1: clip with precise subtitle timestamps      │
│    ▼                                                    │
│  Step 6: step_merge_audio_subtitle() [pub(crate)]        │
│    ▼                                                    │
│  Step 7: step_concat() [pub(crate)]                      │
│    ▼                                                    │
│  Step 8: step_composite() [pub(crate)]                   │
│    ▼                                                    │
│  output: PathBuf (final video path)                      │
└─────────────────────────────────────────────────────────┘
```

### Recommended Project Structure
```
src/sde/
├── mod.rs             # Public API: re-exports run_sde(), SdeRequest, SdeError, SdeProgressStep
├── pipeline.rs        # run_sde() orchestrator + SdePipelineState + 9 step functions
├── script_gen.rs      # step_analyze_plot(), step_generate_script(), repair_json(), parse_script()
├── subtitle.rs        # read_subtitle_file(), detect_encoding(), normalize_subtitle_text()
├── clip.rs            # Thin wrappers around documentary::pipeline::step_clip
├── audio.rs           # Thin wrappers around documentary::pipeline::step_merge_audio_subtitle
├── timestamp.rs       # parse_srt_timestamp(), SubtitleSegment, find_precise_timestamp()
├── types.rs           # SdeRequest, SdePipelineState, SdeProgressStep
├── error.rs           # SdeError enum with PipelineError embedding + From impls

Files to modify:
├── src/lib.rs         # + pub mod sde;
├── src/documentary/pipeline.rs  # Mark 5 step functions as pub(crate)
├── src/prompt/register.rs       # Register plot_analysis (v1.0) + upgrade script_generation to v2.0

New prompt templates:
├── src/prompt/templates/short_drama_narration/
│   ├── plot_analysis_v1.0.md    # NEW — from Python version
│   ├── script_generation_v2.0.md # NEW — upgrade from v1.0 with full short drama conventions
```

### Pattern 1: Step-by-Step Async Function Chain (Phase 6 D-01)
**What:** Each pipeline step is an independent async function. The orchestrator (`run_sde()`) calls them sequentially. Each step receives `&mut SdePipelineState` and writes its outputs into the state.
**When to use:** All pipeline steps.
**Source:** [VERIFIED: Phase 6 codebase] `src/documentary/pipeline.rs`

```rust
// Pattern from Phase 6, replicated for SDE:
async fn step_parse_subtitle(
    state: &mut SdePipelineState,
    subtitle_path: &Path,
) -> Result<(), SdeError> {
    // 1. Read file bytes
    let raw_bytes = tokio::fs::read(subtitle_path).await?;
    // 2. CPU-bound encoding detection (spawn_blocking)
    let (segments, text) = tokio::task::spawn_blocking(move || {
        detect_encoding_and_parse(&raw_bytes)
    }).await.map_err(|e| SdeError::Io { source: e })??;
    // 3. Update state
    state.subtitle_text = text;
    state.subtitle_segments = segments;
    Ok(())
}
```

### Pattern 2: Two-Step LLM with PromptTemplate (D-14, D-15, D-18)
**What:** Step 1 renders `plot_analysis` template (gets subtitle_text), calls `generate_text(text)`, stores raw analysis text. Step 2 renders `script_generation` template (injects plot_analysis as `$plot_analysis`), calls `generate_text(json)`, stores raw JSON.
**When to use:** The SDE-specific LLM orchestration.
**Source:** [VERIFIED: Python `short_drama_explanation.py`] `analyze_subtitle()` + `generate_narration_script()`

```rust
// src/sde/script_gen.rs
use std::collections::HashMap;

// Step 2a: Plot analysis
pub async fn step_analyze_plot(
    state: &mut SdePipelineState,
    subtitle_text: &str,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<(), SdeError> {
    let prompt = prompt_manager.render_prompt(
        "short_drama_narration", "plot_analysis", Some("v1.0"),
        &HashMap::from([("subtitle_content", subtitle_text)]),
    )?;
    let result = provider.generate_text(
        &prompt,
        Some("你是一个专业的短剧剧情分析师。提取人物关系、冲突转折、剧情高潮。"),
        Some(temperature as f32), Some(4096), None,
    ).await?;
    // Save intermediate artifact
    tokio::fs::write(state.task_dir.join("plot_analysis.txt"), &result).await?;
    state.plot_analysis = result;
    Ok(())
}

// Step 2b: Script generation (JSON output)
pub async fn step_generate_script(
    state: &mut SdePipelineState,
    provider: &dyn LlmProvider,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<(), SdeError> {
    let prompt = prompt_manager.render_prompt(
        "short_drama_narration", "script_generation", Some("v2.0"),
        &HashMap::from([
            ("plot_analysis", &state.plot_analysis),
            ("language", "zh-CN"),
            ("style", "幽默风趣"),
        ]),
    )?;
    let result = provider.generate_text(
        &prompt,
        Some("你是一个短剧解说脚本生成专家。严格按 JSON 格式输出 ScriptClip 数组。"),
        Some(temperature as f32), Some(8192),
        Some(LlmResponseFormat::Json),
    ).await?;
    // Save raw JSON (D-17)
    tokio::fs::write(state.task_dir.join("narration_raw.json"), &result).await?;
    state.narration_raw = result;
    Ok(())
}
```

### Anti-Patterns to Avoid
- **Duplicate video processing steps:** Do NOT reimplement TTS/clip/merge/concat/composite in SDE — call Phase 6's pub(crate) functions directly (D-01, D-03).
- **Holding RwLock across .await:** `PromptManager.render_prompt()` releases its read lock before returning. Never hold a `std::sync::RwLockReadGuard` across `.await` — it blocks the tokio worker thread. [CITED: AI-SPEC.md Section 4b.2]
- **Over-abstracting common logic:** Do NOT extract shared pipeline infrastructure between documentary and SDE into a common module — wait for Phase 8 (SDP) to identify true duplication (D-01, D-08).
- **Passing subtitle_text as template variable to script_generation:** Python v2.0 passes `${subtitle_content}` to the template. The current Rust register.rs for `script_generation` v1.0 does NOT have this parameter. Must ensure v2.0 template either includes it or the prompt encodes timestamp info in `plot_analysis`.
- **Unbounded max_tokens:** Always set explicit max_tokens (4096 for plot analysis, 8192 for script generation). Never pass None. [CITED: AI-SPEC.md Section 4b.3]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LLM chat completion | Custom HTTP requests with retry logic | `LlmProvider::generate_text()` via `src/llm/` | Already handles JSON fallback, retry, proxy, streaming. Phase 2 delivered verified abstraction. |
| Prompt template rendering | Custom string replacement engine | `PromptManager::render_prompt()` via `src/prompt/` | Already handles variable interpolation, parameter validation, versioning, registry. Phase 4 delivered verified abstraction. |
| Script loading/saving/validation | Custom JSON parse + validate | `crate::script::load_script()` / `save_script()` / `validate_script()` | Phase 5 delivered verified functions with round-trip tests. Includes ScriptError and per-field ValidationError. |
| Video clipping FFmpeg | Custom ffmpeg arguments | `clip_all_videos()` / `clip_ost1_original_sound()` in `src/documentary/clip.rs` | Phase 6 handles HW acceleration detection, OS-specific path escaping, error handling. |
| Audio merge FFmpeg | Custom ffmpeg filter chain | `merge_audio_files()` in `src/documentary/audio.rs` | Phase 6 handles silent base generation, delay calculation, amix filter. |
| SRT timestamp generation | Manual SRT block construction | `generate_srt_from_word_boundaries()` in `src/documentary/subtitle.rs` | Phase 6 handles word boundary parsing, offset application, formatting. |

**Key insight:** Video processing (steps 4-8) is identical between documentary and SDE pipelines. The only difference is script provenance (frame analysis vs. subtitle analysis) and OST ratio (SDE uses OST=0/1, documentary uses OST=0/1/2). Sharing these steps via `pub(crate)` avoids ~400 lines of duplicate FFmpeg code.

## Common Pitfalls

### Pitfall 1: "播放原片+N" Convention Mismatch
**What goes wrong:** OST=1 clips have `narration` = "播放原片+N" (e.g., "播放原片3"). The TTS step must detect this prefix and skip synthesis. If the LLM outputs a different format (e.g., "播放原片" without number, or "原声" instead), the pipeline either synthesizes unwanted TTS or skips actually-needed synthesis.
**Why it happens:** The convention is enforced only in the prompt template. LLM may deviate from the requested format.
**How to avoid:** `step_tts()` (or its SDE wrapper) must check: `clip.narration.starts_with("播放原片")` to suppress TTS for OST=1 (D-23). Also validate in `parse_script()` that OST=1 clips have narration starting with "播放原片". Log a warning if they don't.
**Warning signs:** TTS is generated for all clips including OST=1 (wrong), or OST=1 clips produce silent audio segments.

### Pitfall 2: GBK/GB2312 Subtitle Encoding Fails Silently
**What goes wrong:** Chinese subtitle files are commonly GBK-encoded (especially on Windows from Chinese subtitle tools). Rust's `std::str::from_utf8()` rejects them silently. The pipeline produces empty or garbled text, leading to meaningless plot analysis.
**Why it happens:** Rust's std library only handles UTF-8 natively. UTF-16 can be handled via `String::from_utf16()`. GBK/GB2312 require a third-party crate.
**How to avoid:** Implement the full encoding fallback chain (D-08): UTF-8 -> UTF-8-SIG -> UTF-16 -> GBK -> GB2312. For GBK/GB2312, add `encoding_rs` crate to Cargo.toml. Wrap the CPU-bound detection in `tokio::task::spawn_blocking()`. [CITED: AI-SPEC.md Section 4b.2]
**Warning signs:** `subtitle_text` is empty or contains garbled characters. Plot analysis output mentions unrelated topics.

### Pitfall 3: PromptManager.render_prompt() Parameter Mismatch
**What goes wrong:** `render_prompt("short_drama_narration", "script_generation", Some("v2.0"), &vars)` fails because the newly registered v2.0 template expects a parameter (e.g., `drama_name` or `subtitle_content`) that SDE code does not pass.
**Why it happens:** Python v2.0's `script_generation` template expects `${drama_name}`, `${plot_analysis}`, and `${subtitle_content}`. The current Rust v1.0 template expects `${plot_analysis}`, `${language}`, `${style}`. These are incompatible parameter sets.
**How to avoid:** When upgrading the template to v2.0 in `register.rs`, define its `parameters` list to match exactly what `step_generate_script()` will pass. The Python v2.0 template has parameters `["drama_name", "plot_analysis", "subtitle_content"]`; the SDE code only has `plot_analysis` at that point (subtitle text is parsed in step 0). Either pass `subtitle_content` too, or restructure the template to not need it.
**Warning signs:** `PromptError::Validation("缺少必需参数: drama_name")` at runtime.

### Pitfall 4: Timestamp Overlap in SDE-Generated Scripts
**What goes wrong:** LLM outputs overlapping timestamps (e.g., clip 3 ends at 00:05:30 but clip 4 starts at 00:05:25), causing frame duplication in the output video.
**Why it happens:** LLMs are notoriously bad at precise numeric boundary calculations, especially with time ranges.
**How to avoid:** Add the overlap guard (G2) immediately after `parse_script()` in `run_sde()` — linear O(n) scan checking that each clip's start >= previous clip's end. Abort with `SdeError::Validation` before any compute-intensive video processing begins. [CITED: AI-SPEC.md Section 6.1 G2]
**Warning signs:** TTS generation time seems high (because many overlapping clips), or final video has repeated frames.

### Pitfall 5: OST=1 Precise Clipping with Imprecise LLM Timestamps
**What goes wrong:** LLM outputs approximate timestamps for OST=1 segments (e.g., "00:00:05,000-00:00:15,000" when the actual subtitle spans "00:00:05,200-00:00:14,800"). Using LLM timestamps directly produces slightly misaligned clips.
**Why it happens:** LLMs round timestamps to convenient boundaries. Subtitle timestamps from SRT files are frame-accurate.
**How to avoid:** During `step_parse_subtitle()`, store parsed `SubtitleSegment` entries with precise timestamps (D-28). In `step_clip()` (or its SDE wrapper), for OST=1 clips, use `find_precise_range()` from `timestamp.rs` to snap LLM timestamps to the nearest original subtitle boundaries (D-29). [CITED: AI-SPEC.md Section 4b.4 `find_precise_range()`]
**Warning signs:** OST=1 clips appear slightly before or after the intended dialogue.

## Prompt Template Analysis

### Register.rs Current State
```rust
// In register_prompts_short_drama_narration():
// Registers ONLY "script_generation" v1.0 — NO "plot_analysis" registered
// v1.0 template params: ["plot_analysis", "language", "style"]
```

### Required Changes to register.rs (D-18)
1. Register new `plot_analysis` (v1.0) with parameters: `["subtitle_content"]`
2. Upgrade `script_generation` from v1.0 to v2.0. Python v2.0 template expects parameters: `["drama_name", "plot_analysis", "subtitle_content"]`
3. Decision point: should `script_generation` v2.0 accept `drama_name`? The SDE code passes only `plot_analysis`, `language`, `style` currently. D-07 says SdeRequest contains `drama_name`. Option A: Pass drama_name to template. Option B: Make drama_name optional in the template. **Recommendation: Option A** — align with Python v2.0.

### Template Variables Comparison
| Variable | plot_analysis (Python) | script_generation (Python v2.0) | script_generation (Rust v1.0, current) | Notes |
|----------|----------------------|--------------------------------|----------------------------------------|-------|
| subtitle_content | required | required | — | Full subtitle text with timestamps |
| plot_analysis | — | required | required | Output of step 1 |
| drama_name | — | required | — | Short drama name from SdeRequest |
| language | — | — | optional (default zh-CN) | |
| style | — | — | optional (default 幽默风趣) | |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Python SubtitleAnalyzer (requests-based HTTP) | Rust LlmProvider (async-openai) | Phase 2 | SDE uses generate_text() instead of manual HTTP calls. JSON fallback handled by LlmResponseFormat::Json. |
| Python PromptManager.get_prompt() | Rust PromptManager.render_prompt() | Phase 4 | SDE uses PromptManager for template rendering. Parameter validation done before LLM call. |
| Python `parse_and_fix_json()` (7 strategies) | Rust `repair_json()` (6 strategies) | This phase | Rust version drops the "default fallback with single item" strategy. Fail-fast instead. |
| Python `analyze_subtitle()` + `generate_narration_script()` | Rust `step_analyze_plot()` + `step_generate_script()` | This phase | Same two-step pattern. Rust adds explicit max_tokens and system prompt. |

**Deprecated/outdated:**
- Python `_call_native_gemini_api()`: Rust unified via OpenAI-compatible protocol. No native Gemini API calls in Rust.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `encoding_rs` crate is needed for GBK subtitle decoding | Standard Stack | If std's encoding detection heuristics suffice (e.g., `String::from_utf8_lossy` + byte pattern analysis), we can avoid the dependency. But GBK detection without a proper decoder will have false positives. |
| A2 | Phase 6's `step_tts()` can be called as-is with OST=1 narration = "播放原片+N" | Architecture Patterns | If `step_tts()` does not check narration prefix and attempts TTS on OST=1 clips, it will try to synthesize "播放原片3" as audio. The SDE wrapper around step_tts() must filter out OST=1 clips before calling. |
| A3 | `script_generation` v2.0 template should accept `drama_name` parameter | Prompt Template Analysis | If the template does not reference drama_name (or makes it optional), there's no issue. But Python v2.0 uses it, and Rust should align. |
| A4 | No additional Rust crates needed for SDE beyond encoding_rs | Standard Stack | If we need ASS subtitle format parsing (beyond basic text extraction), a subtitle parsing crate may be needed. ASS format has complex styling that simple text extraction may miss. |

## Open Questions

1. **Should `encoding_rs` be added to Cargo.toml?**
   - What we know: Python's encoding chain tries UTF-8 -> UTF-8-SIG -> UTF-16 -> GBK -> GB2312. Rust std supports UTF-8 and UTF-16 natively. GBK/GB2312 require a decoder.
   - What's unclear: Whether the alternative approach (byte-level pattern heuristics + `String::from_utf8_lossy`) would work for the common Chinese SRT files encountered in practice.
   - Recommendation: Add `encoding_rs = "0.8"` to Cargo.toml during planning. It's a pure-Rust crate, well-maintained, and adds ~100KB to binary. The safety of correct GBK detection outweighs the minor dependency cost.

2. **Should `script_generation` v2.0 template include `subtitle_content`?**
   - What we know: Python v2.0 template expects `${subtitle_content}` for precise timestamp mapping. The Rust SDE code parses subtitle segments in step 0 and stores them in `SdePipelineState.subtitle_segments`.
   - What's unclear: Whether passing the full subtitle text to the script generation prompt improves timestamp accuracy or just adds noise to the context window.
   - Recommendation: Include `${subtitle_content}` in the v2.0 template (align with Python). Pass from SDE code as: `vars.insert("subtitle_content", &state.subtitle_text)`. An empty template variable check in render_prompt() will catch mismatch at compile time.

3. **SDE progress percentage distribution?**
   - What we know: 9 steps. Documentary uses linear distribution (0, 20, 60, 70, 80, 100).
   - What's unclear: Optimal allocation for SDE's 9 steps. ParseSubtitle and LLM calls are variable-duration.
   - Recommendation: ParseSubtitle=0, PlotAnalysis=15, ScriptGenerate=30, LoadScript=40, Tts=50, Clip=70, MergeAudio=80, Concat=90, Composite=100. Adjust if LLM calls dominate timing.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust built-in) |
| Config file | none — see Wave 0 |
| Quick run command | `cargo test --lib sde` |
| Full suite command | `cargo test --lib sde && cargo test --test sde_e2e` (if e2e test created) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SDE-01 | Pipeline orchestrator executes steps in correct order | unit | `cargo test sde::pipeline::tests` | ❌ Wave 0 |
| SDE-02 | Subtitle parsing handles UTF-8/UTF-16/GBK encoding | unit | `cargo test sde::subtitle::tests` | ❌ Wave 0 |
| SDE-02 | Encoding fallback chain exhausts all candidates | unit | `cargo test sde::subtitle::tests` | ❌ Wave 0 |
| SDE-02 | SRT/ASS timestamp parsing | unit | `cargo test sde::timestamp::tests` | ❌ Wave 0 |
| SDE-03 | `repair_json()` handles all 6 strategies | unit | `cargo test sde::script_gen::tests` | ❌ Wave 0 |
| SDE-03 | `parse_script()` handles OST=0/1 defaults correctly | unit | `cargo test sde::script_gen::tests` | ❌ Wave 0 |
| SDE-03 | "播放原片" prefix detection skips TTS for OST=1 | unit | `cargo test sde::pipeline::tests` | ❌ Wave 0 |
| SDE-01 | Timestamp overlap detection guard (G2) | unit | `cargo test sde::pipeline::tests` | ❌ Wave 0 |
| SDE-01 | Empty script guard (G5) | unit | `cargo test sde::pipeline::tests` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib sde`
- **Per wave merge:** `cargo test --lib sde`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/sde/pipeline.rs` — test module with pipeline orchestration tests (overlap guard, empty script guard)
- [ ] `src/sde/subtitle.rs` — test module with encoding detection tests (UTF-8, UTF-16, GBK fixtures)
- [ ] `src/sde/script_gen.rs` — test module with repair_json strategy tests + parse_script tests
- [ ] `src/sde/error.rs` — test module verifying Chinese Display messages
- [ ] `src/sde/types.rs` — test module verifying SdeRequest validation + default values

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | File path validation for subtitle_path + video_path in SdeRequest.validate() |

### Known Threat Patterns for SDE Pipeline

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via subtitle_path | Tampering | Validate path is absolute, within expected directories. Already handled by documentary patterns. |
| Malformed subtitle file causing DoS | Denial of Service | spawn_blocking with timeout for encoding detection. FFmpeg input bounds-checked by ffmpeg-sidecar. |

## Sources

### Primary (HIGH confidence)
- [VERIFIED: Phase 6 codebase] `src/documentary/pipeline.rs` — step function signatures and PipelineState pattern
- [VERIFIED: Phase 6 codebase] `src/documentary/types.rs` — DocumentaryRequest fields and defaults
- [VERIFIED: Phase 6 codebase] `src/documentary/error.rs` — PipelineError enum (SdeError embedding target)
- [VERIFIED: Phase 5 codebase] `src/script/types.rs` — ScriptClip + OstType data model
- [VERIFIED: Phase 5 codebase] `src/script/mod.rs` — validate_script(), load_script(), save_script()
- [VERIFIED: Phase 2 codebase] `src/llm/provider.rs` — LlmProvider trait with generate_text() signature
- [VERIFIED: Phase 2 codebase] `src/llm/types.rs` — LlmResponseFormat enum
- [VERIFIED: Phase 4 codebase] `src/prompt/manager.rs` — PromptManager::render_prompt() signature
- [VERIFIED: Phase 4 codebase] `src/prompt/register.rs` — Current short_drama_narration registration (only script_generation v1.0)
- [VERIFIED: Python codebase] `app/services/SDE/short_drama_explanation.py` — SubtitleAnalyzer two-step pattern
- [VERIFIED: Python codebase] `app/services/subtitle_text.py` — Encoding detection chain (7 candidates)
- [VERIFIED: Python codebase] `webui/tools/generate_short_summary.py` — parse_and_fix_json() 7 strategies
- [VERIFIED: Python codebase] `app/services/prompts/short_drama_narration/plot_analysis.py` — Template expects ${subtitle_content}
- [VERIFIED: Python codebase] `app/services/prompts/short_drama_narration/script_generation.py` — Template v2.0 expects ${drama_name}, ${plot_analysis}, ${subtitle_content}
- [CITED: CONTEXT.md D-01 through D-29] All 29 implementation decisions

### Secondary (MEDIUM confidence)
- [CITED: AI-SPEC.md Section 4b.2] `spawn_blocking` for CPU-bound encoding detection
- [CITED: AI-SPEC.md Section 6.1] Guardrails G1-G6 (JSON parse, overlap, OST ratio, encoding fallback, empty script, duration)
- [CITED: AI-SPEC.md Section 5] 7-dimension evaluation strategy (E1-E7)

### Tertiary (LOW confidence)
- [ASSUMED] `encoding_rs` crate needed for GBK decoding — not yet verified against actual subtitle files in the test dataset
- [ASSUMED] ASS subtitle format does not require special parsing beyond text extraction — ASS styling info may be useful for future features

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all verified against existing Cargo.toml and Phase 2/4 codebase
- Architecture: HIGH — patterns verified against Phase 6 codebase execution
- Pitfalls: HIGH — sourced from both Python version bugs and Phase 6 operational knowledge
- Encoding detection: MEDIUM — encoding_rs dependency is unverified

**Research date:** 2026-05-05
**Valid until:** 2026-06-05 (30 days — stable Rust ecosystem, but LLM API behavior evolves)
