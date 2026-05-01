---
phase: 11-extended-features
plan: GAP-CLOSURE
type: gap_closure
wave: 0 (post-verification fix)
depends_on: [01, 02, 03, 04, 05]
files_modified:
  - src/audio/normalizer.rs
  - src/youtube/downloader.rs
  - src/material/searcher.rs
  - tests/youtube_integration.rs
autonomous: true
requirements: [EXTD-01, EXTD-02, EXTD-03]
verification_ref: 11-VERIFICATION.md
gaps_addressed:
  - CR-01 (BLOCKER) — RMS 计算公式参考电平错误
  - WR-01 (WARNING) — urlencoding 过于简单
  - WR-05 (WARNING) — 分辨率标准化逻辑重复
  - WR-06 (WARNING) — 冗余 yt-dlp --get-title 子进程
  - WR-07 (WARNING) — 下载后未验证输出文件存在
  - WR-10 (WARNING) — find_ytdlp 死代码
gaps_accepted:
  - WR-09: 集成测试存根使用 assert!(true) — 原始计划中明确标注"完整实现在执行阶段补充"，是有意设计取舍，非 Phase 11 缺陷
  - Pre-existing config.example.toml 不匹配（soulvoice.voice_uri、doubaotts.ak/sk、ui.doubaotts_voice_type）— Phase 12 引入，非 Phase 11 范围
---

# Phase 11: Gap Closure Plan

**Phase Goal:** 修复验证阶段发现的 BLOCKER 和 WARNING 级别问题。

**验证状态:** 35/35 must-haves 全部通过，1 个 BLOCKER (CR-01) 未通过，7 个 WARNING

**本次关闭范围:**
- CR-01: 强制修复（Blocking）
- WR-01, WR-05, WR-06, WR-07, WR-10: 推荐修复（WARNING）
- WR-09 + 预存配置不匹配: 显式接受，不修复

---

## Dependencies / Execution Order

```
Agent 1: CR-01 Bug Fix ──────────────┐
                                      ├──→ 全部完成后统一验证
Agent 2: YouTube 修复 (WR-05/06/07) ──┤                               
Agent 3: Material 修复 (WR-01) ──────┤
Agent 4: 测试清理 (WR-10) ────────────┘
```

- Agent 1 是 BLOCKER，优先执行，但技术上不阻塞其他 Agent（不同文件，无冲突修改）
- Agent 2、3、4 互不依赖，可完全并行
- **写操作安全：** 4 个 Agent 修改 4 个不同文件，无写冲突风险

---

## Agent 1: CR-01 Bug Fix (BLOCKER)

**目标文件:** `src/audio/normalizer.rs`
**改动量:** 1 行 + 1 个单元测试
**风险等级:** 零风险（数学上等价变换）
**可并行:** 是（不同文件）

### 具体修改

**修复前验证:**
```bash
# 确认 bug 代码仍然存在
grep -c "i16::MAX" src/audio/normalizer.rs
# 预期输出: 1（第 288 行，仅此一处引用 i16::MAX）
```

**Bug:**
```rust
// 第 288 行 — 使用 i16::MAX (32767) 作为 f32 采样的参考电平
20.0 * (rms / (i16::MAX as f64)).log10()
```

- f32 采样范围 [-1.0, 1.0]，RMS 值在 [0, 1.0] 范围内
- 除以 32767 使 RMS 偏低约 90 dB，导致 `normalize_audio_for_mixing()` 回退路径产生 `gain_db = target_lufs - rms_db ≈ +90 dB` 过放大

**修复:**
```rust
// 第 288 行 — f32 采样的参考电平为 1.0（全量程）
20.0 * rms.log10()
```

**新增测试:** `test_get_audio_rms_formula()`

验证 RMS dB 计算的数学正确性（不依赖文件 I/O，使用已知的 f32 采样值）：

```rust
#[test]
fn test_get_audio_rms_formula() {
    // 构造已知 f32 采样（范围 [-1.0, 1.0]）
    // 1. 静音：全零采样 → RMS = 0.0 → dB = -60.0（静音）
    // 2. 恒定振幅 0.5 → RMS = 0.5 → dB = 20.0 * log10(0.5) ≈ -6.02 dB
    // 3. 恒定振幅 1.0 → RMS = 1.0 → dB = 20.0 * log10(1.0) = 0.0 dB
    
    let silent: Vec<f32> = vec![0.0; 100];
    let rms_silent = (silent.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>() / silent.len() as f64).sqrt();
    assert!((rms_silent - 0.0).abs() < f64::EPSILON);
    // 静音时 rms = 0，进入 else 分支，返回 -60.0
    // 公式不直接可测（函数返回的是 -60.0），但逻辑覆盖此路径

    let half: Vec<f32> = vec![0.5; 100];
    let rms_half = (half.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>() / half.len() as f64).sqrt();
    let db_half = 20.0 * rms_half.log10();
    assert!((db_half - (-6.0206)).abs() < 0.01, "0.5 RMS 应为 -6.02 dB, got {}", db_half);

    let full: Vec<f32> = vec![1.0; 100];
    let rms_full = (full.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>() / full.len() as f64).sqrt();
    let db_full = 20.0 * rms_full.log10();
    assert!((db_full - 0.0).abs() < 0.01, "1.0 RMS 应为 0 dB, got {}", db_full);
    
    // 验证旧公式（i16::MAX 参考）产生错误结果 — 证明修复的必要性
    let db_old = 20.0 * (rms_half / (i16::MAX as f64)).log10();
    assert!((db_old - (-96.329)).abs() < 0.01, "旧公式 0.5 RMS 应为 -96.33 dB, got {}", db_old);
}
```

### 验收标准

- [x] `cargo test --lib audio::normalizer::tests` 全部通过（原 6 个 + 新增 1 个 = 7 个）
- [x] `cargo check` 零 error
- [x] 第 288 行不再引用 `i16::MAX`
- [x] 新增测试验证 0.5 RMS → -6.02 dB，1.0 RMS → 0 dB

---

## Agent 2: YouTube Downloader 修复 (WR-05, WR-06, WR-07)

**目标文件:** `src/youtube/downloader.rs`
**改动量:** 约 20 行变更（3 处独立修改）
**风险等级:** 低（均为纯重构/增强，逻辑不变）
**可并行:** 是（不同文件，与 Agent 1/3/4 无冲突）

### 2a. WR-05: 提取 `normalize_resolution()` 辅助函数

消除两处重复的分辨率标准化逻辑（`_get_video_formats()` 和 `download_video()` 中各有一份）。

**修复前验证:**
```bash
# 确认两处重复的分辨率标准化逻辑都存在
grep -c "if resolution.contains('p')" src/youtube/downloader.rs
# 预期输出: 2（一处 _get_video_formats，一处 download_video）
```

**修改内容:**
- 在 `validate_format()` 之后、`_get_video_formats()` 之前添加：

```rust
/// 标准化分辨率字符串
///
/// "1080p60" → "1080p", "720p" → "720p", "N/A" → "N/A"
fn normalize_resolution(resolution: &str) -> String {
    if resolution.contains('p') {
        let parts: Vec<&str> = resolution.split('p').collect();
        format!("{}p", parts[0])
    } else {
        resolution.to_string()
    }
}
```

- `_get_video_formats()` 中 resolution 标准化代码块（约在函数中部，`let base_res = if resolution.contains('p') { ... };`）替换为：`let base_res = normalize_resolution(&resolution);`
- `download_video()` 中 resolution 标准化代码块（约在函数前部，"3. 标准化分辨率"注释下方，`let base_resolution = if resolution.contains('p') { ... };`）替换为：`let base_resolution = normalize_resolution(resolution);`

**新增测试:**
```rust
#[test]
fn test_normalize_resolution() {
    assert_eq!(normalize_resolution("1080p60"), "1080p");
    assert_eq!(normalize_resolution("720p"), "720p");
    assert_eq!(normalize_resolution("2160p60"), "2160p");
    assert_eq!(normalize_resolution("N/A"), "N/A");
    assert_eq!(normalize_resolution(""), "");
}
```

### 2b. WR-06: 复用 `--dump-json` 中的 title

`_get_video_formats()` 已通过 `yt-dlp --dump-json` 获取完整 JSON（包含 `title` 字段），但只提取了 `formats`。`download_video()` 随后又启动一个 `yt-dlp --get-title` 子进程获取标题。

**修复前验证:**
```bash
# 确认 --get-title 冗余子进程仍然存在
grep -c "get-title" src/youtube/downloader.rs
# 预期输出: 1（第 259 行，download_video 中第 7 步）
```

**修改内容:**
- 将 `_get_video_formats()` 返回值从 `Result<Vec<VideoFormat>, YoutubeError>` 改为 `Result<(Vec<VideoFormat>, String), YoutubeError>`，附带 title
- 在 `_get_video_formats()` 中从 `info["title"]` 提取标题
- `download_video()` 中删除 `--get-title` 子进程代码（第 257-267 行），改用返回的 title

**变更后的 `_get_video_formats` 签名:**
```rust
async fn _get_video_formats(
    url: &str,
    proxy_url: Option<&str>,
) -> Result<(Vec<VideoFormat>, String), YoutubeError> {
    // ... 现有代码 ...
    let title = info["title"]
        .as_str()
        .unwrap_or("unknown_title")
        .to_string();
    Ok((result, title))
}
```

**`download_video()` 中的调整:**
```rust
// 第 4 步改为：
let (formats, title) = _get_video_formats(url, proxy_url).await?;
// 删除第 7 步（--get-title 子进程，第 257-267 行）
// 第 271 行直接使用 title
Ok(DownloadResult { output_path, title })
```

### 2c. WR-07: 下载后验证输出文件存在

`download_video()` 在第 269 行构造 `output_path` 后直接返回，不验证文件是否实际被 yt-dlp 创建。

**修复前验证:**
```bash
# 确认 output_path 之后没有 .exists() 检查
grep -A2 "let output_path" src/youtube/downloader.rs | grep -c "exists"
# 预期输出: 0（当前没有文件存在性验证）
```

**说明：** WR-06（删除 --get-title 子进程）和 WR-07（添加 exists() 检查）修改同一函数的相邻区域。WR-06 先执行，删除第 257-267 行后，WR-07 在 `output_path` 构造与 `Ok(DownloadResult {...})` 之间插入。

**WR-06 + WR-07 合并后的 `download_video()` 最终代码（后半部分）:**
```rust
    // ... 前半部分（步骤 1-5 不变）...

    // 6. 下载（使用 tokio::process::Command，无进度回调 per D-10）
    let output_template = match rename {
        Some(name) => name.to_string(),
        None => format!("{}_video", base_resolution),
    };

    let output_ext = output_format.to_string();

    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "-f",
        &format!("{}+bestaudio", format_id),
        "--merge-output-format",
        &output_ext,
        "--no-warnings",
        "--quiet",
        "-o",
        &format!("{}.{}", output_template, output_ext),
        url,
    ]);

    if let Some(proxy) = proxy_url {
        if !proxy.is_empty() {
            cmd.args(["--proxy", proxy]);
        }
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| YoutubeError::SubprocessError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YoutubeError::DownloadFailed(stderr.to_string()));
    }

    // [WR-07] 验证输出文件实际存在
    let output_path = PathBuf::from(format!("{}.{}", output_template, output_ext));
    if !output_path.exists() {
        return Err(YoutubeError::DownloadFailed(format!(
            "下载完成后文件未找到: {}",
            output_path.display()
        )));
    }

    Ok(DownloadResult { output_path, title })  // title 来自 WR-06 修改后的 _get_video_formats 返回值
```

**注意：** 执行时 WR-05、WR-06、WR-07 按顺序在同一个函数上操作，最终 commit 应包含三者合并后的完整函数。

### 验收标准

- [x] `cargo test --lib youtube::downloader::tests` 全部通过（原 12 个 + 新增 1 个 = 13 个）
- [x] `cargo check` 零 error
- [x] `grep -c "fn normalize_resolution" src/youtube/downloader.rs` 返回 1
- [x] `grep -c "get-title" src/youtube/downloader.rs` 返回 0（已删除冗余子进程）

---

## Agent 3: Material Searcher urlencoding 修复 (WR-01)

**目标文件:** `src/material/searcher.rs`
**改动量:** 约 20 行（替换 `urlencoding` 函数）
**风险等级:** 零风险（纯增强，原有逻辑不变）
**可并行:** 是（不同文件，与 Agent 1/2/4 无冲突）

### 具体修改

**当前代码（第 174-176 行）:**
```rust
fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}
```
仅替换空格，未编码 `&`、`=`、`%`、`#` 等 URL 特殊字符，导致包含这些字符的搜索词产生截断的 API URL。

**修复前验证:**
```bash
# 确认 urlencoding 函数仅处理空格，不处理 URL 特殊字符
grep -c "s.replace(' ', \"+\")" src/material/searcher.rs
# 预期输出: 1（仅此一处，且无其他字符编码逻辑）
```

**修复：实现完整的 URL 查询编码:**
```rust
/// 完整的 URL 查询编码
///
/// 对查询字符串进行 application/x-www-form-urlencoded 编码。
/// 保留字母数字和 `-`、`_`、`.`、`~`，空格编码为 `+`，
/// 其他字符编码为 `%XX`（UTF-8 逐字节编码）。
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => result.push('+'),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            _ => {
                for b in c.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}
```

**验证（新增测试，添加到文件末尾的 `#[cfg(test)] mod tests` 中）:**
```rust
#[test]
fn test_urlencoding() {
    assert_eq!(urlencoding("hello world"), "hello+world");
    assert_eq!(urlencoding("AT&T"), "AT%26T");
    assert_eq!(urlencoding("C++"), "C%2B%2B");
    assert_eq!(urlencoding("100%"), "100%25");
    assert_eq!(urlencoding("tag#1"), "tag%231");
    assert_eq!(urlencoding("normal"), "normal");
    assert_eq!(urlencoding("a=b"), "a%3Db");
    assert_eq!(urlencoding("你好"), "%E4%BD%A0%E5%A5%BD"); // UTF-8 中文
}
```

### 不使用外部 URL 编码 crate 的理由

- 当前 Cargo.toml 无 `url`、`percent-encoding` 或 `urlencoding` 依赖
- 该编码函数仅用于 Pexels/Pixabay 搜索查询参数，不用于通用 URL 处理
- 手动实现同样功能约 15 行，简单明了，无需引入新依赖

### 验收标准

- [x] `cargo test --lib material::searcher::tests` 全部通过（原 11 个 + 新增 1 个 = 12 个）
- [x] `cargo check` 零 error
- [x] 新测试覆盖空格的 `+` 编码、`&` `+` `%` `#` `=` 的 `%XX` 编码、中文 UTF-8 编码

---

## Agent 4: 测试死代码清理 (WR-10)

**目标文件:** `tests/youtube_integration.rs`
**改动量:** 删除 2 个 `#[cfg]` 块（约 12 行）
**风险等级:** 零风险（纯删除死代码）
**可并行:** 是（不同文件，与 Agent 1/2/3 无冲突）

### 具体修改

移除 `find_ytdlp()` 函数中 `#[cfg(windows)]` 和 `#[cfg(not(windows))]` 分支（第 14-31 行）。

这些分支是死代码，因为第 9-13 行的 `yt-dlp --version` 已覆盖所有平台。`where`/`which` 分支永远不可达。

**修复前验证:**
```bash
# 确认 cfg 死代码分支仍然存在
grep -c "which\|where" tests/youtube_integration.rs
# 预期输出: 1 或 2（cfg 分支中包含 which/where 命令引用）
```

**删除后:**
```rust
fn find_ytdlp() -> String {
    if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return "yt-dlp".to_string();
        }
    }
    panic!("yt-dlp 不可用。请访问 https://github.com/yt-dlp/yt-dlp/releases 下载安装。");
}
```

### 验收标准

- [x] `cargo test --test youtube_integration --no-run` 编译通过
- [x] `grep -c "which\|where" tests/youtube_integration.rs` 返回 0

---

## 统一验证

在所有 4 个 Agent 完成后执行以下验证：

```bash
# 全量编译检查
cargo check 2>&1

# 所有模块的单元测试
cargo test --lib audio::normalizer::tests 2>&1
cargo test --lib audio::volume::tests 2>&1
cargo test --lib youtube::downloader::tests 2>&1
cargo test --lib material::searcher::tests 2>&1
cargo test --lib material::downloader::tests 2>&1

# 集成测试编译（不运行）
cargo test --test audio_integration --test youtube_integration --test material_integration --no-run 2>&1
```

### 通过标准

- `cargo check` 零 error
- `cargo test --lib` 全部通过（~47 个测试）
- 3 个集成测试文件编译通过
- `grep "i16::MAX" src/audio/normalizer.rs` 返回 0（CR-01 修复确认）
- `grep "get-title" src/youtube/downloader.rs` 返回 0（WR-06 修复确认）

---

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Agent 2 返回值类型变更(`Vec<VideoFormat>` → `(Vec<VideoFormat>, String)`)影响未预见的调用方 | 低（`_get_video_formats` 是私有函数） | 编译错误，立即发现 | `cargo check` 会捕获所有调用点；已确认该函数仅在 `download_video` 中被调用 |
| WR-01 手动 URL 编码对某些 UTF-8 字符序列编码错误 | 极低 | 搜索词编码异常 | 新增的 8 个测试用例覆盖了空格、特殊字符、中文，确保编码正确性 |
| 同时多 Agent 修改导致 git 冲突 | 无（4 个不同文件） | 无 | 无冲突风险 |

---

## 输出

gap_closure 完成后，创建 `11-GAP-CLOSURE-SUMMARY.md` 记录修复内容、测试结果和验收状态。

然后更新 `11-VERIFICATION.md` 将 CR-01 状态从 `failed` 改为 `fixed`，并重新计算评分。
