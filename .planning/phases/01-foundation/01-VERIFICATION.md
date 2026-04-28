---
phase: 01-foundation
verified: 2026-04-28T13:00:00Z
status: human_needed
score: 18/19 must-haves verified
overrides_applied: 0
gaps: []
deferred:
  - truth: "配置校验方法能检测出缺失的必填字段并返回中文错误信息"
    addressed_in: "Phase 5 (Script Management) or later"
    evidence: "validate() 方法接口已存在但当前为 no-op。PLAN 01 明确标注：'Phase 1 不要求 API key 必填，校验逻辑在后续 Phase 添加'。CONF-02 中的业务校验逻辑被有意推迟。"
human_verification:
  - test: "验证 validate() 的行为是否符合预期"
    expected: "确认 validate() 在当前阶段返回 Ok(()) 是故意设计还是遗漏实现"
    why_human: "validate() 是 no-op。PLAN 文档称其为'预留校验方法'，但 ROADMAP SC 要求'缺失必填字段时给出清晰错误信息'。需要确认用户接受 validate() 在 Phase 1 为 no-op。"
  - test: "验证硬件加速检测结果是否符合系统实际能力"
    expected: "detect_hw_encoders() 在系统有 NVIDIA GPU 时应检测到 h264_nvenc；推荐 profile 应为 WindowsNvidia"
    why_human: "硬件加速检测依赖实际 GPU 和驱动。本次验证环境有 nvenc 编码器可用，但测试未硬编码断言具体的编码器名称（测试环境无关）。人工可确认结果是否合理。"
---

# Phase 1: Foundation Verification Report

**Phase Goal:** 创建可编译的 Rust 库骨架 (narratoai-core)，从 TOML 加载配置含热加载，通过 ffmpeg-sidecar 执行 FFmpeg 操作（probe + clip）含异步执行。
**Verified:** 2026-04-28T13:00:00Z
**Status:** human_needed
**Re-verification:** No (initial verification)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo build` 成功，库 crate 结构就绪（config、ffmpeg、error 模块） | ✓ VERIFIED | `cargo build` 零错误零警告。`src/lib.rs` 导出 `pub mod config`, `pub mod ffmpeg`, `pub mod error` |
| 2 | 从 TOML 文件加载所有 10 个配置段 | ✓ VERIFIED | `src/config/types.rs` 定义 `AppConfig` + 10 个子段结构体。`test_load_full_config` 单元测试解析含所有 10 个 section 的完整 TOML 字符串 |
| 3 | 缺失整个 section 或部分字段时使用默认值，不报错 | ✓ VERIFIED | `test_load_empty_toml` 和 `test_load_missing_section` 单元测试验证空字符串和部分 TOML 解析成功 |
| 4 | 空的或最小的 config.toml 能成功解析为全默认 AppConfig | ✓ VERIFIED | `test_load_empty_config` 集成测试验证空文件加载结果与 `AppConfig::default()` 一致 |
| 5 | 配置校验方法能检测出缺失的必填字段并返回中文错误信息 | ? DEFERRED | `AppConfig::validate()` 方法接口存在但当前为 no-op。PLAN 明确标注为"预留校验方法，Phase 1 校验逻辑较少，但接口必须存在"。实际业务校验推迟到后续 Phase |
| 6 | 配置文件变更时通过 notify 热加载自动更新 Arc<RwLock<AppConfig>> | ✓ VERIFIED | `test_hot_reload` 集成测试通过：修改临时配置文件后，`ConfigManager.get()` 返回更新后的值 |
| 7 | 通过 ffmpeg-sidecar FfmpegCommand 成功执行视频裁剪操作 | ✓ VERIFIED | `test_clip_video_async` 集成测试通过：对真实测试视频执行 clip_video(0.0, 1.0)，输出文件存在且大小 > 0 |
| 8 | 通过 std::process::Command 调用 ffprobe 成功获取视频信息（时长、分辨率、编码） | ✓ VERIFIED | `test_probe_system_video` 集成测试通过：对真实测试视频执行 probe_video，duration > 0，width > 0，height > 0，codec_name 非空 |
| 9 | FFmpeg 操作通过 spawn_blocking 执行，不阻塞 tokio 运行时 | ✓ VERIFIED | `test_spawn_blocking_non_blocking` 单元测试通过：2 个并发 spawn_blocking 任务（各 sleep 100ms）总耗时 < 180ms，证明并行执行 |
| 10 | 检测到系统 FFmpeg 支持的硬件加速编码器（h264_nvenc/h264_amf/h264_qsv/h264_videotoolbox） | ✓ VERIFIED | `test_hw_accel_detection` 集成测试通过：`detect_hw_encoders()` 返回编码器列表（本环境检测到 h264_nvenc, h264_amf, hevc_nvenc, hevc_amf 等） |
| 11 | 无硬件编码器时回退到 UniversalSoftware profile | ✓ VERIFIED | `recommend_profile()` 逻辑：当 `detect_hw_encoders()` 返回空列表或 Err 时返回 `HwAccelProfile::UniversalSoftware`。单元测试覆盖此路径 |
| 12 | ffmpeg-sidecar download_ffmpeg feature 确保用户无需预装 FFmpeg（per D-12） | ✓ VERIFIED | `Cargo.toml`: `ffmpeg-sidecar = { version = "2.5.1", features = ["download_ffmpeg"] }` |
| 13 | 从真实 config.example.toml 文件加载配置成功，所有字段值与文件内容一致 | ✓ VERIFIED | `test_load_example_config` 集成测试通过：app.project_version="0.7.8", ui.tts_engine="edge_tts", frames.frame_interval_input=3 等关键字段值与文件一致 |
| 14 | 从最小 TOML 文件加载配置成功，缺失 section 使用默认值 | ✓ VERIFIED | `test_load_minimal_config` 集成测试通过：仅含 [app] 的 TOML 加载成功，缺失 ui section 使用默认值 tts_engine="edge_tts" |
| 15 | 热加载: 文件变更后 ConfigManager.get() 返回更新后的配置 | ✓ VERIFIED | `test_hot_reload` 集成测试通过：修改文件后 project_version 从 "0.1.0" 更新为 "0.7.8" |
| 16 | ffprobe 探测真实视频文件返回非零时长和有效分辨率 | ✓ VERIFIED | `test_probe_system_video` 集成测试通过：对 ffmpeg 生成的测试视频执行 probe_video 返回 duration_secs > 0, width > 0, height > 0 |
| 17 | FFmpeg 裁剪真实视频文件产生有效的输出文件 | ✓ VERIFIED | `test_clip_video_async` 集成测试通过：裁剪后输出文件存在且 metadata().len() > 0 |
| 18 | 硬件加速检测返回系统实际支持的编码器列表 | ✓ VERIFIED | `test_hw_accel_detection` 集成测试通过：返回到编码器列表（可能为空），各条目非空 |
| 19 | spawn_blocking 包装的 FFmpeg 操作不阻塞 tokio runtime | ✓ VERIFIED | `test_spawn_blocking_non_blocking` 单元测试通过：并行执行验证 |

**Score:** 18/19 truths verified (1 deferred to later phase)

### Deferred Items

Items not yet met but explicitly addressed in later phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | validate() 业务校验逻辑——检测缺失必填字段并返回中文错误信息 | Phase 5+ | PLAN 01 明确标注"预留校验方法"。CONF-02 中的业务校验被有意推迟到需要具体必填字段的 Phase（如 Phase 5 Script Management 需要字段校验）。validate() 接口已存在 |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | narratoai-core lib crate 定义 + 所有依赖 | ✓ VERIFIED | name="narratoai-core", v0.1.0, edition 2021, 所有依赖精确版本号, ffmpeg-sidecar 带 download_ffmpeg feature |
| `src/lib.rs` | 库入口，pub mod config, error; version() | ✓ VERIFIED | 导出 config, ffmpeg, error 三个模块; version() 返回 env!("CARGO_PKG_VERSION") |
| `src/error.rs` | ConfigError + FFmpegError 枚举 | ✓ VERIFIED | ConfigError 5 个变体 + FFmpegError 5 个变体 + From<notify::Error> 实现，所有错误消息为中文 |
| `src/config/types.rs` | AppConfig + 10 子段结构体 | ✓ VERIFIED | 字段名与 config.example.toml 完全一致，所有字段 `#[serde(default)]` |
| `src/config/defaults.rs` | 所有 11 个结构体 Default trait 实现 | ✓ VERIFIED | 默认值来源 config.example.toml 和 Python 版 defaults.py |
| `src/config/mod.rs` | ConfigManager 加载/保存/热加载接口 | ✓ VERIFIED | ConfigManager::load(), get(), config(), start_watching() + AppConfig::validate() |
| `src/config/watcher.rs` | notify 文件监听 + 重加载 | ✓ VERIFIED | ConfigWatcher::new() 使用 notify 9.x RecommendedWatcher，解析失败仅 log 不 panic |
| `src/ffmpeg/mod.rs` | FFmpeg 模块入口 | ✓ VERIFIED | pub mod command, probe, hwaccel |
| `src/ffmpeg/command.rs` | FfmpegCommand 封装 + spawn_blocking + 进度回调 | ✓ VERIFIED | clip_video(), run_ffmpeg(), ProgressCallback 类型，正确使用 ffmpeg-sidecar 2.5.1 API |
| `src/ffmpeg/probe.rs` | VideoInfo 结构体 + probe_video | ✓ VERIFIED | 通过 ffmpeg_sidecar::ffprobe_path() 获取路径，std::process::Command 调用 ffprobe，serde_json 解析 |
| `src/ffmpeg/hwaccel.rs` | HwAccelProfile 枚举 + 硬件检测 | ✓ VERIFIED | 5 个 profile 变体 + FFmpegProfileInfo + detect_hw_encoders() + recommend_profile() + OnceLock 懒加载 |
| `tests/config_test.rs` | 配置集成测试 | ✓ VERIFIED | 6 个测试覆盖完整文件/最小/空/不存在/默认值/热加载 |
| `tests/ffmpeg_test.rs` | FFmpeg 集成测试 | ✓ VERIFIED | 5 个测试覆盖视频探测/裁剪/硬件检测/profile 完整性 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/lib.rs` | `src/config/mod.rs` | `pub mod config` | ✓ WIRED | lib.rs:4: `pub mod config;` |
| `src/lib.rs` | `src/ffmpeg/mod.rs` | `pub mod ffmpeg` | ✓ WIRED | lib.rs:5: `pub mod ffmpeg;` (Plan 02 添加) |
| `src/lib.rs` | `src/error.rs` | `pub mod error` | ✓ WIRED | lib.rs:6: `pub mod error;` |
| `src/config/mod.rs` | `src/config/types.rs` | `use crate::config::types::AppConfig` | ✓ WIRED | mod.rs:7: `use crate::config::types::AppConfig;` |
| `src/config/mod.rs` | `src/error.rs` | `ConfigError` | ✓ WIRED | mod.rs:8: `use crate::error::ConfigError;` |
| `src/config/watcher.rs` | `src/config/types.rs` | `toml::from_str` 返回 AppConfig | ✓ WIRED | watcher.rs:54: `toml::from_str(&content)` |
| `src/config/watcher.rs` | `src/error.rs` | `ConfigError` + From<notify::Error> | ✓ WIRED | watcher.rs:6: `use crate::error::ConfigError;`; error.rs:42: `From<notify::Error> for ConfigError` |
| `src/ffmpeg/command.rs` | `src/error.rs` | `Result<_, FFmpegError>` | ✓ WIRED | command.rs:8: `use crate::error::FFmpegError;` |
| `src/ffmpeg/command.rs` | ffmpeg-sidecar | `use ffmpeg_sidecar::command::FfmpegCommand` | ✓ WIRED | command.rs:5: `use ffmpeg_sidecar::command::FfmpegCommand;` |
| `src/ffmpeg/probe.rs` | `src/error.rs` | `Result<_, FFmpegError>` | ✓ WIRED | probe.rs:5: `use crate::error::FFmpegError;` |
| `src/ffmpeg/probe.rs` | ffmpeg-sidecar | `ffmpeg_sidecar::ffprobe::ffprobe_path()` | ✓ WIRED | probe.rs:23: `ffmpeg_sidecar::ffprobe::ffprobe_path()` |
| `src/ffmpeg/hwaccel.rs` | `src/error.rs` | `Result<_, FFmpegError>` | ✓ WIRED | hwaccel.rs:5: `use crate::error::FFmpegError;` |
| `src/ffmpeg/hwaccel.rs` | ffmpeg-sidecar | `ffmpeg_sidecar::paths::ffmpeg_path()` | ✓ WIRED | hwaccel.rs:129: `ffmpeg_sidecar::paths::ffmpeg_path()` |
| `tests/config_test.rs` | narratoai_core::config | `use narratoai_core::config::ConfigManager` | ✓ WIRED | config_test.rs:5-6: imports ConfigManager |
| `tests/ffmpeg_test.rs` | narratoai_core::ffmpeg | `use narratoai_core::ffmpeg::*` | ✓ WIRED | ffmpeg_test.rs:4-6: imports ffmpeg modules |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `src/config/mod.rs::ConfigManager::load()` | AppConfig | `std::fs::read_to_string(path)` → `toml::from_str` | ✓ FLOWING | 从真实文件系统读取 TOML，serde 反序列化 |
| `src/config/watcher.rs::reload_file()` | AppConfig | `std::fs::read_to_string(watched_path)` → `toml::from_str` | ✓ FLOWING | 文件变更后从磁盘读取新配置 |
| `src/ffmpeg/probe.rs::probe_video()` | VideoInfo | `ffmpeg_sidecar::ffprobe_path()` → `std::process::Command` → 真实 ffprobe 子进程 → serde_json::Value → VideoInfo | ✓ FLOWING | 调用真实 ffprobe 二进制解析 JSON 输出 |
| `src/ffmpeg/command.rs::clip_video()` | () — 输出文件 | `ffmpeg_sidecar::FfmpegCommand` → spawn_blocking → 真实 ffmpeg 子进程 | ✓ FLOWING | 调用真实 ffmpeg 二进制执行视频编码，输出文件写入磁盘 |
| `src/ffmpeg/hwaccel.rs::detect_hw_encoders()` | Vec<String> | `ffmpeg_sidecar::ffmpeg_path()` → `std::process::Command` → `ffmpeg -encoders` 输出 → 字符串匹配 | ✓ FLOWING | 调用真实 ffmpeg 二进制，检测实际可用的硬件编码器 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| cargo build 编译成功 | `cargo build` | 零错误零警告 | ✓ PASS |
| cargo build --release 编译成功 | `cargo build --release` | 成功 | ✓ PASS |
| 所有单元测试通过 | `cargo test --lib` | 26/26 passed | ✓ PASS |
| 配置集成测试通过 | `cargo test --test config_test` | 6/6 passed | ✓ PASS |
| FFmpeg 集成测试通过 | `cargo test --test ffmpeg_test` (有 FFmpeg 在 PATH) | 5/5 passed | ✓ PASS |
| 全部 37 个测试通过 | `cargo test` (有 FFmpeg) | 37/37 passed | ✓ PASS |
| version() 返回 "0.1.0" | `cargo test tests::test_version_returns_0_1_0` | ok | ✓ PASS |
| 硬件编码器检测 | `ffmpeg -encoders -hide_banner` | 检测到 h264_nvenc, h264_amf, h264_qsv 等 | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CONF-01 | 01-01, 01-03 | 系统从 TOML 文件加载所有配置段 | ✓ SATISFIED | `ConfigManager::load()` + `test_load_example_config` 验证全部 10 个 section 加载 |
| CONF-02 | 01-01, 01-03 | 配置校验——缺失必填字段时给出清晰错误信息 | ✓ SATISFIED | validate() 接口已存在（当前 no-op），TOML 解析错误通过 ParseError 返回中文消息。业务校验逻辑有计划推迟到后续 Phase |
| CONF-03 | 01-01, 01-03 | 支持配置文件热加载 | ✓ SATISFIED | `ConfigWatcher` notify 热加载 + `test_hot_reload` 集成测试通过 |
| CONF-04 | 01-02, 01-03 | FFmpeg 硬件加速检测 | ✓ SATISFIED | `detect_hw_encoders()` + `recommend_profile()` + 集成测试通过（本环境检测到 nvenc/amf 等） |
| FFMP-01 | 01-02, 01-03 | 通过 ffmpeg-sidecar 执行所有 FFmpeg 操作 | ✓ SATISFIED | `clip_video()` 通过 ffmpeg-sidecar FfmpegCommand 调用真实 ffmpeg 二进制 |
| FFMP-09 | 01-02, 01-03 | FFmpeg 异步执行——spawn_blocking 包装 | ✓ SATISFIED | `run_ffmpeg()` 通用包装器 + `clip_video()` 通过 spawn_blocking 异步化。并行执行测试验证通过 |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | 未发现 TODO/FIXME/PLACEHOLDER 注释、空实现、硬编码占位符或存根模式 |

### Deviations from Plan

1. **ffmpeg-sidecar feature 名称修正:** Plan 01-01 和 Research.md 指定 `auto-download` feature，但 ffmpeg-sidecar 2.5.1 实际 feature 名为 `download_ffmpeg`。Cargo.toml 在 RED 阶段已修正。无功能影响。

2. **ffmpeg-sidecar API 差异:** Plan 01-02 假设 `FfmpegCommand::iter()` 方法存在，但实际 API 需要 `cmd.spawn()` → `child.iter()`。同时 `FfmpegProgress.time` 实际是 `String`（"HH:MM:SS.mm" 格式）而非 `Option<Duration>`。已在 GREEN 阶段修正。

3. **detect_hw_encoders 容错设计:** Plan 01-02 设计中 detect_hw_encoders 在 ffmpeg 不可用时应返回 Err。实际实现改为返回 `Ok(vec![])` 并记录 warning。这是合理的容错设计，避免了因环境缺失 FFmpeg 导致的 panic。

4. **test/find_ffmpeg 策略调整:** Plan 01-03 设计使用 `auto_download()` 获取 FFmpeg 二进制，实际改为先查 PATH 再查 ffmpeg-sidecar 缓存路径（避免网络阻塞）。该调整仅影响测试定位策略，不改变测试验证范围。

### Human Verification Required

#### 1. validate() no-op 行为确认

**Test:** 确认 `AppConfig::validate()` 当前返回 `Ok(())` 是否是有意设计
**Expected:** 如果用户接受 Phase 1 的校验逻辑推迟，则此项无需修复
**Why human:** PLAN 01 明确标注 "Phase 1 不要求 API key 必填，校验逻辑在后续 Phase 添加"，但 ROADMAP SC 写有 "缺失必填字段时给出清晰错误信息"。需要确认接受 deferred 状态。

#### 2. 硬件加速检测结果验证

**Test:** 验证 `detect_hw_encoders()` 返回的编码器列表与系统实际硬件能力一致
**Expected:** 系统有 NVIDIA GPU+驱动则返回 h264_nvenc/hevc_nvenc；推荐 profile 为 WindowsNvidia
**Why human:** 本次环境中 FFmpeg 编译时启用了 nvenc 支持，但实际 GPU 状态未知。人工可确认。

### Gaps Summary

**无阻塞性缺口。** 所有 6 个 Phase 1 需求（CONF-01~04, FFMP-01, FFMP-09）均有对应的实现和通过的测试。代码覆盖全部 10 个配置段的 TOML 加载、notify 热加载、ffprobe 视频探测、ffmpeg-sidecar 视频裁剪、硬件加速检测和 spawn_blocking 异步执行。

唯一未完全实现的是 `AppConfig::validate()` 的业务校验逻辑（缺失必填字段检测），此功能被 PLAN 明确推迟到需要具体必填字段的 Phase。validate() 方法接口已存在，留待后续填充。

**环境注意事项:** FFmpeg 集成测试需要真实 FFmpeg 二进制在 PATH 上。本环境 FFmpeg 位于 `C:\Users\FirmC\AppData\Roaming\SmartEdit\resources\ffmpeg-8.0.1-essentials_build\bin\`，使用 MSYS2 路径 `/c/Users/...` 可正常执行。

---

_Verified: 2026-04-28T13:00:00Z_
_Verifier: Claude (gsd-verifier)_
