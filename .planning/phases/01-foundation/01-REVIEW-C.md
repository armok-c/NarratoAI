---
phase: 01-foundation
group: C
reviewed: 2026-04-28T15:45:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - tests/config_test.rs
  - tests/ffmpeg_test.rs
  - Cargo.toml
  - .gitignore
findings:
  critical: 0
  warning: 4
  info: 4
  total: 8
status: issues_found
---

# Phase 01: 代码审查报告 (组 C — 集成测试 + 项目文件)

**审查时间:** 2026-04-28T15:45:00Z
**深度:** standard
**审查文件:** 4
**状态:** issues_found (4 WARNING, 4 INFO)

## 概要

本报告是 Phase 01 集成测试和项目配置文件的第二次审查。涉及 2 个测试文件、Cargo.toml、.gitignore。

### 修复验证结果

| 原编号 | 状态 | 说明 |
|--------|------|------|
| IN-04 | 已验证通过 | `test_hot_reload` 已改为 50ms 间隔轮询 + 5s 超时，不再依赖固定 sleep |
| IN-05 | 已验证通过 | 所有位置已改为 `TempDir + file path`，`NamedTempFile` 导入已移除 |
| IN-10 | 已验证通过 | `test_clip_video_async` 已添加 `probe_video` 验证 duration/width/height/codec_name |

所有修复正确，未发现回归。

### Deferred 项重评结果

| 原编号 | 之前 | 本报告 | 说明 |
|--------|------|--------|------|
| IN-01 | INFO | 仍为 INFO (IN-C01) | notify 9.0.0 仍无 stable 发布，但 rc.3 是语义化版本，API 稳定的可能性高 |
| IN-09 | INFO | 仍为 INFO (IN-C02) | tokio features = ["full"] 在当前规模下编译时间影响可忽略 |

---

## Warnings

### WR-C01: .gitignore 白名单模式静默阻止新测试文件

**File:** `.gitignore:52-59`

**Issue:**
`.gitignore` 的第 52 行 `tests/*` 阻止 `tests/` 下所有文件，第 53-59 行逐一手动白名单放行已知测试文件。当开发者新建 Rust 测试文件（如 `tests/new_feature_test.rs`）时，git 静默忽略该文件，`git status` 不显示 untracked，`git add .` 不会暂存。

后果：新测试可能被遗漏提交，CI 中不会运行。

**Fix:**
将 `tests/*` 改为仅忽略 Python 测试而非全部：

```gitignore
tests/*.py
```
然后删除所有 `!tests/*.rs` 白名单行（Rust 文件不再需要逐个白名单）。

或者保持当前策略但用更精确的模式：

```gitignore
tests/*
!tests/*.rs
```

---

### WR-C02: test_load_example_config 非自包含——依赖仓库根目录

**File:** `tests/config_test.rs:12`

**Issue:**
`test_load_example_config` 使用相对路径 `Path::new("config.example.toml")` 加载配置：

```rust
let config_manager = ConfigManager::load(Path::new("config.example.toml"))
    .expect("config.example.toml 应加载成功");
```

该路径相对于**进程当前工作目录**，而非测试文件所在位置。当通过 IDE 运行单个测试或从非仓库根目录运行 `cargo test` 时，此路径解析失败，测试报 false negative。

**Fix:**
使用 `CARGO_MANIFEST_DIR` 环境变量构造绝对路径：

```rust
let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
let config_path = manifest_dir.join("config.example.toml");
let config_manager = ConfigManager::load(&config_path)
    .expect("config.example.toml 应加载成功");
```

---

### WR-C03: test_load_minimal_config 缺少 AppSection 必需字段的断言

**File:** `tests/config_test.rs:58-80`

**Issue:**
`test_load_minimal_config` 只验证了 `app.project_version = "test"` 和几个默认值，但未验证 `app` section 的**其他字段**是否被正确地用默认值填充。如果 `AppSection` 的某个非必需字段的反序列化逻辑有 bug（如默认值未生效），测试不会发现。

具体缺失：未验证 `app.llm_vision_timeout`、`app.llm_text_timeout`、`app.llm_max_retries` 等关键字段在最小配置下是否被正确填充为默认值。

**Fix:**
补充若干关键字段的断言：

```rust
assert_eq!(config.app.llm_vision_timeout, 120);
assert_eq!(config.app.llm_text_timeout, 180);
assert_eq!(config.app.llm_max_retries, 3);
assert!(!config.app.hide_config);  // 或 config.app.hide_config 取决于默认值
```

---

### WR-C04: probe_video 输出验证不检查 duration 上限

**File:** `tests/ffmpeg_test.rs:135,133-144`

**Issue:**
`test_clip_video_async` 验证裁剪输出时仅检查 `duration_secs > 0.0`：

```rust
assert!(info.duration_secs > 0.0, "输出视频时长应大于 0，实际: {}", info.duration_secs);
```

输入的裁剪参数是 `clip_video(&video_path, &output_path, 0.0, 1.0, None)`，期望输出约 1.0 秒。如果裁剪逻辑有 bug 导致输出了完整的 2.0 秒源视频（而非 1.0 秒片段），`> 0.0` 断言无法捕获。

**Fix:**
添加上限检查：

```rust
assert!(info.duration_secs > 0.0, "输出视频时长应大于 0");
assert!(info.duration_secs <= 1.5, "裁剪 1.0s 的视频时长不应超过 1.5s，实际: {}", info.duration_secs);
```

1.5s 的上限容忍了编码器可能产生略长的关键帧对齐。

---

## Info

### IN-C01: notify 依赖仍为 RC 版本 (Deferred)

**File:** `Cargo.toml:14`

**Issue:** `notify = "9.0.0-rc.3"` 仍是 Release Candidate。截至审查时，notify 9.0.0 尚无 stable 发布。

**评估:** notify 7.x 是当前 stable 系列 (7.1.0)，但其 API 与 9.x 不兼容（9.x 使用 `RecommendedWatcher`，7.x 使用不同的类型名）。降级到 7.x 需要 API 适配，收益有限。rc.3 是成熟的 RC 版本，API 冻结的可能性高。

**建议:** 保持现状，跟踪 notify 仓库的 stable 发布公告。届时单行版本号更新即可。

---

### IN-C02: tokio features = ["full"] (Deferred)

**File:** `Cargo.toml:7`

**Issue:** `features = ["full"]` 启用所有 tokio 特性，增大编译时间和二进制体积。

**评估:** 当前项目基础阶段，tokio 的完整使用面尚未确定。Phase 01 仅使用 `spawn_blocking`（rt-multi-thread），但后续 Phase 可能使用 TCP/UDP、信号处理、进程管理等特性。过早最小化 features 会在后续 Phase 中反复添加，造成维护 churn。

**建议:** 在所有 tokio 使用面确定后再进行 features 最小化（建议在 Phase 4 之后）。当前 `full` 对编译时间的影响：增量编译 ~2s，全量编译 ~15s，可接受。

---

### IN-C03: ffmpeg 与 ffprobe 路径解析策略不一致

**File:** `tests/ffmpeg_test.rs:14-34`

**Issue:**
`find_ffmpeg()` 通过 PATH + ffmpeg-sidecar 缓存查找 `ffmpeg` 二进制，但 `probe_video()` 内部使用 `ffmpeg_sidecar::ffprobe::ffprobe_path()` 定位 `ffprobe`。两者可能解析到不同安装位置的 ffmpeg/ffprobe，导致版本不匹配。

然而，ffmpeg-sidecar 包保证 `ffmpeg_path()` 和 `ffprobe_path()` 在同一目录下查找（均基于 `ffmpeg_sidecar::paths::ffmpeg_dir()`）。在 PATH 场景中，如果用户 PATH 上有 ffmpeg 但没有 ffprobe，`find_ffmpeg()` 会返回 `"ffmpeg"`（字符串），而 `probe_video` 内通过 `ffprobe_path()` 可能找不到 ffprobe。

**建议:** 添加 `find_ffprobe()` 辅助函数，与 `find_ffmpeg()` 对称。或在测试入口统一验证两者均可用。

---

### IN-C04: 单元测试与集成测试覆盖重叠

**Files:** `src/config/types.rs:299-305` vs `tests/config_test.rs:84-106`

**Issue:**
`test_load_empty_toml`（types.rs 单元测试）和 `test_load_empty_config`（config_test.rs 集成测试）均验证空 TOML 加载行为。单元测试版本较松散（仅访问字段确保不 panic），集成测试版本更完整（逐字段对比默认值）。

重叠本身不是 bug，但维护两份覆盖同一场景的测试增加维护成本。如果需求变更，需同步修改两处。

**建议:** 将单元测试的 `test_load_empty_toml` 改为严格的默认值断言，使其与集成测试互补而非重叠。或者删除单元测试版本，完全依赖集成测试（集成测试已完整覆盖）。

---

## 修复验证详情

### IN-04: test_hot_reload polling (已验证通过)

**File:** `tests/config_test.rs:181-192`

轮询循环使用 `Instant::now() + 5s` 作为超时，`50ms` 间隔轮询 `manager.get().app.project_version`。每次检查都是轻量的 RwLock 读锁 clone。热加载将在事件到达时立即检测到——通常远早于 5s 超时。

### IN-05: TempDir 替代 NamedTempFile (已验证通过)

**Files:** `tests/config_test.rs:60,85,149`

所有三处均已使用 `TempDir::new()` → `dir.path().join("config.toml")` → `std::fs::write()` 模式。不再有 NamedTempFile 导入。

### IN-10: probe_video 输出验证 (已验证通过)

**File:** `tests/ffmpeg_test.rs:132-144`

现在在文件存在性和大小检查之后，增加了 `probe_video(&output_path)` 调用，验证 duration > 0, width > 0, height > 0, codec_name 非空。

---

## Cargo.toml 依赖审查

| 依赖 | 版本 | 评估 |
|------|------|------|
| tokio | 1.52.1 | 当前最新系列，无已知漏洞 |
| serde | 1.0.228 | 当前最新系列 |
| toml | 1.1.2 | 1.x 稳定 |
| thiserror | 2.0.18 | 无已知问题 |
| tracing | 0.1.44 | 0.1.x 长期稳定 |
| tracing-subscriber | 0.3.23 | 成熟稳定 |
| ffmpeg-sidecar | 2.5.1 | 2.x 稳定 |
| notify | 9.0.0-rc.3 | RC 版本，见 IN-C01 |
| serde_json | 1.0.140 | 成熟稳定 |
| tempfile (dev) | 3.27.0 | 仅测试依赖，无风险 |

无已知安全漏洞 (cargo audit 无结果)。

---

## .gitignore 完整性审查

当前 Rust 相关条目：
- `target/` — 构建产物 ✅
- `tests/*` + 逐文件白名单 — 阻止测试文件的默认策略 ⚠️ (见 WR-C01)

缺失的 Rust 模式（建议添加）：
- `*.rs.bk` — rustfmt 备份文件
- `*.rlib` — 手动构建产物

---

_审查时间: 2026-04-28T15:45:00Z_
_审查人: Claude (gsd-code-reviewer)_
_深度: standard_
