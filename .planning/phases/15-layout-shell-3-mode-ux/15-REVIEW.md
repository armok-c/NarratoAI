---
phase: 15-layout-shell-3-mode-ux
reviewed: 2026-05-13T12:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - src/components/AppHeader.vue
  - src/components/WindowControls.vue
  - src/components/SettingSection.vue
  - src/components/SettingsDrawer.vue
  - src/components/SystemMonitorBar.vue
  - src/layouts/DefaultLayout.vue
  - src/views/HomeView.vue
  - src/stores/mode.ts
  - src/composables/useTauri.ts
  - src/composables/useTheme.ts
  - src/stores/app.ts
  - src/stores/pipeline.ts
  - vitest.config.ts
  - src/components/__tests__/AppHeader.test.ts
  - src/components/__tests__/WindowControls.test.ts
  - src/components/__tests__/SettingSection.test.ts
  - src/components/__tests__/SettingsDrawer.test.ts
  - src-tauri/src/commands/system.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
findings:
  critical: 2
  warning: 8
  info: 7
  total: 17
status: issues_found
---

# Phase 15: Code Review Report (Full Stack -- Frontend + Rust Backend)

**Reviewed:** 2026-05-13T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20 (17 frontend + 3 Rust backend)
**Status:** issues_found

## Summary

审查了 Phase 15 的 20 个文件：7 个 Vue 组件、3 个 Pinia store、2 个 composable、1 个配置文件、4 个前端测试 stub、以及 3 个 Rust 后端文件（`system.rs`、`commands/mod.rs`、`lib.rs`）。

发现 2 个 Critical、8 个 Warning、7 个 Info 级别问题。

**Critical #1（前端）:** `useTauri.ts` 使用顶层静态 import 加载 `@tauri-apps/api/core`，与项目中所有其他 Tauri API 调用使用动态 `import()` 的模式不一致。在非 Tauri 环境（浏览器 dev mode）中会导致整个 JS bundle 加载失败。

**Critical #2（后端）:** `get_system_stats` 是项目中唯一返回 `Result<..., String>` 的 Tauri 命令，其他 14 条命令统一使用 `CommandError`。这导致错误响应的 JSON 格式不一致（`String` 序列化为裸字符串，`CommandError` 序列化为 `{code, message}` 对象）。

---

## Critical Issues

### CR-01: Static import of Tauri API crashes app in browser dev mode

**File:** `src/composables/useTauri.ts:1` + `src/components/SystemMonitorBar.vue:20`
**Issue:** `useTauri.ts` performs a **top-level static import** of `@tauri-apps/api/core`:
```typescript
import { invoke } from '@tauri-apps/api/core'
```
This is fundamentally inconsistent with the pattern used everywhere else in Phase 15 -- all other Tauri API calls use dynamic `import()` with try/catch:
- `AppHeader.vue:34`: `await import('@tauri-apps/api/window')`
- `SettingsDrawer.vue:74`: `await import('@tauri-apps/api/app')`
- `DefaultLayout.vue:41,50,59,70,79`: `await import('@tauri-apps/api/window')`

When the app runs in a plain browser (e.g., `vite dev` without the Tauri shell), the `@tauri-apps/api/core` module will attempt to initialize the Tauri IPC bridge. If it fails at import time, the entire JavaScript bundle fails to load -- no component can render. Even if the import succeeds at module level, every subsequent `tauriInvoke()` call to `get_system_stats` will throw, and `SystemMonitorBar` will endlessly cycle through error state every 2 seconds.

**Fix:** Change `useTauri.ts` to use dynamic import of `invoke`:
```typescript
// Replace static import:
// import { invoke } from '@tauri-apps/api/core'

export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    debugLog(`[tauriInvoke] 调用命令: ${command}`, args || '')
    const normalizedArgs = normalizeTauriArgs(args)
    const result = await invoke<T>(command, normalizedArgs as Record<string, unknown> | undefined)
    debugLog(`[tauriInvoke] 命令 ${command} 成功`, result)
    return result
  } catch (error) {
    debugError(`[tauriInvoke] 命令 ${command} 失败`, error)
    throw error
  }
}
```

### CR-02: get_system_stats 错误类型 String 与项目 CommandError 不一致

**File:** `src-tauri/src/commands/system.rs:34`
**Issue:** `get_system_stats` 是唯一一个返回 `Result<SystemStats, String>` 的命令。项目中其他所有 14 条 Tauri 命令（pipeline 模块的 5 条、script 模块的 6 条、export 模块的 1 条）统一返回 `Result<..., CommandError>`。Tauri 序列化 `String` 时产生的 JSON 结构为裸字符串，而 `CommandError` 序列化为 `{ "code": "...", "message": "..." }` 对象。这违反了项目 D-18 统一错误格式的约定。

前端 `SystemMonitorBar.vue:33` 的 `catch` 块虽然没有解析错误结构体，但任何未来添加的错误 code 解析逻辑都会因为格式不一致而失败。

**Fix:**
```rust
use crate::error::CommandError;

#[tauri::command]
pub async fn get_system_stats(state: State<'_, SystemState>) -> Result<SystemStats, CommandError> {
    let mut system = state
        .0
        .lock()
        .map_err(|e| CommandError {
            code: "INTERNAL_ERROR".into(),
            message: format!("无法锁定系统状态: {}", e),
        })?;

    system.refresh_all();

    let cpu_percent = system.global_cpu_usage() as f64;

    let total_mem = system.total_memory();
    let ram_percent = if total_mem > 0 {
        (system.used_memory() as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };

    Ok(SystemStats {
        cpu_percent,
        ram_percent,
    })
}
```

---

## Warnings

### WR-01: Window maximized state goes stale after user resizes window

**File:** `src/layouts/DefaultLayout.vue:30,42`
**Issue:** `isMaximized` is only set in `onMounted()` and after `toggleMaximize()`. If the user maximizes via the OS title bar, double-clicks the title bar natively, or uses a keyboard shortcut (Alt+Space, then Maximize), the `isMaximized` ref will never update. The `v-btn` maximize/restore icon in `WindowControls` will show the wrong icon state indefinitely.

**Fix:** Listen for Tauri's `onResized` event to keep `isMaximized` in sync:
```typescript
onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    isMaximized.value = await win.isMaximized()
    unlisten = await win.onResized(async () => {
      isMaximized.value = await win.isMaximized()
    })
  } catch {
    // Non-Tauri environment
  }
})
```

### WR-02: mode.ts directly imports and instantiates usePipelineStore -- cross-store coupling

**File:** `src/stores/mode.ts:3,21`
**Issue:** `mode.ts` directly imports `usePipelineStore` and calls `pipelineStore.reset()` inside `resetPipeline()`. This creates tight coupling between the mode store and the pipeline store. If `usePipelineStore()` is called before the Pinia plugin is installed, it throws. The mode store cannot be tested in isolation.

**Fix:** Remove cross-store import. Let the caller (e.g., `HomeView.vue`) handle pipeline reset:
```typescript
// In HomeView.vue:
function handleModeChange(val: WorkMode) {
  modeStore.setMode(val)
  usePipelineStore().reset()
}
```

### WR-03: v-main min-height 100vh causes vertical scrollbar with app-bar + footer

**File:** `src/layouts/DefaultLayout.vue:11`
**Issue:** `v-main` has `style="min-height: 100vh"`. Combined with `v-app-bar` (40px) and `v-footer` (28px), total content height is 100vh + 68px, creating a permanent vertical scrollbar. Vuetify's `v-layout` already manages `v-main` padding for app-bar and footer.

**Fix:** Remove inline `min-height: 100vh` from `v-main`.

### WR-04: handleMaximize and handleToggleMaximize are identical functions

**File:** `src/layouts/DefaultLayout.vue:57-66` and `src/layouts/DefaultLayout.vue:77-86`
**Issue:** `handleMaximize()` and `handleToggleMaximize()` have identical implementations -- both call `win.toggleMaximize()` and then update `isMaximized`. Dead code duplication.

**Fix:** Consolidate into a single function and use it for both events.

### WR-05: SystemMonitorBar silently swallows all errors without logging

**File:** `src/components/SystemMonitorBar.vue:33-35`
**Issue:** The `catch` block in `fetchStats()` sets `error.value = true` but does not log the error. Makes debugging production issues difficult.

**Fix:** Add at minimum a `console.warn` in development mode.

### WR-06: async 函数持有 std::sync::Mutex -- 维护风险

**File:** `src-tauri/src/commands/system.rs:34-55`
**Issue:** `get_system_stats` 声明为 `async fn`，但在整个函数体内持有 `std::sync::Mutex` 的 `MutexGuard`。虽然当前没有 `.await` 点，但将函数标记为 `async` 意味着未来维护者可能添加 `.await` 调用而忘记改用 tokio::sync::Mutex。注释（第 18-19 行）正确解释了选择 std::sync::Mutex 的理由，但未显式标注 "不得引入 .await"。

**Fix:** 改为同步函数（推荐，因为无 .await），或在注释中增加 WARNING 标记此函数不得在锁持有期间引入 .await 点：
```rust
// 方案 A: 改为同步命令
#[tauri::command]
pub fn get_system_stats(state: State<'_, SystemState>) -> Result<SystemStats, CommandError> {
    // ...
}
```

### WR-07: CPU 使用率首次调用返回 0 -- sysinfo delta 问题

**File:** `src-tauri/src/commands/system.rs:40-42` + `src-tauri/src/lib.rs:76`
**Issue:** `sysinfo` 0.33.1 的 `global_cpu_usage()` 依赖两次 `refresh` 调用之间的 delta 计算。`System::new_all()` 初始化时没有基准时间戳，因此第一次调用 `refresh_all()` + `global_cpu_usage()` 返回 `0.0`。应用启动后前端 SystemMonitorBar 的第一次 fetch 显示 `CPU 0%`，约 2 秒后才显示真实值。

**Fix:** 在 `lib.rs` 的 setup 中初始化后立即调用一次 `refresh_all()` 建立基准：
```rust
let mut init_system = System::new_all();
init_system.refresh_all(); // 建立首次基准
app.manage(SystemState(Mutex::new(init_system)));
```

### WR-08: SystemState pub newtype 暴露内部实现

**File:** `src-tauri/src/commands/system.rs:21`
**Issue:** `SystemState(pub Mutex<System>)` 使用 pub tuple struct 字段，直接暴露内部 `Mutex<System>` 的实现细节。更换锁策略时需要修改所有 `.0` 访问点。

**Fix:** 使用私有字段 + 显式方法：
```rust
pub struct SystemState {
    inner: Mutex<System>,
}

impl SystemState {
    pub fn new(system: System) -> Self {
        Self { inner: Mutex::new(system) }
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, System>, std::sync::PoisonError<std::sync::MutexGuard<'_, System>>> {
        self.inner.lock()
    }
}
```

---

## Info

### IN-01: All 8 test files are empty stubs with no real assertions

**Files:** All files in `src/**/__tests__/*.test.ts`
**Issue:** Every test file contains a single placeholder test with `expect(true).toBe(true)`. Zero coverage. Exist as scaffolding.

**Fix:** Replace with actual tests when Phase 15 is considered complete.

### IN-02: HomeView has redundant watch for modeStore.currentMode

**File:** `src/views/HomeView.vue:79-81`
**Issue:** `selectedMode` is already updated by `v-model` on `v-select`. The watch on `modeStore.currentMode` sets it again redundantly. Only useful if `currentMode` could change from outside `HomeView`.

**Fix:** Harmless. Keep if external mode changes are planned.

### IN-03: vitest.config.ts uses node environment for Vue component tests

**File:** `vitest.config.ts:6`
**Issue:** `environment: 'node'` means Vue component tests cannot access browser APIs. Will need `jsdom` or `happy-dom` when real tests are added.

**Fix:** Change to `environment: 'jsdom'` when real component tests are added.

### IN-04: AppHeader dark mode CSS uses fragile selector

**File:** `src/components/AppHeader.vue:122-125`
**Issue:** `:root[class*="dark"]` selector assumes dark class on `<html>`. Vuetify 4 applies theme classes to `<html>`, so it works, but `.v-theme--dark` alone would suffice.

**Fix:** Simplify to `.v-theme--dark .logo-glow { ... }`.

### IN-05: test_ram_percent_zero_total 复制生产逻辑而非测试函数行为

**File:** `src-tauri/src/commands/system.rs:83-96`
**Issue:** 测试复制了生产代码的计算逻辑（`if total_mem > 0 { ... } else { 0.0 }`），但没有调用 `get_system_stats` 函数本身。这种测试验证的是"复制的逻辑和原逻辑一致"而非"函数行为正确"。

**Fix:** 提取纯函数 `calc_ram_percent(used, total) -> f64` 并单独测试，或直接构造 `SystemState` 调用 `get_system_stats`。

### IN-06: register_all_commands! 宏中列表与 pub mod / pub use 列表无结构性保障

**File:** `src-tauri/src/commands/mod.rs:1-9 vs 17-36`
**Issue:** 文件顶部手动维护 `pub mod` / `pub use` 列表，底部宏中手动维护命令全路径列表。添加新命令时需同时修改两处，无编译期检查确保同步。

**Fix:** 这是 Tauri 生态的常见模式，无完美解决方案。可在宏注释中添加 checklist 提醒。

### IN-07: get_system_stats 缺少服务端速率限制

**File:** `src-tauri/src/commands/system.rs:34`
**Issue:** 无服务端速率限制。虽然当前仅 SystemMonitorBar 每 2 秒调用一次，但任何前端代码均可高频调用。`refresh_all()` 在某些平台有系统调用开销。风险较低，记录为 Info。

**Fix:** 如果将来有性能问题，在 Tauri 中间件层添加 rate limit。

---

## Summary Table

| ID | Severity | File | Line(s) | Description |
|----|----------|------|---------|-------------|
| CR-01 | Critical | useTauri.ts | 1 | 静态 import Tauri API 在浏览器 dev mode 崩溃 |
| CR-02 | Critical | system.rs | 34 | 错误类型 String 与项目 CommandError 不一致 |
| WR-01 | Warning | DefaultLayout.vue | 30,42 | 最大化状态在 OS 级操作后过时 |
| WR-02 | Warning | mode.ts | 3,21 | 跨 store 耦合（直接 import usePipelineStore） |
| WR-03 | Warning | DefaultLayout.vue | 11 | min-height: 100vh 导致永久滚动条 |
| WR-04 | Warning | DefaultLayout.vue | 57-66, 77-86 | handleMaximize 与 handleToggleMaximize 完全相同 |
| WR-05 | Warning | SystemMonitorBar.vue | 33-35 | 错误被静默吞没无日志 |
| WR-06 | Warning | system.rs | 34-55 | async 函数持有 std::sync::Mutex，维护风险 |
| WR-07 | Warning | system.rs + lib.rs | 40, 76 | CPU 首次调用返回 0（sysinfo delta 问题） |
| WR-08 | Warning | system.rs | 21 | pub newtype 暴露内部实现 |
| IN-01 | Info | __tests__/*.test.ts | all | 8 个测试文件为空 stub |
| IN-02 | Info | HomeView.vue | 79-81 | watch on currentMode 冗余 |
| IN-03 | Info | vitest.config.ts | 6 | node 环境无法测试 DOM |
| IN-04 | Info | AppHeader.vue | 122-125 | dark mode CSS 选择器可简化 |
| IN-05 | Info | system.rs | 83-96 | 测试复制生产逻辑 |
| IN-06 | Info | commands/mod.rs | 1-36 | mod/use 与宏列表无结构性同步 |
| IN-07 | Info | system.rs | 34 | 无服务端速率限制 |

**Critical: 2 | Warning: 8 | Info: 7 | Total: 17**

最需要关注的问题：
1. **CR-01**（静态 import）应在合并前修复，否则浏览器 dev mode 完全不可用。
2. **CR-02**（错误类型不一致）应在合并前修复，修复简单（替换 String 为 CommandError + map_err）。
3. **WR-06/WR-07**（async + Mutex / CPU 首次返回 0）建议在本 phase 或下个 phase 修复。

---

## Fix Log (--fix --all applied 2026-05-13)

| ID | Severity | Fix Applied | Files Changed |
|----|----------|-------------|---------------|
| CR-01 | Critical | Static import → dynamic import `await import('@tauri-apps/api/core')` | `src/composables/useTauri.ts` |
| CR-02 | Critical | `Result<..., String>` → `Result<..., CommandError>` + `map_err` | `src-tauri/src/commands/system.rs` |
| WR-01 | Warning | Added `onResized` listener + `onUnmounted` cleanup | `src/layouts/DefaultLayout.vue` |
| WR-02 | Warning | Removed cross-store coupling; pipeline reset moved to HomeView caller | `src/stores/mode.ts`, `src/views/HomeView.vue` |
| WR-03 | Warning | Removed `min-height: 100vh` from `v-main` | `src/layouts/DefaultLayout.vue` |
| WR-04 | Warning | Consolidated `handleMaximize` → `handleToggleMaximize` (single function) | `src/layouts/DefaultLayout.vue` |
| WR-05 | Warning | Added `console.warn` in dev mode for fetchStats errors | `src/components/SystemMonitorBar.vue` |
| WR-06 | Warning | Changed `async fn` → `fn` (no .await points) | `src-tauri/src/commands/system.rs` |
| WR-07 | Warning | Added initial `refresh_all()` in setup for CPU baseline | `src-tauri/src/lib.rs` |
| WR-08 | Warning | Encapsulated `SystemState` with private field + `new()`/`lock()` methods | `src-tauri/src/commands/system.rs`, `src-tauri/src/lib.rs` |
| IN-02 | Info | Removed redundant `watch` on `modeStore.currentMode` | `src/views/HomeView.vue` |
| IN-03 | Info | Changed vitest environment from `node` to `jsdom` | `vitest.config.ts` |
| IN-04 | Info | Simplified dark mode CSS to `.v-theme--dark` only | `src/components/AppHeader.vue` |
| IN-05 | Info | Extracted `calc_ram_percent()` pure function + proper test | `src-tauri/src/commands/system.rs` |
| IN-06 | Info | Added sync checklist comment to macro doc | `src-tauri/src/commands/mod.rs` |
| IN-07 | Info | No code change — documented as future consideration if perf issues arise | — |

**Not fixed (by design):**
- IN-01: Test stubs are scaffolding for future phases, not replaced

**Verification:**
- Rust: `cargo check` passes (0 new warnings, 2 pre-existing)
- TypeScript: Pre-existing environment issues (missing node_modules) — no new type errors introduced

---

_Reviewed: 2026-05-13T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Fix applied: 2026-05-13_
_Depth: standard_
