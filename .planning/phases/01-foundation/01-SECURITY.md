---
phase: 01-foundation
slug: foundation
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-28
---

# Phase 01 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| config.toml → AppConfig | 用户可编辑的 TOML 文件传入 serde 反序列化 | 配置数据 (含 API 密钥) |
| 视频路径 → FFmpeg CLI | 用户提供的文件路径传入 FFmpeg 子进程 | 文件系统路径 |
| ffprobe JSON → VideoInfo | 外部进程 stdout 解析为结构化数据 | 视频元数据 |
| 测试视频生成 → FFmpeg | 测试代码生成命令参数调用 FFmpeg | 临时视频文件 |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-01-01 | Tampering | config.toml 反序列化 | mitigate | serde 强类型结构体 + `#[serde(deny_unknown_fields)]` 拒绝未知字段 | closed |
| T-01-02 | Information Disclosure | config.toml API 密钥 | accept | config.toml 在 .gitignore 中，桌面单用户应用 | closed |
| T-01-03 | Tampering | ConfigWatcher 热加载 | mitigate | 解析失败时仅 tracing::error! 记录日志，不替换当前有效配置 | closed |
| T-02-01 | Tampering | FFmpeg 文件路径参数 | mitigate | ffmpeg-sidecar builder API 正确转义参数，不使用字符串拼接命令 | closed |
| T-02-02 | Tampering | ffprobe JSON 输出解析 | mitigate | serde_json 安全提取字段，缺失字段使用默认值 (0.0, 0, "unknown")，不 panic | closed |
| T-02-03 | Denial of Service | spawn_blocking 长时间阻塞 | accept | FFmpeg 操作本身可能耗时，spawn_blocking 使用独立线程池不阻塞 tokio runtime；clip_video 已接入 600s 超时 (WR-01) | closed |
| T-03-01 | Denial of Service | 集成测试占用系统资源 | accept | 测试创建小视频 (2s 320x240)，资源消耗极低 | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01 | T-01-02 | config.toml 已在 .gitignore 中，桌面单用户应用，密钥泄露风险可接受 | Armok | 2026-04-28 |
| AR-02 | T-02-03 | FFmpeg 操作天然耗时 (视频编码)，spawn_blocking 置于独立线程池；clip_video 已添加 600 秒超时防护 | Armok | 2026-04-28 |
| AR-03 | T-03-01 | 集成测试使用 2 秒 320x240 极小视频，资源消耗可忽略 | Armok | 2026-04-28 |

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-28 | 7 | 7 | 0 | gsd-security-auditor (automated) |

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-28
