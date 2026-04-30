# Phase 12: Additional TTS Engines - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-30
**Phase:** 12-additional-tts-engines
**Areas discussed:** Azure Speech 路由策略, HTTP 调用 vs 官方 SDK, IndexTTS2 参考音频入参, 引擎文件组织方式

---

## Azure Speech 路由策略

| Option | Description | Selected |
|--------|-------------|----------|
| 内部智能回退（对齐 Python） | voice_name 匹配 Neural 格式走 Azure SDK，不匹配回退 Edge-TTS | ✓ |
| 只做 V2，不回退 | voice_name 不匹配直接报错，用户手动切换引擎 | |
| 拆分为两个独立引擎名 | Azure Neural 用 edge_tts，Azure SDK 单独暴露 | |

**User's choice:** 内部智能回退（对齐 Python）
**Notes:** 对齐 `should_use_azure_speech_services()` 行为

---

| Option | Description | Selected |
|--------|-------------|----------|
| 路由器内联分派 | match "azure_speech" 分支直接内联 V1/V2 判断 | ✓ |
| Trait 实现内部分派 | AzureSpeechEngine 内部分派，多一层包装 | |

**User's choice:** 路由器内联分派
**Notes:** 不创建额外的包装结构体

---

| Option | Description | Selected |
|--------|-------------|----------|
| 硬编码音色列表（对齐 Python） | 字符串数组硬编码，提供公开查询函数 | ✓ |
| 不提供音色列表 | 用户手动输入 | |
| 外部可编辑列表文件 | JSON/TOML 文件运行时加载 | |

**User's choice:** 硬编码音色列表（对齐 Python）
**Notes:** 对齐 `get_all_azure_voices()` 行为

---

## HTTP 调用 vs 官方 SDK

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 reqwest（全 HTTP） | 所有 6 个引擎用 reqwest，手动实现认证签名 | ✓ |
| Azure/Tencent 用 SDK，其余 HTTP | 两个引擎用社区 Rust SDK，其余 reqwest | |
| 按引擎逐一定 | 研究阶段决定每个引擎方案 | |

**User's choice:** 统一 reqwest（全 HTTP）
**Notes:** 零额外 SDK 依赖，风格一致

---

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 3 次重试 | 所有引擎 60s 超时 + 3 次重试 | ✓ |
| 各引擎自定义重试 | 每个引擎自行决定 | |
| 无重试，由调用方处理 | TTS 层直接返回错误 | |

**User's choice:** 统一 3 次重试
**Notes:** 对齐 Python 版各引擎重试行为

---

## IndexTTS2 参考音频入参

| Option | Description | Selected |
|--------|-------------|----------|
| 前缀编码在 voice_name 内（对齐 Python） | indextts2:/path/to/ref.wav → 解析路径 | ✓ |
| 扩展 trait 增加参数 | 改 synthesize() 签名加 reference_audio_path | |
| 构造时预设参考音频 | 引擎 struct 字段预设路径 | |

**User's choice:** 前缀编码在 voice_name 内（对齐 Python）
**Notes:** 不改 trait 签名，soulvoice/tencent/qwen3 同样走前缀解析

---

## 引擎文件组织方式

| Option | Description | Selected |
|--------|-------------|----------|
| 每个引擎独立文件（对齐 Phase 3） | 6 个文件 + 1 个共享模块 | ✓ |
| 按复杂度分组 | 小引擎合并，大引擎独立 | |
| 合并为一个文件 | 所有引擎放一个 engines.rs | |

**User's choice:** 每个引擎独立文件（对齐 Phase 3）
**Notes:** 与 edge_tts.rs 模式一致

---

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python 引擎名 | 文件名与 voice.py match 字符串一致 | ✓ |
| 短文件名 | azure.rs, tencent.rs 等 | |
| 研究阶段决定 | 待研究后确定 | |

**User's choice:** 对齐 Python 引擎名
**Notes:** azure_speech.rs, tencent_tts.rs, soulvoice.rs, qwen_tts.rs, indextts2.rs, doubaotts.rs

---

| Option | Description | Selected |
|--------|-------------|----------|
| 提取共享模块 | src/tts/common.rs 放代理/重试/写入逻辑 | ✓ |
| 各自独立实现（对齐 Python） | 每个引擎文件自包含 | |

**User's choice:** 提取共享模块
**Notes:** 减少 6 个文件中的重复代码

---

## Claude's Discretion

- 各引擎 HTTP 请求体构造细节
- reqwest Client 实例复用策略
- 共享模块具体函数签名
- IndexTTS2 HTTP 超时值

## Deferred Ideas

None — discussion stayed within phase scope
