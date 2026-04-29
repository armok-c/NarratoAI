# Phase 4: Prompt System + Visual Analyzer - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

实现两个相关子系统：

1. **Prompt 系统** — 版本化 Prompt 模板注册表（category/name/version 三级索引），Jinja 风格 `${variable}` 模板渲染（含过滤器），模板完整性校验，按关键词/分类/模型类型搜索。三类 Prompt：documentary、short_drama_editing、short_drama_narration。

2. **视觉分析器** — 视频关键帧提取（通过 FFmpeg，含 4 级回退策略），帧文件管理，批量帧编码（base64 data URL），通过 Phase 2 的 LlmProvider 发送给视觉 LLM，返回结构化 JSON 分析报告。Semaphore 并发控制。

覆盖需求：PRMP-01, PRMP-02, PRMP-03, PRMP-04, VISL-01, VISL-02, VISL-03

</domain>

<decisions>
## Implementation Decisions

### Prompt 存储方式
- **D-01:** 编译期嵌入——用 `include_str!()` 将 `.md` 模板文件在编译时嵌入二进制。Prompt 更新走代码发布流程，与 Python 版行为一致（模板不可被用户随意修改）。
- **D-02:** Rust 函数注册——每个 Prompt 通过 Rust 函数注册到 Registry，元数据以结构体字段传入（name/category/version/model_type/output_format/tags/parameters），模板内容通过 `include_str!()` 嵌入。
- **D-03:** 按分类分文件——模板文件 `src/prompt/templates/{category}/{name}_v{version}.md`，注册函数 `src/prompt/registry/{category}.rs`，通过 `register_all_prompts()` 集中注册。
- **D-04:** 版本号用字符串——`String` 类型，格式如 `"v1.0"`、`"v2.1"`。完全对齐 Python 版，Registry key 可直接使用。

### 模板渲染引擎
- **D-05:** 自实现简单替换——用正则实现 `${variable}` 和 `$variable` 两种占位符替换。完全对齐 Python 版行为，Prompt 模板从 Python 迁移时零修改。
- **D-06:** 全部 6 个过滤器——`upper`、`lower`、`title`、`strip`、`truncate`、`json`。通过 `HashMap<String, fn>` 注册自定义过滤器，支持 `${variable|filter_name}` 语法。
- **D-07:** 缺失变量返回错误——渲染时所有模板变量必须由调用方提供，缺失返回 `PromptError::TemplateRender`，明确指出缺失参数名。与 Python 版抛出 `TemplateRenderError` 行为一致。
- **D-08:** 两种占位符都支持——`${variable}` 和 `$variable` 写法均可用。注意 `$` 字符转义避免误匹配。

### 帧提取与 Phase 1 集成
- **D-09:** 新建 `src/visual/` 模块——帧提取和视觉分析放同一模块，内部调用 Phase 1 `ffmpeg` 模块的工具函数。按领域分离：ffmpeg 模块是底层操作，visual 模块是业务编排。
- **D-10:** 完整 4 级回退——1) 硬件加速 JPEG → 2) 软件解码 JPEG → 3) PNG 格式（再转 JPEG）→ 4) BMP/MJPEG。完全对齐 Python 版 `_extract_single_frame_optimized()` 行为。
- **D-11:** 函数参数 + config 默认值——`extract_frames()` 接收明确参数（`interval_seconds`, `quality`, `output_dir`），参数未提供时从 `config.toml` `[frames]` 段读默认值。灵活可测试。
- **D-12:** 快路径 + 逐帧回退——先尝试单条 ffmpeg `fps=1/{interval}` filter 命令（快速），失败后自动回退到逐帧提取（可靠）。完全对齐 Python 版 `extract_frames_by_interval_with_fallback()`。

### 视觉分析批量策略
- **D-13:** 磁盘文件→加载→批量发送——帧保存为 JPEG 文件到磁盘 → 按 `batch_size` 分批加载 → 用 Phase 2 的图片预处理（thumbnail 1024px + JPEG quality 85 + base64 data URL）→ 调用 `LlmProvider.analyze_images()`。与 Python 版完全一致。
- **D-14:** 收集错误继续执行——单帧提取失败用占位符标记，LLM 调用失败重试后仍失败则返回部分结果 + 错误统计。每个批次失败不影响其他批次。
- **D-15:** 对齐 Python JSON 格式 + Rust 强类型——定义 `FrameObservation` 和 `BatchAnalysisResult` 结构体，serde 反序列化 LLM 返回的 JSON。字段对齐 Python 版的 `frame_observations` + `overall_activity_summary`。
- **D-16:** 复用回调模式——视觉分析接受 `Option<ProgressCallback>`，在帧提取和每个批次分析完成后上报进度百分比和步骤描述。与 Phase 1 D-15 一致，后续 Tauri events 统一对接。

### Prompt 注册时机
- **D-17:** 统一初始化函数——`register_all_prompts()` 公开函数，由库初始化入口显式调用。与 Phase 2 的 `register_all_providers()` 模式一致，调用方控制初始化顺序。
- **D-18:** `Arc<RwLock<PromptRegistry>>`——全局 Registry 实例用读写锁保护，与 Phase 1 Config 和 Phase 2 Provider Registry 模式一致。支持并发读取。

### 视觉分析 Prompt 来源
- **D-19:** 混合模式——帧内容分析的通用 Prompt（如 frame_analysis）从 Registry 获取并渲染；批量帧处理的 JSON 格式约束 prompt 硬编码在视觉分析服务中。完全对齐 Python 版行为。
- **D-20:** 包含输出验证——`PromptManager` 提供 `validate_output()` 方法，校验 LLM 返回的 JSON 格式、解说文案结构、剧情分析结果。对齐 Python 版 `validate_narration_script()` / `validate_plot_analysis()` / `validate_json()`。

### 错误类型定义
- **D-21:** 按领域分两个 enum——`PromptError`（prompt 领域）和 `VisualError`（visual 领域）。遵循 Phase 1 D-17 的按领域分错误枚举模式，thiserror 派生，中文 `#[error("...")]` 消息。
- **D-22:** 对齐 Python 变体——`PromptError` 包含：`TemplateRender`、`NotFound`、`Registration`、`Validation`、`Version`。`VisualError` 包含：`FrameExtraction`、`Analysis`、`BatchPartial`。完全对齐 Python 版 8 个异常类。

### 帧文件目录管理
- **D-23:** 调用方指定输出目录——`extract_frames()` 接收 `output_dir: &Path` 参数，由调用方（如 Phase 6 流水线）决定输出位置。模块不负责创建/清理目录。
- **D-24:** 对齐 Python 命名——帧文件命名 `keyframe_{frame_number:06d}_{HHMMSSmmm}.jpg`。如 `keyframe_000120_000500000.jpg`。与 Python 版一致，便于跨版本调试比对。

### Claude's Discretion
无——所有决策均由用户明确选择。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 Prompt 系统参考
- `app/services/prompts/base.py` — Prompt 基类、元数据、ModelType/OutputFormat 枚举定义
- `app/services/prompts/registry.py` — PromptRegistry：三级索引 `{category: {name: {version: prompt}}}`
- `app/services/prompts/template.py` — TemplateRenderer：`${variable}` 替换 + 过滤器语法 + 变量提取
- `app/services/prompts/manager.py` — PromptManager 统一管理接口（get_prompt/register_prompt/search/validate_output）
- `app/services/prompts/validators.py` — 输出验证器（JSON/解说文案/剧情分析格式校验）
- `app/services/prompts/exceptions.py` — 5 种 Prompt 异常类：TemplateRenderError/PromptNotFoundError/PromptRegistrationError/PromptValidationError/PromptVersionError
- `app/services/prompts/__init__.py` — Prompt 系统模块入口
- `app/services/prompts/documentary/frame_analysis.py` — FrameAnalysisPrompt：纪录片帧分析模板（实际 Prompt 内容参考）
- `app/services/prompts/documentary/narration_generation.py` — 解说文案生成 Prompt 模板
- `app/services/prompts/short_drama_editing/plot_extraction.py` — 短剧剧情提取 Prompt 模板
- `app/services/prompts/short_drama_narration/script_generation.py` — 短剧脚本生成 Prompt 模板

### Python 版视觉分析器参考
- `app/utils/video_processor.py` — VideoProcessor：帧提取（4 级回退 + 快路径）、视频信息探测
- `app/services/documentary/frame_analysis_service.py` — DocumentaryFrameAnalysisService：帧分析 → 文案生成编排
- `app/services/documentary/frame_analysis_models.py` — FrameBatchResult 数据模型
- `app/utils/ffmpeg_utils.py` — FFmpeg 工具函数（硬件加速检测、命令构建）

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库 + Tauri 命令、文件存储、完全不复用 smartEdit）
- `.planning/REQUIREMENTS.md` — Phase 4 需求：PRMP-01~04, VISL-01~03
- `.planning/ROADMAP.md` — Phase 4 定义、成功标准、依赖关系（Phase 1 + Phase 2）
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（crate 结构、tokio/tracing、serde/toxic、thiserror、spawn_blocking）
- `.planning/phases/02-llm-service-layer/02-CONTEXT.md` — Phase 2 决策（async-openai、LlmProvider trait、Semaphore、图片预处理、batch_size/max_concurrency）
- `.planning/phases/05-script-management/05-CONTEXT.md` — 下游参考：Phase 5 脚本模型（Phase 6 将同时依赖 Phase 4 + Phase 5）

### 代码库映射
- `.planning/codebase/ARCHITECTURE.md` — Python 版 Prompt 系统架构和视觉分析数据流
- `.planning/codebase/STACK.md` — Python 版依赖栈（Pillow/ffmpeg 等帧处理工具）
- `.planning/codebase/INTEGRATIONS.md` — LLM 服务集成点（视觉分析通过 LlmProvider 调用）

### Rust 依赖参考
- `Cargo.toml` — Phase 1/2 已定义的依赖（tokio、serde、tracing、thiserror、toml、async-openai、image、base64）
- `src/ffmpeg/mod.rs` — Phase 1 FFmpeg 封装（spawn_blocking + ffmpeg-sidecar），帧提取直接复用
- `src/llm/mod.rs` — Phase 2 LLM 模块（LlmProvider trait，视觉分析入口）
- `src/config/types.rs` — 配置类型（AppSection.vision_*、FramesSection 帧提取参数）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/ffmpeg/mod.rs` — Phase 1 的 `spawn_blocking` + ffmpeg-sidecar 封装，帧提取直接复用其命令构建和执行模式
- `src/llm/mod.rs` — Phase 2 的 `LlmProvider` trait（`analyze_images()` 方法），视觉分析批量调用入口
- `src/llm/` — Phase 2 的图片预处理（thumbnail 1024px + JPEG quality 85 + base64 data URL），帧文件编码直接复用
- `src/config/types.rs` — `FramesSection`（帧间隔等参数）、`AppSection`（vision LLM 配置），Phase 4 读取这些配置段

### Established Patterns
- **模块级结构**：Phase 1/2/5 均是 `src/{domain}/mod.rs` + 子模块。Phase 4 新增 `src/prompt/` 和 `src/visual/`
- **Arc<RwLock<T>> 全局实例**：Phase 1 Config (`Arc<RwLock<AppConfig>>`) 和 Phase 2 Registry 模式，Prompt Registry 对齐此模式
- **thiserror 领域错误**：Phase 1 (ConfigError/FFmpegError) 和 Phase 2 (LLMError) 模式，PromptError/VisualError 遵循相同约定
- **异步回调进度上报**：Phase 1 D-15 的 `ProgressCallback` 模式，视觉分析进度上报复用
- **显式初始化函数**：Phase 2 的 `register_all_providers()`，Phase 4 的 `register_all_prompts()` 对齐

### Integration Points
- `src/lib.rs` — 新增 `pub mod prompt;` + `pub mod visual;`
- `src/error.rs` — 新增 `PromptError` 和 `VisualError` enum（或放在各自模块内）
- `src/ffmpeg/mod.rs` — visual 模块调用 FFmpeg 工具函数进行帧提取
- `src/llm/mod.rs` — visual 模块调用 `LlmProvider::analyze_images()` 进行批量视觉分析
- `Cargo.toml` — 新增 `regex`（模板渲染）、`serde_json`（分析结果）依赖（大多数已有）
- Phase 6 Documentary Pipeline — 消费 Prompt 系统获取解说文案模板 + 消费视觉分析器进行帧分析

</code_context>

<specifics>
## Specific Ideas

- 用户明确要求帧提取完全对齐 Python 版的 4 级回退策略（硬件加速→软件解码→PNG→BMP/MJPEG），不可简化
- 模板占位符语法必须同时支持 `${variable}` 和 `$variable`，与 Python 版兼容
- 视觉分析 prompt 采用混合模式——通用分析 prompt 从 Registry 获取，批量 JSON 格式 prompt 硬编码在服务中
- 帧文件命名必须对齐 Python 版 `keyframe_{frame_number:06d}_{HHMMSSmmm}.jpg` 格式
- Prompt 系统需包含 `validate_output()` 输出验证功能，验证解说文案和剧情分析 JSON 格式

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-prompt-system-visual-analyzer*
*Context gathered: 2026-04-29*
