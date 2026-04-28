# Roadmap: TTS Engine Stack

## Overview

实现 NarratoAI 所有 TTS 引擎（从核心 trait/路由器到全部 7 个引擎），覆盖 Edge-TTS、Azure、Tencent、SoulVoice、Qwen、IndexTTS2、Doubao。

**父里程碑:** NarratoAI Rust Rewrite (Phase 1 → Phase 3 → Phase 12)
**前置依赖:** Phase 1 (Foundation) 完成

## Phases

- [ ] **Phase 3: TTS Core + Edge-TTS** — TtsProvider trait、路由器、Edge-TTS WebSocket 实现
- [ ] **Phase 12: Additional TTS Engines** — Azure、Tencent、SoulVoice、Qwen、IndexTTS2、Doubao 六个引擎

## Phase Details

### Phase 3: TTS Core + Edge-TTS
**Goal**: 系统能将文本通过 TTS 引擎转换为音频文件，默认的 Edge-TTS 引擎可正常工作
**Depends on**: Phase 1 (Foundation)
**Requirements**: TTS-01, TTS-02, TTS-03
**Success Criteria**:
  1. TtsProvider trait 定义了统一的 TTS 引擎接口（输入文本+参数，输出音频文件路径）
  2. 按引擎名字符串（如 `edge_tts`）通过路由器分发到对应 TTS 实现
  3. Edge-TTS 引擎通过 WebSocket 协议生成中文语音音频文件，音频可在播放器中正常播放

### Phase 12: Additional TTS Engines
**Goal**: 除 Edge-TTS 外的 6 个 TTS 引擎全部实现
**Depends on**: Phase 3
**Requirements**: TTS-04, TTS-05, TTS-06, TTS-07, TTS-08, TTS-09
**Success Criteria**:
  1. Azure Speech TTS 引擎通过 REST API 生成语音音频
  2. Tencent TTS 引擎生成语音音频
  3. SoulVoice TTS 引擎生成语音音频
  4. Qwen TTS 引擎生成语音音频
  5. IndexTTS2 语音克隆引擎用参考音频生成语音
  6. Doubao TTS 引擎生成语音音频

## Progress

| Phase | Plans | Status |
|-------|-------|--------|
| 3. TTS Core + Edge-TTS | 0/? | Not started |
| 12. Additional TTS Engines | 0/? | Not started |
