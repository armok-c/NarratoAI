---
phase: 01-foundation
plan: 01
type: execute
wave: 1
subsystem: core
tags: [config, error, rust-skeleton, tdd]
requires: []
provides: [narratoai-core-lib, config-system, error-types]
affects: [CONF-01, CONF-02, CONF-03]
key-files:
  created:
    - Cargo.toml
    - src/lib.rs
    - src/error.rs
    - src/config/types.rs
    - src/config/defaults.rs
    - src/config/mod.rs
    - src/config/watcher.rs
    - Cargo.lock
  modified:
    - .gitignore
decisions:
  - "ffmpeg-sidecar feature is download_ffmpeg, not auto-download (Research.md had wrong name)"
  - "AppConfig::validate() placeholder in mod.rs (not types.rs) to avoid duplicate impl blocks"
  - "Cargo.lock committed for reproducible builds"
  - "target/ added to .gitignore"
metrics:
  duration: "~12 minutes"
  tasks: 2
  commits: 5
  tests-added: 18
  completed-date: "2026-04-28"
git-log:
  - 887afd2 chore(01-01): add Cargo.lock and gitignore Rust build artifacts
  - 085e082 feat(01-01): implement config system with defaults, loading, and hot-reload
  - 517d473 test(01-01): add failing tests for config system
  - 2cf1403 feat(01-01): implement library skeleton
  - 527e351 test(01-01): add failing tests for library skeleton
---

# Phase 1 Plan 1: Rust 库骨架 + 配置系统

创建 narratoai-core lib crate 骨架和完整的 TOML 配置系统：10 个配置段的反序列化、默认值填充、文件加载、notify 热加载。

## 执行摘要

按计划完成 2 个 TDD 任务，共 5 次提交（2 RED + 2 GREEN + 1 chore），18 个测试全部通过，cargo build 零警告。

## 文件结构

```
Cargo.toml                — package name: narratoai-core, v0.1.0, edition 2021
Cargo.lock                — 依赖版本锁定（148 个 crate）
src/
  lib.rs                  — 库入口：pub mod config, error; version()
  error.rs                — ConfigError (5 variants) + FFmpegError (5 variants)
  config/
    mod.rs                — ConfigManager (load/get/start_watching/config)
    types.rs              — AppConfig + 10 subsection structs
    defaults.rs           — Default trait impls for all 11 structs
    watcher.rs            — ConfigWatcher via notify 9.x hot-reload
.gitignore                — added target/ for Rust build artifacts
```

## 配置段对应

| TOML Section  | Rust Struct      | 字段数 |
|---------------|------------------|--------|
| [app]         | AppSection       | 13     |
| [ui]          | UiSection        | 11     |
| [azure]       | AzureSection     | 2      |
| [tencent]     | TencentSection   | 3      |
| [soulvoice]   | SoulVoiceSection | 4      |
| [tts_qwen]    | TtsQwenSection   | 2      |
| [indextts2]   | IndexTTS2Section | 9      |
| [doubaotts]   | DoubaoTTSSection | 8      |
| [proxy]       | ProxySection     | 3      |
| [frames]      | FramesSection    | 3      |

## 任务完成情况

### Task 1: 库骨架 (TDD)

- **RED** `527e351` — 创建 Cargo.toml、src/lib.rs、src/error.rs，version() 返回空字符串，错误消息使用英文，测试全部失败（11/11）
- **GREEN** `2cf1403` — 实现 version() 返回 env!("CARGO_PKG_VERSION")，所有错误消息改为中文，11 个测试全部通过
- **偏离:** ffmpeg-sidecar feature 名称为 `download_ffmpeg` 而非 `auto-download`（Research.md 笔误）。此 feature 已包含在 default features 中。

### Task 2: 配置类型系统 + 热加载 (TDD)

- **RED** `517d473` — 创建 types/defaults/watcher/mod 配置文件，Default 值故意错误，ConfigManager load() 返回错误错误类型，3 个测试按预期失败
- **GREEN** `085e082` — 实现正确默认值（来自 config.example.toml）、ConfigManager 完整加载逻辑、ConfigWatcher notify 热加载，12 个配置测试全部通过

### 附加

- `887afd2` — 提交 Cargo.lock（可复现构建），更新 .gitignore 添加 target/

## 测试结果

全部 18 个测试通过（error.rs 的 10 个错误消息测试 + config 模块的 8 个测试）

```text
cargo test
running 18 tests
... all ok
```

## 威胁模型履行

| Threat | Disposition | Status |
|--------|-------------|--------|
| T-01-01 配置文件篡改 | mitigate: 强类型 serde 反序列化 | 已实现 |
| T-01-02 API key 泄露 | accept: .gitignore 已排除 config.toml | 已确认 |
| T-01-03 热加载篡改 | mitigate: 解析失败仅记录日志，不替换配置 | 已实现 |

## 偏离

### 已自动修复

**1. [RULE 2 - 遗漏关键功能] ffmpeg-sidecar feature 名称修正**

- **发现位置:** Task 1 RED phase
- **问题:** Research.md 指定 `auto-download` feature，但 ffmpeg-sidecar 2.5.1 实际 feature 名为 `download_ffmpeg`
- **修复:** Cargo.toml: `features = ["auto-download"]` → `features = ["download_ffmpeg"]`
- **文件修改:** Cargo.toml
- **提交:** 527e351 (RED commit 已包含修正)

**2. [RULE 2 - 遗漏关键功能] .gitignore 缺少 target/**

- **发现位置:** Task 2 GREEN 后清理
- **问题:** Rust 构建产物目录 `target/` 不在 .gitignore 中，可能被误提交
- **修复:** 添加 `target/` 到 .gitignore
- **文件修改:** .gitignore
- **提交:** 887afd2

## 自检

- [x] Cargo.toml 存在: package name=narratoai-core, version=0.1.0
- [x] src/lib.rs 存在: pub mod config, error; version() 返回 "0.1.0"
- [x] src/error.rs 存在: ConfigError (5 variants), FFmpegError (5 variants)
- [x] src/config/types.rs 存在: AppConfig + 10 个子段，字段名与 config.example.toml 一致
- [x] src/config/defaults.rs 存在: 所有 11 个 struct 的 Default trait 实现
- [x] src/config/mod.rs 存在: ConfigManager 提供 load/get/start_watching/config 接口
- [x] src/config/watcher.rs 存在: notify 9.x 热加载
- [x] cargo build: 零警告零错误
- [x] cargo test: 全部 18 个测试通过
- [x] ffmpeg-sidecar: features = ["download_ffmpeg"]
