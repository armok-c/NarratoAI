# Phase 11: Extended Features - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-30
**Phase:** 11-extended-features
**Areas discussed:** FFmpeg loudnorm 策略, yt-dlp 集成方式, 素材 API 覆盖范围, 智能音量配置存储, 模块顶层组织, 错误类型策略, 测试与外部依赖策略, 代理与网络配置

---

## FFmpeg loudnorm 策略

| Option | Description | Selected |
|--------|-------------|----------|
| Two-pass | 先分析后应用，精确对齐目标 LUFS，完全对齐 Python 版 | ✓ |
| 单次线性标准化 | 更快但精度稍低，Python 版未使用 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 可配置 + 默认值 | 从 config.toml [audio] 段读取 target_lufs，默认 -23.0 | ✓ |
| 硬编码 -20.0 | 视频混合场景固定值 | |
| 硬编码 -23.0 | 广播标准固定值 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 纯数学 RMS 标准化 | 用 symphonia 解码 → 计算 RMS → 增益系数 | ✓ |
| FFmpeg loudnorm 单 pass 回退 | 不传测量值，仍在 FFmpeg 体系内 | |
| 无回退，返回错误 | 鲁棒性差 | |

| Option | Description | Selected |
|--------|-------------|----------|
| symphonia | 纯 Rust，支持 MP3/AAC/WAV/FLAC/OGG | ✓ |
| ffmpeg-sidecar 解码 | 复用现有依赖但多次子进程调用 | |
| rodio + symphonia | 重型 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 新建 src/audio/ | 独立模块，放 normalizer.rs + volume.rs | ✓ |
| 放入 src/ffmpeg/ | 混合语义（RMS 非 FFmpeg） | |
| 工具函数散放 | 分散 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 纯函数 + 参数传入配置 | 所有参数显式传入，无隐藏状态 | ✓ |
| 带 Builder 的结构体 | 更灵活但复杂 | |
| 同 Python 的 struct + 字段 | 与 Python 最接近 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 全部复刻 5 函数 | analyze_lufs + normalize_lufs + get_audio_rms + calculate_volume_adjustment + normalize_audio_for_mixing | ✓ |
| 核心两个 | 仅 normalize_lufs + analyze_lufs | |
| 仅 normalize_lufs | 最简但不可复用 | |

---

## yt-dlp 集成方式

| Option | Description | Selected |
|--------|-------------|----------|
| CLI 子进程 | tokio::process::Command，与 ffmpeg-sidecar 模式一致 | ✓ |
| Rust 绑定库 | 功能可能不全 | |
| reqwest 直接调 YouTube API | 无法下载视频流 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 完整复刻 Python 逻辑 | list_formats → 匹配分辨率 → 手动指定 format_id | ✓ |
| 简化 bestvideo+bestaudio | yt-dlp 内置选择器，代码量最小 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 无进度回调 | 与 Python 版一致 | ✓ |
| 解析 stderr 百分比 | 脆弱且随 yt-dlp 版本变化 | |

| Option | Description | Selected |
|--------|-------------|----------|
| src/youtube/ + 纯函数 | download_video(url, resolution, output_format) | ✓ |
| 放入 src/utils/ | 不满足领域模块约定 | |
| struct YoutubeService | 不必须面向对象 | |

---

## 素材 API 覆盖范围

| Option | Description | Selected |
|--------|-------------|----------|
| Pexels + Pixabay 双源 | 与 Python 版完全对齐 | ✓ |
| 仅 Pexels | 聚焦验证模式 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 多 key 轮换 | 从 config.toml 数组读取，轮换应对限流 | ✓ |
| 单 key 简化 | 简单但无轮换能力 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Python 风格合并 | download_videos() 捆绑搜索+下载+时长追踪 | ✓ |
| 拆分为独立函数 | 更 Rust 风格 | |

| Option | Description | Selected |
|--------|-------------|----------|
| src/material/ + MD5 缓存 | 新建模块，复用 Python MD5 URL 缓存逻辑 | ✓ |
| 放入 src/utils/ | 不推荐 | |

---

## 智能音量配置存储

| Option | Description | Selected |
|--------|-------------|----------|
| 代码硬编码 + config 覆盖 | Profile 默认值在代码中，config.toml [audio] 可覆盖 | ✓ |
| 纯 config.toml 驱动 | 用户配置文件变重 | |
| 纯代码硬编码 | 用户无法定制 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 同一模块分开文件 | normalizer.rs + volume.rs，共享 AudioError | ✓ |
| 合并为单文件 | 单文件变长 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 全部 4 类函数 | get_optimized_volumes + apply_volume_profile + get_recommended_volumes_for_content + validate_volume | ✓ |
| 仅 profile 系统 | 去掉 video_type 和 content_type | |
| 仅内容类型推荐 | 最简单的智能音量 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 新建 [audio] 段 | 独立配置段，清晰独立 | ✓ |
| 放在 [app] 段 | 与 LLM 配置混在一起 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 全部实现 | PROCESSING_CONFIG + MIXING_CONFIG | ✓ |
| 仅 PROCESSING_CONFIG | MIXING 留给 Phase 6 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 专用 struct | VolumeConfig { tts_volume, original_volume, bgm_volume } | ✓ |
| HashMap | 灵活但无编译时检查 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 仅定义配置 + 类型 | 交叉淡化实现留给 Phase 6 | ✓ |
| 完整实现交叉淡化 | 越界进入 Phase 6 | |

---

## 模块顶层组织

| Option | Description | Selected |
|--------|-------------|----------|
| 顶级 src/{module}/ | 与 ffmpeg/llm/tts 同级 | ✓ |
| src/extended/ 父模块 | 多一层嵌套 | |
| src/services/ | 与现有结构不一致 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 扩展 src/config/types.rs | 新增 AudioSection/MaterialSection，与 FramesSection 模式一致 | ✓ |
| 模块各自定义 | 配置加载逻辑分散 | |

---

## 错误类型策略

| Option | Description | Selected |
|--------|-------------|----------|
| 各自独立 enum | AudioError/YoutubeError/MaterialError | ✓ |
| 统一 ExtendedFeatureError | 耦合四个不相关领域 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 对齐 Python 异常 | 参考 Python 版异常情况定义变体 | ✓ |
| 粗粒度 | 每模块 2-3 变体 | |

---

## 测试与外部依赖策略

| Option | Description | Selected |
|--------|-------------|----------|
| 集成测试标记 + mock 单元测试 | 单元测试 mock 外部调用，集成测试 #[ignore] | ✓ |
| 全量需要真实服务 | CI 不稳定 | |
| 仅单元测试 | 范围过窄 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 模块内 #[cfg(test)] + tests/ 集成 | 与 Phase 1/3/5 模式一致 | ✓ |
| 仅 tests/ 目录 | 单元测试不便 | |

---

## 代理与网络配置

| Option | Description | Selected |
|--------|-------------|----------|
| yt-dlp CLI 传递 --proxy | 从 [proxy] 段读取 | ✓ |
| 不传代理 | 依赖系统环境变量 | |

| Option | Description | Selected |
|--------|-------------|----------|
| reqwest 复用 [proxy] 段 | 读取代理 URL 构建 reqwest::Proxy | ✓ |
| reqwest 默认系统代理 | 依赖环境变量 | |
| 不支持代理 | 过于简化 | |

---

## Claude's Discretion

- 各错误枚举的具体变体数量和定义
- 函数签名细节参数（如默认值处理方式）
- Cargo.toml 新增依赖（symphonia 及其 feature flags）
- 测试用例具体设计和 mock 实现
- `[audio]` 配置段的精确字段名和类型
- `download_videos()` 编排函数内部调用细粒度函数的拆分方式

## Deferred Ideas

None
