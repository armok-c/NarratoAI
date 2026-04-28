---
phase: 01-foundation
plan: 03
type: execute
subsystem: foundation
tags: [integration-test, config, ffmpeg, verification]
depends_on:
  - 01-01 (config system)
  - 01-02 (FFmpeg operations)
provides:
  - Integration test coverage for CONF-01, CONF-02, CONF-03
  - Integration test coverage for FFMP-01, FFMP-09, CONF-04
  - End-to-end config loading and hot reload verification
  - End-to-end FFmpeg probe/clip/hwaccel verification
affects:
  - tests/config_test.rs
  - tests/ffmpeg_test.rs
tech-stack:
  added: []
  patterns:
    - "Integration tests in tests/ use narratoai_core::* imports"
    - "FFmpeg-dependent tests search PATH then ffmpeg-sidecar cache"
    - "Hot reload test uses TempDir + notify sleep-based timing"
key-files:
  created:
    - tests/config_test.rs (189 lines, 6 tests)
    - tests/ffmpeg_test.rs (210 lines, 5 tests)
  modified:
    - .gitignore (whitelist Rust integration test files)
decisions: []
metrics:
  duration: ~15 minutes
  completed: 2026-04-28
---

# Phase 1 Plan 3: Integration Tests Summary

**One-liner:** 11 项集成测试验证配置系统完整文件加载/默认值/热加载和 FFmpeg 视频探测/裁剪/硬件加速检测，全部在真实 FFmpeg 进程上完成。

---

## Execution Summary

达成 Phase 1 的最终验收目标：用真实 `config.example.toml` 文件和真实 FFmpeg 进程验证配置系统和 FFmpeg 操作的所有需求已满足。

### Task 1: 编写配置系统集成测试 (tests/config_test.rs)

**Commit:** `ca8a30e`

6 个 `#[test]` 函数覆盖全部配置场景：

| 测试 | 场景 | 验证点 |
|------|------|--------|
| `test_load_example_config` | 加载真实 config.example.toml | 所有 10 个 section 字段值与文件一致 |
| `test_load_minimal_config` | 仅含 `[app]` section 的 TOML | 缺失 section 使用默认值 |
| `test_load_empty_config` | 空文件 | 全部字段等于 AppConfig::default() |
| `test_load_nonexistent` | 不存在路径 | ConfigError::NotFound 返回 |
| `test_default_values` | AppConfig::default() | 关键字段值与 config.example.toml 一致 |
| `test_hot_reload` | 修改文件后重新加载 | ConfigManager.get() 返回更新值 |

### Task 2: 编写 FFmpeg 操作集成测试 (tests/ffmpeg_test.rs)

**Commit:** `baecd73`

5 个 `#[test]` 函数覆盖 FFmpeg 操作（含 1 个 `#[tokio::test]`）：

| 测试 | 类型 | 验证点 |
|------|------|--------|
| `test_probe_system_video` | unit | 对 ffmpeg 生成的测试视频执行 probe_video，验证 duration/width/height/codec_name/format_name 有效 |
| `test_clip_video_async` | async | 对测试视频执行 clip_video(0.0, 1.0)，输出文件存在且大小 > 0 |
| `test_hw_accel_detection` | unit | detect_hw_encoders 返回编码器列表（可能为空），各条目非空 |
| `test_recommend_profile` | unit | recommend_profile 返回 5 个变体之一，get_profile 返回有效信息 |
| `test_profiles_completeness` | unit | 遍历 5 个 HwAccelProfile，验证 name/encoder/pixel_format/compatibility_level 完整 |

---

## Verification Results

| Step | Command | Result |
|------|---------|--------|
| 1 | `cargo build` | 0 errors |
| 2 | `cargo test --test config_test` | 6 passed |
| 3 | `cargo test --test ffmpeg_test` | 5 passed |
| 4 | `cargo test` | 37 passed (26 unit + 6 config int + 5 ffmpeg int) |
| 5 | `cargo test 2>&1 | grep -E "test result|running"` | All 3 test files reported OK |

### Full Test Count

- **Unit tests:** 26 (config/mod.rs: 2, config/types.rs: 5, error.rs: 8, ffmpeg/command.rs: 3, ffmpeg/probe.rs: 1, ffmpeg/hwaccel.rs: 5, lib.rs: 1)
- **Config integration tests:** 6 (tests/config_test.rs)
- **FFmpeg integration tests:** 5 (tests/ffmpeg_test.rs)
- **Total:** 37 passed, 0 failed, 0 ignored

---

## Requirement Coverage

| Req ID | Description | Coverage |
|--------|-------------|----------|
| CONF-01 | 从 TOML 文件加载所有配置段 | `test_load_example_config` — 验证 10 个 section 全部加载 |
| CONF-02 | 配置校验——缺失字段使用默认值 | `test_load_minimal_config`, `test_load_empty_config` |
| CONF-03 | 支持配置文件热加载 | `test_hot_reload` — 文件变更后 ConfigManager.get() 更新 |
| CONF-04 | FFmpeg 硬件加速检测 | `test_hw_accel_detection`, `test_recommend_profile` |
| FFMP-01 | 通过 ffmpeg-sidecar 执行 FFmpeg 操作 | `test_probe_system_video`, `test_clip_video_async` |
| FFMP-09 | FFmpeg 异步执行不阻塞 tokio | `test_clip_video_async` spawn_blocking 包装 |

---

## Known Stubs

None detected. 所有测试使用真实文件或临时文件，无硬编码空值/占位符。

---

## Threat Flags

None. 测试代码不引入新的安全边界。FFmpeg 子进程调用复用库模块已有的 ffmpeg-sidecar builder API（非字符串拼接），无命令注入风险。

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] .gitignore 阻止 Rust 测试文件提交**

- **Found during:** Task 1 commit
- **Issue:** `.gitignore` line 52 (`tests/*`) 忽略 tests/ 下所有文件，仅 whitelist Python 测试文件。新创建的 `tests/config_test.rs` 和 `tests/ffmpeg_test.rs` 无法被 git 跟踪。
- **Fix:** 在 `.gitignore` 的 `tests/*` whitelist 列表中增加 `!tests/config_test.rs` 和 `!tests/ffmpeg_test.rs`
- **File modified:** `.gitignore`
- **Commit:** `ca8a30e`

### Plan Adjustments

**2. find_ffmpeg 策略调整：跳过 auto_download**

- **Trigger:** ffmpeg-sidecar 的 `auto_download()` 在 Windows 环境网络阻塞，导致测试进程挂起
- **Adjustment:** `find_ffmpeg()` 不再调用 `auto_download()`，改为先搜索 PATH，再检查 ffmpeg-sidecar 缓存路径。若两者皆不可用则 panic 输出清晰提示。
- **Rationale:** auto_download 下载 ~100MB 二进制需要网络连接，在受限环境中会无限等待。PATH 检测足够覆盖系统安装和 ffmpeg-sidecar 预下载两种场景。
- **Files affected:** `tests/ffmpeg_test.rs`
- **NOTE:** 该调整不改变测试验证范围，仅改变 FFmpeg 定位策略

---

## Decisions Made

1. **路径搜索优先于 auto_download**: 集成测试的 FFmpeg 定位策略从 `auto_download()` 改为 PATH/缓存路径搜索，避免网络阻塞风险
2. **Libx264 而非硬件编码器检测**: `detect_hw_encoders` 仅返回硬件编码器列表（不含 libx264），`test_hw_accel_detection` 据此调整验证逻辑

## Self-Check: PASSED

- Created files: tests/config_test.rs, tests/ffmpeg_test.rs, 01-03-SUMMARY.md
- Commits: ca8a30e (config tests), baecd73 (ffmpeg tests)
- All 37 tests pass: 26 unit + 6 config integration + 5 ffmpeg integration

## DEPENDENCY NOTE: FFmpeg Binary

The FFmpeg integration tests (`tests/ffmpeg_test.rs`) require a real FFmpeg binary.
During development, the test harness discovered FFmpeg at:
`C:\Users\FirmC\AppData\Roaming\SmartEdit\resources\ffmpeg-8.0.1-essentials_build\bin\`

To run these tests in CI or a fresh environment, either:
- Install FFmpeg on PATH, OR
- Run `ffmpeg_sidecar::download::auto_download()` once before tests, OR
- Set `PATH` to include a directory with `ffmpeg.exe` and `ffprobe.exe`
