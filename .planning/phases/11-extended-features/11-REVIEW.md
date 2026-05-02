---
phase: 11-extended-features
reviewed: 2026-05-01
depth: standard
files_reviewed: 16
files_reviewed_list:
  - Cargo.toml
  - config.example.toml
  - src/audio/mod.rs
  - src/audio/normalizer.rs
  - src/audio/volume.rs
  - src/config/defaults.rs
  - src/config/types.rs
  - src/lib.rs
  - src/material/downloader.rs
  - src/material/mod.rs
  - src/material/searcher.rs
  - src/youtube/downloader.rs
  - src/youtube/mod.rs
  - tests/audio_integration.rs
  - tests/material_integration.rs
  - tests/youtube_integration.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 11 Code Review (Iteration 10 — Fresh Review)

**Reviewer:** gsd-code-reviewer
**Date:** 2026-05-01
**Depth:** standard
**Scope:** 16 files, ~3050 lines (all Phase 11 source + config + integration tests)
**Previous iterations:** 9 (4 CR fixed, 5 WR fixed, 2 WR fixed in iteration 9)

---

## Summary

Phase 11 引入三个新模块（audio / youtube / material）共 ~3050 行 Rust 代码，整体质量良好。前 9 轮审查共修复了 4 个 Critical、7 个 Warning。本次为全新审查，聚焦当前代码状态。

**无 Critical 和 Warning 级别问题。** WR-08（get_video_formats 超时）和 WR-09（task_id 路径遍历）的修复已验证正确。3 个 Info 为设计观察，不影响功能和安全。

---

## Previous Fix Verification

### WR-08 (Fixed) — `get_video_formats` 超时

**文件:** `src/youtube/downloader.rs:130-134`

`tokio::time::timeout(Duration::from_secs(120), cmd.output())` 正确包裹异步调用。120 秒超时合理（元数据查询通常 < 10 秒）。超时返回 `SubprocessError("元数据查询超时（120 秒）")`，与 `download_video` 的 600 秒超时策略一致。

### WR-09 (Fixed) — `download_videos` task_id 路径遍历

**文件:** `src/material/downloader.rs:146-149`

入口处校验 `task_id` 不包含 `/`、`\`、`..`，阻止路径遍历。使用 `MaterialError::IoError` 报错，语义合理。与 YouTube 模块 `rename` 参数校验策略保持一致。

---

## Findings

### INFO-01: `search_videos_pixabay` 无 401 专用错误处理

**文件:** `src/material/searcher.rs:292-294`
**严重度:** Info
**类别:** 用户体验

`search_videos_pexels` 对 HTTP 401 返回 `AuthenticationFailed("Pexels API key 无效")`，而 `search_videos_pixabay` 统一返回 `ApiRequest("HTTP 401")`。用户在 Pixabay key 失效时无法获得友好的认证错误提示。

---

### INFO-02: `save_video` 在 `rename` 失败时遗留临时文件

**文件:** `src/material/downloader.rs:95`
**严重度:** Info
**类别:** 资源清理

若 `std::fs::rename` 失败（如跨设备链接），带有 UUID 后缀的临时文件将残留在磁盘上。长时间运行可能导致临时文件累积。可在 `rename` 失败后添加 `let _ = std::fs::remove_file(&temp_path);` 清理。

---

### INFO-03: `build_client` 每次搜索调用创建新 reqwest Client

**文件:** `src/material/searcher.rs:144-167`
**严重度:** Info
**类别:** 性能

`reqwest::Client` 设计为可复用（内置连接池）。每次 `search_videos_*` 调用都创建新 Client，失去连接池优势。在低频调用场景下影响可忽略。若未来搜索频率增加，可考虑缓存 Client 实例。

---

## Files Reviewed (No Issues)

| File | Lines | Notes |
|------|-------|-------|
| `src/audio/mod.rs` | 2 | 纯模块声明 |
| `src/audio/normalizer.rs` | 519 | loudnorm 两遍 + RMS 回退实现完整，JSON 提取安全 |
| `src/audio/volume.rs` | 329 | 纯函数配置，NaN 检查已就绪 |
| `src/config/defaults.rs` | 134 | 默认值与 config.example.toml 一致 |
| `src/config/types.rs` | 425 | `skip_serializing` 保护密钥字段，`deny_unknown_fields` 防拼写错误 |
| `src/lib.rs` | 28 | 模块导出 + version() |
| `Cargo.toml` | 42 | 依赖版本合理，无已知漏洞版本 |
| `config.example.toml` | 218 | 中文注释完善，API key 默认为空 |
| `tests/audio_integration.rs` | 32 | 有意存根，待后续填充 |
| `tests/material_integration.rs` | 26 | 有意存根，待后续填充 |
| `tests/youtube_integration.rs` | 37 | 有意存根，待后续填充 |
| `src/youtube/downloader.rs` | 468 | URL 前缀校验、换行符检查、args 数组模式、rename 路径遍历防护完整 |
| `src/youtube/mod.rs` | 1 | 纯模块声明 |
| `src/material/mod.rs` | 2 | 纯模块声明 |
| `src/material/searcher.rs` | 495 | API key 轮换、代理配置、urlencoding 实现正确 |
| `src/material/downloader.rs` | 291 | MD5 缓存、temp+rename、ffprobe 验证、task_id 校验完整 |

---

## Comparison with Previous Reviews

| Metric | Iteration 9 | Iteration 10 (Current) |
|--------|-------------|----------------------|
| Critical | 0 | 0 |
| Warning | 2 | 0 |
| Info | 7 | 3 |
| Total | 9 | 3 |

Iteration 9 的 2 个 Warning（WR-08 超时、WR-09 路径遍历）已修复。Info 项从 7 减少到 3（保留了最有实际价值的观察，其他 4 项为已知设计决策：`get_audio_rms` 内存加载、`.mp3` 硬编码、`AudioSection` 无范围验证、代理验证重复）。

---

## Conclusion

Phase 11 代码经过 10 轮审查后质量优秀。所有安全相关和健壮性相关的问题均已修复。剩余 3 个 Info 为低优先级设计观察，可在后续迭代中按需处理。
