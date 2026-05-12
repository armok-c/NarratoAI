# Phase 13: Gap Closure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 13-gap-closure
**Areas discussed:** 音频标准化集成点, 智能音量集成策略, 集成范围, 配置驱动 vs 显式调用, Prompt 调用链验证, 配置与命令层更新, Phase 11 模块微调, SDE/SDP Prompt 调用链验证, 已有测试影响分析, 共享辅助函数测试

---

## 音频标准化集成点

| Option | Description | Selected |
|--------|-------------|----------|
| 合并后统一标准化 | step 4 合并完成后做一次 loudnorm，高效一次调用 | ✓ |
| 逐片段标准化 | 每个 TTS 片段单独标准化，调用次数多 | |
| 两者都做 | 先逐片段粗调 + 再合并后精调 | |

| Option | Description | Selected |
|--------|-------------|----------|
| RMS 回退 | loudnorm 失败时 symphonia 计算 RMS 增益 | ✓ |
| 静默跳过 | 失败时保留原音频，仅 log 警告 | |
| 硬失败 | 失败即终止流水线 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 原地覆盖 | normalize 覆盖 merger_audio.mp3，向后兼容 | ✓ |
| 保留两份 | 生成 merger_audio_normalized.mp3 保留原始 | |

| Option | Description | Selected |
|--------|-------------|----------|
| -14 LUFS | 流媒体/YouTube 标准 | ✓ |
| -20 LUFS | Phase 11 ProcessingConfig 当前默认值 | |
| -23 LUFS | EBU R128 广播标准 | |

| Option | Description | Selected |
|--------|-------------|----------|
| config.toml 驱动 | 从 [audio] 段读取参数 | ✓ |
| DocumentaryRequest 传参 | 请求结构体新增字段 | |
| 两者结合 | config 默认 + request 覆盖 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 嵌入 step 4 内部 | merge_audio_files 完成后内联调用 | ✓ |
| 独立 step 4.5 | 新步骤插入流水线 | |

**Notes:** 直接调用 normalize_audio_for_mixing() 而非分步调用；长音频不做特殊处理；全部三个流水线都集成标准化；SDP 对原始视频音频做标准化。

---

## 智能音量集成策略

| Option | Description | Selected |
|--------|-------------|----------|
| config 开关控制 | enable_smart_volume=true 时自动覆盖音量 | ✓ |
| 始终自动覆盖 | 忽略请求体积参数 | |
| 仅当未显式设置 | 默认值走智能，显式传参尊重用户 | |

| Option | Description | Selected |
|--------|-------------|----------|
| get_optimized_volumes | 按 video_type 返回预调音量 | ✓ |
| apply_volume_profile | 按命名 profile 选择 | |
| 两者都支持 | profile + video_type 兜底 | |

| Option | Description | Selected |
|--------|-------------|----------|
| step_composite 中替换 | composite 开头根据 config 覆盖字段 | ✓ |
| run_documentary 入口处理 | 主编排函数入口统一处理 | |

**Notes:** 扩展 get_optimized_volumes() 的 documentary/sde/sdp 分支；智能音量覆盖请求字段不改 API；BGM 音量与淡出独立；log 最终音量值；新增 volume_profile 到 AudioSection；不需要 OST 感知；三个流水线全部集成。

---

## 集成范围

| Option | Description | Selected |
|--------|-------------|----------|
| 全部三个流水线 | Documentary + SDE + SDP | ✓ |
| 仅 Documentary + SDE | 这两者有 merge_audio_files | |
| 仅 Documentary | 先旗舰验证 | |

**Notes:** SDP 标准化原始视频音频轨道。智能音量同样覆盖全部三个流水线。

---

## 配置驱动 vs 显式调用

| Option | Description | Selected |
|--------|-------------|----------|
| 默认关闭 | enable_* 默认 false，向后兼容 | ✓ |
| 默认开启 | 新用户开箱即用 | |

| Option | Description | Selected |
|--------|-------------|----------|
| AudioSection 新增字段 | volume_profile: String = "balanced" | ✓ |
| 不需要 | 用 video_type 推断 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 保持不变 | 请求结构体默认值不动 | ✓ |
| 全部统一 | 与 AudioSection 统一 | |

---

## Prompt 调用链验证

**重大发现：** Documentary 的 generate_narration() 和 analyze_video() 硬编码 prompt，完全绕过 PromptManager。SDE/SDP 已验证正确使用。

| Option | Description | Selected |
|--------|-------------|----------|
| 重构为 PromptManager | generate_narration + analyze_video 改用 render_prompt | ✓ |
| 保持硬编码 | 模板仅作参考 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 新增参数 | prompt_manager: &PromptManager | ✓ |
| 全局 Arc 访问 | 函数内部获取全局实例 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 合并到模板 | narration_generation_v2.0.md 增加 JSON 格式要求 | ✓ |
| 保持模板不变 | JSON 格式由 response_format 参数控制 | |

---

## 配置与命令层更新

| Option | Description | Selected |
|--------|-------------|----------|
| 全面更新 | target_lufs + volume_profile + 注释全部更新 | ✓ |
| 最小必要更新 | 仅新增字段 + 注释更新 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 不需要命令层变更 | config 已通过 Tauri State 传递 | ✓ |
| 需要新增配置命令 | get/set audio config 命令 | |

---

## Phase 11 模块微调

| Option | Description | Selected |
|--------|-------------|----------|
| 更新 ProcessingConfig 默认值 | target_lufs: -20.0 → -14.0 | ✓ |
| 更新 AudioSection 默认值 | target_lufs: -23.0 → -14.0 + volume_profile | ✓ |
| 扩展 get_optimized_volumes | 新增 documentary/sde/sdp 分支 | ✓ |
| 更新配置测试断言 | -23.0 → -14.0 断言修正 | ✓ |

---

## SDE/SDP Prompt 调用链验证

**验证结果：** SDE 和 SDP 均正确使用 `prompt_manager.render_prompt()` 调用注册模板。无需修改。

| Option | Description | Selected |
|--------|-------------|----------|
| SDE/SDP 也需要检查 | 验证通过——正确使用 PromptManager | ✓ |
| 确认只需修 Documentary | Documentary 是唯一缺口 | |

---

## 已有测试与辅助函数测试

| Option | Description | Selected |
|--------|-------------|----------|
| 全面排查 | cargo test 全量运行找受影响测试 | ✓ |
| 仅 config 测试 | 只改已知的 target_lufs 断言 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 单元测试 | normalize_merged_audio + resolve_volumes 核心逻辑 | ✓ |
| 仅集成测试 | 通过流水线测试间接覆盖 | |

---

## Claude's Discretion

- `get_optimized_volumes()` 中 documentary/sde/sdp 分支的精确音量值
- `src/audio/pipeline.rs` 的具体函数签名和参数设计
- `narration_generation_v2.0.md` JSON 格式说明的具体措辞
- 集成测试中 ffprobe LUFS 读数的容差范围

## Deferred Ideas

None — discussion stayed within phase scope
