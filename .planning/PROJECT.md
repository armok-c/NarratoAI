# NarratoAI Rust Rewrite

## What This Is

NarratoAI 是一站式 AI 影视解说+自动化剪辑工具（当前 Python 版 v0.7.8）。本项目将其后端完全用 Rust 重写，支持三大业务模式：纪录片解说（Documentary）、短剧解说（SDE）、短剧混剪（SDP）。后续里程碑将用 Tauri 2.0 + Vue 3 重构前端，打包为原生桌面应用。

## Core Value

用 Rust 重写核心视频剪辑流水线（LLM 文案生成 → TTS 配音 → FFmpeg 视频处理 → 合成输出），实现比 Python 版更高性能的视频自动化生产。

## Requirements

### Validated

<!-- 从现有 Python 代码库推断已验证的能力 -->

- ✓ 纪录片解说流水线（Documentary） — Python 版已实现
- ✓ 短剧解说流水线（SDE） — Python 版已实现
- ✓ 短剧混剪流水线（SDP） — Python 版已实现
- ✓ LLM 服务层（OpenAI 兼容协议，多 provider） — Python 版已实现
- ✓ 7 个 TTS 引擎（edge_tts、azure_speech、tencent_tts、soulvoice、tts_qwen、indextts2、doubaotts） — Python 版已实现
- ✓ FFmpeg 视频裁剪/合并/混音 — Python 版已实现
- ✓ 版本化 Prompt 管理系统 — Python 版已实现
- ✓ 字幕生成与合并 — Python 版已实现
- ✓ 剪映草稿导出 — Python 版已实现

### Active

<!-- Rust 重写目标：代码级对齐 -->

- [ ] Rust 核心视频处理库（FFmpeg 绑定、裁剪、合并、混音）
- [ ] Rust LLM 服务层（OpenAI 兼容协议、Provider/Registry 模式、Vision+Text 支持）
- [x] Rust TTS 路由层（7 个引擎全部重写） — Validated in Phase 12
- [ ] Rust Prompt 系统（版本化、分类注册表、模板渲染）
- [ ] Rust 纪录片解说流水线（6 步编排）
- [ ] Rust 短剧解说流水线
- [ ] Rust 短剧混剪流水线
- [ ] Rust 配置系统（TOML 加载）
- [x] Rust 脚本管理（JSON 加载/保存/编辑/校验） — Validated in Phase 5
- [ ] Rust 状态管理（文件存储，JSON/TOML）
- [ ] Rust 视觉分析器（帧提取 + LLM 视觉分析）
- [ ] Tauri 命令层（暴露 Rust 函数给前端）
- [ ] 剪映草稿导出（Rust 实现）

### Out of Scope

- Streamlit Web UI — 后续里程碑用 Tauri + Vue 3 替代
- Redis 状态后端 — Rust 版用文件存储
- Docker 部署 — 桌面应用不需要
- smartEdit 代码复用 — 从零开始，仅参考技术栈选型

## Context

- 现有 Python 版本在 `E:\GitLib\NarratoAI`，是成熟的可运行系统，作为功能参考和代码级对齐的基准
- smartEdit（`E:\GitLib\smartEdit`）是基于 Tauri 2.0 + Vue 3 + Rust 的桌面视频编辑应用，是失败的尝试（目标是做出自动化 NarratoAI），但其技术栈验证了 Tauri + Rust 的可行性
- smartEdit 的技术选型可参考：Tauri 2.0、Vue 3 + Vuetify 3 + TypeScript + Vite、Rust + SQLite + sqlx
- 用户要求完全从零开始，不复用 smartEdit 代码

## Constraints

- **Tech stack**: Rust（后端库）+ Tauri 2.0 命令层（后续前端里程碑用 Vue 3 + Vite）
- **No code reuse**: 从零开始，不复用 smartEdit 的 Rust 代码，仅参考技术栈
- **Feature parity**: 代码级对齐——每个 Python 模块都有对应 Rust 实现
- **Storage**: 文件存储（JSON/TOML），不用数据库
- **Architecture**: Rust 库 + Tauri 命令，不是 HTTP API 服务

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 从零重写而非增量替换 | 完全发挥 Rust 性能优势，避免 Python/Rust 混合架构复杂性 | — Pending |
| 库 + Tauri 命令架构 | 桌面应用，无需 HTTP 层，Tauri 直接调用 Rust 函数 | — Pending |
| 文件存储替代 Redis | 桌面单用户场景，不需要分布式状态 | — Pending |
| 参考但不复用 smartEdit | smartEdit 已验证技术栈但代码不可用，从零开始更干净 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-30 after Phase 12 completion — all 7 TTS engines implemented*
