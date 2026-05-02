---
phase: 11-extended-features
reviewed: 2026-05-01T19:30:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src/youtube/mod.rs
  - src/youtube/downloader.rs
  - tests/youtube_integration.rs
findings:
  critical: 1
  warning: 4
  info: 2
  total: 7
status: issues_found
---

# Phase 11: YouTube 下载模块代码审查报告

**Reviewed:** 2026-05-01
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

审查了 YouTube 下载模块的 3 个文件，包括 `downloader.rs`（核心下载逻辑）、`mod.rs`（模块声明）和集成测试。代码整体质量良好：CLI 参数使用 `args` 数组传递（无 shell 注入风险）、URL 前缀校验到位、错误类型覆盖较全。发现 1 个 Critical 级别问题（`rename` 参数路径遍历）、4 个 Warning 级别问题（URL 验证不防御空白字符、格式选择 fallback 冗余查找、代理 URL 未校验、使用 format_note 而非 resolution 字段）、2 个 Info 级别问题。

注意：前几轮审查中的 WR-18（分辨率回退缺少日志）已在代码中修复（第 227-232 行添加了 `tracing::warn!`），本轮验证通过。

## Critical Issues

### CR-01: `rename` 参数未做路径遍历校验，允许写入任意相对路径

**File:** `src/youtube/downloader.rs:239-255`
**Issue:** `rename` 参数直接作为 `yt-dlp -o` 的输出模板，未过滤路径分隔符。攻击者可传入 `"../../../tmp/evil"` 作为 rename 值，导致 yt-dlp 将下载的视频写入 CWD 之外的任意位置。第 275 行的 `PathBuf::from` 和第 278 行的 `output_path.exists()` 会使用遍历后的路径，确认写入成功后返回。

具体攻击路径：
1. 调用 `download_video(url, "1080p", "mp4", Some("../../../tmp/evil"), None)`
2. `output_template = "../../../tmp/evil"`（第 239 行）
3. `yt-dlp -o "../../../tmp/evil.mp4" url`（第 255 行）-- 写入 CWD 之外的文件
4. `PathBuf::from("../../../tmp/evil.mp4")`（第 275 行）-- 路径遍历
5. `output_path.exists()` 返回 true（第 278 行）-- 确认写入成功
6. 返回 `Ok(DownloadResult { output_path: "../../../tmp/evil.mp4", ... })`（第 285 行）

**Fix:**
```rust
let output_template = match rename {
    Some(name) => {
        // 防止路径遍历：拒绝包含路径分隔符的文件名
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(YoutubeError::InvalidUrl(
                "rename 参数不允许包含路径分隔符或 '..'".into(),
            ));
        }
        name.to_string()
    }
    None => format!("{}_video", base_resolution),
};
```

## Warnings

### WR-01: URL 验证未过滤空白字符和换行符

**File:** `src/youtube/downloader.rs:107-109, 193-195`
**Issue:** URL 前缀校验仅检查 `starts_with("http://")` 和 `starts_with("https://")`，未过滤 URL 中的换行符、回车符或尾随空格。虽然 `tokio::process::Command` 使用 `args` 数组传递参数（不经过 shell 解释），因此不构成命令注入，但含换行符的 URL 会被传递给 yt-dlp，yt-dlp 会因无效 URL 报错，错误信息不易追溯。

更重要的是，如果 `proxy_url` 参数（第 114-117 行和第 259-262 行）包含换行符或恶意内容，同样会原样传递给 `--proxy` 参数。虽然不构成命令注入，但属于纵深防御缺失。

**Fix:**
```rust
// 在 URL 校验处添加 trim 和换行符检查
let trimmed = url.trim();
if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
    return Err(YoutubeError::InvalidUrl(url.to_string()));
}
if trimmed.contains('\n') || trimmed.contains('\r') {
    return Err(YoutubeError::InvalidUrl("URL 不允许包含换行符".into()));
}
// 后续使用 trimmed 而非原始 url
```

### WR-02: 格式选择 fallback 逻辑中第二次查找冗余，且存在逻辑不一致

**File:** `src/youtube/downloader.rs:216-234`
**Issue:** fallback 路径中，第 216-219 行先通过 `find(|f| f.vcodec != "none")` 找到第一个非纯音频格式的 `format_id`（克隆为 `String`），然后第 226 行又用 `formats.iter().find(|f| f.format_id == fallback)` 做第二次查找来获取 `&VideoFormat` 用于日志输出。两次查找逻辑等价，第二次查找理论上不可能返回 `None`（因为 `fallback` 来自第一次查找的结果），但代码仍然使用 `if let Some(fb)` 防御。

这不仅是代码冗余（O(n) 变 O(2n)），更是一个逻辑问题：第一次 `find` 返回的是 `Option<String>`，丢失了 `&VideoFormat` 引用，导致需要第二次查找。应直接保留引用。

**Fix:**
```rust
None => {
    // fallback: 匹配任何有视频的格式
    let fallback_fmt = formats
        .iter()
        .find(|f| f.vcodec != "none")
        .ok_or_else(|| {
            YoutubeError::FormatNotFound(format!(
                "无可用视频格式（目标分辨率: {}）",
                base_resolution
            ))
        })?;

    tracing::warn!(
        "目标分辨率 {} 无匹配格式，回退到: {} ({})",
        base_resolution,
        fallback_fmt.format_id,
        fallback_fmt.resolution
    );
    fallback_fmt.format_id.clone()
}
```

### WR-03: `proxy_url` 参数未做格式校验

**File:** `src/youtube/downloader.rs:114-117, 259-262`
**Issue:** `proxy_url` 仅检查 `!proxy.is_empty()` 后直接传给 `yt-dlp --proxy`。未验证是否为合法的代理 URL 格式（如 `http://host:port`、`socks5://host:port`）。如果配置文件中代理值为非法字符串（如 `"not-a-proxy"`），yt-dlp 会报错，但错误信息为 yt-dlp 内部错误，不够友好。

此外，与 URL 参数不同，`proxy_url` 完全没有前缀校验，允许任意字符串传入（包括含换行符的字符串，见 WR-01）。

**Fix:**
```rust
if let Some(proxy) = proxy_url {
    if !proxy.is_empty() {
        // 基本代理 URL 格式校验
        let valid_schemes = ["http://", "https://", "socks5://", "socks5h://"];
        if !valid_schemes.iter().any(|s| proxy.starts_with(s)) {
            return Err(YoutubeError::SubprocessError(format!(
                "无效的代理 URL: {}（支持: http, https, socks5, socks5h）",
                proxy
            )));
        }
        cmd.args(["--proxy", proxy]);
    }
}
```

### WR-04: 使用 `format_note` 而非 `resolution` 字段解析分辨率，匹配不健壮

**File:** `src/youtube/downloader.rs:140`
**Issue:** yt-dlp `--dump-json` 输出中，`format_note` 和 `resolution` 是两个不同字段。`format_note` 是用户可读的标签（如 `"1080p"`, `"720p60"`, `"tiny"`, `"medium"`, `"DASH video"`），而 `resolution` 是标准化的 `"WIDTHxHEIGHT"` 格式（如 `"1920x1080"`）。代码使用 `format_note` 作为分辨率匹配依据，这依赖于 yt-dlp 的标签命名约定，不够健壮。

当 `format_note` 为非标准值（如 `"DASH video"`、空字符串、`"only audio"`）时，`normalize_resolution` 不会修改它，但也不会匹配任何用户请求的分辨率（如 `"1080p"`），导致格式选择静默失败进入 fallback 路径。

**Fix:** 同时解析 `resolution` 字段（`"WIDTHxHEIGHT"` 格式）作为备选匹配依据，或在 `format_note` 为非标准值时使用 `height` 字段构造分辨率字符串：
```rust
let resolution = f["format_note"]
    .as_str()
    .filter(|s| !s.is_empty() && *s != "tiny" && *s != "small" && *s != "medium")
    .map(|s| s.to_string())
    .or_else(|| {
        // 回退到 resolution 字段（"1920x1080" 格式）
        f["resolution"].as_str().map(|r| {
            r.split('x').next().unwrap_or("0").to_string() + "p"
        })
    })
    .unwrap_or_else(|| "N/A".to_string());
```

## Info

### IN-01: `normalize_resolution` 对非标准 `format_note` 值的处理可能导致匹配遗漏

**File:** `src/youtube/downloader.rs:78-85`
**Issue:** `normalize_resolution` 仅在字符串包含 `'p'` 时才做标准化。当 `format_note` 为 `"tiny"`, `"only audio"`, `""` 等非标准值时，函数原样返回。这些值在后续的格式匹配中不会匹配任何用户请求的分辨率（如 `"1080p"`），最终全部进入 fallback 路径。虽然 fallback 会处理（选第一个有视频的格式），但用户请求的分辨率语义完全被忽略。

这本身不是 bug（fallback 行为正确），但意味着对于 format_note 非标准化的视频，分辨率选择功能实际上不生效。

**Fix:** 考虑在 `VideoFormat` 结构体中增加 `height: u32` 字段（从 yt-dlp JSON 的 `height` 字段解析），用于更可靠的分辨率匹配。或在文档中说明此限制。

### IN-02: 集成测试 `test_get_video_formats` 为空存根

**File:** `tests/youtube_integration.rs:30-37`
**Issue:** `test_get_video_formats` 仅包含 `assert!(true, "存根 -- 执行阶段填充具体测试逻辑")`，未对 `get_video_formats` 函数进行任何实际验证。`find_ytdlp()` 已正确实现（检测 yt-dlp 是否可用），但测试本身不调用被测函数。

由于 `get_video_formats` 是 `async fn`（私有函数），集成测试需要使用公开的 `download_video` API 或将 `get_video_formats` 暴露为 `pub` 以供测试。

**Fix:** 实现真实的集成测试，使用公开可用的 YouTube 视频 URL 验证格式列表非空、标题非空：
```rust
#[test]
#[ignore]
fn test_download_video_real() {
    let _yt_dlp = find_ytdlp();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(narratoai::youtube::downloader::download_video(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "720p",
        "mp4",
        None,
        None,
    ));
    match result {
        Ok(r) => {
            assert!(r.output_path.exists(), "下载文件应存在");
            assert!(!r.title.is_empty(), "标题不应为空");
            let _ = std::fs::remove_file(r.output_path);
        }
        Err(e) => {
            // 网络失败是可接受的，但不应 panic
            println!("下载失败（网络原因可接受）: {}", e);
        }
    }
}
```

---

_Reviewed: 2026-05-01T19:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
