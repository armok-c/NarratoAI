# NarratoAI Rust Rewrite

## What This Is

NarratoAI 是一站式 AI 影视解说+自动化剪辑工具。后端已用 Rust 完全重写（v1.0 交付），包含库 crate + Tauri 2.0 命令层，支持三大业务模式：纪录片解说（Documentary）、短剧解说（SDE）、短剧混剪（SDP）。前端将在 v2.0 里程碑用 Tauri + Vue 3 重构。

## Core Value

高性能 Rust 后端驱动完整的视频自动化生产流水线：LLM 文案生成 → TTS 配音 → FFmpeg 视频处理 → 音频标准化 → 合成输出。

## Requirements

### Validated

- ✓ 配置系统（TOML 加载/热加载/硬件加速检测） — v1.0 Phase 1
- ✓ LLM 服务层（OpenAI 兼容协议，Provider/Registry，Vision+Text+Stream） — v1.0 Phase 2
- ✓ TTS 路由层（7 个引擎：Edge-TTS、Azure、Tencent、SoulVoice、Qwen、IndexTTS2、Doubao） — v1.0 Phase 3+12
- ✓ Prompt 系统（版本化注册表、模板渲染、分类管理） — v1.0 Phase 4+13
- ✓ 视觉分析器（帧提取 + 批量 LLM 视觉分析） — v1.0 Phase 4
- ✓ 脚本管理（JSON 加载/保存/编辑/校验，不可变更新） — v1.0 Phase 5
- ✓ 纪录片解说流水线（6 步完整编排） — v1.0 Phase 6
- ✓ 短剧解说流水线（9 步编排，字幕解析+LLM 分析） — v1.0 Phase 7
- ✓ 短剧混剪流水线（多片段裁剪+拼接） — v1.0 Phase 8
- ✓ 剪映草稿导出 — v1.0 Phase 9
- ✓ Tauri 2.0 命令层（15 个 IPC 命令+进度事件） — v1.0 Phase 10
- ✓ 扩展功能（音频标准化、YouTube 下载、Pexels 素材、智能音量） — v1.0 Phase 11+13
- ✓ FFmpeg 完整操作（裁剪/合并/混音/字幕/拼接/硬件加速编码） — v1.0 Phase 1+6

### Active

- [ ] Vue 3 + Vuetify 3 桌面 UI
- [ ] 纪录片模式工作面板
- [ ] 短剧解说模式工作面板
- [ ] 短剧混剪模式工作面板
- [ ] 脚本编辑器（带时间轴预览）
- [ ] 任务进度实时展示
- [ ] 配置管理界面
- [ ] 视频预览播放器
- [ ] YouTube/Pexels Tauri 命令暴露
- [ ] ConfigManager 热加载激活
- [ ] CONF-02 业务级校验实现

### Out of Scope

- Streamlit Web UI — 已被 Tauri + Vue 3 替代
- Redis 状态后端 — 桌面单用户不需要分布式状态
- Docker 部署 — 桌面应用打包为原生安装包
- Python 旧 LLM 接口 — Rust 版直接实现新架构
- Gemini 原生 SDK — 统一用 OpenAI 兼容协议
- HTTP API 服务 — Tauri 命令层直接调用
- 多用户/并发任务管理 — 桌面单用户
- PyO3 Python 桥接 — 完全重写
- faster-whisper 本地 STT — Python 版已注释掉

## Context

Shipped v1.0 with ~24,609 LOC Rust (23,532 core + 1,077 Tauri).
Tech stack: Rust + ffmpeg-sidecar + reqwest + tokio + Tauri 2.0.
Tests: 573 passing. Timeline: 15 days (2026-04-27 → 2026-05-12).

Known tech debt (WARNING level):
- Edge-TTS proxy tunnel unimplemented
- Tauri State read lock held across long pipelines
- Integration test stubs using assert!(true)
- _max_retries parameter accepted but unwired

## Constraints

- **Tech stack**: Rust（后端库）+ Tauri 2.0 命令层 + Vue 3 + Vite（前端，v2.0）
- **No code reuse**: 从零开始，不复用 smartEdit 代码
- **Feature parity**: 代码级对齐 Python 版
- **Storage**: 文件存储（JSON/TOML）
- **Architecture**: Rust 库 + Tauri 命令

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 从零重写而非增量替换 | 完全发挥 Rust 性能优势 | ✓ Good — 15 天交付完整后端 |
| 库 + Tauri 命令架构 | 桌面应用无需 HTTP 层 | ✓ Good — 15 个 IPC 命令 |
| 文件存储替代 Redis | 桌面单用户场景 | ✓ Good — 简单可靠 |
| 统一 OpenAI 兼容协议 | 所有 Provider 统一接口 | ✓ Good — 降低复杂度 |
| Phase 13 gap closure 插入 | 补齐 Prompt+音频集成 | ✓ Good — 关闭关键缺口 |
| OST 类型驱动流水线 | 三种模式清晰分离 | ✓ Good — 裁剪/音频/混音策略一致 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

---
*Last updated: 2026-05-12 after v1.0 milestone completion*
