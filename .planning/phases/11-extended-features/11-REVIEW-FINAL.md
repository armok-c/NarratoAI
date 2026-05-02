---
phase: 11-extended-features
reviewed: 2026-05-01T18:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src/material/mod.rs
  - src/material/searcher.rs
  - src/material/downloader.rs
findings:
  critical: 1
  warning: 5
  info: 2
  total: 8
status: issues_found
---

# Phase 11: Code Review Report (Material Search & Download Module)

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

对 `src/material/` 模块进行标准深度审查，聚焦素材搜索下载的三个核心文件。发现 1 个 Critical 问题（Pixabay API key 泄露到 URL 查询参数中，可被日志和中间代理捕获）、5 个 Warning 问题（原子写入非原子、Pexels 精确分辨率过滤导致结果为空、缓存命中 TOCTOU 竞态、累计时长使用搜索值而非实际值、Pixabay 未区分认证错误）、2 个 Info 问题（未使用的导入、客户端每次请求重建）。

## Critical Issues

### CR-01: Pixabay API key 通过 URL 查询参数传输，存在泄露风险

**File:** `src/material/searcher.rs:279-283`
**Issue:** Pixabay API key 被直接拼接到 URL 查询字符串中（`?key=...`）。虽然代码注释声称"Pixabay API 设计限制"（确实是其 API 要求），但这意味着：
1. URL 会被 reqwest 内部日志记录（tracing::info/debug 级别可能暴露完整 URL）
2. HTTP 代理（特别是配置了 `http`/`https` 代理时）可以完整看到请求 URL，包括 API key
3. DNS 查询、CDN 日志等中间环节也可能泄露查询参数
4. `tracing::info!("下载素材: {}", video_url)` (downloader.rs:70) 虽然此处不涉及 Pixabay key，但搜索函数中任何级别的 URL 日志都会泄露 key

Pixabay 确实只支持 URL 参数传 key（不支持 Header），但代码应在请求前对 URL 中的 key 做脱敏处理，并确保任何日志输出不包含完整 URL。

**Fix:**
```rust
// searcher.rs - 在发送请求前确保不记录完整 URL
// 方案 1: 不记录完整 API URL（当前代码已做到，但应加注释明确禁止）
// 方案 2: 在 tracing 级别上禁止 debug 级别输出 API URL

// 在 build_client 中添加日志过滤器，或使用 reqwest 的 tracing subscriber 排除敏感 URL
// 最小改动: 在 search_videos_pixabay 中添加明确的日志安全注释

// 同时建议在 config.example.toml 中添加安全提示：
// [pixabay]
// api_keys = [...]  # 注意: Pixabay API 通过 URL 参数传输 key，请确保代理可信
```

## Warnings

### WR-01: `std::fs::rename` 在 Windows 上非原子操作，可能导致数据损坏

**File:** `src/material/downloader.rs:98`
**Issue:** 代码注释称"原子重命名"，但 `std::fs::rename` 在 Windows 上跨目录时实际上是复制+删除操作，而非真正的原子 rename。如果进程在复制完成后、删除前崩溃，会导致同时存在 `.tmp` 和 `.mp4` 文件（浪费磁盘但不丢失数据）。更严重的是，如果另一个进程在 rename 之前读取到不完整的 `.mp4` 文件（由于 Windows 的文件锁定语义不同于 POSIX），可能导致读取到部分写入的数据。

当前代码 temp 文件和最终文件在同一目录，Windows 上同目录 rename 是原子的，所以这个风险有限。但代码注释"原子重命名"具有误导性。

**Fix:**
```rust
// 修正注释，准确描述行为
// 6. 重命名临时文件为最终路径（同目录内，POSIX 原子；Windows 同卷原子）
std::fs::rename(&temp_path, &video_path)
    .map_err(|e| MaterialError::IoError(e.to_string()))?;
```

### WR-02: Pexels 搜索使用精确分辨率匹配（`==`），极易返回空结果

**File:** `src/material/searcher.rs:249`
**Issue:** Pexels API 返回的 `video_files` 中每个文件的分辨率是固定值（如 1920x1080, 1280x720, 3840x2160 等）。当前代码使用 `f.width == target_width && f.height == target_height` 做精确匹配。如果调用者传入 `target_width=1920, target_height=1080` 但 Pexels 只返回了 `1920x1080` 的 `sd` 质量和 `3840x2160` 的 `hd` 质量，精确匹配会错过所有结果。

此外，`video_files` 可能为空列表（API 返回的视频没有任何可下载文件），此时 `flat_map` 会正确产生空结果，但调用者无法区分"没有匹配分辨率"和"API 没有返回视频"。

**Fix:**
```rust
// 改为选择最接近目标分辨率的 variant，而非精确匹配
// 类似 Pixabay 的 choose_variant 逻辑
fn choose_closest_pexels_file<'a>(
    files: &'a [PexelsVideoFile],
    target_w: u32,
    target_h: u32,
) -> Option<&'a PexelsVideoFile> {
    files.iter().min_by_key(|f| {
        let dw = (f.width as i32 - target_w as i32).unsigned_abs();
        let dh = (f.height as i32 - target_h as i32).unsigned_abs();
        dw * dw + dh * dh  // 欧氏距离平方，避免浮点
    })
}
```

### WR-03: 缓存命中检查存在 TOCTOU 竞态条件

**File:** `src/material/downloader.rs:59-67`
**Issue:** `video_path.exists()` 和后续的 `std::fs::metadata()` 是两次独立的文件系统调用。在两次调用之间，文件可能被另一个进程删除或截断。虽然这在单进程场景下极不可能发生，但如果 `save_video` 被多线程并发调用（通过 `download_videos` 的搜索+下载流水线），同一个 video_url 的两次并发调用可能同时通过缓存检查，导致重复下载。

更重要的是：缓存命中时直接返回路径，但没有验证文件内容是否是有效的视频文件（损坏文件不会在缓存命中时被检测到）。

**Fix:**
```rust
// 方案 1: 对缓存命中也做基础验证
if video_path.exists() {
    match probe_audio(&video_path) {
        Ok(dur) if dur > 0.0 => {
            tracing::info!("素材缓存命中: {}", video_path.display());
            return Ok(video_path);
        }
        _ => {
            tracing::warn!("缓存文件损坏，重新下载: {}", video_path.display());
            let _ = std::fs::remove_file(&video_path);
        }
    }
}
// 缺点: 每次缓存命中都要调用 ffprobe，增加启动开销

// 方案 2: 仅检查文件大小（当前方案已足够，加注释说明权衡）
```

### WR-04: `download_videos` 使用搜索 API 返回的时长而非 ffprobe 实际时长

**File:** `src/material/downloader.rs:203`
**Issue:** 代码注释明确说"使用搜索 API 返回的时长而非 ffprobe 实际时长（与 Python 版行为一致）"。但 `save_video` 已经调用了 `probe_audio` 获取了实际时长却丢弃了结果。这意味着如果搜索 API 返回的时长与实际视频时长不一致（例如 API 报告 15 秒，实际视频 12 秒），累计时长计算会不准确，可能导致下载的视频总时长不足 `audio_duration`。

此外，`save_video` 中 `probe_audio` 的 `duration <= 0.0` 检查与 `probe_audio` 函数内部的 `duration <= 0.0` 检查重复（probe.rs:152 已做此校验并返回 Err），所以 `save_video` 的 `Ok(duration) if duration <= 0.0` 分支永远不会触发。

**Fix:**
```rust
// save_video 应返回实际时长，让 download_videos 使用准确值
pub fn save_video(
    video_url: &str,
    save_dir: &Path,
    proxy_section: &ProxySection,
) -> Result<(PathBuf, f64), MaterialError> {  // 返回 (路径, 实际时长)
    // ... 下载逻辑 ...
    match probe_audio(&video_path) {
        Ok(duration) => {
            tracing::info!("素材下载完成: {} ({}s)", video_path.display(), duration);
            Ok((video_path, duration))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&video_path);
            Err(MaterialError::ValidationFailed(format!("ffprobe 验证失败: {}", e)))
        }
    }
}

// download_videos 中:
match save_video(&item.video_url, &save_dir, proxy_section) {
    Ok((path, actual_duration)) => {
        total_duration += actual_duration;  // 使用实际时长
        downloaded_paths.push(path);
    }
    // ...
}
```

### WR-05: Pixabay 搜索未区分认证失败（401）和其他 HTTP 错误

**File:** `src/material/searcher.rs:292-294`
**Issue:** Pexels 搜索对 401 状态码做了专门处理（`searcher.rs:229-232`），返回 `MaterialError::AuthenticationFailed`。但 Pixabay 搜索对所有非成功状态码统一返回 `MaterialError::ApiRequest`。当 Pixabay API key 无效时，错误信息不够明确，用户难以判断是 key 配置错误还是网络问题。

**Fix:**
```rust
if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
    return Err(MaterialError::AuthenticationFailed(
        "Pixabay API key 无效".into(),
    ));
}
if !response.status().is_success() {
    return Err(MaterialError::ApiRequest(format!("HTTP {}", response.status())));
}
```

## Info

### IN-01: `reqwest::blocking::Client` 每次请求都重新构建

**File:** `src/material/searcher.rs:221,285` 和 `src/material/downloader.rs:71`
**Issue:** `build_client()` 在每次搜索和下载调用时都创建新的 `reqwest::blocking::Client`。虽然 `reqwest::blocking::Client` 内部维护连接池，每次创建新实例意味着连接池无法跨请求复用。在 `download_videos` 的循环中（逐个下载多个视频），每次下载都会创建新 Client，导致 TCP 连接无法 keep-alive 复用，增加连接建立开销。

项目其他模块（TTS 引擎、LLM 服务）都在初始化时创建一次 Client 并复用。

**Fix:**
将 `build_client` 的结果作为参数传入 `download_videos`，在调用层缓存 Client 实例。或者让 `download_videos` 内部只创建一次 Client。

### IN-02: `downloader.rs` 导入了未使用的 `AtomicUsize`

**File:** `src/material/downloader.rs:8`
**Issue:** `use std::sync::atomic::AtomicUsize;` 在 `downloader.rs` 中被导入，但该文件本身并未直接使用 `AtomicUsize`（仅在测试中通过 `super::*` 间接引用）。非测试代码中不需要此导入。

**Fix:**
```rust
// 移除非必要的导入，或将导入移到 #[cfg(test)] 块中
// 如果仅在测试中使用:
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
```

---

_Reviewed: 2026-05-01T18:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
