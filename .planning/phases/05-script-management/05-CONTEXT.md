# Phase 5: Script Management - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

JSON 脚本的数据模型、加载/保存、编辑、校验——三种业务模式（纪录片/SDE/SDP）共享的脚本基础设施。系统能加载 Python 版生成的 JSON 脚本文件，所有字段正确反序列化为 Rust 结构体；能修改脚本的段落文案、OST 类型（Rust 枚举）、时间戳并保存回 JSON；脚本校验能检测缺失必要字段和不合理的时间戳。

</domain>

<decisions>
## Implementation Decisions

### 数据模型设计
- **D-01:** 单个 `ScriptClip` 结构体包含所有字段（5 个核心 + 6 个管道扩展），管道扩展字段用 `Option<T>`。加载时核心字段必须有值，管道字段为 None。
- **D-02:** OST 字段用 Rust 枚举 `OstType { NarrationOnly, OriginalSound, Mixed }`，通过 `serde_repr` 序列化为 0/1/2 整数。
- **D-03:** timestamp 字段保留原始字符串格式（`HH:MM:SS,mmm-HH:MM:SS,mmm`），不解析为 Duration 类型。通过校验函数验证格式正确性。
- **D-04:** 顶层脚本类型为 `type Script = Vec<ScriptClip>`（类型别名），不包装元数据结构。与 Python 版纯数组格式一致。
- **D-05:** `_id` 字段类型为 `i64`（非 usize），与 JSON 数值兼容。
- **D-06:** `picture` 和 `narration` 为 `String` 类型。管道扩展字段：`duration: Option<f64>`, `source_time_range: Option<String>`, `edited_time_range: Option<String>`, `audio: Option<PathBuf>`, `video: Option<PathBuf>`, `subtitle: Option<PathBuf>`。

### 格式兼容性
- **D-07:** 只支持加载纯 JSON 数组格式（`[{...}, {...}]`）。不处理 LLM 输出的 `{"items": [...]}` 信封——信封由各管道模块在调用 Script 加载前自行剥离。
- **D-08:** 保存时只输出核心字段（`_id`, `timestamp`, `picture`, `narration`, `OST`），忽略 Option 管道扩展字段。与 Python 版脚本保存行为一致。
- **D-09:** 保存输出美化 JSON（indent=2），非 ASCII 字符不转义（`ensure_ascii=false` 行为）。与 Python 版 `json.dumps(indent=2, ensure_ascii=False)` 一致。
- **D-10:** 加载/保存使用 UTF-8 编码。与 Python 版 `encoding="utf-8"` 一致。

### 校验策略
- **D-11:** 校验级别与 Python 版 `check_script.py` 对齐：JSON 数组、每个元素有全部 5 个核心字段、`_id` 为正整数、timestamp 格式匹配正则 `^\d{2}:\d{2}:\d{2},\d{3}-\d{2}:\d{2}:\d{2},\d{3}$`、picture/narration 非空字符串、OST 为 0/1/2 有效值。
- **D-12:** 加载时自动校验（`load_script()` 内部调用 `validate()`），校验失败返回 `ScriptError`。不需要显式调用校验方法（但可提供 `validate()` 公开方法供复用）。
- **D-13:** 校验失败时收集所有错误（`Vec<ValidationError>`），一次性返回。不采用第一个错误即停策略。
- **D-14:** 用户可见的错误信息用中文（如"脚本第 2 段缺少必要字段: timestamp"），与 Phase 1 D-18 决策一致。

### 编辑 API
- **D-15:** Script 提供不可变更新方法，每个方法返回新的 `Script` 实例：`update_narration(index: usize, text: &str) -> Result<Script, ScriptError>`、`set_ost(index: usize, ost: OstType) -> Result<Script, ScriptError>`、`update_timestamp(index: usize, ts: &str) -> Result<Script, ScriptError>`、`update_picture(index: usize, pic: &str) -> Result<Script, ScriptError>`。
- **D-16:** 用数组下标 `usize` 定位 clip，不用 `_id` 查找。与 Vec 操作语义一致。
- **D-17:** Phase 5 只实现单字段更新。不包含增/删/重排操作（insert_clip、remove_clip、reorder）。后续阶段按需添加。
- **D-18:** 索引越界时返回 `ScriptError::IndexOutOfBounds`（中文错误信息）。

### 错误类型
- **D-19:** `ScriptError` 枚举包含变体：`JsonParse(json Error)`, `Validation(Vec<ValidationError>)`, `Io(io::Error)`, `IndexOutOfBounds`, `InvalidTimestamp`。使用 thiserror 派生。

### Claude's Discretion
- ScriptClip 的 serde 属性命名策略（snake_case vs 原始 camelCase 字段名如 sourceTimeRange）
- ValidationError 的具体结构和 Display 实现
- 是否提供 `ScriptClip::builder()` 便利方法
- 模块内部文件拆分方式

</decisions>

<specifics>
## Specific Ideas

- Python 版 `narration` 字段在 OST=1 时包含占位文本（`"播放原片N"` 或 `"播放原生_RANDOMHEX"`），Rust 版不做特殊处理——只当作普通字符串
- Python 版脚本文件保存在 `resource/scripts/YYYY-MMDD-HHMMSS.json`，文件命名由调用方决定，Script 模块只负责加载/保存内容
- 校验时不需要检查 `narration` 内容与 OST 的一致性（Python 版也不检查）

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版脚本系统参考
- `app/models/schema.py` — Pydantic 数据模型（VideoClipParams 等脚本相关结构体）
- `app/services/script_service.py` — Python 版脚本服务入口
- `app/services/update_script.py` — 管道处理后脚本字段更新（update_script_timestamps）
- `app/utils/check_script.py` — Python 版脚本格式校验（check_format 函数）
- `app/models/const.py` — 常量定义（任务状态等）
- `resource/scripts/2026-0416-112716.json` — 实际脚本 JSON 样例

### Python 版管道集成参考
- `app/services/task.py` — 核心流水线编排（start_subclip），脚本如何进入管道
- `app/services/clip_video.py` — 视频裁剪，按 OST 类型分支处理
- `app/services/voice.py` — TTS 生成，跳过 OST=1 的片段

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库、文件存储、不可变模式）
- `.planning/REQUIREMENTS.md` — Phase 5 需求：SCRP-01, SCRP-02, SCRP-03
- `.planning/ROADMAP.md` — Phase 5 定义和成功标准
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（错误处理、模块结构、serde 模式）

### 代码库映射
- `.planning/codebase/CONVENTIONS.md` — Provider/Registry 模式、编码约定
- `.planning/codebase/STRUCTURE.md` — 项目目录结构

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/models/schema.py:VideoClipParams` — Python 版脚本 clip 的 Pydantic 模型，Rust ScriptClip 结构体的对齐基准
- `app/utils/check_script.py:check_format()` — 校验规则的权威来源，Rust 版需 1:1 对齐
- `app/services/update_script.py:update_script_timestamps()` — 管道扩展字段的来源，Rust 版 Option 字段的语义参考

### Established Patterns
- **serde + 强类型（Phase 1 D-07）**：ScriptClip 遵循相同的 serde 反序列化模式
- **领域错误枚举（Phase 1 D-16/D-17）**：ScriptError 遵循 thiserror 模式
- **不可变更新（项目规则）**：编辑方法返回新实例，不修改原始数据

### Integration Points
- `src/script/mod.rs` — 新模块入口，由 `src/lib.rs` pub mod 导出
- `src/error.rs` — ScriptError 可能集成到统一错误体系，或作为 script 模块独立错误
- Phase 6 流水线将调用 `load_script()` 加载脚本、管道处理后将更新写入 Option 字段
- Phase 10 Tauri 命令层将调用编辑 API 和保存方法

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-script-management*
*Context gathered: 2026-04-28*
