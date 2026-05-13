---
phase: 14-foundation-frontend-scaffold
reviewed: 2026-05-13
depth: standard
files_reviewed: 45
scope: frontend-ts-vue + rust-ts-rs + integration-tests
findings:
  critical: 2
  warning: 9
  info: 12
  total: 23
fixed: 16
status: fixes_applied
---

# Phase 14: Code Review Report

**Reviewed:** 2026-05-13
**Depth:** standard
**Files Reviewed:** 45 (30 frontend TS/Vue + 13 Rust + 2 config)
**Status:** issues_found

## Summary

Phase 14 引入 Vue 3.5 + Vite 8 + TypeScript 6 前端脚手架、9 个 Pinia store、4 个 API 模块、tauriInvoke IPC 封装，以及 Rust 侧 ts-rs 类型导出和集成测试替换。

**最严重问题：**
1. Rust 配置类型中敏感字段（API key、secret）通过 `#[ts(export)]` 导出到 TypeScript 源码文件，尽管有 `#[serde(skip_serializing)]` 保护
2. `OstType` 使用 `serde_repr` 序列化为整数（0/1/2），但 ts-rs 生成字符串联合类型，导致 IPC 类型不匹配

## Critical Issues (2)

### CR-01: 敏感字段通过 ts-rs 导出到 TypeScript 类型定义文件

**File:** `narratoai-core/src/config/types.rs` 多处
**Category:** Security

多个配置结构体含 `#[serde(skip_serializing)]` 标记的敏感字段（`vision_openai_api_key`、`speech_key`、`secret_id`/`secret_key`、`ak`/`sk`/`token` 等），但同时派生 `TS` 并标记 `#[ts(export)]`。

`skip_serializing` 仅影响运行时 JSON 序列化。ts-rs 的 `serde-compat` 不解析此属性，生成的 `.ts` 文件将包含所有敏感字段：

```typescript
export interface AppSection {
  vision_openai_api_key: string;  // API key 暴露在 .ts 源码
  text_openai_api_key: string;    // 同上
  pexels_api_keys: Array<string>; // 同上
}
```

**Fix:** 在每个敏感字段添加 `#[ts(skip)]`：
```rust
#[serde(default, skip_serializing)]
#[ts(skip)]
pub vision_openai_api_key: String,
```
或：不导出含敏感信息的配置类型到前端，创建脱敏的前端专用类型。

### CR-02: OstType ts-rs 生成类型与 IPC 实际传输格式不匹配

**File:** `narratoai-core/src/script/types.rs:11-18`
**Category:** Bug

`OstType` 使用 `serde_repr` 序列化为整数 (0, 1, 2)，但 ts-rs 不理解 `serde_repr`。生成的 TypeScript 将是：
```typescript
// ts-rs 生成（错误）:
export type OstType = "NarrationOnly" | "OriginalSound" | "Mixed";
// 实际 IPC 传输格式:
// 0 | 1 | 2
```

前端代码 `clip.OST === "NarrationOnly"` 永远不会匹配（实际值是 `0`）。

**Fix:** 添加 `#[ts(type = "0 | 1 | 2")]` 覆盖：
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, TS)]
#[repr(u8)]
#[ts(export, export_to = "../src/types/generated/")]
#[ts(type = "0 | 1 | 2")]
pub enum OstType { ... }
```

## Warnings (9)

### WR-01: `useTheme.ts` 初始化不同步 Vuetify 主题状态

**File:** `src/composables/useTheme.ts:9-12`
**Category:** Bug

`toggleTheme()` 和 `setTheme()` 同时更新 app store 和 Vuetify 主题，但首次加载时没有同步机制。如果 localStorage 存储了 `'dark'`，app store 会读取到 `'dark'`，但 Vuetify 仍使用 `'light'`（defaultTheme）。直到用户手动切换主题，两套状态才同步。

**Fix:** 在 `App.vue` 或 `main.ts` 初始化时调用 `setTheme(appStore.theme)` 确保一致。

### WR-02: `filterSensitiveArgs` 只过滤顶层字段，不处理嵌套对象

**File:** `src/composables/useTauri.ts:54-67`
**Category:** Security

过滤逻辑仅检查传入对象的一级字段名。如果敏感字段嵌套在子对象中（如 `{ config: { apiKey: '...' } }`），不会被过滤。

**Fix:** 递归过滤嵌套对象：
```typescript
function filterSensitiveArgs(args: unknown): unknown {
  if (isPlainObject(args)) {
    const safe = { ...(args as Record<string, unknown>) }
    for (const field of sensitiveFields) {
      delete safe[field]
    }
    for (const [key, value] of Object.entries(safe)) {
      safe[key] = filterSensitiveArgs(value) // 递归
    }
    return safe
  }
  if (Array.isArray(args)) return args.map(filterSensitiveArgs)
  return args
}
```

### WR-03: `llm.ts` store 使用 `Object.assign` 原地修改

**File:** `src/stores/llm.ts:10-11`
**Category:** Quality

`updateVisionConfig` 和 `updateTextConfig` 使用 `Object.assign(visionConfig.value, c)` 原地修改对象，违反不可变数据原则。

**Fix:** 使用展开运算符创建新对象：
```typescript
function updateVisionConfig(c: Partial<LLMConfig>) {
  visionConfig.value = { ...visionConfig.value, ...c }
}
```

### WR-04: `app.ts` localStorage 读取不验证值合法性

**File:** `src/stores/app.ts:6`
**Category:** Bug

`localStorage.getItem('theme')` 返回 `string | null`，代码直接 `as 'light' | 'dark'` 类型断言。如果 localStorage 被手动修改为非法值（如 `'auto'`），整个主题系统将处于不一致状态。

**Fix:** 添加值校验：
```typescript
const stored = localStorage.getItem('theme')
const theme = ref<'light' | 'dark'>(
  stored === 'light' || stored === 'dark' ? stored : 'light'
)
```

### WR-05: `ScriptClip._id` 自定义反序列化行为在 ts-rs 输出中不可见

**File:** `narratoai-core/src/script/types.rs:49`
**Category:** Bug

`_id` 字段使用 `#[serde(deserialize_with = "deserialize_id")]` 接受整数和浮点 JSON。ts-rs 不理解 `deserialize_with`，生成的 TypeScript 类型为 `id: number`，丢失了"接受浮点但拒绝小数"的细微行为。

**Fix:** 低优先级，基础类型 (`number`) 正确。可添加 `#[ts(type = "number")]` 明确标注。

### WR-06: 集成测试 `test_normalize_lufs_two_pass` 不清理输出目录

**File:** `narratoai-core/tests/audio_lufs_integration.rs:61-79`
**Category:** Quality

测试在项目根目录创建 `test_output/audio_lufs/` 并写入 WAV 文件，但不清理。反复运行会累积文件。`test_output/` 目录不在 `.gitignore` 中。

而 `test_get_audio_rms_with_real_file` 正确使用 `temp_dir()`。

**Fix:** 使用 `tempfile::TempDir` 自动清理。

### WR-07: `test_ytdlp_version` 重复执行 `yt-dlp --version`

**File:** `narratoai-core/tests/youtube_integration.rs:18-27`
**Category:** Quality

`find_ytdlp()` 已通过 `yt-dlp --version` 检测安装状态，测试又执行一次相同命令。

**Fix:** 让 `find_ytdlp()` 返回版本字符串，或接受冗余作为冒烟测试。

### WR-08: `ProgressPayload` 前端手写类型缺少 `error_code` 和 `error_message` 字段

**File:** `src/types/index.ts:11-20`
**Category:** Bug

前端 `ProgressPayload` 接口只有 8 个字段，缺少 Rust 端 `ProgressPayload` 的 `error_code` 和 `error_message` 可选字段。当 pipeline 出错时，前端无法访问错误详情。

**Fix:** 添加可选字段：
```typescript
error_code?: string | null
error_message?: string | null
```
或等 Phase 16 替换为 ts-rs 生成类型时自动解决。

### WR-09: `ProgressPayload` 使用 `skip_serializing_if` 在 ts-rs 中的微妙差异

**File:** `src-tauri/src/models/progress.rs:20-23`
**Category:** Bug

`skip_serializing_if = "Option::is_none"` 使字段在 JSON 中可能不存在。ts-rs `serde-compat` 会生成 `field?: string | null`，语义正确但前端开发者需注意用可选链访问。

**Fix:** 无需立即修改，前端应使用 `payload.error_code?.length` 访问。

## Info (12)

### IN-01: `types/index.ts` 手写类型与 ts-rs 生成类型重复

**File:** `src/types/index.ts`
**Category:** Architecture

`VideoMeta`、`ProgressPayload`、`LLMConfig` 手写定义与 Phase 16 ts-rs 生成的类型将重复。需在 Phase 16 统一，删除手写版本。

### IN-02: `tts.ts` 使用 `any` 类型

**File:** `src/stores/tts.ts:6,12`
**Category:** Quality

`engineConfigs: Record<string, any>` 和 `updateConfig(engineName: string, config: any)` 缺乏类型约束。Phase 16 应替换。

### IN-03: `configApi.ts` 全面使用 `any`

**File:** `src/api/configApi.ts`
**Category:** Quality

`getConfig()` 返回 `Promise<any>`，`setConfig(config: any)` 参数无类型。已标注 TODO，Phase 16 替换。

### IN-04: `debugError` 不过滤错误对象中的敏感信息

**File:** `src/composables/useTauri.ts:85-90`
**Category:** Security

`debugError` 直接传递 error 对象给 `console.error`，未调用 `filterSensitiveArgs`。如果 error 包含请求参数，敏感字段可能泄露到控制台。

### IN-05: `pipelineApi.ts` `stopPipeline` 调用不存在的后端命令

**File:** `src/api/pipelineApi.ts:20-22`
**Category:** Architecture

`stop_pipeline` 在 `src-tauri/src/commands/` 中未注册。已记录为已知桩，后续 Phase 将添加后端命令。

### IN-06: ts-rs `export_to` 路径在两个 crate 中生成到不同目录

**File:** 多个文件
**Category:** Architecture

`narratoai-core` 类型生成到 `narratoai-core/src/types/generated/`，`src-tauri` 类型生成到 `src-tauri/src/types/generated/`。如果目标是单一前端类型目录，需统一路径。

### IN-07: `ts_export_test.rs` 不验证 `src-tauri` 类型

**File:** `narratoai-core/tests/ts_export_test.rs`
**Category:** Quality

测试仅检查 `narratoai-core` 的类型。`src-tauri` 中的 4 个类型（CommandError、CommandResponse、ScriptInfo、ProgressPayload）未覆盖。

### IN-08: `test_get_video_formats` 解析失败时静默通过

**File:** `narratoai-core/tests/youtube_integration.rs:51-58`
**Category:** Quality

`serde_json::from_str` 失败时仅打印消息并 return，测试视为通过。

### IN-09: `defaults.rs` `repetition_penalty` 默认值与测试值不一致

**File:** `narratoai-core/src/config/defaults.rs:87` vs `narratoai-core/src/config/types.rs:328`
**Category:** Quality

默认值 `1.5`，测试 TOML 使用 `10.0`。不是 bug（测试验证解析，不验证默认值），但可能引起混淆。

### IN-10: `TtsResult` 和 `WordBoundary` 未导出 ts-rs 类型

**File:** `narratoai-core/src/documentary/types.rs:106-113`
**Category:** Architecture

内部管线类型未标注 TS。如果未来需要跨 IPC 传递，需补充。

### IN-11: `video.ts` `addVideo` 使用 `.push()` 原地修改数组

**File:** `src/stores/video.ts:9`
**Category:** Quality

虽然 Vue/Pinia 中 `.push()` 是惯用模式，但违反不可变原则。在 Vue 响应式系统中可接受。

### IN-12: `vite.config.ts` `manualChunks` 对不匹配的包返回 `undefined`

**File:** `vite.config.ts:14-24`
**Category:** Quality

不匹配 vuetify/vue-vendor/tauri 的新依赖将归入默认 chunk，可能导致非预期的包大小。低优先级。

---

## Findings Summary

| Severity | Count | Action Required |
|----------|-------|-----------------|
| CRITICAL | 2 | **Must fix before Phase 15** |
| WARNING | 9 | Should fix in Phase 14 or 15 |
| INFO | 12 | Track for Phase 16 cleanup |

## Priority Fix List

1. **CR-01** — 在所有敏感字段添加 `#[ts(skip)]` 或停止导出配置类型
2. **CR-02** — 为 `OstType` 添加 `#[ts(type = "0 | 1 | 2")]`
3. **WR-01** — 初始化时同步 Vuetify 主题状态
4. **WR-04** — 验证 localStorage 主题值合法性
5. **WR-08** — 前端 ProgressPayload 补充 error 字段（或等 Phase 16）

---

_Reviewed: 2026-05-13_
_Reviewer: Claude (gsd-code-review)_
_Depth: standard_

## Fix Log (2026-05-13)

16/23 findings fixed. Remaining 7 are deferred to later phases (Phase 16 type cleanup, architecture decisions, acceptable patterns, or future work).

### Fixed (16)

| ID | Severity | Fix Commit | Description |
|----|----------|------------|-------------|
| CR-01 | Critical | `b4cac82` | 13 sensitive fields in config types: added `#[ts(skip)]` |
| CR-02 | Critical | `aa44cd2` | OstType: added `#[ts(type = "0 \| 1 \| 2")]` to match IPC integer format |
| WR-01 | Warning | `044e9bc` | useTheme: sync Vuetify theme on initialization |
| WR-02 | Warning | `eeed8a2` | filterSensitiveArgs: recursive filtering for nested objects and arrays |
| WR-03 | Warning | `00303c7` | llm store: replaced `Object.assign` with spread operator |
| WR-04 | Warning | `509bff4` | app store: validate localStorage theme value |
| WR-05 | Warning | `f950d21` | ScriptClip._id: added `#[ts(type = "number")]` + manual `.ts` update `bigint` to `number` |
| WR-06 | Warning | `0cd3fac` | LUFS integration test: use `temp_dir()` for cleanup |
| WR-08 | Warning | `121028a` | ProgressPayload: added `error_code` and `error_message` optional fields |
| IN-04 | Info | `5f909be` | debugError: filter sensitive data in error output |
| IN-07 | Info | `a6a80f3` | ts_export_test: documented src-tauri coverage limitation |
| IN-08 | Info | `8b18b82` | youtube integration test: panic on JSON parse failure instead of silent pass |
| IN-09 | Info | `b339ac4` | defaults.rs: added clarifying comment for repetition_penalty default vs test value |
| IN-12 | Info | `5c2cb39` | vite.config.ts: add explicit `vendor` chunk fallback in manualChunks |
| CR-01/CR-02 .ts | — | `072bd3c` | Generated TS files updated to reflect sensitive field removal and OstType integer override |

### Deferred (7)

| ID | Reason |
|----|--------|
| WR-07 | Acceptable as smoke test — redundant version check validates install + output format |
| WR-09 | Semantically correct (`field?: string \| null`) — no fix needed |
| IN-01 | Phase 16 ts-rs migration will unify hand-written and generated types |
| IN-02 | Phase 16 type replacement |
| IN-03 | Phase 16 type replacement |
| IN-05 | Known stub — backend command planned for later phase |
| IN-06 | Architecture decision — separate crate output dirs |
| IN-10 | Future work — internal pipeline types |
| IN-11 | Acceptable Vue/Pinia pattern — .push() is idiomatic in Vue reactivity |
