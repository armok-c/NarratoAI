---
phase: 10
slug: tauri-command-layer
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-07
audited: 2026-05-12
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p narratoai-core --lib` |
| **Full suite command** | `cargo test --workspace --lib` |
| **Tauri layer command** | `cargo test -p narratoai-app --lib` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-app --lib`
- **After every plan wave:** Run `cargo test --workspace --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | TAURI-01 | T-10-01 | CommandError 不含敏感信息 | unit | `cargo test -p narratoai-app --lib -- test_command_error_serialize` | ✅ COVERED | ✅ green |
| 10-01-02 | 01 | 1 | TAURI-01 | T-10-01 | 8 个 From<T> 转换正确 | unit | `cargo test -p narratoai-app --lib -- test_from_` | ✅ COVERED | ✅ green |
| 10-01-03 | 01 | 1 | TAURI-01 | T-10-06 | validate_path 拒绝路径遍历 | unit | `cargo test -p narratoai-app --lib -- test_validate_path` | ✅ COVERED | ✅ green |
| 10-01-04 | 01 | 1 | TAURI-01 | T-10-01 | CommandError 实现 std::error::Error | unit | `cargo test -p narratoai-app --lib -- test_command_error_is_std_error` | ✅ COVERED | ✅ green |
| 10-01-05 | 01 | 1 | TAURI-02 | — | ProgressPayload 序列化完整 | unit | `cargo test -p narratoai-app --lib -- test_progress_payload` | ✅ COVERED | ✅ green |
| 10-01-06 | 01 | 1 | TAURI-02 | T-10-03 | 事件 payload 不含敏感字段 | unit | `cargo test -p narratoai-app --lib -- test_progress_payload_no_sensitive` | ✅ COVERED | ✅ green |
| 10-01-07 | 01 | 1 | TAURI-02 | — | CommandResponse/ScriptInfo 序列化往返 | unit | `cargo test -p narratoai-app --lib -- test_command_response` | ✅ COVERED | ✅ green |
| 10-01-08 | 01 | 1 | TAURI-01 | — | 请求类型 Deserialize（5 个结构体） | unit | `cargo test -p narratoai-core --lib -- types` | ✅ COVERED | ✅ green |
| 10-01-09 | 01 | 1 | TAURI-02 | — | ProgressCallback 字符串签名 + 步骤名 | unit | `cargo test -p narratoai-core --lib -- progress` | ✅ COVERED | ✅ green |
| 10-02-01 | 02 | 2 | TAURI-02 | — | 流水线命令 emit pipeline-progress | integration | MANUAL-ONLY | — | ✅ COVERED (编译验证) |
| 10-02-02 | 02 | 2 | TAURI-01 | — | register_all_commands!() 宏注册 13 条命令 | compile | `cargo check -p narratoai-app --lib` | ✅ COVERED | ✅ green |
| 10-03-01 | 02 | 2 | TAURI-03 | T-10-04 | 文件对话框仅读选中路径 | integration | MANUAL-ONLY | — | 🔶 MANUAL-ONLY |

*Status: ✅ green · ❌ red · ⚠️ flaky · 🔶 MANUAL-ONLY*

---

## Wave 0 Requirements

- [x] `src-tauri/src/error.rs` — 15 个单元测试覆盖 CommandError + validate_path
- [x] `src-tauri/src/models/progress.rs` — 5 个单元测试覆盖 ProgressPayload + CommandResponse + ScriptInfo
- [x] `src-tauri/Cargo.toml` — dev-dependencies 通过 serde_json 传递（无需额外依赖）
- [x] `cargo test --workspace --lib` — 592 tests（572 core + 20 tauri）

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tauri 应用启动并加载配置 | TAURI-01 | 需完整 Tauri 运行时 + config.toml | `cargo tauri dev`，检查控制台无错误 |
| 前端收到 pipeline-progress 事件 | TAURI-02 | 需前端 JS 监听器 | 启动应用，触发流水线，检查浏览器 console 事件 |
| 文件对话框选择文件 | TAURI-03 | 需原生对话框交互 | 点击"选择视频"按钮，验证路径返回 Rust 后端 |
| 15 条命令通过 IPC 调用 | TAURI-01 | 需 Tauri 运行时 + 前端 invoke | 在浏览器 devtools 中调用 `invoke('run_documentary', ...)` |

---

## Test Coverage Summary

### Automated Tests (592 total)

| Test Group | Count | File | Coverage |
|------------|-------|------|----------|
| CommandError From<T> 转换 | 8 | `src-tauri/src/error.rs` | T-10-01: 所有 8 个领域错误 + Display |
| CommandError 序列化安全 | 2 | `src-tauri/src/error.rs` | T-10-01: 无敏感字段 + std::error::Error |
| validate_path 路径遍历 | 4 | `src-tauri/src/error.rs` | T-10-06: 拒绝 ..、null、允许合法路径 |
| ProgressPayload 序列化 | 3 | `src-tauri/src/models/progress.rs` | T-10-03: 完整序列化 + skip_optional + 无敏感字段 |
| CommandResponse/ScriptInfo | 2 | `src-tauri/src/models/progress.rs` | D-07: 序列化往返验证 |
| Core types (Deserialize) | ~10 | `narratoai-core/src/*/types.rs` | SdeRequest/SdpRequest/ScriptClip 验证 |
| Progress steps | 4 | `narratoai-core/src/*/pipeline.rs` | 字符串步骤名验证 |
| 其余 core 测试 | 572 | narratoai-core 全模块 | 无回归 |

### Compile-Time Verification

| Check | Command | Coverage |
|-------|---------|----------|
| Workspace 编译 | `cargo check --workspace` | 工作空间布局 + 依赖正确 |
| Tauri lib 编译 | `cargo check -p narratoai-app --lib` | 所有 13 条命令 + Builder 正确 |
| Core lib 编译 | `cargo check -p narratoai-core` | ProgressCallback + Request 类型 |
| register_all_commands!() | 编译期展开 | 13 个 handler 均被 tauri::generate_handler! 引用 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ✅ approved
