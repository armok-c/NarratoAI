# Phase 3: TTS Core + Edge-TTS - Discussion Log

**Gathered:** 2026-04-28
**Mode:** Default (interactive)
**Workstream:** ws-tts

## Area 1: Edge-TTS 实现策略

| Question | Options | Selected | Notes |
|----------|---------|----------|-------|
| 实现方案？ | 原生 WebSocket / 现成 crate / CLI 子进程 | 原生 WebSocket（推荐） | tokio-tungstenite 直接实现协议 |
| 代理支持？ | 支持代理 / 暂不支持 | 支持代理（推荐） | 复用 config.proxy，中国网络环境必需 |
| 重试策略？ | 3次重试 / 可配置 / 不重试 | 3次重试（推荐） | 与 Python 行为一致，间隔 1s |
| 音频格式？ | MP3 / SSML 指定 | MP3（推荐） | 与 Python 版一致 |

## Area 2: TtsProvider trait 设计

| Question | Options | Selected | Notes |
|----------|---------|----------|-------|
| async vs sync？ | async trait / sync trait / 不统一 | async trait（推荐） | #[async_trait]，同步引擎用 spawn_blocking |
| 返回值？ | TtsOutput 结构体 / 仅路径 / 字节+时序 | TtsOutput 结构体（推荐） | 含 audio_file_path + word_boundaries + duration |
| 参数形式？ | TtsParams 结构体 / 独立参数 / Builder | 独立参数 | 与 Python 一致，大参数列表 |
| 词边界结构？ | Vec<WordBoundary> / 三元组 / Option | Vec<WordBoundary>（推荐） | start_offset, end_offset, text，100ns 单位 |

## Area 3: SubMaker / 字幕时序保留

| Question | Options | Selected | Notes |
|----------|---------|----------|-------|
| 保留独立 SubMaker？ | 内嵌在 TtsOutput / 保留独立结构 | 内嵌 TtsOutput（推荐） | Vec<WordBoundary> 直接放在返回值中 |
| 引擎无词边界时？ | 估算词边界 / 返回空 Vec | 估算词边界（推荐） | 整段文本一个 WordBoundary，与 Python 一致 |

## Area 4: 路由器分发机制

| Question | Options | Selected | Notes |
|----------|---------|----------|-------|
| 枚举 vs 字符串？ | 枚举+match / 字符串匹配 / HashMap | 字符串匹配 | 与 Python 一致，简单直接 |
| 路由器形式？ | 模块级函数 / TtsRouter 结构体 | 模块级函数（推荐） | pub fn synthesize() |
| 未知引擎处理？ | 返回错误 / Fallback Edge-TTS | 返回错误（推荐） | TTSError::UnknownEngine，不静默 |
| Phase 12 扩展？ | 接受现状 / 提前设计注册表 | 接受，Phase 12 再考虑（推荐） | YAGNI |

## Deferred Ideas

None
