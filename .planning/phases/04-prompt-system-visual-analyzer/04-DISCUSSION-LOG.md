# Phase 4: Prompt System + Visual Analyzer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 04-prompt-system-visual-analyzer
**Areas discussed:** Prompt 存储方式, 模板渲染引擎, 帧提取与 Phase 1 集成, 视觉分析批量策略, Prompt 注册时机, 视觉分析 Prompt 来源, 错误类型定义, 帧文件目录管理

---

## Prompt 存储方式

| Option | Description | Selected |
|--------|-------------|----------|
| 编译期嵌入 | 用 include_str! 将模板文件嵌入二进制，与 Python 行为一致，零运行时开销 | ✓ |
| 运行时外部文件 | 从 prompts/ 目录运行时加载，用户可直接编辑 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 编译期嵌入（推荐）
**Notes:** Prompt 更新走代码发布流程

| Option | Description | Selected |
|--------|-------------|----------|
| Rust 函数注册 | 每个 Prompt 通过函数注册到 Registry，元数据以结构体字段传入 | ✓ |
| Markdown Frontmatter | YAML frontmatter 定义元数据，正文是模板内容 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** Rust 函数注册（推荐）
**Notes:** 与 Python 版 Prompt 子类 `__init__` 模式一致

| Option | Description | Selected |
|--------|-------------|----------|
| 按分类分文件 | 模板文件按分类目录分组，注册函数按分类分文件 | ✓ |
| 单文件集中注册 | 所有 Prompt 在一个文件中注册 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 按分类分文件（推荐）
**Notes:** 清晰可扩展

| Option | Description | Selected |
|--------|-------------|----------|
| 字符串版本号 | String 类型，如 "v1.0"，对齐 Python | ✓ |
| Semver 结构体 | semver crate，结构化版本比较 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 字符串版本号（推荐）
**Notes:** Prompt 不需要 semver 兼容性检查

---

## 模板渲染引擎

| Option | Description | Selected |
|--------|-------------|----------|
| 自实现简单替换 | regex 实现 ${variable} 语法，完全对齐 Python 行为 | ✓ |
| Tera | Jinja2 风格，功能强大但语法不兼容现有模板 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 自实现简单替换（推荐）
**Notes:** Prompt 模板从 Python 迁移时零修改

| Option | Description | Selected |
|--------|-------------|----------|
| 全部 6 个过滤器 | upper/lower/title/strip/truncate/json | ✓ |
| 仅基础 4 个 | upper/lower/title/strip | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 全部 6 个（推荐）
**Notes:** 实现简单，保证迁移兼容性

| Option | Description | Selected |
|--------|-------------|----------|
| 返回错误 | 缺失变量返回 RenderError，明确指出参数名 | ✓ |
| 替换为空字符串 | 静默替换，可能产生不完整 Prompt | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 返回错误（推荐）
**Notes:** 与 Python 版行为一致

| Option | Description | Selected |
|--------|-------------|----------|
| 仅 ${variable} | 只支持花括号格式 | |
| 两种都支持 | ${variable} 和 $variable 都可用 | ✓ |
| 你决定 | Claude 自主选择 | |

**User's choice:** 两种都支持
**Notes:** 完全对齐 Python 版行为

---

## 帧提取与 Phase 1 集成

| Option | Description | Selected |
|--------|-------------|----------|
| 新建 visual 模块 | 帧提取和视觉分析在 src/visual/，调用 ffmpeg 工具函数 | ✓ |
| 扩展 Phase 1 ffmpeg | 在 src/ffmpeg/ 中添加帧提取，但违反领域分离 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 新建 visual 模块（推荐）
**Notes:** 按领域分离——ffmpeg 是底层操作，visual 是业务编排

| Option | Description | Selected |
|--------|-------------|----------|
| 基础 2 级回退 | JPEG → PNG 回退 | |
| 完整对齐 Python 4 级 | HW加速→软件解码JPEG→PNG→BMP/MJPEG | ✓ |
| 你决定 | Claude 自主选择 | |

**User's choice:** 完整对齐 Python 4 级
**Notes:** 完全对齐 Python 版行为

| Option | Description | Selected |
|--------|-------------|----------|
| 函数参数 + config 默认值 | 参数未提供时从 config.toml [frames] 段读取 | ✓ |
| 仅从 config.toml 读取 | 所有参数从配置文件读取 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 函数参数 + config 默认值（推荐）
**Notes:** 灵活可测试

| Option | Description | Selected |
|--------|-------------|----------|
| 快路径 + 回退 | 先尝试 fps filter 单命令，失败后逐帧提取 | ✓ |
| 仅逐帧提取 | 只用逐帧提取 + 4 级回退 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 快路径 + 回退（推荐）
**Notes:** 完全对齐 Python 版 behavior

---

## 视觉分析批量策略

| Option | Description | Selected |
|--------|-------------|----------|
| 磁盘文件→加载→批量发送 | 帧保存为 JPEG，分批加载编码发送 | ✓ |
| 内存管道直通 | 内存中 base64 编码，不保存磁盘文件 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 磁盘文件→加载→批量发送（推荐）
**Notes:** 与 Python 版一致，帧文件可保留供调试

| Option | Description | Selected |
|--------|-------------|----------|
| 收集错误继续执行 | 部分失败标记占位符，返回部分结果+统计 | ✓ |
| 遇错即停 | 任何失败立即终止 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 收集错误继续执行（推荐）
**Notes:** 符合批量处理场景

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python + Rust 强类型 | FrameObservation/BatchAnalysisResult struct | ✓ |
| 自由格式 JSON | serde_json::Value 不做结构化校验 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 对齐 Python 格式 + Rust 强类型（推荐）
**Notes:** 与 Python 版 frame_observations + overall_activity_summary 一致

| Option | Description | Selected |
|--------|-------------|----------|
| 复用回调模式 | Option<ProgressCallback>，与 Phase 1 D-15 一致 | ✓ |
| tracing span + events | 仅日志记录，无程序化回调 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 复用回调模式（推荐）
**Notes:** 后续 Tauri events 统一对接

---

## Prompt 注册时机

| Option | Description | Selected |
|--------|-------------|----------|
| 统一初始化函数 | register_all_prompts() 公开函数，显式调用 | ✓ |
| static 惰性初始化 | OnceLock/OnceCell 首次访问时自动注册 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 统一初始化函数（推荐）
**Notes:** 与 Phase 2 register_all_providers() 模式一致

| Option | Description | Selected |
|--------|-------------|----------|
| Arc<RwLock<PromptRegistry>> | 与 Config 和 Provider Registry 一致 | ✓ |
| &'static 全局引用 | OnceLock 初始化后不可变引用 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** Arc<RwLock<PromptRegistry>>（推荐）
**Notes:** 支持并发读，保持运行时动态注册能力

---

## 视觉分析 Prompt 来源

| Option | Description | Selected |
|--------|-------------|----------|
| 全部通过 Prompt 系统 | 帧分析和批量格式 prompt 都注册在 Registry | |
| 混合模式 | 通用分析 prompt 从 Registry 获取，批量 JSON 格式 prompt 硬编码 | ✓ |
| 你决定 | Claude 自主选择 | |

**User's choice:** 混合模式（对齐 Python）
**Notes:** 完全对齐 Python 版行为

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 4 不包含 | 输出验证交给 Phase 5 Script Management | |
| 包含在 Prompt 系统 | PromptManager 提供 validate_output() | ✓ |
| 你决定 | Claude 自主选择 | |

**User's choice:** 包含在 Prompt 系统
**Notes:** 校验 JSON/解说文案/剧情分析

---

## 错误类型定义

| Option | Description | Selected |
|--------|-------------|----------|
| 按领域分两个 enum | PromptError + VisualError | ✓ |
| 单一 Phase4Error | 一个 enum 包含所有变体 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 按领域分两个 enum（推荐）
**Notes:** 遵循 Phase 1 D-17 的按领域分错误枚举模式

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python 5 + 3 变体 | PromptError(5) + VisualError(3) | ✓ |
| 精简版 | 最小化变体，后续按需添加 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 对齐 Python 5 + 3 变体（推荐）
**Notes:** PromptError: TemplateRender/NotFound/Registration/Validation/Version。VisualError: FrameExtraction/Analysis/BatchPartial

---

## 帧文件目录管理

| Option | Description | Selected |
|--------|-------------|----------|
| 调用方指定 | extract_frames() 接收 output_dir: &Path | ✓ |
| 模块自动管理临时目录 | 自动创建 tempdir，分析后清理 | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 调用方指定（推荐）
**Notes:** 与 Python 版一致，调用方可保留帧文件供调试

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python 命名 | keyframe_{frame_number:06d}_{HHMMSSmmm}.jpg | ✓ |
| 简化命名 | frame_{index:04d}.jpg | |
| 你决定 | Claude 自主选择 | |

**User's choice:** 对齐 Python 命名（推荐）
**Notes:** 便于跨版本调试比对

---

## Claude's Discretion

None — all decisions were made by the user.

## Deferred Ideas

None — discussion stayed within phase scope
