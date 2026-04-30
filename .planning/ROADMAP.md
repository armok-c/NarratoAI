# Roadmap: NarratoAI Rust Rewrite

## Overview

将 NarratoAI 的核心视频剪辑流水线从 Python 重写为 Rust，交付一个库 crate + Tauri 2.0 命令层的桌面应用后端。路线从基础设施（配置、错误类型、FFmpeg 核心）开始，逐层构建 LLM 服务、TTS 引擎、Prompt 系统、视觉分析器，然后实现三大业务流水线（纪录片、短剧解说、短剧混剪），最后通过 Tauri 命令层暴露给前端，并补齐扩展功能。共 12 个阶段、56 个需求、100% 覆盖。

## Phases

**Phase Numbering:**
- Integer phases (1-12): Planned milestone work
- Decimal phases: Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Foundation** - 项目脚手架、配置系统、错误类型、FFmpeg 核心操作
- [ ] **Phase 2: LLM Service Layer** - OpenAI 兼容协议客户端、Provider/Registry 模式、Vision+Text 支持
- [ ] **Phase 3: TTS Core + Edge-TTS** - TtsProvider trait、路由器、Edge-TTS WebSocket 实现
- [ ] **Phase 4: Prompt System + Visual Analyzer** - Prompt 注册表/模板渲染、视频帧提取+批量视觉分析
- [x] **Phase 5: Script Management** - JSON 脚本加载/保存/编辑/校验 *(completed 2026-04-29)*
- [ ] **Phase 6: Documentary Pipeline** - 纪录片完整 6 步流水线（旗舰模式）
- [ ] **Phase 7: SDE Pipeline** - 短剧解说流水线（字幕解析、LLM 分析、OST 脚本）
- [ ] **Phase 8: SDP Pipeline** - 短剧混剪流水线（多片段混剪）
- [x] **Phase 9: JianYing Export** - 剪映草稿 JSON 生成和项目时间线导出 *(completed 2026-04-29)*
- [ ] **Phase 10: Tauri Command Layer** - Tauri 2.0 命令注册、进度推送、文件对话框
- [ ] **Phase 11: Extended Features** - 音频标准化、YouTube 下载、Pexels 素材、智能音量
- [ ] **Phase 12: Additional TTS Engines** - Azure、Tencent、SoulVoice、Qwen、IndexTTS2、Doubao 六个引擎

## Phase Details

### Phase 1: Foundation
**Goal**: 项目拥有可编译的 Rust 库骨架，能从 TOML 文件加载配置，能通过 ffmpeg-sidecar 执行基本的视频操作
**Depends on**: Nothing (first phase)
**Requirements**: CONF-01, CONF-02, CONF-03, CONF-04, FFMP-01, FFMP-09
**Success Criteria** (what must be TRUE):
  1. `cargo build` 成功，库 crate 结构就绪（config、ffmpeg、error 模块）
  2. 从 TOML 文件加载所有配置段（app、ui、TTS 后端、proxy、frames）且缺失必填字段时给出清晰错误信息
  3. 通过 ffmpeg-sidecar CLI 子进程成功执行一次视频信息探测（ffprobe）和一次视频裁剪操作
  4. FFmpeg 操作通过 spawn_blocking 执行，不阻塞 tokio 运行时
  5. 检测到系统 FFmpeg 支持的硬件加速编码器（NVIDIA/AMD/Intel profile 匹配）
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — 库骨架 + 配置系统（TOML 加载/默认值/校验/热加载）
- [x] 01-02-PLAN.md — FFmpeg 操作层（异步裁剪/视频探测/硬件检测）
- [x] 01-03-PLAN.md — 集成测试（配置 + FFmpeg 端到端验证）

### Phase 2: LLM Service Layer
**Goal**: 系统能通过 OpenAI 兼容协议与任意 LLM provider 通信，支持纯文本和视觉（图片+文本）两种输入模式
**Depends on**: Phase 1
**Requirements**: LLM-01, LLM-02, LLM-03, LLM-04, LLM-05, LLM-06, LLM-07
**Success Criteria** (what must be TRUE):
  1. 通过 `provider/model_name` 格式（如 `deepseek/deepseek-chat`）路由请求到正确的 API 端点
  2. 纯文本对话请求返回完整的 LLM 响应（非流式）
  3. 流式请求通过 SSE 解析逐 token 输出，调用方能拿到完整的 token 流
  4. 视觉请求能发送 base64 编码的图片+文本，获得 LLM 的分析结果
  5. API 错误、超时、限流场景下自动重试，重试耗尽后返回结构化错误信息
**Plans**: 3 plans

Plans:
- [x] 02-01-PLAN.md — 核心基础设施：依赖声明、LLMError（9变体）、LlmProvider trait、数据类型
- [x] 02-02-PLAN.md — Provider 实现：Registry 注册中心、图片预处理、OpenAiCompatibleProvider（文本/流式/视觉）
- [x] 02-03-PLAN.md — 模块组装+测试：register_all_providers()、模块声明、完整集成测试套件

### Phase 3: TTS Core + Edge-TTS
**Goal**: 系统能将文本通过 TTS 引擎转换为音频文件，默认的 Edge-TTS 引擎可正常工作
**Depends on**: Phase 1
**Requirements**: TTS-01, TTS-02, TTS-03
**Success Criteria** (what must be TRUE):
  1. TtsProvider trait 定义了统一的 TTS 引擎接口（输入文本+参数，输出音频文件路径）
  2. 按引擎名字符串（如 `edge_tts`）通过路由器分发到对应 TTS 实现
  3. Edge-TTS 引擎通过 WebSocket 协议生成中文语音音频文件，音频可在播放器中正常播放
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md — TTS Core: TTSError + WordBoundary/TtsOutput + TtsProvider trait + synthesize 路由器
- [x] 03-02-PLAN.md — Edge-TTS 引擎: 原生 WebSocket 实现（SSML/音频流/词边界/重试/代理）
- [x] 03-03-PLAN.md — TTS 集成测试: 8 项测试覆盖 TTS-01/02/03

### Phase 4: Prompt System + Visual Analyzer
**Goal**: 系统能管理版本化 Prompt 模板并按需渲染，能从视频中提取关键帧并批量发送给视觉 LLM 获取结构化分析结果
**Depends on**: Phase 1, Phase 2
**Requirements**: PRMP-01, PRMP-02, PRMP-03, PRMP-04, VISL-01, VISL-02, VISL-03
**Success Criteria** (what must be TRUE):
  1. Prompt 按 category/name/version 三级索引存储，加载时校验模板完整性
  2. 模板渲染能正确替换 Jinja 风格占位符变量，输出完整 prompt 文本
  3. 从视频中按时间间隔通过 FFmpeg 提取关键帧图片
  4. 批量帧图片发送给视觉 LLM 后，返回 JSON 格式的结构化帧分析报告
  5. 视觉分析批量请求受信号量限制，不超出并发控制阈值
**Plans**: 4 plans

Plans:
- [ ] 04-01-PLAN.md — Prompt 系统核心：类型/错误/注册表/模板渲染（PRMP-01, PRMP-02）
- [ ] 04-02-PLAN.md — 视觉分析器核心：帧提取快路径 + 4 级回退（VISL-01）
- [ ] 04-03-PLAN.md — Prompt 系统完成：Manager/验证器/注册函数/模板文件（PRMP-03, PRMP-04）
- [ ] 04-04-PLAN.md — 视觉分析器完成 + 模块集成（VISL-02, VISL-03）

### Phase 5: Script Management
**Goal**: 系统能加载、编辑、校验、保存 JSON 格式的视频脚本，且与 Python 版格式兼容
**Depends on**: Phase 1
**Requirements**: SCRP-01, SCRP-02, SCRP-03
**Success Criteria** (what must be TRUE):
  1. 能加载 Python 版生成的 JSON 脚本文件，所有字段正确反序列化为 Rust 结构体
  2. 能修改脚本的段落文案、OST 类型（使用 Rust 枚举）、时间戳并保存回 JSON
  3. 脚本校验能检测缺失必要字段和不合理的时间戳，返回明确的错误描述
**Plans**: 2 plans

Plans:
- **Wave 1** *(no blockers — can execute immediately)*
  - [x] 05-01-PLAN.md — 数据模型 + 加载/保存/校验（ScriptClip, OstType, ScriptError, load/save/validate）
- **Wave 2** *(blocked on Wave 1 completion)*
  - [x] 05-02-PLAN.md — 编辑 API + 集成测试（不可变更新方法 + round-trip 测试）

**Cross-cutting constraints:** 不可变数据更新（所有编辑方法返回新 Script 实例）

### Phase 6: Documentary Pipeline
**Goal**: 纪录片模式完整流水线可端到端运行——从视频输入到最终合成输出，验证整体架构
**Depends on**: Phase 1, Phase 2, Phase 3, Phase 4, Phase 5
**Requirements**: FFMP-02, FFMP-03, FFMP-04, FFMP-05, FFMP-06, FFMP-07, FFMP-08, DOCS-01, DOCS-02, DOCS-03
**Success Criteria** (what must be TRUE):
  1. 6 步流水线（加载脚本 -> TTS 生成 -> 视频裁剪 -> 音频字幕合并 -> 片段拼接 -> 最终合成）完整运行并输出最终视频文件
  2. 按 OST 类型正确分支处理——OST=0（仅解说音）、OST=1（仅原声）、OST=2（混合）三种模式各产生正确的音频轨道组合
  3. 从视频帧分析结果通过 LLM 生成解说文案，文案内容与帧画面相关
  4. SRT 字幕文件从 TTS 词边界时间戳生成，时间戳与音频同步
  5. 每步完成后状态更新并可被外部查询（进度追踪）
**Plans**: TBD

### Phase 7: SDE Pipeline
**Goal**: 短剧解说模式完整流水线可运行——从字幕文件解析到最终带解说的视频输出
**Depends on**: Phase 6
**Requirements**: SDE-01, SDE-02, SDE-03
**Success Criteria** (what must be TRUE):
  1. 从 SRT/ASS 字幕文件提取中文对话内容，正确处理中文编码
  2. LLM 分析剧情后生成带 OST 类型标记的解说脚本
  3. 短剧解说完整流水线（字幕解析 -> LLM 分析 -> 带 OST 脚本 -> 视频处理）端到端输出最终视频
**Plans**: TBD

### Phase 8: SDP Pipeline
**Goal**: 短剧混剪模式完整流水线可运行——支持跨多个视频源的裁剪和拼接
**Depends on**: Phase 6
**Requirements**: SDP-01, SDP-02
**Success Criteria** (what must be TRUE):
  1. 从多个视频源文件中按脚本裁剪片段并拼接为完整输出视频
  2. 短剧混剪完整流水线（字幕解析 -> LLM 分析 -> 多片段脚本 -> 视频处理）端到端运行
**Plans**: TBD

### Phase 9: JianYing Export
**Goal**: 系统能将项目时间线导出为剪映草稿 JSON 格式，可在剪映专业版中打开编辑
**Depends on**: Phase 5
**Requirements**: JYNG-01, JYNG-02
**Success Criteria** (what must be TRUE):
  1. 生成的剪映草稿 JSON 可被剪映专业版正确导入，显示完整的时间线结构
  2. 视频片段、音频轨道正确映射到剪映格式的时间线层级（字幕轨道在 D-07 中明确排除）
**Plans**: 4 plans

Plans:
- **Wave 1** *(no blockers — can execute immediately)*
  - [x] 09-01-PLAN.md — 基础设施（error.rs, time.rs, types.rs, template.rs, Cargo.toml uuid, lib.rs 模块导出）
- **Wave 2** *(blocked on Wave 1 completion)*
  - [x] 09-02-PLAN.md — 核心构建块（material.rs, segment.rs, track.rs）
- **Wave 3** *(blocked on Wave 2 completion)*
  - [x] 09-03-PLAN.md — Builder API + 导出（builder.rs, probe_audio, ExportRequest, export_draft）
- **Wave 4** *(blocked on Wave 3 completion)*
  - [x] 09-04-PLAN.md — 集成测试（OST 分支 + JSON 结构 + ID 引用一致性验证）

**Cross-cutting constraints:** 所有时间值为微秒整数（SEC=1,000,000），素材路径必须绝对路径，每个 segment 必须有 Speed 素材对象

### Phase 10: Tauri Command Layer
**Goal**: 所有 Rust 业务功能通过 Tauri 2.0 命令暴露给前端，支持实时进度推送和文件选择
**Depends on**: Phase 6, Phase 7, Phase 8
**Requirements**: TAURI-01, TAURI-02, TAURI-03
**Success Criteria** (what must be TRUE):
  1. Tauri 2.0 应用启动后，所有业务函数（配置加载、三种流水线启动、脚本管理）可通过 Tauri 命令从 JS 端调用
  2. 流水线执行过程中，前端通过 Tauri events 实时收到每步进度更新
  3. 前端可通过 Tauri 文件对话框选择视频文件和字幕文件，路径传递给 Rust 命令
**Plans**: TBD
**UI hint**: yes

### Phase 11: Extended Features
**Goal**: 补齐 Python 版的辅助功能——音频标准化、YouTube 下载、Pexels 素材搜索、智能音量控制
**Depends on**: Phase 1
**Requirements**: EXTD-01, EXTD-02, EXTD-03, EXTD-04
**Success Criteria** (what must be TRUE):
  1. 音频文件经过 LUFS 响度标准化后，响度符合目标值（如 -14 LUFS）
  2. 通过 yt-dlp 子进程下载 YouTube 视频到本地
  3. 通过 Pexels API 搜索素材并下载到本地
  4. 根据内容类型（解说/原声/BGM）应用预设音量参数
**Plans**: TBD

### Phase 12: Additional TTS Engines
**Goal**: 除 Edge-TTS 外的 6 个 TTS 引擎全部实现，达到与 Python 版一致的 TTS 功能覆盖
**Depends on**: Phase 3
**Requirements**: TTS-04, TTS-05, TTS-06, TTS-07, TTS-08, TTS-09
**Success Criteria** (what must be TRUE):
  1. Azure Speech TTS 引擎通过 REST API 生成语音音频
  2. Tencent TTS 引擎生成语音音频
  3. SoulVoice TTS 引擎生成语音音频
  4. Qwen TTS 引擎生成语音音频
  5. IndexTTS2 语音克隆引擎用参考音频生成语音
  6. Doubao TTS 引擎生成语音音频
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12
(Phases 2 & 3 can overlap; Phases 9, 11, 12 are independent of each other)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 3/3 | Complete | 2026-04-28 |
| 2. LLM Service Layer | 3/3 | Complete | 2026-04-29 |
| 3. TTS Core + Edge-TTS | 3/3 | Complete | 2026-04-28 |
| 4. Prompt System + Visual Analyzer | 0/4 | Planning complete | - |
| 5. Script Management | 2/2 | Complete | 2026-04-29 |
| 6. Documentary Pipeline | 0/? | Not started | - |
| 7. SDE Pipeline | 0/? | Not started | - |
| 8. SDP Pipeline | 0/? | Not started | - |
| 9. JianYing Export | 4/4 | Complete | 2026-04-29 |
| 10. Tauri Command Layer | 0/? | Not started | - |
| 11. Extended Features | 0/? | Not started | - |
| 12. Additional TTS Engines | 0/? | Not started | - |
