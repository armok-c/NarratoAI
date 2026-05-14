---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
created: 2026-05-13
source: 16-RESEARCH.md → Validation Architecture + Security Domain
---

# Phase 16 Validation

## Nyquist Validation Dimensions

### D1 — Existence (Artifacts Exist)

| Check | Target | Verification |
|-------|--------|-------------|
| C-01 | `src-tauri/src/commands/config.rs` — 5 Tauri 命令定义 | `grep -c "#\[tauri::command\]" src-tauri/src/commands/config.rs` == 5 |
| C-02 | `src-tauri/src/commands/mod.rs` — config 模块注册 | `grep -c "pub mod config" src-tauri/src/commands/mod.rs` == 1 |
| C-03 | `src-tauri/tauri.conf.json` — externalBin 声明 | `grep -c "externalBin" src-tauri/tauri.conf.json` >= 1 |
| C-04 | `src-tauri/capabilities/default.json` — shell 权限 | `grep -c "shell:allow-execute" src-tauri/capabilities/default.json` == 1 |
| C-05 | `src/stores/llm.ts` — 扩展实现 | `wc -l < src/stores/llm.ts` >= 80 |
| C-06 | `src/stores/tts.ts` — 扩展实现 | `wc -l < src/stores/tts.ts` >= 80 |
| C-07 | `src/composables/useConfig.ts` | `wc -l < src/composables/useConfig.ts` >= 60 |
| C-08 | `src/components/config/*Panel.vue` — 7 个面板 | `ls src/components/config/*Panel.vue 2>/dev/null | wc -l` == 7 |
| C-09 | `tests/stores/llm.test.ts` — 单元测试 | `wc -l < tests/stores/llm.test.ts` >= 30 |
| C-10 | `tests/composables/useConfig.test.ts` | `wc -l < tests/composables/useConfig.test.ts` >= 30 |
| C-11 | `src-tauri/binaries/.gitkeep` | `test -f src-tauri/binaries/.gitkeep` |
| C-12 | `src/types/generated/AppConfig.ts` — ts-rs 同步 | `grep -c "export interface AppConfig" src/types/generated/AppConfig.ts` == 1 |

### D2 — Behavior (Right Behavior)

| Check | Requirement | Verification |
|-------|-------------|-------------|
| B-01 | get_config 返回脱敏 AppConfig JSON | `cargo test -p narratoai-app -- commands::config::get_config 2>&1 | grep -q "PASSED"` |
| B-02 | save_config 增量更新合并 | `cargo test -p narratoai-app -- commands::config::save_config 2>&1 | grep -q "PASSED"` |
| B-03 | save_config 冲突检测 (expected_mtime) | `cargo test -p narratoai-app -- commands::config::conflict_detection 2>&1 | grep -q "PASSED"` |
| B-04 | get_system_proxy 检测环境变量 | `cargo test -p narratoai-core -- tts::common::tests 2>&1 | grep -q "test_from_env"` |
| B-05 | test_llm_connection 构造有效请求 | `cargo test -p narratoai-app -- commands::config::test_connection 2>&1 | grep -q "PASSED"` |
| B-06 | Edge-TTS voice list 支持代理 | `cargo test -p narratoai-core -- edge_tts::tests 2>&1 | grep -q "PASSED"` |
| B-07 | LLM Vision 面板仅 Documentary 可见 | `grep -c "v-if.*currentMode.*documentary" src/components/config/LlmVisionPanel.vue` >= 1 |
| B-08 | TTS 面板在 SDP 模式隐藏 | `grep -c "v-if.*currentMode.*!==.*sdp" src/components/SettingsDrawer.vue` >= 1 |
| B-09 | Draft save/restore/clear 生命周期 | `npx vitest run tests/composables/useConfig.test.ts 2>&1 | grep -q "PASSED"` |
| B-10 | Settings 面板共 7 个 SettingSection | `grep -c "SettingSection" src/components/SettingsDrawer.vue` >= 7 |

### D3 — Constraints (Within Boundaries)

| Check | Constraint | Verification |
|-------|-----------|-------------|
| CN-01 | API Key 仅在后端脱敏（前端不重复脱敏） | `grep -c "desensitize\|maskKey\|脱敏" src/composables/useConfig.ts` — 结果应为 0（脱敏逻辑在 Rust 侧） |
| CN-02 | 无前端直接文件系统读写 | `grep -c "fs\|readFile\|writeFile" src/composables/useConfig.ts` == 0 |
| CN-03 | Draft 仅通过 localStorage 管理 | `grep -c "localStorage" src/composables/useConfig.ts` >= 3 |
| CN-04 | 配置读写仅通过 Tauri IPC | `grep -c "invoke.*config" src/composables/useConfig.ts` >= 2 |

### D4 — Security (STRIDE Mitigations Applied)

| Check | Threat | Verification |
|-------|--------|-------------|
| S-01 | T-16-01: save_config 冲突检测 | `grep -c "expected_mtime\|conflict" src-tauri/src/commands/config.rs` >= 1 |
| S-02 | T-16-02: API Key 脱敏 | `grep -c "desensitize_key" src-tauri/src/commands/config.rs` >= 1 |
| S-03 | T-16-10: API Key type=password | `grep -c "type.*password" src/components/config/LlmVisionPanel.vue` >= 1 |

### D5 — Integration (Connections Work)

| Check | Connection | Verification |
|-------|-----------|-------------|
| I-01 | App.vue → get_config → Pinia stores | `grep -c "loadFromBackend\|get_config" src/App.vue` >= 1 |
| I-02 | SettingsDrawer Footer → save_config | `grep -c "save_config\|handleSave" src/components/SettingsDrawer.vue` >= 1 |
| I-03 | LLM Panel → test_llm_connection | `grep -c "test_llm_connection\|testConnection" src/components/config/LlmVisionPanel.vue` >= 1 |

## Test Commands

```bash
# Rust 后端
cd src-tauri && cargo check
cd narratoai-core && cargo test -- tts::common::tests

# TypeScript 前端
npx vue-tsc --noEmit
npx vitest run tests/stores/llm.test.ts tests/stores/tts.test.ts
npx vitest run tests/composables/useConfig.test.ts tests/components/SettingSection.test.ts
```

## Key Risks

1. **ts-rs 生成文件路径不对** — 如果 `cargo test` 未生成文件到预期位置，Plan 16-03 需后备手动定义接口
2. **tauri-plugin-shell 编译兼容性** — 需要 Tauri 2.0 特定版本匹配；`cargo check` 是初次验证
3. **FFmpeg sidecar 二进制缺失** — 开发阶段可用 system FFmpeg 8.1.1 替代测试；生产构建前需用户手动放置
