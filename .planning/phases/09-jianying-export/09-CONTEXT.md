# Phase 9: JianYing Export - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

将项目时间线导出为剪映草稿 JSON 格式，可在剪映专业版中打开编辑。Phase 9 只负责导出/格式转换职责——接收已处理的脚本数据（含视频/音频/字幕文件路径），生成兼容剪映的草稿文件夹。不包含 TTS 生成、视频裁剪等流水线步骤（属于 Phase 6/7/8）。

</domain>

<decisions>
## Implementation Decisions

### 剪映格式逆向策略
- **D-01:** 通过 pyJianYingDraft 源码逆向获取剪映草稿 JSON 格式规范。阅读 pyJianYingDraft 的 Python 源码，理解其内部数据结构到 JSON 的映射关系，然后在 Rust 中直接生成相同格式的 JSON。
- **D-02:** 跟随 pyJianYingDraft 最新版本，确保与当前剪映专业版兼容。
- **D-03:** 完整复制 pyJianYingDraft API——在 Rust 中实现对等的 `DraftFolder`、`VideoSegment`、`AudioSegment`、`TrackType`、`trange` 等 builder 类型。提供类似的高级 API 而非直接生成 JSON。

### 导出 API 输入设计
- **D-04:** 导出函数仅接收处理后的数据。不做 TTS 生成和视频裁剪——管道编排属于 Phase 6/7/8。导出函数接收已处理的 Script（所有 Option 字段都有值）+ 视频源路径 + 配置参数。
- **D-05:** 新建 `ExportRequest` 结构体作为导出函数的输入参数，包含 Script、视频源路径、草稿配置等。清晰明确，下游调用方一目了然。
- **D-06:** 导出时严格校验 Option 字段。检查 `video`、`audio`、`duration`、`source_time_range` 等字段是否有值，缺失则报错。确保导出完整性，不生成不完整的草稿。

### 时间线轨道映射
- **D-07:** 视频轨道 + 音频轨道双轨布局，与 Python 版一致。不加字幕轨、BGM 轨。
- **D-08:** OST 类型映射与 Python 版 1:1 对齐——OST=0（仅解说）加视频段+音频段，OST=1（仅原声）只加视频段，OST=2（混合）加视频段+音频段。
- **D-09:** 视频片段来源智能回退——有处理后的视频文件（`clip.video` 有值）则用它，没有则用原始视频路径 + `source_time_range` 截取。与 Python 版行为一致。
- **D-10:** 音频片段时长使用 ffprobe 精确获取——调用 Phase 1 已实现的 FFmpeg probe 能力获取音频文件实际时长，取 `min(音频实际时长, 视频时长)` 作为安全时长，确保不超出素材。

### 草稿元数据配置
- **D-11:** 分辨率配置化 + 默认 1080p。ExportRequest 中包含分辨率参数（宽、高），默认 1920x1080。后续可扩展支持竖屏等其他分辨率。
- **D-12:** 草稿命名采用用户指定 + 自动回退策略。用户提供草稿名称，为空则用 `NarratoAI_{timestamp}` 格式。由 ExportRequest 结构体的字段传入。
- **D-13:** 帧率使用剪映默认值（30fps），不额外配置。减少逆向工程复杂度。
- **D-14:** 草稿保存路径由调用方传入（ExportRequest 结构体的字段）。不依赖 config.toml。

### Claude's Discretion
- ExportRequest 结构体的具体字段设计（哪些必填、哪些可选）
- JianYingError 枚举的具体变体定义
- builder API 的方法命名和调用风格
- 模块内部文件拆分方式

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版剪映导出参考
- `app/services/jianying_task.py` — Python 版剪映草稿导出完整实现（start_export_jianying_draft），包含 OST 分支处理、时间线构建、音频时长策略
- pyJianYingDraft 库源码 — 逆向工程的目标，需要阅读其 DraftFolder/VideoSegment/AudioSegment/trange/TrackType 的实现

### Rust 代码库依赖
- `src/script/types.rs` — ScriptClip 数据模型，Phase 9 输入数据的核心类型
- `src/script/mod.rs` — load_script/save_script/validate 函数
- `src/ffmpeg/probe.rs` — FFmpeg probe 能力，用于获取音频时长
- `src/ffmpeg/mod.rs` — FFmpeg 模块入口
- `src/lib.rs` — 库入口，Phase 9 需添加 `pub mod jianying`
- `src/error.rs` — 错误类型基础

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库、文件存储、不可变模式）
- `.planning/REQUIREMENTS.md` — Phase 9 需求：JYNG-01, JYNG-02
- `.planning/ROADMAP.md` — Phase 9 定义和成功标准
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（模块结构、错误处理、FFmpeg 集成）
- `.planning/phases/05-script-management/05-CONTEXT.md` — Phase 5 决策（ScriptClip 模型、编辑 API、校验策略）

### 代码库映射
- `.planning/codebase/CONVENTIONS.md` — 代码风格约定
- `.planning/codebase/STRUCTURE.md` — 项目目录结构

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/ffmpeg/probe.rs` — Phase 1 已实现 ffprobe 视频探测能力，可直接复用获取音频时长
- `src/script/types.rs:ScriptClip` — Phase 5 已实现的脚本数据模型，Phase 9 导出的输入
- `src/script/types.rs:OstType` — OST 枚举（NarrationOnly/OriginalSound/Mixed），决定轨道映射逻辑
- `app/services/jianying_task.py` — Python 版导出逻辑的完整参考

### Established Patterns
- **领域模块结构（Phase 1 D-02）**：Phase 9 创建 `src/jianying/` 模块，由 `src/lib.rs` 导出
- **领域错误枚举（Phase 1 D-16/D-17）**：`JianYingError` 使用 thiserror 派生
- **中文用户可见错误信息（Phase 1 D-18）**：JianYingError 的 Display 实现用中文
- **serde + 强类型（Phase 1 D-07）**：剪映 JSON 数据结构使用 serde 序列化
- **不可变数据更新（项目规则）**：builder 模式返回新实例

### Integration Points
- `src/jianying/mod.rs` — 新模块入口，由 `src/lib.rs` pub mod 导出
- `src/ffmpeg/probe.rs` — 调用 ffprobe 获取音频时长
- `src/script/types.rs` — Script/ScriptClip 作为导出输入
- Phase 6/7/8 流水线将调用导出函数，将处理后的脚本 + 文件路径传给 Phase 9
- Phase 10 Tauri 命令层将暴露导出 API 给前端

</code_context>

<specifics>
## Specific Ideas

- Python 版 `jianying_task.py` 中音频文件路径的构造方式是 `output_dir/audio_{timestamp_formatted}.mp3`，timestamp 中的冒号替换为下划线。Rust 版应复用相同的路径逻辑，确保与 Python 版文件命名一致。
- Python 版使用 `trange(f"{current_time}s", f"{duration}s")` 的时间表示方式，Rust 版需要实现等效的时间范围表示。
- 剪映草稿文件夹结构需要通过 pyJianYingDraft 源码逆向确认——可能包含 `draft_content.json`、`draft_meta_info.json` 等多个文件。

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-jianying-export*
*Context gathered: 2026-04-29*
