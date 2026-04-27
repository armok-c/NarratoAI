# Requirements: NarratoAI Rust Rewrite

**Defined:** 2026-04-27
**Core Value:** 用 Rust 重写核心视频剪辑流水线（LLM 文案生成 → TTS 配音 → FFmpeg 视频处理 → 合成输出），实现高性能 AI 视频自动化生产

## v1 Requirements

Requirements for Rust rewrite milestone. Each maps to roadmap phases. Code-level alignment with Python v0.7.8.

### Configuration

- [ ] **CONF-01**: 系统从 TOML 文件加载所有配置段（app、ui、各 TTS 后端、proxy、frames）
- [ ] **CONF-02**: 配置校验——缺失必填字段时给出清晰错误信息
- [ ] **CONF-03**: 支持配置文件热加载（运行时检测变更）
- [ ] **CONF-04**: FFmpeg 硬件加速检测（NVIDIA/AMD/Intel profile 匹配）

### FFmpeg Pipeline

- [ ] **FFMP-01**: 通过 ffmpeg-sidecar（CLI 子进程）执行所有 FFmpeg 操作
- [ ] **FFMP-02**: 按 OST 类型裁剪视频片段（OST=0 仅解说音、OST=1 仅原声、OST=2 混合）
- [ ] **FFMP-03**: 音频视频合并——将解说音频与视频片段合成
- [ ] **FFMP-04**: 字幕生成（SRT 格式，从 TTS 词边界生成时间戳）
- [ ] **FFMP-05**: 字幕烧录到视频（FFmpeg subtitles filter）
- [ ] **FFMP-06**: BGM 混音（FFmpeg amix filter，音量控制）
- [ ] **FFMP-07**: 视频片段拼接（concat demuxer）
- [ ] **FFMP-08**: 最终合成输出（含硬件加速编码）
- [ ] **FFMP-09**: FFmpeg 异步执行——用 spawn_blocking 包装，不阻塞 tokio 运行时

### LLM Service

- [ ] **LLM-01**: OpenAI 兼容协议客户端（/v1/chat/completions 端点）
- [ ] **LLM-02**: Provider/Registry 模式——通过 `provider/model_name` 格式路由到不同 API 端点
- [ ] **LLM-03**: Vision 模型支持——发送图片（base64）+ 文本给视觉 LLM
- [ ] **LLM-04**: Text 模型支持——纯文本对话（流式响应）
- [ ] **LLM-05**: 流式响应处理（SSE 解析，逐 token 输出）
- [ ] **LLM-06**: 并发控制——视觉分析批量请求的信号量限制
- [ ] **LLM-07**: 统一错误处理——API 错误、超时、限流的重试机制

### TTS

- [ ] **TTS-01**: TtsProvider trait——统一 TTS 引擎接口（输入文本，输出音频文件）
- [ ] **TTS-02**: TTS 路由器——按引擎名字符串分发到具体实现
- [ ] **TTS-03**: Edge-TTS 引擎实现（WebSocket 协议，SSML 生成，音频接收）
- [ ] **TTS-04**: Azure Speech TTS 引擎实现（REST API）
- [ ] **TTS-05**: Tencent TTS 引擎实现
- [ ] **TTS-06**: SoulVoice TTS 引擎实现
- [ ] **TTS-07**: Qwen TTS 引擎实现
- [ ] **TTS-08**: IndexTTS2 语音克隆引擎实现
- [ ] **TTS-09**: Doubao TTS 引擎实现

### Prompt System

- [ ] **PRMP-01**: Prompt 注册表——按 category/name/version 三级索引存储 prompt 模板
- [ ] **PRMP-02**: 模板渲染——变量替换（Jinja 风格占位符）
- [ ] **PRMP-03**: Prompt 分类——documentary/short_drama_editing/short_drama_narration
- [ ] **PRMP-04**: Prompt 校验——加载时检查模板完整性

### Visual Analyzer

- [ ] **VISL-01**: 视频帧提取——按时间间隔通过 FFmpeg 提取关键帧
- [ ] **VISL-02**: 批量帧分析——将提取的帧批量发送给视觉 LLM
- [ ] **VISL-03**: 结构化分析结果生成——输出 JSON 格式的帧分析报告

### Script Management

- [ ] **SCRP-01**: JSON 脚本加载/保存——与 Python 版格式兼容
- [ ] **SCRP-02**: 脚本编辑——修改段落文案、OST 类型、时间戳
- [ ] **SCRP-03**: 脚本校验——检查必要字段、时间戳合理性

### Documentary Pipeline

- [ ] **DOCS-01**: 纪录片完整流水线编排（6 步：加载脚本 → TTS → 视频裁剪 → 音频字幕合并 → 片段拼接 → 最终合成）
- [ ] **DOCS-02**: 视觉分析→文案生成链路——帧分析结果输入 LLM 生成解说文案
- [ ] **DOCS-03**: 任务进度追踪——每步完成后更新状态并通知前端

### SDE Pipeline

- [ ] **SDE-01**: 短剧解说完整流水线编排（字幕解析 → LLM 分析 → 带OST脚本 → 视频处理）
- [ ] **SDE-02**: 字幕文本解析——从 SRT/ASS 文件提取对话内容
- [ ] **SDE-03**: LLM 剧情分析——生成带 OST 类型标记的解说脚本

### SDP Pipeline

- [ ] **SDP-01**: 短剧混剪完整流水线编排（字幕解析 → LLM 分析 → 多片段脚本 → 视频处理）
- [ ] **SDP-02**: 多片段混剪——跨多个视频源裁剪和拼接

### JianYing Export

- [ ] **JYNG-01**: 生成剪映草稿 JSON 格式（逆向工程 pyJianYingDraft 格式）
- [ ] **JYNG-02**: 导出项目时间线——片段、字幕、音频轨道映射到剪映格式

### Tauri Command Layer

- [ ] **TAURI-01**: Tauri 2.0 命令注册——暴露所有 Rust 业务函数给前端
- [ ] **TAURI-02**: 进度事件推送——通过 Tauri events 实时推送任务进度
- [ ] **TAURI-03**: 文件对话框集成——视频/字幕文件选择

### Extended Features

- [ ] **EXTD-01**: 音频响度标准化（LUFS-based loudness normalization）
- [ ] **EXTD-02**: YouTube 视频下载（调用 yt-dlp 子进程）
- [ ] **EXTD-03**: Pexels 素材搜索和下载
- [ ] **EXTD-04**: 智能音量控制——根据内容类型（解说/原声/BGM）提供预设音量

## v2 Requirements

Deferred to Tauri + Vue 3 frontend milestone.

### Frontend

- **UI-01**: Vue 3 + Vuetify 3 桌面 UI
- **UI-02**: 纪录片模式工作面板
- **UI-03**: 短剧解说模式工作面板
- **UI-04**: 短剧混剪模式工作面板
- **UI-05**: 脚本编辑器（带时间轴预览）
- **UI-06**: 任务进度实时展示
- **UI-07**: 配置管理界面
- **UI-08**: 视频预览播放器

## Out of Scope

| Feature | Reason |
|---------|--------|
| Streamlit Web UI | 被 Tauri + Vue 3 替代 |
| Redis 状态后端 | 桌面单用户不需要分布式状态 |
| Docker 部署 | 桌面应用打包为原生安装包 |
| Python 旧 LLM 接口（llm.py） | Rust 版直接实现新架构，无需迁移适配器 |
| Gemini 原生 SDK | 统一用 OpenAI 兼容协议 |
| HTTP API 服务 | Tauri 命令层直接调用 Rust 函数 |
| 多用户/并发任务管理 | 桌面单用户，一次一个任务 |
| PyO3 Python 桥接 | 完全重写，不保留 Python 依赖 |
| faster-whisper 本地 STT | Python 版已注释掉，桌面版不需要 |

## Traceability

Which phases cover which requirements.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONF-01 | Phase 1: Foundation | Pending |
| CONF-02 | Phase 1: Foundation | Pending |
| CONF-03 | Phase 1: Foundation | Pending |
| CONF-04 | Phase 1: Foundation | Pending |
| FFMP-01 | Phase 1: Foundation | Pending |
| FFMP-09 | Phase 1: Foundation | Pending |
| LLM-01 | Phase 2: LLM Service Layer | Pending |
| LLM-02 | Phase 2: LLM Service Layer | Pending |
| LLM-03 | Phase 2: LLM Service Layer | Pending |
| LLM-04 | Phase 2: LLM Service Layer | Pending |
| LLM-05 | Phase 2: LLM Service Layer | Pending |
| LLM-06 | Phase 2: LLM Service Layer | Pending |
| LLM-07 | Phase 2: LLM Service Layer | Pending |
| TTS-01 | Phase 3: TTS Core + Edge-TTS | Pending |
| TTS-02 | Phase 3: TTS Core + Edge-TTS | Pending |
| TTS-03 | Phase 3: TTS Core + Edge-TTS | Pending |
| PRMP-01 | Phase 4: Prompt System + Visual Analyzer | Pending |
| PRMP-02 | Phase 4: Prompt System + Visual Analyzer | Pending |
| PRMP-03 | Phase 4: Prompt System + Visual Analyzer | Pending |
| PRMP-04 | Phase 4: Prompt System + Visual Analyzer | Pending |
| VISL-01 | Phase 4: Prompt System + Visual Analyzer | Pending |
| VISL-02 | Phase 4: Prompt System + Visual Analyzer | Pending |
| VISL-03 | Phase 4: Prompt System + Visual Analyzer | Pending |
| SCRP-01 | Phase 5: Script Management | Pending |
| SCRP-02 | Phase 5: Script Management | Pending |
| SCRP-03 | Phase 5: Script Management | Pending |
| FFMP-02 | Phase 6: Documentary Pipeline | Pending |
| FFMP-03 | Phase 6: Documentary Pipeline | Pending |
| FFMP-04 | Phase 6: Documentary Pipeline | Pending |
| FFMP-05 | Phase 6: Documentary Pipeline | Pending |
| FFMP-06 | Phase 6: Documentary Pipeline | Pending |
| FFMP-07 | Phase 6: Documentary Pipeline | Pending |
| FFMP-08 | Phase 6: Documentary Pipeline | Pending |
| DOCS-01 | Phase 6: Documentary Pipeline | Pending |
| DOCS-02 | Phase 6: Documentary Pipeline | Pending |
| DOCS-03 | Phase 6: Documentary Pipeline | Pending |
| SDE-01 | Phase 7: SDE Pipeline | Pending |
| SDE-02 | Phase 7: SDE Pipeline | Pending |
| SDE-03 | Phase 7: SDE Pipeline | Pending |
| SDP-01 | Phase 8: SDP Pipeline | Pending |
| SDP-02 | Phase 8: SDP Pipeline | Pending |
| JYNG-01 | Phase 9: JianYing Export | Pending |
| JYNG-02 | Phase 9: JianYing Export | Pending |
| TAURI-01 | Phase 10: Tauri Command Layer | Pending |
| TAURI-02 | Phase 10: Tauri Command Layer | Pending |
| TAURI-03 | Phase 10: Tauri Command Layer | Pending |
| EXTD-01 | Phase 11: Extended Features | Pending |
| EXTD-02 | Phase 11: Extended Features | Pending |
| EXTD-03 | Phase 11: Extended Features | Pending |
| EXTD-04 | Phase 11: Extended Features | Pending |
| TTS-04 | Phase 12: Additional TTS Engines | Pending |
| TTS-05 | Phase 12: Additional TTS Engines | Pending |
| TTS-06 | Phase 12: Additional TTS Engines | Pending |
| TTS-07 | Phase 12: Additional TTS Engines | Pending |
| TTS-08 | Phase 12: Additional TTS Engines | Pending |
| TTS-09 | Phase 12: Additional TTS Engines | Pending |

**Coverage:**
- v1 requirements: 56 total
- Mapped to phases: 56
- Unmapped: 0

---
*Requirements defined: 2026-04-27*
*Last updated: 2026-04-27 after roadmap creation with traceability*
