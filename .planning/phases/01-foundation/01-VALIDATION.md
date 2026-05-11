---
phase: 01
slug: foundation
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-11
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust cargo test (built-in) |
| **Config file** | narratoai-core/Cargo.toml |
| **Quick run command** | `cargo test -p narratoai-core --lib -- config ffmpeg` |
| **Full suite command** | `cargo test -p narratoai-core` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p narratoai-core --lib`
- **After every plan wave:** Run `cargo test -p narratoai-core`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-T1 | 01 | 1 | CONF-01, CONF-02 | T-01-01 | serde 强类型反序列化，不执行任意代码 | unit | `cargo test -p narratoai-core --lib -- error config::types::tests` | ✅ | ✅ green |
| 01-01-T2 | 01 | 1 | CONF-01, CONF-03 | T-01-03 | 解析失败仅 log，不替换有效配置 | unit+integ | `cargo test -p narratoai-core --test config_hot_reload_test` | ✅ | ✅ green |
| 01-02-T1 | 02 | 2 | FFMP-01, FFMP-09 | T-02-01 | ffmpeg-sidecar builder API 转义参数 | unit | `cargo test -p narratoai-core --lib -- ffmpeg::command` | ✅ | ✅ green |
| 01-02-T2 | 02 | 2 | CONF-04, FFMP-01 | T-02-02 | serde_json Value 安全提取，缺失字段默认值 | unit | `cargo test -p narratoai-core --lib -- ffmpeg::probe ffmpeg::hwaccel` | ✅ | ✅ green |
| 01-03-T1 | 03 | 3 | CONF-01, CONF-02, CONF-03 | — | 配置端到端验证 | integ | `cargo test -p narratoai-core --test config_hot_reload_test` | ✅ | ✅ green |
| 01-03-T2 | 03 | 3 | CONF-04, FFMP-01, FFMP-09 | — | FFmpeg 端到端验证 | integ | `cargo test -p narratoai-core --test infra_real_test` | ✅ | ✅ green |

---

## Requirement → Test Cross-Reference

| Requirement | Description | Tests | Status |
|-------------|-------------|-------|--------|
| CONF-01 | TOML 配置加载所有 10 个配置段 | `config::types::tests::test_load_full_config`, `config::tests::test_config_manager_load_valid_file` | ✅ COVERED |
| CONF-02 | 配置校验（缺失字段→中文错误） | `config::types::tests::test_validate_empty_api_key_ok`, `error::tests::test_config_error_*` (6 tests) | ⚠️ DEFERRED |
| CONF-03 | notify 热加载 | `config_hot_reload_test::test_hot_reload`, `config_hot_reload_test::test_hot_reload_invalid_toml_preserves_config` | ✅ COVERED |
| CONF-04 | FFmpeg 硬件加速检测 | `ffmpeg::hwaccel::tests::*` (4 tests) | ✅ COVERED |
| FFMP-01 | ffmpeg-sidecar FFmpeg 操作 | `ffmpeg::command::tests::*`, `ffmpeg::probe::tests::*`, `infra_real_test::test_video_probe`, `infra_real_test::test_ffmpeg_clip_video_segment` | ✅ COVERED |
| FFMP-09 | spawn_blocking 异步执行 | `ffmpeg::command::tests::test_spawn_blocking_non_blocking`, `infra_real_test::test_ffmpeg_clip_video_segment` | ✅ COVERED |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| validate() 业务校验逻辑 | CONF-02 | 按计划推迟到后续 Phase（Phase 5+）。validate() 接口已存在，当前为 no-op。PLAN 01 明确标注："Phase 1 不要求 API key 必填，校验逻辑在后续 Phase 添加" | 确认 AppConfig::validate() 在当前阶段返回 Ok(()) 是故意设计 |
| 硬件加速检测结果 | CONF-04 | 检测结果依赖实际 GPU 和驱动 | 运行 `cargo test -p narratoai-core --test infra_real_test -- test_ffmpeg_and_ffprobe_available`，人工确认编码器列表与系统硬件匹配 |

---

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-11

---

## Validation Audit 2026-05-11

| Metric | Count |
|--------|-------|
| Gaps found | 1 (CONF-03 hot reload test orphaned) |
| Resolved | 1 (migrated to narratoai-core/tests/config_hot_reload_test.rs) |
| Escalated | 0 |
