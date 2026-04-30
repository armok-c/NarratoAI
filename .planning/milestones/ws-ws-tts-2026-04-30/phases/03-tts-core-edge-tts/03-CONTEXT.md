# Phase 3: TTS Core + Edge-TTS - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

创建 TTS 核心抽象层（TtsProvider async trait + 路由器函数）并实现第一个具体引擎（Edge-TTS via WebSocket），使系统能将中文文本转为 MP3 语音音频文件。输出包含词边界时序数据供 Phase 6 字幕生成使用。

Phase 3 只实现 Edge-TTS 一个引擎。其余 6 个引擎在 Phase 12 实现。
</domain>

<decisions>
## Implementation Decisions

### Edge-TTS 实现策略
- **D-01:** 原生 WebSocket 实现，用 tokio-tungstenite 直接与微软 Edge TTS 服务通信（wss:// 端点、SSML 请求、流式接收 audio + WordBoundary chunk）。不依赖 Python edge-tts 库，不使用 CLI 子进程。
- **D-02:** WebSocket 连接支持代理，从 `config.proxy` 读取代理设置，通过 tokio-tungstenite 的 connector 配置。
- **D-03:** 3 次重试，间隔 1 秒，与 Python 版行为一致。重试耗尽后返回 `TTSError`。
- **D-04:** 音频输出为 MP3 格式。Edge TTS 服务端默认返回 MP3 音频流，直接写入 `.mp3` 文件。

### TtsProvider Trait 设计
- **D-05:** async trait，用 `#[async_trait]` 宏实现。同步引擎（后续 Phase 12）内部用 `tokio::task::spawn_blocking` 包装。
- **D-06:** 返回值：`Result<TtsOutput, TTSError>`，其中 `TtsOutput { audio_file_path: PathBuf, word_boundaries: Vec<WordBoundary>, duration: f64 }`。调用方从返回值获取所有必要数据。
- **D-07:** 独立参数：`text: &str, voice_name: &str, rate: f64, pitch: f64, output_path: &Path`。不支持的参数由各引擎实现忽略。
- **D-08:** `WordBoundary { start_offset: u64, end_offset: u64, text: String }`。单位 100 纳秒（与 Python 版一致，便于 Phase 6 字幕生成）。

### SubMaker / 字幕时序
- **D-09:** 不保留独立 SubMaker 结构。词边界数据直接内嵌在 `TtsOutput.word_boundaries: Vec<WordBoundary>` 中。Phase 6 字幕生成直接消费此列表。
- **D-10:** Edge-TTS 通过 WebSocket 流式接收 `WordBoundary` / `SentenceBoundary` 事件，实时填充 `Vec<WordBoundary>`。不支持词边界的引擎（SoulVoice 等，Phase 12）返回估算词边界：整段文本一个 WordBoundary，start=0，end=基于文本长度的估算时长。

### 路由器分发机制
- **D-11:** 字符串匹配分发，与 Python 版 `tts()` 函数一致：`match engine { "edge_tts" => ... }`。
- **D-12:** 模块级 `pub async fn synthesize(engine: &str, ...) -> Result<TtsOutput, TTSError>` 函数，不封装为结构体。
- **D-13:** 未知引擎字符串返回 `TTSError::UnknownEngine { engine: String }`。不静默 fallback 到 Edge-TTS（与 Python 不同——Rust 用类型安全的错误传播）。
- **D-14:** Phase 3 只实现 Edge-TTS 一个 match 分支。Phase 12 有 7 个分支时若觉得冗长再重构为注册表模式。YAGNI。

### Claude's Discretion
- D-06 TtsOutput 的 duration 字段：用户选择返回结构体包含 duration，Claude 决定用 `f64`（秒，与 Python 一致）。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 TTS 参考
- `app/services/voice.py` — Python 版 TTS 路由器和全部 7 个引擎实现。核心函数：`tts()`（路由分发，行 1238-1276）、`azure_tts_v1()`（Edge-TTS WebSocket 实现，行 1309-1368）、`azure_tts_v2()`（Azure Speech SDK，行 1371-1462）、`tts_multiple()`（批量 TTS 调用，行 1693-1777）
- `app/services/voice.py:SubMaker` — Python 版字幕事件收集器，`subs: list[str]` + `offset: list[tuple[int, int]]`
- `app/services/voice.py:new_sub_maker()` / `add_subtitle_event()` — SubMaker 创建和填充辅助函数
- `config.example.toml` — proxy 配置段（`[proxy]` 下的 `enabled`、`http`、`https` 字段）

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（技术栈、架构决策）
- `.planning/REQUIREMENTS.md` — Phase 3 需求：TTS-01（trait）、TTS-02（路由器）、TTS-03（Edge-TTS 引擎）
- `.planning/ROADMAP.md` — Phase 3 定义和成功标准
- `.planning/workstreams/ws-tts/ROADMAP.md` — ws-tts 工作流内里程碑

### Phase 1 决策（前置依赖）
- `.planning/phases/01-foundation/01-CONTEXT.md` — 项目结构（D-01~D-06）、配置系统（D-07~D-10）、FFmpeg 集成（D-11~D-15）、错误处理（D-16~D-18）、测试与版本（D-19~D-20）

### 代码库映射
- `.planning/codebase/ARCHITECTURE.md` — Python 版分层架构、TTS 在流水线中的位置
- `.planning/codebase/STACK.md` — Python 版依赖（edge-tts 7.2.7 + 其他 TTS SDK）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/services/voice.py:azure_tts_v1()` — Edge-TTS 的完整 WebSocket 通信逻辑（Communicate 构造、stream() 异步迭代、audio/WordBoundary chunk 处理、3 次重试）。Rust 版需等价实现。
- `app/services/voice.py:get_edge_tts_proxy()` — 代理地址提取逻辑（从 `config.proxy` 读取）
- `app/services/voice.py:convert_rate_to_percent()` / `convert_pitch_to_percent()` — rate/pitch 参数转换（1.0 → "+0%"，>1.0 → "+N%"）
- `config.example.toml` `[proxy]` 段 — 代理配置字段名和格式的权威来源

### Established Patterns
- **字符串路由**：Python 版 `tts()` 用 if/else 字符串匹配分发，Rust 版保持 match 语句分发
- **重试模式**：所有 TTS 引擎统一 3 次重试 + 1 秒间隔，Rust 版保持一致
- **SubMaker 模式**：Python 版在 TTS 流式接收过程中实时填充词边界事件，Rust 版直接在 TtsOutput 中累积 `Vec<WordBoundary>`

### Integration Points
- `src/tts/mod.rs` — TTS 模块入口，定义 trait + router + WordBoundary + TtsOutput + TTSError
- `src/tts/edge_tts.rs` — Edge-TTS 引擎具体实现
- `src/config/mod.rs` (Phase 1) — TTS 读取 proxy 配置
- `src/error.rs` (Phase 1) — 新增 `TTSError` 枚举
- Phase 6 纪录片流水线通过 `pub async fn synthesize()` 调用 TTS

</code_context>

<specifics>
## Specific Ideas

- 用户要求 Edge-TTS 用原生 Rust WebSocket 实现，不依赖 Python 子进程
- 代理必须支持（中国大陆网络环境）
- 输出 MP3 格式，与 Python 版一致
- 未知引擎返回错误而非静默 fallback（用户偏好明确错误 > 隐藏问题）

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 3-TTS Core + Edge-TTS*
*Context gathered: 2026-04-28*
