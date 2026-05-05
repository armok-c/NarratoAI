# Phase 8: SDP Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 08-sdp-pipeline
**Areas discussed:** 字幕解析策略, 多视频源, 流水线结构, 脚本生成, 依赖策略, 流程拆分, BGM

---

## 字幕解析策略

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 8 先建公共模块 | 在 src/subtitle/ 下建共享模块，供 Phase 7+8 共用 | |
| Phase 8 内联实现 | 字幕解析放 src/sdp/ 内部，Phase 7 独立实现 | |
| Phase 7 先做，Phase 8 复用 | 调整顺序，Phase 7 先建字幕模块，Phase 8 复用 | ✓ |

**User's choice:** Phase 7 先做，Phase 8 复用
**Notes:** 导致 Phase 8 新增对 Phase 7 的依赖，需更新 ROADMAP.md

---

## 多视频源

| Option | Description | Selected |
|--------|-------------|----------|
| 单视频输入 | 先只支持单个视频文件，与 Python 版对齐 | ✓ |
| 视频-片段映射 | 用户提供映射规则 | |
| 自动搜索匹配 | LLM 输出 video_source 字段 | |

**User's choice:** 单视频输入
**Notes:** 后续可扩展为多视频源

---

## 流水线结构

| Option | Description | Selected |
|--------|-------------|----------|
| 沿用 Phase 6 模式 | src/sdp/ 模块 + SdpPipelineState + step 函数链 | ✓ |
| 轻量流水线 | 精简函数链，无 PipelineState | |

**User's choice:** 沿用 Phase 6 模式
**Notes:** 保持架构一致性，即使 SDP 比纪录片简单

---

## 脚本生成

| Option | Description | Selected |
|--------|-------------|----------|
| 1:1 对齐两步 LLM | subtitle_analysis → plot_extraction | ✓ |
| 单次 LLM 调用 | 合并为一个 prompt | |

**User's choice:** 1:1 对齐两步 LLM
**Notes:** 完全复刻 Python 版流程

---

## 依赖策略

| Option | Description | Selected |
|--------|-------------|----------|
| 更新依赖，先讨论后等 | ROADMAP 改为依赖 Phase 6+7，等 Phase 7 完成再执行 | ✓ |
| 用 stub 解耦，可并行 | Phase 8 先建临时实现 | |

**User's choice:** 更新依赖，先讨论后等
**Notes:** Phase 8 CONTEXT.md 先写好，Phase 7 完成后才开始执行

---

## 流程拆分

| Option | Description | Selected |
|--------|-------------|----------|
| 拆分为两个独立函数 | generate_sdp_script() + run_sdp() | ✓ |
| 合为一个完整流水线 | run_sdp() 内部完成全部 | |

**User's choice:** 拆分为两个独立函数
**Notes:** 与 Phase 6 的 D-12/D-13 模式一致

---

## BGM

| Option | Description | Selected |
|--------|-------------|----------|
| 支持 BGM | 最终合成支持 BGM 混音 | ✓ |
| 纯原声（无 BGM） | 仅 concat 拼接 | |

**User's choice:** 支持 BGM
**Notes:** 与 Phase 6 step_composite 模式一致

---

## Claude's Discretion

- SdpPipelineState 的具体字段定义
- step 函数的拆分粒度
- SdpRequest 参数结构体和默认值
- SdpPipelineError 枚举的具体变体
- JSON 修复逻辑实现方式
- custom_clips 参数默认值
- Prompt 模板 Rust 版本对齐方式

## Deferred Ideas

- 多视频源支持 — 后续阶段
- 跨模式公共流水线抽象 — Phase 6/7/8 完成后评估
