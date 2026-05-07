---
phase: 07-sde-pipeline
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/pipeline.rs
  - src/sde/script_gen.rs
  - src/sde/timestamp.rs
  - src/sde/types.rs
  - src/subtitle/error.rs
  - src/subtitle/mod.rs
  - src/subtitle/parser.rs
  - src/subtitle/timestamp.rs
  - src/subtitle/types.rs
  - src/documentary/pipeline.rs
  - src/documentary/types.rs
  - src/documentary/audio.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
---

# Phase 07: 代码审查报告（第十轮）

**审查时间:** 2026-05-07T12:00:00Z
**审查深度:** standard
**文件数量:** 14
**状态:** clean（仅遗留 Info 项，无新增问题）

## 摘要

第十轮审查重点验证 Phase 08 字幕模块重构对 Phase 07 代码的影响。字幕功能从 `src/sde/subtitle.rs` 提取到独立的 `src/subtitle/` 共享模块后，SDE 模块通过 re-export 层正确引用，接口无变化。

**重构影响评估：**
- `src/sde/subtitle.rs` 已删除，解析/时间戳功能迁移至 `src/subtitle/` 模块
- `src/sde/clip.rs` 和 `src/sde/audio.rs` 已删除（前期审查确认为死代码）
- SDE 模块通过 re-export 层（`sde/timestamp.rs`, `sde/types.rs`）保持 API 兼容
- `sde/pipeline.rs` 的 import 路径从 `crate::sde::subtitle::parse_subtitle_file` 更新为 `crate::subtitle::parser::parse_subtitle_file`

**新增问题：** 无 BLOCKER 或 WARNING。2 个遗留 Info 项继续存在，均无变化。

## 审查组 A：共享字幕模块 `src/subtitle/`（5 文件）

| 文件 | 行数 | 结论 |
|------|------|------|
| error.rs | 61 | 通过 -- 错误枚举简洁，覆盖解析和 IO |
| mod.rs | 10 | 通过 -- 公共 API 导出完整 |
| types.rs | 40 | 通过 -- SubtitleSegment 结构体设计合理 |
| timestamp.rs | 217 | 通过 -- 解析和范围查找逻辑正确 |
| parser.rs | 747 | 通过 -- 编码检测链健壮，SRT/ASS 解析正确 |

### 字幕模块质量总结

- 编码检测链顺序合理：BOM > UTF-16-LE(无BOM) > UTF-8 > GBK > GB18030
- `parse_subtitle_file` 标注为 CPU 密集型，文档明确要求 `spawn_blocking` 包装
- SRT/ASS 双格式支持，解析器跳过无效块而非 panic
- `normalize_subtitle_text` 正确处理换行、BOM、NUL、毫秒分隔符
- 测试覆盖率良好：编码检测、格式解析、边界情况均有覆盖

## 审查组 B：SDE 核心模块 `src/sde/`（6 文件）

| 文件 | 行数 | 结论 |
|------|------|------|
| error.rs | 97 | 通过 -- 6 变体覆盖所有错误场景，From<SubtitleError> 映射正确 |
| mod.rs | 10 | 通过 -- 已删除 subtitle/clip/audio 模块，清理完整 |
| timestamp.rs | 2 | 通过 -- re-export 层，SDE API 兼容 |
| types.rs | 235 | 通过 -- 参数校验完整，SubtitleSegment 通过 re-export 引用 |
| script_gen.rs | 601 | 通过 -- JSON 修复策略完整，LLM 调用链正确 |
| pipeline.rs | 840 | 通过 -- 9 步编排正确，所有 async I/O 使用 tokio::fs |

### SDE 模块质量总结

- 重构后通过 re-export 层保持 API 兼容，外部调用方（如 SDP `src/sdp/clip.rs`）无需修改
- 已删除模块的引用已完全清理（`pub mod subtitle`/`clip`/`audio` 不再存在）
- 所有 `create_dir_all` 和文件写入均使用 `tokio::fs`（async）
- `parse_subtitle_file` 正确通过 `spawn_blocking` 包装
- G1/G2/G3 Guard（JSON 解析、时间戳非重叠、OST 比例）实现正确
- `run_sde` 约 610 行（编排器），结构清晰，按步骤分段

## 审查组 C：纪录片依赖 `src/documentary/`（3 文件）

| 文件 | 行数 | 结论 |
|------|------|------|
| pipeline.rs | 592 | 通过 -- 6 步流水线正确，FFmpeg 命令构建与 SDE 一致 |
| types.rs | 109 | 通过 -- 参数校验与 SDE 一致 |
| audio.rs | 275 | 通过 -- 音频/字幕合并正确，calculate_clip_duration 统一逻辑 |

### 跨模块一致性

| 检查项 | SDE | 纪录片 | 状态 |
|--------|-----|--------|------|
| SubtitleSegment 类型 | `subtitle::types` re-export | 本地 `documentary::subtitle` | 两者职责不同，不冲突 |
| subtitle_color 校验 | types.rs:89-92 | types.rs:77-80 | 一致 |
| subtitle_force_style 构建 | pipeline.rs:424-451 | pipeline.rs:300-323 | 一致 |
| concat 逻辑 | pipeline.rs:301-386 | pipeline.rs:181-269 | 一致 |
| composite 逻辑 | pipeline.rs:453-602 | pipeline.rs:325-463 | 一致 |
| create_dir_all | `tokio::fs` | `tokio::fs` | 一致 |
| 文件写入 | `tokio::fs::write` | `tokio::fs::write` | 一致 |

**类型命名说明：** 纪录片模块有独立的 `SubtitleSegment`（`{ srt_content, offset_secs }`）用于字幕合并偏移，与共享模块的 `SubtitleSegment`（`{ index, start_secs, end_secs, text }`）用途不同。Rust 类型系统确保不会误用。

## Info

### IN-01: `write_srt_file` 死代码（遗留，原 IN-03）

**文件:** `src/documentary/subtitle.rs:64`
**问题:** `pub fn write_srt_file` 标记为 `pub` 但仅在同模块的测试中使用。生产代码中无任何调用方引用此函数。WR-08 修复已将唯一生产调用方 `merge_subtitle_files` 改用 `tokio::fs::write`，因此该函数现在是完全的死代码。
**建议:** 可安全移除 `write_srt_file` 函数及其测试，或将可见性降为 `pub(crate)` 并标注 `#[deprecated]`。
**状态:** 未变化（不在本次重构影响范围内）。

### IN-02: `parse_script` 的 `_task_dir` 未使用参数（遗留，原 IN-04）

**文件:** `src/sde/script_gen.rs:302`
**问题:** `parse_script(raw_json: &str, _task_dir: &Path)` 的 `_task_dir` 参数从未在函数体中使用。这是历史 API 设计遗留，函数注释中提到"调用方负责将结果异步保存到 `task_dir/script_final.json`"，但实际上这个参数对函数行为无任何影响。
**建议:** 可在未来版本中移除此参数，让调用方自行管理路径。当前不会导致 bug，仅影响 API 清晰度。
**状态:** 未变化。

## 安全 / Unsafe / Panic 扫描

| 检查项 | 结果 |
|--------|------|
| `unsafe` 块 | 无 |
| 生产代码中的裸 `.unwrap()` | 无（仅 `LazyLock` regex 初始化，编译期安全） |
| 硬编码密钥 | 无 |
| Panic 路径 | `script_gen.rs:253` 的 `unreachable!()` 在穷尽匹配后，逻辑安全 |
| sync-in-async | 无（`parse_subtitle_file` 通过 `spawn_blocking` 包装） |
| 大函数（>50 行） | `run_sde`: ~610 行（编排器，结构清晰但偏大） |

## Phase 08 重构影响确认

以下变更已验证无回归：

1. **`src/sde/subtitle.rs` 删除** -- 功能完整迁移至 `src/subtitle/` 模块
2. **`src/sde/clip.rs` 删除** -- 确认为死代码，SDE 流水线使用 `documentary::clip`
3. **`src/sde/audio.rs` 删除** -- 确认为死代码，SDE 流水线使用 `documentary::audio`
4. **import 路径更新** -- `sde/pipeline.rs` 使用 `crate::subtitle::parser::parse_subtitle_file`
5. **re-export 层建立** -- `sde/timestamp.rs` 和 `sde/types.rs` 提供兼容性 re-export

---
_审查时间: 2026-05-07T12:00:00Z_
_审查者: Claude (agent-organizer + code-reviewer)_
_审查深度: standard_
_第十轮 -- Phase 08 字幕模块重构后验证，0 新增 BLOCKER/WARNING，2 遗留 Info 项不变_
