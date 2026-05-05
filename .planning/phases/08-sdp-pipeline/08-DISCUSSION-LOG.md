# Phase 8: SDP Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 08-sdp-pipeline
**Areas discussed:** Subtitle 复用, 流水线步骤, 预处理复用, 模块提取, 视频裁剪复用, 脚本生成模式, 进度百分比, SdpRequest, BGM 混音, 错误类型, PipelineState, 脚本格式, 输入验证, Python 对齐, 测试策略, 模块结构, 新依赖, Prompt 迁移, 保留决策

---

## Subtitle 复用策略

| Option | Description | Selected |
|--------|-------------|----------|
| 直接引用 src/sde::subtitle | 从 src/sde/subtitle.rs 直接调用，最简单但有跨业务域耦合 | |
| 提取公共模块 src/subtitle/ | 将 subtitle.rs 移出为独立公共模块，Phase 7 和 8 各自引用 | ✓ |
| SDP 自己实现字幕解析 | 独立实现，避免依赖但代码重复 | |

**User's choice:** 提取公共模块 src/subtitle/

---

## 流水线步骤数量

| Option | Description | Selected |
|--------|-------------|----------|
| 6 步 | 加载脚本→视频裁剪→片段拼接→BGM混音→最终合成 | |
| 5 步 | 视频裁剪→片段拼接→BGM混音→最终合成 | |
| 4 步 | 视频裁剪→片段拼接→最终合成(+BGM)，BGM 合并到合成步骤 | ✓ |

**User's choice:** 4 步——最精简设计，与 Python 版 SDP 对齐

---

## 预处理复用策略

| Option | Description | Selected |
|--------|-------------|----------|
| 复用 SDE 字幕+LLM基础设施 | 调用 parse_subtitle_file() + PromptManager + LlmProvider | ✓ |
| 完全独立实现 | 在 src/sdp/ 内独立编写所有逻辑 | |

**User's choice:** 复用 SDE 基础设施

---

## 模块提取范围

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 8 前置步骤 | 在 Phase 8 计划内完成 subtitle 模块迁移 | ✓ |
| 独立小数 Phase | 创建 Phase 7.1 专门处理模块提取 | |
| Phase 8 边做边提取 | 实现 SDP 时先迁移再构建 | |

**User's choice:** Phase 8 前置步骤

---

## 视频裁剪复用

| Option | Description | Selected |
|--------|-------------|----------|
| 直接复用 sde_step_clip() | SDP 全部 OST=1，与 SDE 裁剪逻辑完全一致 | ✓ |
| SDP 独立实现裁剪 | 在 src/sdp/clip.rs 内独立实现 | |
| 由 Claude 判断 | Planner 评估代码重复程度后决定 | |

**User's choice:** 直接复用 sde_step_clip()

---

## 脚本生成 LLM 调用模式

| Option | Description | Selected |
|--------|-------------|----------|
| 复用调用模式 | 参考 SDE 代码结构但用 SDP 专属 Prompt | ✓ |
| 完全独立实现 | 在 src/sdp/ 内独立编写 | |

**User's choice:** 复用调用模式——参考 SDE step_analyze_plot/step_generate_script 的结构

---

## 进度百分比分配

| Option | Description | Selected |
|--------|-------------|----------|
| 25/50/75/100% | 等分 4 步 | |
| 30/60/90/100% | 裁剪权重最高，反映 FFmpeg 实际耗时 | ✓ |
| 由 Claude 判断 | Planner 根据实际耗时分配 | |

**User's choice:** 30/60/90/100%

---

## SdpRequest 参数设计

| Option | Description | Selected |
|--------|-------------|----------|
| video_path + script_path + config | 最简设计，与 SdeRequest 模式一致 | |
| video_path + script_path + bgm_path | 显式 BGM 参数，与 DocumentaryRequest 一致 | |
| 由 Claude 判断 | Planner 合并出合理参数集 | ✓ |

**User's choice:** 由 Claude/Planner 裁量

---

## BGM 混音位置

| Option | Description | Selected |
|--------|-------------|----------|
| 合并到第 4 步 | 最终合成步骤内通过 amix filter 混入，减少一次编码 | ✓ |
| 独立第 5 步 | 先合成无 BGM 版本再混入，更灵活 | |
| 可选跳过 | BGM 可选，未配置则 4 步 | |

**User's choice:** 合并到第 4 步合成步骤

---

## 错误类型设计

| Option | Description | Selected |
|--------|-------------|----------|
| 独立 SdpError 枚举 | 新建 SdpError 涵盖所有阶段，与 SdeError 模式一致 | ✓ |
| 复用+扩展 | 主要复用 SdeError 和 PipelineError | |

**User's choice:** 独立 SdpError 枚举

---

## PipelineState 设计

| Option | Description | Selected |
|--------|-------------|----------|
| 独立 SdpPipelineState | 精简的 4 步状态结构体 | ✓ |
| 极致简化 | 不用 PipelineState，4 步直接传临时变量 | |

**User's choice:** 独立 SdpPipelineState

---

## 脚本输出格式

| Option | Description | Selected |
|--------|-------------|----------|
| merged_subtitle.json | 与 Python 版对齐 | ✓ |
| sdp_script.json | 独立文件名 | |
| 由 Claude 判断 | | |

**User's choice:** merged_subtitle.json

---

## 输入验证策略

| Option | Description | Selected |
|--------|-------------|----------|
| 两阶段都验证 | generate 和 run 各自验证输入 | ✓ |
| 仅入口验证 | 只在函数入口做基本检查 | |
| 由 Claude 判断 | | |

**User's choice:** 两阶段都验证

---

## Python 版对齐策略

| Option | Description | Selected |
|--------|-------------|----------|
| 1:1 对齐核心逻辑 | LLM 调用、JSON 修复、脚本合并严格对齐 | ✓ |
| 允许合理改进 | 可借鉴 Phase 7 repair_json 6 步改进 | |

**User's choice:** 1:1 对齐核心逻辑

---

## 测试策略

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 SDE 测试密度 | 各模块都有单元测试，与 SDE 密度一致 | ✓ |
| 最小化测试 | 只测试关键路径 | |

**User's choice:** 对齐 SDE 测试密度

---

## 模块文件结构

| Option | Description | Selected |
|--------|-------------|----------|
| 精简结构 | ~5 个文件（无 clip.rs） | |
| 对齐 SDE 结构 | error + types + pipeline + clip + script_gen + mod | ✓ |
| 由 Claude 判断 | | |

**User's choice:** 对齐 SDE 结构

---

## 新依赖

| Option | Description | Selected |
|--------|-------------|----------|
| 无需新依赖 | encoding_rs + regex 已由 SDE 添加 | |
| 由 Claude 判断 | Planner 评估 | ✓ |

**User's choice:** 由 Claude/Planner 评估

---

## Prompt 迁移策略

| Option | Description | Selected |
|--------|-------------|----------|
| 翻译为 Rust 模板 | Python Prompt 内容翻译为 PromptTemplate 格式 | ✓ |
| 直接内嵌字符串 | Prompt 作为 const &str | |

**User's choice:** 翻译为 Rust 模板

---

## 保留现有决策

| Option | Description | Selected |
|--------|-------------|----------|
| 全部保留 | D-03~D-08 全部有效，仅更新模块路径和 Phase 7 状态 | ✓ |
| 部分调整 | | |

**User's choice:** 全部保留

---

## Claude's Discretion

- SdpRequest 参数结构体设计（参考 SdeRequest + DocumentaryRequest 合并）
- SdpPipelineState 的具体字段定义
- SdpError 枚举的具体变体
- step 函数的拆分粒度
- JSON 修复逻辑实现（可直接参考 Phase 7 repair_json 6步策略）
- 是否需要新增 Cargo.toml 依赖
- Prompt 模板的 Rust 版本具体内容
- custom_clips 参数默认值

## Deferred Ideas

- 多视频源支持（SDP-02 完整含义）——未来阶段
- 跨模式公共流水线抽象——等 Phase 6/7/8 全部完成后评估
