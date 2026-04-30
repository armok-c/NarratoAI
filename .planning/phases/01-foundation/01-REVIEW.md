---
phase: 01-foundation
reviewed: 2026-04-30T00:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/error.rs
  - src/config/types.rs
  - src/config/defaults.rs
  - src/config/mod.rs
  - src/config/watcher.rs
  - Cargo.lock
  - .gitignore
  - src/ffmpeg/mod.rs
  - src/ffmpeg/command.rs
  - src/ffmpeg/probe.rs
  - src/ffmpeg/hwaccel.rs
  - tests/config_test.rs
  - tests/ffmpeg_test.rs
findings:
  critical: 2
  warning: 1
  info: 3
  total: 6
status: issues_found
---

# Phase 01: Foundation — Code Review Report

**Reviewed:** 2026-04-30T00:00:00Z
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

审查了 Phase 01（基础层）的全部 16 个文件，包括 Rust 库骨架、配置系统、FFmpeg 操作层和集成测试。整体质量较好，代码结构清晰，错误处理覆盖完善，测试结构合理。

发现 2 个 BLOCKER 问题、1 个 WARNING 问题、3 个 INFO 问题。核心 Bug 集中在 `src/ffmpeg/command.rs`：

1. **CR-01** — FFmpeg 参数顺序错误导致 `-c:v copy` / `-c:a copy` 被 FFmpeg 忽略，静默降级为重编码
2. **CR-02** — `spawn_blocking` 超时后 FFmpeg 子进程变成孤儿进程泄漏资源，可能持文件锁导致后续操作失败

---

## CRITICAL Issues

### CR-01: FFmpeg codec copy 参数顺序错误导致静默重编码（而非流复制）

**File:** `src/ffmpeg/command.rs:62-68`
**Issue:** `clip_video` 构建 FFmpeg 参数时，`codec_video("copy")` 和 `codec_audio("copy")` 在 `.output()` **之后**调用。通过阅读 `ffmpeg-sidecar` v2.5.1 源码（`command.rs:67-70,91-107,182-186`）确认，所有 builder 方法仅通过 `self.arg(...)` 向 `Command` 追加参数，**不进行参数重排序**。

实际生成的命令行：
```sh
ffmpeg -loglevel level+info -ss 0.000 -i input.mp4 -t 10.000 output.mp4 -y -c:v copy -c:a copy
```

在 FFmpeg 的参数解析规则中，非选项参数（输出 URL `output.mp4`）是输出选项的作用域分隔符。出现在 `output.mp4` **之后**的 `-c:v copy` 和 `-c:a copy` 被解释为"应用于下一个输出"的选项。由于没有下一个输出，这两个选项被**静默忽略**，FFmpeg 回退到默认编码器（libx264 重编码）。

**影响：**
- `clip_video` 始终执行完整重编码而不是流复制，违背了 `codec: copy` 的设计意图
- 处理速度下降 5-50 倍（取决于源视频）
- 重编码引入质量损失
- 测试未检测到此 Bug（重编码也产生有效输出）

**修复方案：** 将 `codec_video`、`codec_audio` 和 `overwrite` 移至 `.output()` 调用**之前**：

```rust
let mut cmd = FfmpegCommand::new();
cmd.seek(format!("{:.3}", start))
    .input(&input_path)
    .duration(format!("{:.3}", duration))
    .codec_video("copy")
    .codec_audio("copy")
    .overwrite()
    .output(&output_path);
```

生成正确的命令行：
```sh
ffmpeg -loglevel level+info -ss 0.000 -i input.mp4 -t 10.000 -c:v copy -c:a copy -y output.mp4
```

---

### CR-02: `spawn_blocking` 超时后 FFmpeg 子进程变成孤儿进程泄漏系统资源

**File:** `src/ffmpeg/command.rs:59`
**Line:**
```rust
tokio::time::timeout(Duration::from_secs(600), tokio::task::spawn_blocking(move || {
    let mut child = cmd.spawn() ... // FFmpeg 子进程在此处启动
    ...
}))
```

**Issue:** `tokio::time::timeout` 包裹 `spawn_blocking`。当 600 秒超时触发时，`JoinHandle` 被 drop，但内部已经在运行的 blocking 线程**不会被取消**（Tokio 不提供 spawn_blocking 任务的强制取消机制）。该线程中的 FFmpeg 子进程继续运行，导致：

1. **文件锁泄漏** — 孤儿 FFmpeg 进程持有输出文件的写入锁，后续操作无法访问或删除该文件
2. **残留进程** — 孤儿进程持续消耗 CPU 和内存
3. **数据损坏** — 调用者收到超时错误后可能尝试读取不完整/损坏的输出文件
4. **资源耗尽** — 批量处理视频时，孤儿进程积累可能导致系统资源耗尽

**影响：** 在长时间运行或批量视频处理场景中，孤儿进程积累会造成确定性资源泄漏。这是功能性 Bug 而非理论风险。

**修复方案：** 使用 `CancellationToken` 在 timeout 和 blocking 任务之间建立协作式取消：

```rust
use tokio_util::sync::CancellationToken;

pub async fn clip_video(
    input: &Path, output: &Path,
    start: f64, duration: f64,
    progress: Option<ProgressCallback>,
) -> Result<(), FFmpegError> {
    let input_path = input.to_string_lossy().to_string();
    let output_path = output.to_string_lossy().to_string();
    let progress = progress.map(Arc::new);
    let cancel = CancellationToken::new();

    let handle = tokio::task::spawn_blocking({
        let cancel = cancel.clone();
        move || {
            let mut cmd = FfmpegCommand::new();
            cmd.seek(format!("{:.3}", start))
                .input(&input_path)
                .duration(format!("{:.3}", duration))
                .codec_video("copy")
                .codec_audio("copy")
                .overwrite()
                .output(&output_path);

            let mut child = cmd.spawn()
                .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

            let iter = match child.iter() {
                Ok(it) => it,
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(FFmpegError::SpawnFailed(e.to_string()));
                }
            };

            for event in iter {
                if cancel.is_cancelled() {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(FFmpegError::Timeout("Cancelled".into()));
                }
                // 处理 FfmpegEvent
            }
            // 正常完成逻辑...
        }
    });

    tokio::select! {
        result = handle => {
            result.map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
        }
        _ = tokio::time::sleep(Duration::from_secs(600)) => {
            cancel.cancel();
            Err(FFmpegError::Timeout("FFmpeg clip_video timed out".into()))
        }
    }
}
```

---

## WARNING Issues

### WR-01: `ffmpeg-sidecar` 的 `download_ffmpeg` 特性引入构建时供应链风险

**File:** `Cargo.toml:13`
**Line:**
```toml
ffmpeg-sidecar = { version = "2.5.1", features = ["download_ffmpeg"] }
```
**Issue:** `download_ffmpeg` 特性会在构建时（如果系统未安装 FFmpeg）从 GitHub Releases 自动下载预编译的 FFmpeg 二进制文件。这是一个外部来源的二进制分发机制，存在以下风险：

1. **供应链攻击面** — 如果 `ffmpeg-sidecar` 的发布流程被攻破，或其引用的 GitHub Release 资源被篡改，用户会在编译阶段执行恶意代码
2. **不可复现构建** — 下载的 FFmpeg 版本取决于下载时间，不同时刻的构建可能获得不同版本的 FFmpeg
3. **离线构建失败** — 无 GitHub 访问权限的环境无法完成构建

**建议：** 将 `download_ffmpeg` 设置为可选 feature，默认关闭：

```toml
[features]
default = []
download-ffmpeg = ["ffmpeg-sidecar/download_ffmpeg"]

[dependencies]
ffmpeg-sidecar = "2.5.1"
```

生产环境应要求系统预装 FFmpeg，并通过 `build.rs` 校验版本。

---

## INFO Issues

### IN-01: 热加载测试 `test_hot_reload` 依赖 OS 文件通知机制，可能不稳定

**File:** `tests/config_test.rs:153-199`
**Issue:** `test_hot_reload` 依赖 `notify` 的文件系统通知，该机制在不同平台上行为不一致：
- Windows 上某些编辑器使用原子保存（写入临时文件后重命名），触发 `Create` + `Remove` 事件而非 `Modify`
- WSL、Docker 或 CI 环境中 inotify 可能不可用或需要额外配置
- 文件系统通知可能被延迟或合并（coalesced）

测试已使用 readiness signal 和轮询模式（50ms 间隔，5s 超时）做了适当防护，但对于不可靠的通知系统仍可能偶发失败。建议添加 `#[ignore]` 标注或列入 CI 的非关键测试套件。

---

### IN-02: error.rs 中 17 个错误消息测试高度重复

**File:** `src/error.rs:150-319`
**Issue:** 每个错误变体对应一个独立测试函数，结构完全一致（创建实例 → 检查 `to_string()` 包含中文）。170 行中超过 90% 是重复样板。新增错误变体时容易遗漏测试。

**建议：** 使用宏合并：

```rust
macro_rules! test_chinese_message {
    ($name:ident, $error:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let msg = $error.to_string();
            assert!(msg.contains($expected), "消息应包含中文: {}", msg);
        }
    };
}
test_chinese_message!(test_config_error_io, ConfigError::IoError("test".into()), "配置文件读取失败");
// 其他测试同理
```

---

### IN-03: ffprobe duration 解析假设值始终为字符串格式

**File:** `src/ffmpeg/probe.rs:58-63`
**Issue:** 代码使用 `.as_str()` 解析 ffprobe JSON 中的 `format.duration`。当前 ffprobe 版本将其作为字符串输出（如 `"120.500000"`），但 JSON 规范中持续时间用数字表示更合理。如果 ffprobe 未来变更输出格式，`.as_str()` 返回 `None` 会导致整个探测失败。

**建议：** 同时尝试字符串和数字解析：

```rust
let duration_secs = json["format"]["duration"]
    .as_str()
    .and_then(|s| s.parse().ok())
    .or_else(|| json["format"]["duration"].as_f64())
    .ok_or_else(|| FFmpegError::OutputParseError(
        "Missing or invalid duration in ffprobe output".into()
    ))?;
```

---

## 文件级别摘要

| 文件 | 行数 | 问题 | 备注 |
|------|------|------|------|
| Cargo.toml | 31 | WR-01 | `download_ffmpeg` 供应链风险 |
| src/lib.rs | 22 | 0 | 结构清晰，模块声明完备 |
| src/error.rs | 321 | IN-02 | 错误类型覆盖完整，测试重复 |
| src/config/types.rs | 363 | 0 | `deny_unknown_fields` 严格但有取舍 |
| src/config/defaults.rs | 113 | 0 | 默认值与 Python 版一致 |
| src/config/mod.rs | 112 | 0 | `RwLock` 使用正确，poison 恢复模式标准 |
| src/config/watcher.rs | 76 | 0 | 热加载逻辑正确，文件名过滤 + 事件过滤完备 |
| src/ffmpeg/mod.rs | 4 | 0 | 模块分发 |
| src/ffmpeg/command.rs | 189 | CR-01, CR-02 | 参数顺序 Bug + 孤儿进程泄漏 |
| src/ffmpeg/probe.rs | 139 | IN-03 | ffprobe JSON 解析健壮性 |
| src/ffmpeg/hwaccel.rs | 268 | 0 | 编码器检测与推荐逻辑正确 |
| tests/config_test.rs | 200 | IN-01 | 热加载测试平台依赖性 |
| tests/ffmpeg_test.rs | 231 | 0 | 覆盖充分 |

---

## 未发现问题的区域

以下方面经确认没有缺陷：

- **配置反序列化** — serde `deny_unknown_fields` + `#[serde(default)]` 组合正确，默认值在 `defaults.rs` 和 `types.rs` 测试中双重验证
- **热加载线程安全** — `Arc<RwLock<AppConfig>>` 跨线程共享正确，poison 恢复使用 `into_inner()` 标准模式
- **文件监听过滤** — `watcher.rs` 按文件名和事件类型双重过滤，避免不相关事件触发重加载
- **编码器检测** — `detect_hw_encoders` 对 FFmpeg 未安装和 ffmpeg 执行失败有不同处理路径
- **测试隔离** — `TempDir` 在 `config/mod.rs` 和 `tests/` 中使用正确，确保测试间不互相污染
- **`spawn_blocking` 并发验证** — `test_spawn_blocking_non_blocking` 使用 Barrier 正确验证并行执行

---

_Reviewed: 2026-04-30T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
