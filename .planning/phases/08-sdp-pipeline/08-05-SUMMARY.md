---
phase: 08-sdp-pipeline
plan: 05
type: execute
status: complete
completed: 2026-05-06T13:52:00Z
files_modified:
  - src/sdp/error.rs
  - src/sdp/clip.rs
---

## Plan 08-05: From<ScriptError> + 清除未使用导入

**目标:** 修复 Phase 8 的 BLOCKER 编译错误——为 `SdpError` 实现 `From<ScriptError>` trait，并清除 `clip.rs` 中未使用的 `SubtitleSegment` 导入。

**结果:** 2 处修改，~18 行新增代码。

### 修改详情

1. **src/sdp/error.rs** — 新增 `impl From<crate::script::error::ScriptError> for SdpError`
   - `ScriptError::JsonParse` → `SdpError::JsonRepair`
   - `ScriptError::Validation` → `SdpError::Validation`
   - `ScriptError::Io` → `SdpError::Io`
   - `ScriptError::IndexOutOfBounds` → `SdpError::Validation`
   - `ScriptError::InvalidTimestamp` → `SdpError::Validation`

2. **src/sdp/clip.rs** — 删除第 5 行未使用的 `use crate::sde::types::SubtitleSegment;`，保留测试模块内 import

### 验证结果

| 检查项 | 结果 |
|--------|------|
| `cargo build -p narratoai-core` | ✅ 编译成功，零错误 |
| `cargo test -- sdp` | ✅ 37 passed |
| `cargo test -- subtitle` | ✅ 57 passed（无回归） |
| `cargo test -- sde` | ✅ 54 passed（无回归） |
| `cargo test -- prompt::register` | ✅ 4 passed（无回归） |
| `cargo test -- documentary` | ✅ 58 passed（无回归） |

### 偏差

无。严格按 PLAN.md 中的精确代码执行。

### Self-Check: PASSED
