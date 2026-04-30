# Phase 11: Extended Features - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

补齐 Python 版的四个辅助功能：音频响度标准化（LUFS-based loudness normalization）、YouTube 视频下载（yt-dlp 子进程）、Pexels/Pixabay 素材搜索下载、智能音量控制（内容类型预设）。四个功能各自独立，仅依赖 Phase 1（FFmpeg 操作、配置系统、错误类型）。覆盖需求：EXTD-01, EXTD-02, EXTD-03, EXTD-04。

</domain>

<decisions>
## Implementation Decisions

### 音频标准化 (EXTD-01)
- **D-01:** Two-pass FFmpeg loudnorm — 第一遍分析获取 measured_I/LRA/TP/thresh，第二遍应用精确标准化。与 Python 版行为完全对齐。
- **D-02:** target_lufs 可配置 + 默认 -23.0。从 config.toml 新建 `[audio]` 段读取，调用方可按场景覆盖。
- **D-03:** FFmpeg loudnorm 分析失败时回退到纯数学 RMS 标准化。用 symphonia 解码音频 → 计算 RMS → 乘以增益系数。
- **D-04:** RMS 回退使用 symphonia（纯 Rust 音频解码库，支持 MP3/AAC/WAV/FLAC/OGG），不引入外部音频工具依赖。
- **D-05:** 新建 `src/audio/` 模块，放 normalizer.rs + volume.rs。与 Phase 1 模块结构约定一致。
- **D-06:** 纯函数 API —— 所有参数显式传入（如 `pub fn normalize_lufs(input: &Path, output: &Path, target_lufs: f64, max_peak: f64) -> Result<()>`），无隐藏状态。
- **D-07:** 全部 5 个函数 1:1 复刻 Python 版：`analyze_lufs()`、`normalize_lufs()`、`get_audio_rms()`、`calculate_volume_adjustment()`、`normalize_audio_for_mixing()`。

### yt-dlp 集成 (EXTD-02)
- **D-08:** CLI 子进程方式集成，通过 `tokio::process::Command` 调用 yt-dlp。用户需系统安装 yt-dlp，与 ffmpeg-sidecar 外部依赖模式一致。
- **D-09:** 完整复刻 Python 版的格式/分辨率匹配逻辑：先 `_get_video_formats()` 获取格式列表 → 匹配目标分辨率 → 手动指定 format_id 下载。
- **D-10:** 无下载进度回调。与 Python 版行为一致。
- **D-11:** 新建 `src/youtube/` 模块，纯函数 API：`download_video(url, resolution, output_format, rename) -> Result<(PathBuf, String)>`。

### 素材 API (EXTD-03)
- **D-12:** Pexels + Pixabay 双源支持。与 Python 版功能完全对齐。
- **D-13:** 多 API key 轮换机制。从 config.toml `[app]` 段 `pexels_api_keys` / `pixabay_api_keys` 数组读取，轮换应对限流。
- **D-14:** Python 风格合并式 `download_videos()` —— 搜索 + 下载 + 时长追踪捆绑为单一编排函数。同时提供更细粒度的 `search_videos()` 和 `save_video()` 供独立复用。
- **D-15:** 新建 `src/material/` 模块。视频缓存复用 MD5 URL 去重逻辑（文件已存在则跳过下载）。HTTP 客户端使用 reqwest（已在 Cargo.toml）。

### 智能音量 (EXTD-04)
- **D-16:** 音量 profile 预设硬编码在代码中（balanced/voice_focused/original_focused/quiet_background），config.toml `[audio]` 段可覆盖 tts_volume/original_volume/bgm_volume。与 Python AudioConfig 类模式一致。
- **D-17:** `src/audio/normalizer.rs`（LUFS 标准化 + RMS 回退）+ `src/audio/volume.rs`（profile/预设/验证）。共享 `AudioError` 枚举。
- **D-18:** 全部 4 类函数 1:1 复刻：`get_optimized_volumes(video_type)`、`apply_volume_profile(profile_name)`、`get_recommended_volumes_for_content(content_type)`、`validate_volume(volume, name)`。
- **D-19:** 新建 `[audio]` 配置段：target_lufs、max_peak、tts_volume、original_volume、bgm_volume、enable_smart_volume、enable_audio_normalization、sample_rate、channels、bitrate、crossfade_duration、bgm_fade_out。
- **D-20:** PROCESSING_CONFIG + MIXING_CONFIG 全部实现为 Rust 类型。但 MIXING_CONFIG 中 crossfade/bgm_fade_out 仅定义配置 struct，实际 FFmpeg 交叉淡化留给 Phase 6 音频合并 pipeline。
- **D-21:** 音量值使用专用 struct `VolumeConfig { tts_volume: f64, original_volume: f64, bgm_volume: f64 }`，类型安全，编译时字段检查。
- **D-22:** `MixingConfig { crossfade_duration: f64, bgm_fade_out: f64, dynamic_range_compression: bool }` struct 定义 + 配置读取。FFmpeg 实现留给 Phase 6。

### 模块组织
- **D-23:** 四个新模块均为顶级 `src/{module}/`（src/audio/、src/youtube/、src/material/），与现有 ffmpeg/llm/tts/ 同级。lib.rs 分别 `pub mod` 导出。
- **D-24:** 配置类型扩展 `src/config/types.rs`，新增 `AudioSection`、`MaterialSection` 等 struct，与 Phase 1 FramesSection/ProxySection 模式一致。

### 错误类型
- **D-25:** 各自独立错误枚举：`AudioError`、`YoutubeError`、`MaterialError`，定义在各自模块内。与 Phase 1 ConfigError/FFmpegError 和 Phase 5 ScriptError 模式一致。
- **D-26:** thiserror 派生，用户可见错误信息用中文 Display。与 Phase 1 D-18 决策一致。
- **D-27:** 错误变体粒度对齐 Python 版异常情况。AudioError：LoudnormAnalysisFailed/LoudnormNormalizeFailed/RmsFallbackFailed/FileNotFound。YoutubeError：SubprocessError/FormatNotFound/InvalidUrl/DownloadFailed。

### 测试策略
- **D-28:** 单元测试 mock 外部调用（FFmpeg/yt-dlp/HTTP），不依赖真实服务。集成测试用 `#[ignore]` 标记，需真实外部服务时手动运行。
- **D-29:** 单元测试在各模块内 `#[cfg(test)] mod tests`，集成测试放 `tests/audio_integration.rs` 等。与 Phase 1/3/5 模式一致。

### 代理与网络配置
- **D-30:** yt-dlp CLI 子进程从 config.toml `[proxy]` 段读取代理地址，拼接到 `--proxy` 参数。
- **D-31:** reqwest（Pexels/Pixabay API 调用）从 `[proxy]` 段读取 http/https 代理 URL，构建 `reqwest::Proxy` 注入 Client。

### Claude's Discretion
- 各错误枚举的具体变体数量和定义
- 函数签名细节参数（如默认值处理方式）
- Cargo.toml 新增依赖（symphonia 及其 feature flags）
- 测试用例具体设计和 mock 实现
- `[audio]` 配置段的精确字段名和类型
- `download_videos()` 编排函数内部调用细粒度函数的拆分方式

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版参考实现
- `app/services/audio_normalizer.py` — AudioNormalizer 类：LUFS 分析、two-pass 标准化、RMS 回退、音量调整系数计算
- `app/config/audio_config.py` — AudioConfig 类：音量预设/Profile/内容类型推荐/PROCESSING_CONFIG/MIXING_CONFIG
- `app/services/youtube_service.py` — YoutubeService 类：格式列表/分辨率匹配/yt-dlp 下载
- `app/services/material.py` — Pexels/Pixabay 搜索+下载+缓存完整流程、save_clip_video、merge_videos
- `app/services/task.py` — 核心流水线编排（音频标准化和智能音量的调用点参考）
- `app/services/generate_video.py` — 视频生成中音频标准化调用点

### Rust 代码库依赖
- `src/config/types.rs` — 需扩展 AudioSection/MaterialSection，参考 FramesSection/ProxySection 模式
- `src/ffmpeg/mod.rs` — FFmpeg 子进程执行模式参考（spawn_blocking + ffmpeg-sidecar），音频 loudnorm 调用复用此模式
- `src/error.rs` — 错误类型模式参考（thiserror + 中文 Display）
- `src/lib.rs` — 新增 `pub mod audio`、`pub mod youtube`、`pub mod material`
- `Cargo.toml` — 新增 symphonia 依赖（音频解码）、现有 reqwest/tokio/serde 直接复用

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库 + Tauri 命令、文件存储、不可变模式）
- `.planning/REQUIREMENTS.md` — Phase 11 需求：EXTD-01, EXTD-02, EXTD-03, EXTD-04
- `.planning/ROADMAP.md` — Phase 11 定义、成功标准、依赖关系（Phase 1）
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（模块结构、错误处理、FFmpeg spawn_blocking、serde 模式）
- `.planning/phases/05-script-management/05-CONTEXT.md` — Phase 5 决策（领域模块约定、thiserror、中文错误）

### 代码库映射
- `.planning/codebase/STACK.md` — Python 版依赖栈（pydub/moviepy/requests 等）
- `.planning/codebase/INTEGRATIONS.md` — YouTube/Pexels/代理集成点
- `.planning/codebase/ARCHITECTURE.md` — Python 版分层架构

### 外部依赖
- yt-dlp CLI — 需系统安装，文档：https://github.com/yt-dlp/yt-dlp
- Pexels API — https://api.pexels.com/videos/search
- Pixabay API — https://pixabay.com/api/videos/
- symphonia — Rust 音频解码库：https://crates.io/crates/symphonia
- FFmpeg loudnorm filter — https://ffmpeg.org/ffmpeg-filters.html#loudnorm

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/ffmpeg/mod.rs` — Phase 1 的 spawn_blocking + ffmpeg-sidecar 封装，音频 loudnorm 两遍调用直接复用其命令构建和执行模式
- `src/config/types.rs` — 现有 FramesSection/ProxySection，新增 AudioSection 遵循相同 serde + 默认值模式
- `src/error.rs` — thiserror 模式参考，新错误枚举遵循相同约定
- `Cargo.toml` — reqwest（HTTP 客户端）/tokio（异步运行时）/serde（序列化）已就绪

### Established Patterns
- **spawn_blocking 外部进程（Phase 1）**：yt-dlp CLI 和 FFmpeg loudnorm 均走此模式
- **模块级结构（Phase 1 D-02）**：`src/{domain}/mod.rs` + 子模块
- **领域错误枚举（Phase 1 D-16/D-17）**：各自模块内 thiserror 派生
- **中文用户可见错误（Phase 1 D-18）**：所有 Error 的 Display 用中文
- **serde + 强类型（Phase 1 D-07）**：配置 struct 和 API 响应类型使用 serde 反序列化
- **纯函数 API**：无全局状态，参数显式传入
- **配置扩展**：新增 section struct 并在 AppConfig 中引用

### Integration Points
- `src/lib.rs` — 新增 `pub mod audio`、`pub mod youtube`、`pub mod material`
- `src/config/types.rs` — 新增 AudioSection、扩展 AppConfig
- `Cargo.toml` — 新增 symphonia 依赖
- Phase 6 Documentary Pipeline — 消费音频标准化和智能音量功能
- Phase 10 Tauri Command Layer — 暴露扩展功能 API 给前端

</code_context>

<specifics>
## Specific Ideas

- 用户明确要求与 Python 版 1:1 功能对齐，所有 4 个功能的函数数量和粒度完全复刻
- 音频模块外部依赖仅 FFmpeg + symphonia，不引入 pydub 等价物
- MIXING_CONFIG 只在 Phase 11 定义类型，FFmpeg 交叉淡化实现留给 Phase 6
- yt-dlp 格式匹配要复刻 Python 的完整分辨率标准化逻辑（如 "1080p60" → "1080p"）
- 素材下载 MD5 缓存 key 复用 Python 版逻辑：取 URL 去掉 query string → MD5 → `vid-{hash}`
- 代理支持从现有 `[proxy]` 段读取，不新建配置段
- 音量 profile 命名和数值与 Python AudioConfig 类完全一致

</specifics>

<deferred>
## Deferred Ideas

None — all discussion stayed within Phase 11 scope

</deferred>

---

*Phase: 11-extended-features*
*Context gathered: 2026-04-30*
