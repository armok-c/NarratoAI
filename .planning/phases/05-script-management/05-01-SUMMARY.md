---
phase: 05-script-management
plan: 01
subsystem: script
tags: [serde, serde_repr, serde_json, thiserror, validation, json]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: "thiserror 错误枚举模式、serde derive 模式、模块导出约定"
provides:
  - "ScriptClip 数据模型（5 核心字段 + 6 管道扩展字段）"
  - "OstType 枚举（NarrationOnly/OriginalSound/Mixed 序列化为 0/1/2）"
  - "ScriptError 错误类型（JsonParse/Validation/Io/IndexOutOfBounds/InvalidTimestamp）"
  - "ValidationError 结构体（中文 Display 输出）"
  - "load_script / save_script / validate 公开函数"
  - "validate_timestamp 手写校验函数"
affects: [06-documentary-pipeline, 07-sde-pipeline, 08-sdp-pipeline, 10-tauri-commands]

# Tech tracking
tech-stack:
  added: [serde_repr 0.1.20]
  patterns: ["serde_repr 枚举整数序列化", "手写 timestamp 校验避免 regex", "ValidationError 收集所有错误一次性返回"]

key-files:
  created:
    - narratoai-core/src/script/types.rs
    - narratoai-core/src/script/error.rs
    - narratoai-core/src/script/mod.rs
  modified:
    - Cargo.toml
    - narratoai-core/src/lib.rs

key-decisions:
  - "OstType 字段使用 ost (snake_case) + serde rename='OST' 消除 non_snake_case 警告"
  - "validate_timestamp 手写校验替代 regex 依赖"
  - "validate 收集所有错误一次性返回，不第一个错误即停"

patterns-established:
  - "script 模块文件拆分: types.rs (数据模型) + error.rs (错误类型) + mod.rs (I/O + 校验)"
  - "serde skip_serializing_if=Option::is_none 确保保存时省略 None 字段"

requirements-completed: [SCRP-01, SCRP-03]

# Metrics
duration: 9min
completed: 2026-04-29
---

# Phase 5 Plan 01: Script Data Model & I/O Summary

**ScriptClip/OstType serde 数据模型 + load_script/save_script/validate 函数，对齐 Python 版 check_script.py 校验规则，24 个测试全部通过**

## Performance

- **Duration:** 9 min
- **Started:** 2026-04-29T01:13:31Z
- **Completed:** 2026-04-29T01:22:39Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- ScriptClip 结构体正确反序列化 Python 版 JSON 脚本（5 核心字段 + 6 管道扩展字段）
- OstType 枚举通过 serde_repr 序列化为 0/1/2 整数，与 Python 版 OST 字段兼容
- validate() 收集所有错误一次性返回，对齐 Python 版 check_script.py 的校验规则
- save_script 输出美化 JSON（indent=2），中文原样输出，Option::None 字段省略
- save + load round-trip 一致性验证通过

## Task Commits

Each task was committed atomically:

1. **Task 1: 数据模型 + 错误类型 + Cargo 依赖** - `59c8df2` (feat)
2. **Task 2: 加载/保存/校验函数** - `d4b4b69` (feat)

## Files Created/Modified
- `Cargo.toml` - 添加 serde_repr = "0.1.20" 依赖
- `src/lib.rs` - 添加 pub mod script 模块导出
- `src/script/types.rs` - ScriptClip, OstType, Script 类型别名, ValidationError
- `src/script/error.rs` - ScriptError 枚举（5 个变体，中文错误消息）
- `src/script/mod.rs` - load_script, save_script, validate 公开函数 + validate_timestamp 辅助函数

## Decisions Made
- OstType 字段名使用 `ost` (snake_case) 配合 `#[serde(rename = "OST")]` 消除 Rust non_snake_case 警告，同时保持与 Python 版 JSON 的兼容性
- validate_timestamp 采用手写字符串解析，避免引入 regex 依赖（与 RESEARCH.md 推荐一致）
- 校验错误收集策略：遍历所有 clip 收集全部错误后一次性返回 Vec<ValidationError>

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] OstType 字段名 non_snake_case 警告**
- **Found during:** Task 1 (RED 阶段编译)
- **Issue:** Rust 字段名 `OST`（全大写）触发 non_snake_case 编译警告
- **Fix:** 将 Rust 字段名改为 `ost`，添加 `#[serde(rename = "OST")]` 保持 JSON 兼容
- **Files modified:** src/script/types.rs
- **Verification:** cargo build 零警告
- **Committed in:** 59c8df2 (Task 1 commit)

**2. [Rule 3 - Blocking] error.rs 测试编译错误**
- **Found during:** Task 1 (RED 阶段)
- **Issue:** `serde_json::from_str` 返回 Result 类型赋值给非 Result 变量
- **Fix:** 添加显式 Result 类型注解
- **Files modified:** src/script/error.rs
- **Verification:** cargo test 编译通过
- **Committed in:** 59c8df2 (Task 1 commit)

**3. [Rule 3 - Blocking] mod.rs 未使用 import 警告**
- **Found during:** Task 2 (GREEN 阶段)
- **Issue:** OstType/ScriptClip 在 mod.rs 非测试代码中未使用但被导入
- **Fix:** 从生产代码 import 中移除 OstType/ScriptClip，仅在 #[cfg(test)] 块导入
- **Files modified:** src/script/mod.rs
- **Verification:** cargo build 零警告
- **Committed in:** d4b4b69 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** 所有修复均为编译正确性必要工作，无范围蔓延。

## Issues Encountered
- serde_repr 对无效 OST 整数值（如 5）自动返回反序列化错误，无需在 validate() 中额外处理——与 Python 版行为一致

## User Setup Required
None - 无外部服务配置需求。

## Next Phase Readiness
- script 模块数据模型和 I/O 函数就绪，Plan 02 (编辑 API) 可直接基于 ScriptClip 和 Script 类型实现
- load_script/save_script/validate 已公开，Phase 6 流水线可直接调用

## Self-Check: PASSED

- [x] src/script/types.rs exists
- [x] src/script/error.rs exists
- [x] src/script/mod.rs exists
- [x] .planning/phases/05-script-management/05-01-SUMMARY.md exists
- [x] Commit 59c8df2 exists
- [x] Commit d4b4b69 exists

---
*Phase: 05-script-management*
*Completed: 2026-04-29*
