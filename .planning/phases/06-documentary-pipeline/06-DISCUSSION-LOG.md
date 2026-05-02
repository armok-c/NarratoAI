# Phase 6: Documentary Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-02
**Phase:** 06-documentary-pipeline
**Areas discussed:** 流水线编排结构, 错误恢复策略, 字幕生成流程, 帧分析→文案生成编排

---

## 流水线编排结构

| Option | Description | Selected |
|--------|-------------|----------|
| Step-by-Step 函数链 | 每步一个独立 async 函数，主函数顺序调用，PipelineState 结构体传递状态 | ✓ |
| Pipeline trait + 注册 | PipelineStep trait + Pipeline 编排器，Phase 7/8 可复用但 Phase 6 有过度设计风险 | |
| 状态机驱动 | PipelineState 枚举状态转移驱动执行，支持断点续跑，复杂度高 | |

**User's choice:** Step-by-Step 函数链
**Notes:** 简单直接，调试方便，Python 版逻辑 1:1 对齐

| Option | Description | Selected |
|--------|-------------|----------|
| 可变 PipelineState | struct PipelineState，每步接收 &mut 并更新字段 | ✓ |
| 不可变链式传递 | 每步返回自己的结果，下一步作为参数接收 | |

**User's choice:** 可变 PipelineState
**Notes:** 与 Python 版行为最接近

| Option | Description | Selected |
|--------|-------------|----------|
| src/documentary/ | 新建 documentary 模块 | ✓ |
| src/pipeline/ | pipeline 模块下 documentary.rs，为 7/8 预留命名空间 | |

**User's choice:** src/documentary/
**Notes:** 与 Python 版 app/services/documentary/ 对齐

| Option | Description | Selected |
|--------|-------------|----------|
| 参数结构体 | DocumentaryRequest 结构体 + ProgressCallback | ✓ |
| 直接参数 | 大量参数直接传入，签名长 | |

**User's choice:** 参数结构体
**Notes:** 与 Phase 9 的 ExportRequest 模式一致

| Option | Description | Selected |
|--------|-------------|----------|
| 不复用，各自独立 | 三种模式各自实现，避免过早抽象 | |
| 抽取公共 step 函数 | 视频裁剪/音频合并/字幕/合成抽取为公共函数 | |
| Claude 定 | | ✓ |

**User's choice:** Claude 定
**Notes:** Claude 决定：Phase 6 先不做跨模式抽象，Phase 7/8 发现重复再抽取到 src/pipeline/common.rs

---

## 错误恢复策略

| Option | Description | Selected |
|--------|-------------|----------|
| 完全 fail-fast | 任何步骤失败立即返回错误，与 Python 版 1:1 | ✓ |
| 步骤级重试 + fail-fast | 每步可配置重试次数，重试耗尽后才 fail-fast | |
| 片段级容错 | 单个片段失败跳过继续，适合长视频但输出可能不完整 | |

**User's choice:** 完全 fail-fast

| Option | Description | Selected |
|--------|-------------|----------|
| 单一流水线错误枚举 | PipelineError 包装各步骤领域错误 | |
| 透传领域错误 | 每步直接返回领域错误，调用方 downcast | |
| Claude 定 | | ✓ |

**User's choice:** Claude 定
**Notes:** Claude 决定：单一 PipelineError 枚举包装领域错误，通过 From 自动转换

| Option | Description | Selected |
|--------|-------------|----------|
| 不清理 | 中间文件留在 task_dir，便于调试 | ✓ |
| 失败时清理 | 删除 task_dir 所有中间文件 | |

**User's choice:** 不清理

---

## 字幕生成流程

| Option | Description | Selected |
|--------|-------------|----------|
| TTS 后即时生成 | TTS 返回 word boundary 后即时为每段生成 SRT 片段 | ✓ |
| 统一后处理 | 所有 TTS 完成后单独一步统一生成 | |

**User's choice:** TTS 后即时生成

| Option | Description | Selected |
|--------|-------------|----------|
| 最终合成时烧录 | 步骤 6 FFmpeg 一次性合并视频+音频+BGM+字幕烧录 | ✓ |
| 单独烧录步骤 | 步骤 5.5 单独烧录，FFmpeg 命令更简单但多一次编码 | |

**User's choice:** 最终合成时烧录

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python 版合并逻辑 | 1:1 实现时间偏移合并 | ✓ |
| Claude 定 | | |

**User's choice:** 对齐 Python 版合并逻辑

---

## 帧分析→文案生成编排

| Option | Description | Selected |
|--------|-------------|----------|
| 独立预处理函数 | generate_documentary_script() 与 6 步流水线解耦 | ✓ |
| 集成步骤 0 | 在 6 步之前加步骤 0，一步到位但强制所有用户先走分析 | |

**User's choice:** 独立预处理函数
**Notes:** 与 Python 版工作流一致（先分析→编辑→再生成视频）

| Option | Description | Selected |
|--------|-------------|----------|
| 直接输出 Script | 两步 LLM 调用后返回 Vec<ScriptClip> | ✓ |
| 分步输出 | 帧分析返回 BatchAnalysisResult，调用方自行组装 | |

**User's choice:** 直接输出 Script

| Option | Description | Selected |
|--------|-------------|----------|
| LLM 输出 OST + timestamp | Prompt 要求 LLM 同时输出 OST 类型和 timestamp | ✓ |
| LLM 只输出文案 + OST | timestamp 由帧分析结果自动推算 | |
| Claude 定 | | |

**User's choice:** LLM 输出 OST + timestamp
**Notes:** 与 Python 版 generate_narration_script.py 一致

| Option | Description | Selected |
|--------|-------------|----------|
| 两步调用 | 先 frame_analysis 再 narration_generation | ✓ |
| Claude 定 | | |

**User's choice:** 两步调用
**Notes:** 与 Python 版 frame_analysis_service.py 流程一致

---

## Claude's Discretion

- Phase 7/8 跨模式复用：Phase 6 先不做抽象，Phase 7/8 发现重复再抽取
- PipelineError 枚举设计：单一枚举包装领域错误，通过 From 自动转换

## Deferred Ideas

None — discussion stayed within phase scope
