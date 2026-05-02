---
phase: 11-extended-features
reviewed: 2026-05-01T18:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - tests/audio_integration.rs
  - tests/youtube_integration.rs
  - tests/material_integration.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 11: 集成测试文件 Code Review Report

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

审查了 3 个集成测试文件（`audio_integration.rs`、`youtube_integration.rs`、`material_integration.rs`）。这些文件均为 `#[ignore]` 标记的集成测试，设计为通过 `cargo test -- --ignored` 手动运行。

主要发现：
- **2 个 Critical 问题**：`material_integration.rs` 中 API key 环境变量名不一致（`PEXELS_API_KEYS` vs 源代码实际使用的复数键数组），且 API key 被读取后未使用，存在信息泄露风险；`audio_integration.rs` 和 `youtube_integration.rs` 中存在存根断言 `assert!(true)` 伪装为真实测试。
- **3 个 Warning 问题**：`find_ffmpeg` 实现不一致且缺少 sidecar 缓存检测；环境变量优先级（复数名在前）与实际使用模式不符；`youtube_integration.rs` 缺少网络错误/无效 URL 的负面测试。
- **2 个 Info 问题**：`find_ytdlp` 的 panic 消息可改进；`material_integration.rs` 中注释与代码意图不完全匹配。

---

## Critical Issues

### CR-01: API key 环境变量名优先级与实际使用模式不匹配，且 key 被读取但未使用

**File:** `tests/material_integration.rs:9-12`

```rust
let api_key = std::env::var("PEXELS_API_KEYS")
    .or_else(|_| std::env::var("PEXELS_API_KEY"))
    .expect("需要设置 PEXELS_API_KEYS 或 PEXELS_API_KEY 环境变量");
```

**Issue:** 存在两个问题叠加：

1. **变量名优先级反转**：代码优先读取 `PEXELS_API_KEYS`（复数），回退到 `PEXELS_API_KEY`（单数）。但源代码中 `search_videos_pexels()` 接收的是 `api_keys: &[String]`（复数数组），且 `MaterialDownloader` 在 `config.example.toml` 中配置的键名为 `api_keys_pexels`（复数）。实际生产环境使用的是复数形式（支持多 key 轮转），测试中应与之对齐。优先尝试单数名而回退到复数名会导致：用户按文档设置复数名 `PEXELS_API_KEYS` 时测试通过，但单数名 `PEXELS_API_KEY` 才是许多 CI 系统的标准约定。这种不一致增加了配置出错的风险。

2. **Key 被读取但完全未使用**：`api_key` 变量在 `test_pexels_search_integration` 中被绑定后，仅检查非空（第 12 行），然后测试即以 `assert!(true)` 结束（第 23 行注释表明"完整实现在执行阶段补充"）。这意味着 API key 被加载到内存中但没有任何实际用途。虽然当前代码不会泄露 key，但这为后续实现埋下了隐患——开发者可能在"补充完整实现"时不经意间将 key 打印到日志或断言消息中。`test_pixabay_search_integration`（第 19-26 行）存在完全相同的问题。

**Fix:**

```rust
#[test]
#[ignore]
fn test_pexels_search_integration() {
    // 与源代码 MaterialDownloader 一致，使用复数形式
    let api_key = std::env::var("PEXELS_API_KEYS")
        .expect("需要设置 PEXELS_API_KEYS 环境变量（支持逗号分隔的多个 key）");
    let keys: Vec<String> = api_key.split(',').map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert!(!keys.is_empty(), "PEXELS_API_KEYS 不能为空");

    // TODO: 调用 search_videos_pexels("test", 5, 1920, 1080, &keys, &counter, &proxy)
    //       验证返回 Vec<MaterialInfo> 非空
}
```

对于 Pixabay 测试同理。同时，移除单数形式 `PEXELS_API_KEY` / `PIXABAY_API_KEY` 的回退，避免维护两套环境变量名约定。

### CR-02: 两个测试文件使用 `assert!(true)` 存根断言，伪装测试通过

**Files:** `tests/audio_integration.rs:23,31` 和 `tests/youtube_integration.rs:36`

```rust
// audio_integration.rs:23
assert!(true, "存根 — 执行阶段填充具体测试逻辑");

// audio_integration.rs:31
assert!(true, "存根 — 执行阶段填充具体测试逻辑");

// youtube_integration.rs:36
assert!(true, "存根 — 执行阶段填充具体测试逻辑");
```

**Issue:** 3 个测试函数体完全由 `assert!(true)` 组成，不执行任何实际验证。这些测试通过 `#[ignore]` 标记不会被常规 `cargo test` 运行，但：

1. 当开发者使用 `cargo test -- --ignored` 手动运行时，这些测试会报告 **PASS**，给人以"集成测试全部通过"的虚假信心。
2. `assert!(true)` 是一个永真断言，无论系统状态如何都会通过。如果 FFmpeg 不可用或网络断开，测试仍然报告成功。
3. `find_ffmpeg()` 的返回值被绑定到 `_ffmpeg`（下划线前缀表示未使用），`find_ytdlp()` 同理——调用 helper 函数的副作用（panic on missing）在当前实现中是唯一的"检测"，但如果 FFmpeg 存在但损坏，测试仍然通过。

这是一个正确性问题：测试声称验证了某个功能，但实际上什么都没验证。

**Fix:**

方案 A（推荐）：将存根测试改为 `todo!()` 宏，使其在运行时 panic 并明确标注未实现：

```rust
#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    let _ffmpeg = find_ffmpeg();
    todo!("集成测试：使用 ffmpeg 生成测试音频 -> normalize_lufs -> 验证输出");
}
```

方案 B：如果保留 `assert!(true)` 格式，至少添加一个实际的环境检测断言：

```rust
#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    let ffmpeg = find_ffmpeg();  // 移除下划线前缀
    assert!(ffmpeg.len() > 0, "FFmpeg 路径不应为空");
    // TODO: 实际测试逻辑
}
```

---

## Warnings

### WR-01: `find_ffmpeg` 实现与 `tests/ffmpeg_test.rs` 不一致，缺少 sidecar 缓存检测

**File:** `tests/audio_integration.rs:8-15`

```rust
fn find_ffmpeg() -> String {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return "ffmpeg".to_string();
        }
    }
    panic!("FFmpeg 不可用。请安装 FFmpeg 或等待 ffmpeg-sidecar 自动下载。");
}
```

**Issue:** `tests/ffmpeg_test.rs:14-34` 中的 `find_ffmpeg()` 返回 `PathBuf`，且包含两个检测步骤（PATH 检测 + ffmpeg-sidecar 缓存路径检测）。`audio_integration.rs` 中的版本仅检查 PATH，返回 `String`，且 panic 消息中提到"等待 ffmpeg-sidecar 自动下载"但实际并未检测 sidecar 缓存路径。

如果 FFmpeg 仅通过 ffmpeg-sidecar 下载（不在 PATH 中），`audio_integration.rs` 中的测试会不必要地 panic，而 `ffmpeg_test.rs` 中的测试可以正常运行。

**Fix:**

```rust
use std::path::PathBuf;

fn find_ffmpeg() -> PathBuf {
    // 1. 先检查 PATH
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return PathBuf::from("ffmpeg");
        }
    }
    // 2. 检查 ffmpeg-sidecar 缓存路径
    let cached = ffmpeg_sidecar::paths::ffmpeg_path();
    if cached.exists() {
        return cached;
    }
    panic!(
        "FFmpeg 不可用。请安装 FFmpeg 并确保在 PATH 上，\n\
         或运行以下命令让 ffmpeg-sidecar 自动下载:\n\
         use ffmpeg_sidecar::download::auto_download;\n\
         auto_download().unwrap();"
    );
}
```

或者更好的做法是将 `find_ffmpeg` 提取到 `tests/common/mod.rs` 共享模块中，避免两个测试文件各自维护一个不一致的实现。

### WR-02: `youtube_integration.rs` 缺少网络错误和无效 URL 的负面测试用例

**File:** `tests/youtube_integration.rs`（整个文件）

**Issue:** 该文件仅包含 2 个测试：`test_ytdlp_version`（验证版本号非空）和 `test_get_video_formats`（存根）。完全缺少以下负面测试场景：

1. **无效 URL**：`get_video_formats` 是 `async fn`，源代码在 `src/youtube/downloader.rs:107` 有 URL 前缀校验（`http://` / `https://`），但没有测试验证 `ftp://` 或 `file://` 等 URL 被正确拒绝。
2. **不存在/不可达 URL**：网络超时或 404 响应时的错误处理。
3. **yt-dlp 不可用**：虽然有 `find_ytdlp()` 的 panic 检测，但没有测试验证当 yt-dlp 不存在时的优雅降级行为。

当前 `test_get_video_formats` 是存根（WR-02 覆盖），但即使补充完整实现，也应包含对应的负面测试。

**Fix:**

```rust
#[tokio::test]
#[ignore]
fn test_get_video_formats_invalid_url() {
    let _yt_dlp = find_ytdlp();
    // 验证非 http(s) URL 返回 InvalidUrl 错误
    // 注意：get_video_formats 是 private，需要通过公共 API 或集成测试 helper 调用
}

#[tokio::test]
#[ignore]
fn test_get_video_formats_unreachable_url() {
    let _yt_dlp = find_ytdlp();
    // 验证对不存在域名（如 http://nonexistent.invalid/video）返回适当的错误
}
```

### WR-03: `material_integration.rs` 未验证 API key 格式有效性

**File:** `tests/material_integration.rs:12,22`

```rust
assert!(!api_key.is_empty(), "API key 不能为空");
```

**Issue:** 仅检查 API key 非空是不够的。Pexels API key 是 56 个字符的十六进制字符串，Pixabay API key 是特定格式的字母数字字符串。空字符串检查无法捕获以下情况：

1. 环境变量设置为空格或空白字符（如 `PEXELS_API_KEYS="  "`）。
2. 环境变量包含分隔符但无实际 key（如 `PEXELS_API_KEYS=","`）。
3. 开发者误将 placeholder 值（如 `"YOUR_API_KEY_HERE"`）设置为环境变量。

在这些情况下，测试会在后续（当完整实现补充后）因 API 认证失败而报错，但错误消息不直观——一个提前的格式校验能更快定位配置问题。

**Fix:**

```rust
assert!(!api_key.trim().is_empty(), "PEXELS_API_KEYS 不能为空或纯空白");
// 可选：基本格式检查
let keys: Vec<&str> = api_key.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
for key in &keys {
    assert!(key.len() >= 20, "Pexels API key 长度异常（至少 20 字符）: {}...", &key[..key.len().min(8)]);
}
```

---

## Info

### IR-01: `find_ytdlp` panic 消息缺少可操作的排查步骤

**File:** `tests/youtube_integration.rs:14`

```rust
panic!("yt-dlp 不可用。请访问 https://github.com/yt-dlp/yt-dlp/releases 下载安装。");
```

**Issue:** Panic 消息仅提供了 GitHub releases 页面链接，但未提及常见的安装方式（pip install、winget、scoop、choco 等）。在 Windows 上（项目的目标平台之一），`pip install yt-dlp` 或 `winget install yt-dlp.yt-dlp` 更方便。

**Fix:**

```rust
panic!(
    "yt-dlp 不可用。安装方式:\n\
     - pip install yt-dlp\n\
     - winget install yt-dlp.yt-dlp\n\
     - 或访问 https://github.com/yt-dlp/yt-dlp/releases 手动下载"
);
```

### IR-02: `audio_integration.rs` 中 `find_ffmpeg` 返回值未使用但保留了调用

**File:** `tests/audio_integration.rs:20,29`

```rust
let _ffmpeg = find_ffmpeg();
```

**Issue:** `find_ffmpeg()` 的调用目的是在测试开始前检测 FFmpeg 是否可用（不可用时 panic）。使用 `_ffmpeg` 绑定是正确的（避免 unused 警告），但结合 CR-02（存根断言），整个测试的实际效果等价于：

```rust
#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    find_ffmpeg(); // 仅此一行有实际效果
    assert!(true); // 无意义断言
}
```

如果测试的唯一目的是检测 FFmpeg 是否可用，应该明确命名为 `test_ffmpeg_available`，而不是伪装成 `test_normalize_lufs_two_pass`。

---

_Reviewed: 2026-05-01_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
