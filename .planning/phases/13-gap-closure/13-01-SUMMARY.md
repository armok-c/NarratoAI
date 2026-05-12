# 13-01 配置系统更新 — 执行摘要

## 目标

为 AudioSection 配置新增 volume_profile 字段，统一默认 target_lufs 为 -14.0，同步更新 config.example.toml。

## 修改文件

| 文件 | 变更说明 |
|------|----------|
| `narratoai-core/src/config/types.rs` | AudioSection 已有 volume_profile: String 字段（#\[serde(default)\]），测试断言已就绪（target_lufs 断言 -14.0，含 volume_profile 断言） |
| `narratoai-core/src/config/defaults.rs` | `target_lufs: -23.0` → `-14.0`；新增 `volume_profile: "balanced".into()` |
| `config.example.toml` | [audio] 段 target_lufs = -14.0，新增 volume_profile = "balanced"，注释反映 YouTube/流媒体标准 |

## 测试结果

- `cargo test -p narratoai-core` 全线通过（603 passed, 0 failed, 3 ignored）
- TOML 解析验证通过（`python -c "import toml; ..."`)
- 旧 config.toml 兼容性：volume_profile 带 `#[serde(default)]`，序列化/反序列化兼容旧配置

## 提交历史

```
d30a476 feat(13-01): update config.example.toml [audio] section — -14.0 LUFS + volume_profile
e709dc7 feat(13-01): AudioSection::default() — target_lufs=-14.0, volume_profile=balanced
```

## 需求覆盖

- PRMP-03: 配置系统支持 volume_profile 字段
- EXTD-01: 默认 target_lufs 对齐 -14.0 YouTube/流媒体标准
- EXTD-04: config.example.toml 包含 volume_profile 配置说明
