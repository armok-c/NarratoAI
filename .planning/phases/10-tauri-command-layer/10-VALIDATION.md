---
phase: 10
slug: tauri-command-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-07
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p narratoai-core` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-tauri --lib`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | TAURI-01 | T-10-01 / — | CommandError 不含敏感信息 | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | TAURI-01 | T-10-02 / — | Config State 不泄露 API key | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 2 | TAURI-02 | T-10-03 / — | 事件 payload 不含 API key | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 10-03-01 | 03 | 2 | TAURI-03 | T-10-04 / — | 文件对话框仅读选中路径 | integration | `cargo test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/tests/` — integration test stubs for Tauri commands
- [ ] `src-tauri/Cargo.toml` — dev-dependencies (tauri test utils)
- [ ] `cargo test --workspace` — verify existing core tests still pass

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tauri 应用启动并加载配置 | TAURI-01 | 需完整 Tauri 运行时 | `cargo tauri dev`，检查控制台无错误 |
| 前端收到 pipeline-progress 事件 | TAURI-02 | 需前端 JS 监听器 | 启动应用，触发流水线，检查浏览器 console 事件 |
| 文件对话框选择文件 | TAURI-03 | 需原生对话框交互 | 点击"选择视频"按钮，验证路径返回 Rust 后端 |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
