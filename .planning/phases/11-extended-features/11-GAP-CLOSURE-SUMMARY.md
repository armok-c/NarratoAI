---
phase: 11-extended-features
plan: GAP-CLOSURE
type: gap_closure
status: complete
commit: 65923af
---

# Phase 11: Gap Closure Summary

**Status:** Complete
**Commit:** 65923af

## Fixes Applied

### CR-01 (BLOCKER): RMS 参考电平修复
- **文件:** `src/audio/normalizer.rs`
- **修复:** 第 288 行 `20.0 * (rms / (i16::MAX as f64)).log10()` → `20.0 * rms.log10()`
- **新增测试:** `test_get_audio_rms_formula` (验证 0.5→-6.02dB, 1.0→0dB, 旧公式→-96.33dB)
- **结果:** 7/7 测试通过

### WR-01: URL 编码修复
- **文件:** `src/material/searcher.rs`
- **修复:** 替换仅处理空格的 `s.replace(' ', "+")` 为完整的 application/x-www-form-urlencoded 编码
- **新增测试:** `test_urlencoding` (8 个断言覆盖空格、&、+、%、#、=、中文)
- **结果:** 11/11 测试通过（含 urlencoding 模块全部测试）

### WR-05: 分辨率标准化去重
- **文件:** `src/youtube/downloader.rs`
- **修复:** 提取 `normalize_resolution()` 辅助函数，消除 `_get_video_formats()` 和 `download_video()` 中的重复逻辑
- **新增测试:** `test_normalize_resolution` (5 个断言)
- **结果:** 12/12 测试通过

### WR-06: 复用 dump-json title
- **文件:** `src/youtube/downloader.rs`
- **修复:** `_get_video_formats()` 返回值增加 title，删除冗余的 `yt-dlp --get-title` 子进程
- **结果:** 编译通过，12/12 测试通过

### WR-07: 下载后文件存在性验证
- **文件:** `src/youtube/downloader.rs`
- **修复:** 在 `download_video()` 返回前检查 `output_path.exists()`
- **结果:** 编译通过

### WR-10: 死代码清理
- **文件:** `tests/youtube_integration.rs`
- **修复:** 删除 `find_ytdlp()` 中不可达的 `#[cfg(windows)]`/`#[cfg(not(windows))]` 分支
- **结果:** 集成测试编译通过

## 验收结果

| 检查项 | 状态 |
|--------|------|
| `cargo check` 零 error | ✓ |
| 全部 lib 单元测试 | ✓ 367/368 (1 个预存 hwaccel 失败，无关) |
| 3 个集成测试文件编译 | ✓ |
| `grep "i16::MAX" src/audio/normalizer.rs` 仅存于 test | ✓ |
| `grep "get-title" src/youtube/downloader.rs` 返回 0 | ✓ |
| `fn normalize_resolution` 存在且唯一 | ✓ |

## 接受的 Gaps（未修复）

- WR-09: 集成测试存根使用 `assert!(true)` — 原始计划中的有意设计取舍
- 预存 config.example.toml 不匹配 — Phase 12 引入，非 Phase 11 范围
