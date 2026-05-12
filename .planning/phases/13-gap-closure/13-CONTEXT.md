# Phase 13: Gap Closure — Documentary Prompt + Phase 11 Integration - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

补齐纪录片模式 Prompt 模板的实际调用（模板已注册但流水线绕过 PromptManager），确保 Phase 11 扩展功能（音频标准化、智能音量）与三大流水线（Documentary、SDE、SDP）正确集成。

**覆盖需求：** PRMP-03（纪录片 Prompt 补全）、EXTD-01（音频标准化集成）、EXTD-04（智能音量集成）

**现状发现：**
- Prompt 模板和注册已完成（Phase 4），但 Documentary 的 `generate_narration()` 和 `analyze_video()` 用硬编码 prompt 绕过了 PromptManager
- Phase 11 音频模块（`src/audio/normalizer.rs`、`volume.rs`）已完整实现但零个流水线引用（零 `use crate::audio` 出现在任何流水线模块）
- SDE 和 SDP 已正确使用 PromptManager ✅
</domain>

<decisions>
## Implementation Decisions

### 音频标准化集成
- **D-01:** 合并后统一标准化——`merge_audio_files()` 完成后立即对输出调用 `normalize_audio_for_mixing()`，嵌入 step 4 内部。不新增独立步骤。
- **D-02:** 直接调用 `normalize_audio_for_mixing()`（Phase 11 已封装 try loudnorm → RMS 回退编排逻辑）。
- **D-03:** 原地覆盖 — normalize 输出直接覆盖 `merger_audio.mp3`，PipelineState 无需新增字段。
- **D-04:** target_lufs 全局统一为 -14.0 LUFS（YouTube/流媒体标准）。
- **D-05:** 参数来源 — 从 `config.toml [audio]` 段读取 `target_lufs` / `max_peak` / `enable_audio_normalization`。
- **D-06:** 长音频不做特殊处理（FFmpeg loudnorm 原生支持任意时长）。
- **D-07:** SDP 标准化策略 — 对输入视频的原声音频轨道做 loudnorm 标准化后再参与混音（SDP 没有 merge_audio 步骤）。

### 智能音量集成策略
- **D-08:** config 开关控制 — `enable_smart_volume = true` 时调用 `get_optimized_volumes(video_type)` 覆盖请求结构体的音量字段；false 时保持现有行为。
- **D-09:** `get_optimized_volumes()` 新增 `"documentary"` / `"sde"` / `"sdp"` 三个匹配分支。保留现有分支做 fallback。
- **D-10:** step_composite 开头检查 config，若启用智能音量则覆盖 `tts_volume` / `original_volume` / `bgm_volume`。
- **D-11:** 保持请求结构体（DocumentaryRequest/SdeRequest/SdpRequest）的字段和默认值不变。智能音量覆盖发生在流水线内部。
- **D-12:** BGM 音量和淡出各管各的——智能音量只负责 `bgm_volume`（响度），MixingConfig 的 `bgm_fade_out` 负责淡出时长。
- **D-13:** step_composite 开头用 `tracing::info!` 输出最终生效的音量值和来源（smart/manual）。
- **D-14:** AudioSection 新增 `volume_profile: String` 字段，默认 `"balanced"`，作为 `apply_volume_profile()` 的备选入口。
- **D-15:** 不需要 OST 感知——音量按 video_type 全局设定，片段 OST 差异由流水线固有 OST 分支处理。

### Prompt 调用链修复（仅 Documentary）
- **D-16:** `generate_narration()` 重构为通过 `PromptManager.render_prompt("documentary", "narration_generation", Some("v2.0"), &vars)` 获取 Prompt。
- **D-17:** `analyze_video()` 重构为通过 `PromptManager.render_prompt("documentary", "frame_analysis", Some("v1.0"), &vars)` 获取 Prompt，移除硬编码 fallback。
- **D-18:** `generate_documentary_script()` 新增 `prompt_manager: &PromptManager` 参数，向下传递给 `analyze_video()` 和 `generate_narration()`。
- **D-19:** `narration_generation_v2.0.md` 模板增强——在「输出格式」中明确要求 JSON 格式、`items` 数组结构和每个 item 的字段（picture、narration、timestamp、OST）。
- **D-20:** SDE 和 SDP 已验证通过 `prompt_manager.render_prompt()` 正确使用注册模板，无需修改。

### 代码架构
- **D-21:** 新建 `src/audio/pipeline.rs`，提供两个共享辅助函数：
  - `normalize_merged_audio(input: &Path, config: &AudioSection) -> Result<PathBuf, AudioError>` — 检查开关 → 调用 normalize_audio_for_mixing
  - `resolve_volumes(request_volumes: &VolumeConfig, video_type: &str, config: &AudioSection) -> VolumeConfig` — 检查开关 → 调用 get_optimized_volumes 或返回原值
- **D-22:** 各流水线 `PipelineError` 新增 `AudioNormalization { details: String }` 变体，实现 `From<AudioError>` 自动转换。
- **D-23:** `step_merge_audio_subtitle()` 和 `step_composite()` 新增 `config: &AppConfig` 参数。主编排函数入口传入。

### Phase 11 模块微调
- **D-24:** `ProcessingConfig::default()` 的 `target_lufs` 从 -20.0 更新为 -14.0。
- **D-25:** `AudioSection::default()` 的 `target_lufs` 从 -23.0 更新为 -14.0，新增 `volume_profile: "balanced".into()`。
- **D-26:** `config.example.toml [audio]` 段全面更新：
  - `target_lufs = -14.0`（注释更新为 YouTube/流媒体标准）
  - 新增 `volume_profile = "balanced"`
- **D-27:** `config/types.rs` 测试中 `target_lufs` 断言从 -23.0 更新为 -14.0，新增 `volume_profile` 断言。

### 配置与开关
- **D-28:** `enable_audio_normalization` / `enable_smart_volume` 默认值保持 `false`（向后兼容）。用户需手动开启。
- **D-29:** Tauri 命令层不需要变更——config 和 prompt_manager 已通过 Tauri State 传递。

### 验证与测试
- **D-30:** 三层验证策略：单元测试（辅助函数逻辑）+ 集成测试（流水线带标准化运行）+ 手动 E2E（真实视频验证 LUFS）。
- **D-31:** 集成测试用 ffprobe loudnorm 分析输出音频，断言 `integrated_lufs ≈ -14.0`。
- **D-32:** 智能音量集成测试构造 config（`enable_smart_volume=true`）→ 调用 `resolve_volumes()` → 断言返回值与 `get_optimized_volumes("documentary")` 一致。
- **D-33:** `cargo test` 全量运行排查受影响测试，更新因默认值变更而失败的断言。
- **D-34:** 共享辅助函数（`normalize_merged_audio` / `resolve_volumes`）需单元测试覆盖：开关开启/关闭、RMS 回退、智能音量覆盖/手动模式。

### Claude's Discretion
- `get_optimized_volumes()` 中 documentary/sde/sdp 分支的精确音量值（可根据各模式特性自行设定合理值）
- `src/audio/pipeline.rs` 的具体函数签名和参数设计
- `narration_generation_v2.0.md` JSON 格式说明的具体措辞
- 集成测试中 ffprobe LUFS 读数的容差范围

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 11 音频模块（已实现，待集成）
- `narratoai-core/src/audio/normalizer.rs` — `normalize_lufs()`、`normalize_audio_for_mixing()`、`analyze_lufs()`、`get_audio_rms()`、`calculate_volume_adjustment()`
- `narratoai-core/src/audio/volume.rs` — `get_optimized_volumes()`、`apply_volume_profile()`、`get_recommended_volumes_for_content()`、`VolumeConfig`、`ProcessingConfig`
- `narratoai-core/src/audio/mod.rs` — 模块导出
- `.planning/phases/11-extended-features/11-CONTEXT.md` — Phase 11 决策（D-01~D-07 音频标准化，D-16~D-22 智能音量，D-25~D-27 错误类型，D-28~D-29 测试策略）

### Documentary 流水线（Phase 6 已交付，本次修改目标）
- `narratoai-core/src/documentary/pipeline.rs` — 6 步流水线：step_load_script、step_tts、step_clip、step_merge_audio_subtitle、step_concat、step_composite
- `narratoai-core/src/documentary/audio.rs` — `merge_audio_files()`、`merge_subtitle_files()`、`calculate_clip_duration()`
- `narratoai-core/src/documentary/script_gen.rs` — `generate_documentary_script()`、`analyze_video()`、`generate_narration()`（**本次重点重构对象**）
- `narratoai-core/src/documentary/types.rs` — `DocumentaryRequest`、`PipelineState`
- `narratoai-core/src/documentary/error.rs` — `PipelineError`（需新增 AudioNormalization 变体）
- `.planning/phases/06-documentary-pipeline/06-CONTEXT.md` — Phase 6 决策（D-01 Step-by-Step、D-02 PipelineState、D-06 fail-fast、D-12 独立预处理）

### SDE/SDP 流水线（本次集成目标）
- `narratoai-core/src/sde/pipeline.rs` — 9 步流水线，复用 `merge_audio_files()`
- `narratoai-core/src/sde/script_gen.rs` — 正确使用 PromptManager ✅
- `narratoai-core/src/sde/types.rs` — SdeRequest（含音量字段）
- `narratoai-core/src/sdp/pipeline.rs` — 4 步流水线（无 merge_audio，有 BGM 混音）
- `narratoai-core/src/sdp/script_gen.rs` — 正确使用 PromptManager ✅
- `narratoai-core/src/sdp/types.rs` — SdpRequest（含音量字段）

### Prompt 系统
- `narratoai-core/src/prompt/register.rs` — `register_all_prompts()` → `register_documentary_prompts()`（已注册 frame_analysis v1.0 + narration_generation v2.0）
- `narratoai-core/src/prompt/manager.rs` — `PromptManager::render_prompt()`（SDE/SDP 已正确调用，Documentary 待修复）
- `narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md` — 帧分析模板（参数：video_description、language）
- `narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md` — 文案生成模板（参数：video_title、frame_analysis_json、language、style）（**本次需增强 JSON 格式要求**）

### 配置系统
- `narratoai-core/src/config/types.rs` — `AudioSection` 定义（需新增 volume_profile 字段、更新测试断言）
- `narratoai-core/src/config/defaults.rs` — `AudioSection::default()`（target_lufs 需更新为 -14.0、新增 volume_profile）
- `config.example.toml` — `[audio]` 段模板（需全面更新默认值+注释+新增字段）

### SDE/SDP Prompt 模板（已验证正确调用）
- `narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md`
- `narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md`
- `narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md`
- `narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md`

### 项目规划文档
- `.planning/ROADMAP.md` — Phase 13 定义、成功标准、依赖（Phase 6 + Phase 11）
- `.planning/REQUIREMENTS.md` — 覆盖需求：PRMP-03、EXTD-01、EXTD-04
- `.planning/PROJECT.md` — 项目约束

### Python 版参考
- `app/services/task.py` — 核心流水线中音频标准化调用点
- `app/services/audio_normalizer.py` — Python 版 LUFS 标准化实现

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/audio/normalizer.rs` — `normalize_audio_for_mixing()` 已封装 loudnorm→RMS 回退，流水线一行调用即可
- `src/audio/volume.rs` — `get_optimized_volumes()` / `apply_volume_profile()` 已实现，待扩展 documentary/sde/sdp 分支
- `src/prompt/manager.rs` — `render_prompt()` API，SDE/SDP 已示范正确调用模式
- `src/prompt/register.rs` — `register_all_prompts()` 已注册所有 Prompt
- `src/documentary/audio.rs` — `merge_audio_files()` + `merge_subtitle_files()` 可内联标准化调用
- `src/documentary/pipeline.rs` — 主编排函数已接收 request + config 参数，新增 config 参数改动量小
- `src/sde/pipeline.rs` — 已示范正确的 `prompt_manager: &PromptManager` 参数传递模式

### Established Patterns
- **PromptManager 参数传递**（SDE/SDP）：`generate_*_script(request, provider, prompt_manager, temp)` — Documentary 应对齐此模式
- **领域错误枚举**（Phase 1 D-16/D-17）：thiserror 派生，中文 Display
- **模块级结构**：`src/audio/pipeline.rs` 新增，沿袭 `src/{domain}/` 约定
- **fail-fast 错误恢复**（Phase 6 D-06）：标准化失败即终止流水线
- **step 函数签名**：逐步扩展参数（已有 request + state，本次新增 config）

### Integration Points
- `src/documentary/pipeline.rs:step_merge_audio_subtitle()` → 调用 `normalize_merged_audio()`
- `src/documentary/pipeline.rs:step_composite()` → 调用 `resolve_volumes()`
- `src/documentary/script_gen.rs:generate_narration()` → 改用 `PromptManager::render_prompt()`
- `src/documentary/script_gen.rs:analyze_video()` → 改用 `PromptManager::render_prompt()`
- `src/sde/pipeline.rs` → 音频标准化+智能音量集成（同上模式）
- `src/sdp/pipeline.rs` → 原始音频标准化+智能音量集成（适配无 merge_audio 架构）
- `src/audio/mod.rs` → 新增 `pub mod pipeline;`
- `src/config/types.rs` → AudioSection 新增 volume_profile
- `config.example.toml` → [audio] 段更新
- `narratoai-core/src/lib.rs` → 无需变更（audio 模块已在 Phase 11 导出）

### Key Patterns from SDE/SDP to Follow
SDE `script_gen.rs` 的正确 Prompt 调用模式：
```rust
let mut vars = HashMap::new();
vars.insert("subtitle_content", &state.subtitle_text as &str);
let prompt = prompt_manager.render_prompt(
    "short_drama_narration", "plot_analysis", Some("v1.0"), &vars
)?;
let result = provider.generate_text(&prompt, Some(system_prompt), ...)?;
prompt_manager.validate_output(&result, &OutputFormat::Text)?;
```
Documentary 应完全对齐此模式。

</code_context>

<specifics>
## Specific Ideas

- Documentary Prompt 调用链修复是 Phase 13 的核心任务——模板已注册但从未被使用，这与 Python 版的 Prompt 系统设计相悖
- 音频标准化嵌入 step 4 的模式（合并后标准化、原地覆盖）与 Python 版 `task.py` 内联模式一致
- SDE/SDP 的 PromptManager 集成是可参考的实现模板——Documentary 只需对齐相同的 `render_prompt()` 调用模式
- 共享辅助函数 `src/audio/pipeline.rs` 避免三个流水线重复相同的开关检查和函数调用
- `narration_generation_v2.0.md` 模板增强 JSON 格式要求后，需要与 `generate_narration()` 中的 JSON 解析逻辑（`parse_script_clips()`）保持一致
- config.example.toml 的 [audio] 段注释提及 EBU R128，更新为 -14.0 后注释应反映 YouTube/流媒体标准
- 集成测试用 ffprobe loudnorm 验证时，容差建议 ±2 LUFS（考虑到 FFmpeg loudnorm 的精度）
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 13-gap-closure*
*Context gathered: 2026-05-12*
