---
phase: 07-sde-pipeline
reviewed: 2026-05-06T17:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/pipeline.rs
  - src/sde/script_gen.rs
  - src/sde/subtitle.rs
  - src/sde/timestamp.rs
  - src/sde/types.rs
  - src/documentary/pipeline.rs
  - src/documentary/types.rs
  - src/documentary/audio.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 07: 代码审查报告（第九轮）

**审查时间:** 2026-05-06T17:00:00Z
**审查深度:** standard
**文件数量:** 10
**状态:** issues_found（仅遗留 Info 项）

## 摘要

第九轮审查重点验证第八轮发现的 2 个 Warning（WR-08、WR-09）的修复情况，并检查是否引入新问题。

**修复验证结果：**
- **WR-08 已修复:** `src/documentary/audio.rs:170` 现在使用 `tokio::fs::write` 替代同步 `write_srt_file`
- **WR-09 已修复:** 所有 4 处 `create_dir_all` 调用已替换为 `tokio::fs::create_dir_all`：
  - `src/sde/pipeline.rs:52` -- 主流水线入口
  - `src/sde/pipeline.rs:701` -- `analyze_subtitle_plot` 独立 API
  - `src/sde/pipeline.rs:734` -- `generate_sde_script` 独立 API
  - `src/documentary/pipeline.rs:484` -- 纪录片流水线入口

**回归检查：**
- 所有修复的错误处理（`.map_err()`）和类型转换正确，无回归
- 未发现新的 sync-in-async 模式（`parse_subtitle_file` 中的 `fs::read` 已通过 `spawn_blocking` 正确包装）
- `write_srt_file`（`src/documentary/subtitle.rs:64`）现在仅被测试代码引用，不再是生产代码路径

**新增问题：** 无 BLOCKER 或 WARNING。2 个 Info 级遗留项继续存在。

## Info

### IN-01: `write_srt_file` 死代码（遗留，原 IN-03）

**文件:** `src/documentary/subtitle.rs:64`
**问题:** `pub fn write_srt_file` 标记为 `pub` 但仅在同模块的测试中使用。生产代码中无任何调用方引用此函数。WR-08 修复已将唯一生产调用方 `merge_subtitle_files` 改用 `tokio::fs::write`，因此该函数现在是完全的死代码。
**建议:** 可安全移除 `write_srt_file` 函数及其测试，或将可见性降为 `pub(crate)` 并标注 `#[deprecated]`。

### IN-02: `parse_script` 的 `_task_dir` 未使用参数（遗留，原 IN-04）

**文件:** `src/sde/script_gen.rs:302`
**问题:** `parse_script(raw_json: &str, _task_dir: &Path)` 的 `_task_dir` 参数从未在函数体中使用。这是历史 API 设计遗留，函数注释中提到"调用方负责将结果异步保存到 `task_dir/script_final.json`"，但实际上这个参数对函数行为无任何影响。
**建议:** 可在未来版本中移除此参数，让调用方自行管理路径。当前不会导致 bug，仅影响 API 清晰度。

## 跨流水线一致性（SDE vs 纪录片）

| 检查项 | SDE | 纪录片 | 状态 |
|--------|-----|--------|------|
| subtitle_color 校验 | types.rs:89-92 | types.rs:77-80 | 一致 |
| TTS 循环中的写入 | tokio::fs::write:184 | tokio::fs::write:78 | 一致 |
| concat 逻辑 | pipeline.rs:301-386 | pipeline.rs:182-269 | 一致 |
| composite 逻辑 | pipeline.rs:453-602 | pipeline.rs:325-463 | 一致 |
| 字幕合并写入 | tokio::fs::write（已修复） | tokio::fs::write（已修复） | 一致 |
| create_dir_all | tokio::fs（已修复） | tokio::fs（已修复） | 一致 |

## 安全 / Unsafe / Panic 扫描

| 检查项 | 结果 |
|--------|------|
| `unsafe` 块 | 无 |
| 生产代码中的裸 `.unwrap()` | 无（仅 `LazyLock` regex 初始化，编译期安全） |
| 硬编码密钥 | 无 |
| Panic 路径 | `script_gen.rs:253` 的 `unreachable!()` 在穷尽匹配后，逻辑安全 |
| 大函数（>50 行） | `run_sde`: ~580 行（编排器，结构清晰但偏大） |

---

_审查时间: 2026-05-06T17:00:00Z_
_审查者: Claude (gsd-code-reviewer)_
_审查深度: standard_
_第九轮 -- 第八轮 2 个 Warning 全部修复确认，无新增 BLOCKER/WARNING_
