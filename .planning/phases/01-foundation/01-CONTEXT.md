# Phase 1: Foundation - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

创建可编译的 Rust 库骨架（narratoai-core），能从 TOML 文件加载配置，能通过 ffmpeg-sidecar 执行基本的视频操作（ffprobe 探测 + 视频裁剪）。配置系统支持热加载。FFmpeg 操作异步执行不阻塞运行时。硬件加速检测通过 FFmpeg 编码器列表完成。

</domain>

<decisions>
## Implementation Decisions

### Rust 项目结构
- **D-01:** 单个 lib crate（narratoai-core），不用 Cargo workspace。理由：单人项目，与 Python 版分层架构一致，后续通过 Tauri Cargo.toml 依赖引入。
- **D-02:** 按领域划分模块：Phase 1 创建 `config/`、`ffmpeg/`、`error/`。后续阶段逐步添加 `llm/`、`tts/`、`prompt/`、`script/`、`pipeline/`。
- **D-03:** src/ 根目录布局：`src/lib.rs` + `src/config/mod.rs` + `src/ffmpeg/mod.rs` + `src/error.rs`。不用 `src/app/` 嵌套。
- **D-04:** 异步运行时选 tokio（Tauri 2.0 依赖 tokio，统一生态）。
- **D-05:** 日志库选 tracing（与 tokio 深度集成，支持 span 结构化追踪，是 loguru 的 Rust 等价物）。
- **D-06:** Cargo.toml 依赖使用精确版本号（如 `tokio = "1.45.0"`），确保可复现构建。

### 配置系统
- **D-07:** 使用 serde + toml 反序列化为强类型结构体。一个顶层 `AppConfig` 结构体包含所有子段（App、Ui、Azure、Tencent、SoulVoice、TtsQwen、IndexTTS2、DoubaoTTS、Proxy、Frames），与 config.toml 的 TOML 结构一一对应。
- **D-08:** 缺失字段通过 `serde(default)` + `Default` trait 自动填充默认值，不报错。API key 等敏感字段用空字符串默认值。
- **D-09:** 热加载通过 notify crate 监听 config.toml 文件变更事件，触发重新加载。
- **D-10:** 配置并发安全用 `Arc<RwLock<AppConfig>>`。热加载时短暂获取写锁，运行中任务获取读锁读取。

### FFmpeg 集成
- **D-11:** 使用 ffmpeg-sidecar crate 封装 FFmpeg CLI 子进程调用。ffprobe 视频信息探测也通过 ffmpeg-sidecar 完成。
- **D-12:** 内置 FFmpeg 二进制，打包时分发，用户无需单独安装。
- **D-13:** 硬件加速检测通过运行 `ffmpeg -encoders` 检测可用硬件编码器（h264_nvenc、h264_amf、h264_qsv、h264_videotoolbox），匹配预定义 Profile。Phase 1 只做检测，Profile 使用留给 Phase 6。
- **D-14:** FFmpeg 操作通过 `tokio::task::spawn_blocking()` 包装在 async fn 中，不阻塞 tokio 异步运行时。
- **D-15:** FFmpeg 进度通过回调函数参数（`Option<ProgressCallback>`）上报。回调接收进度百分比和步骤描述。后续 Tauri events 通过此回调推送进度。

### 错误处理
- **D-16:** 使用 thiserror 定义领域错误枚举。Phase 1 创建 `ConfigError`（配置加载/解析/缺失字段）和 `FFmpegError`（进程启动失败/执行超时/输出解析错误）。
- **D-17:** 按领域分错误枚举。后续阶段添加 `LLMError`、`TTSError`、`PipelineError` 等。
- **D-18:** 用户可见的错误信息用中文（如"配置文件缺失必填字段: app.vision_openai_api_key"），内部 source chain 保留英文。

### 测试与版本
- **D-19:** 单元测试（同文件 `#[cfg(test)] mod tests`）+ 集成测试（`tests/` 目录）。集成测试使用真实 FFmpeg 执行操作，CI 环境通过 ffmpeg-sidecar 自动下载 FFmpeg。
- **D-20:** 版本号在 Cargo.toml 的 `[package].version` 字段管理，通过 `env!("CARGO_PKG_VERSION")` 编译时获取。不再使用 project_version 文件。

### Claude's Discretion
- D-01 crate 组织方式：用户选择"让我决定"，Claude 选定单个 lib crate。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版配置参考
- `config.example.toml` — 完整配置文件模板，所有 10 个配置段的字段定义和默认值
- `app/config/config.py` — Python 版配置加载实现（load_config、save_config、模块级单例模式）
- `app/config/defaults.py` — Python 版默认值构建逻辑（merge_missing_app_defaults）
- `app/config/ffmpeg_config.py` — FFmpeg 硬件加速检测（FFmpegProfile、FFmpegConfigManager、PROFILES 字典、get_recommended_profile）
- `app/models/exception.py` — Python 版自定义异常（HttpException、FileNotFoundException）

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（技术栈、架构决策、范围排除）
- `.planning/REQUIREMENTS.md` — Phase 1 需求：CONF-01~04, FFMP-01, FFMP-09
- `.planning/ROADMAP.md` — Phase 1 定义和成功标准

### 代码库映射
- `.planning/codebase/STACK.md` — Python 版依赖和技术栈
- `.planning/codebase/ARCHITECTURE.md` — Python 版分层架构和核心流水线
- `.planning/codebase/CONVENTIONS.md` — 代码风格约定（中文用户界面、日志模式、Provider/Registry 模式）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/config/ffmpeg_config.py:FFmpegProfile` — 硬件加速 Profile 数据结构，Rust 版可映射为 enum/struct
- `app/config/ffmpeg_config.py:PROFILES` — 5 个预定义 Profile（high_performance、compatibility、windows_nvidia、macos_videotoolbox、universal_software），Rust 版需等价定义
- `config.example.toml` — 所有配置段的字段名、类型和默认值的权威来源

### Established Patterns
- **模块级单例**：Python 版用模块级变量 `_cfg` 存储配置，Rust 版用 `Arc<RwLock<AppConfig>>` 等价实现
- **Provider/Registry 模式**：后续 Phase 2 LLM 服务层将使用，Phase 1 不涉及但库结构需预留扩展空间
- **策略模式（TTS 路由）**：后续 Phase 3 将使用，Phase 1 不涉及
- **Pipeline 模式**：后续 Phase 6 将使用 6 步流水线，Phase 1 的 FFmpeg 操作是其基础组件

### Integration Points
- `src/lib.rs` — 库入口，pub mod 导出各领域模块
- `src/config/mod.rs` — 配置加载器，被所有后续阶段依赖
- `src/ffmpeg/mod.rs` — FFmpeg 操作封装，被 Phase 6 流水线大量使用
- `src/error.rs` — 领域错误类型，被所有模块依赖

</code_context>

<specifics>
## Specific Ideas

- 用户明确要求内置 FFmpeg 二进制，不依赖系统安装
- 硬件加速检测在 Phase 1 只做检测和 Profile 匹配，具体编码器参数使用留给 Phase 6
- FFmpeg 进度回调接口在 Phase 1 定义，为后续 Tauri events 预留接口

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 1-Foundation*
*Context gathered: 2026-04-27*
