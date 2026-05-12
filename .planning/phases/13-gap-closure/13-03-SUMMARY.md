# 13-03 SUMMARY: 纪录片 Prompt 调用链修复

## 目标

修复纪录片流水线的 Prompt 调用链：将 `generate_narration()` 和 `analyze_video()` 的硬编码 prompt 替换为 `PromptManager::render_prompt()` 调用，对齐 SDE/SDP 的已验证模式。

## 变更文件

| 文件 | 变更 |
|------|------|
| `narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md` | 增强模板：明确 JSON items 数组格式、四个必填字段（picture/narration/timestamp/OST），删除与 JSON 输出矛盾的"仅包含解说文案正文"指令 |
| `narratoai-core/src/documentary/script_gen.rs` | 重构三个函数：新增 `use HashMap` 和 `use PromptManager`；`analyze_video()` 和 `generate_narration()` 新增 `prompt_manager` 参数并改用 `render_prompt()`；`generate_documentary_script()` 新增 `prompt_manager` 参数并向下传递 |
| `src-tauri/src/commands/pipeline.rs` | `generate_documentary_script` 命令新增 `prompt_manager` State 参数，锁定后传递给 core 函数 |

## 验证

- `cargo check -p narratoai-core` — 通过
- `cargo check`（workspace 级别，含 Tauri） — 通过，仅 2 条预存在警告（括号冗余）
- `script_gen.rs` 中无残留硬编码 prompt 字符串

## 关键决策

### PromptManager 参数位置
- `analyze_video()`: 参数放在 `provider` 之后，即 `(request, provider, prompt_manager)`
- `generate_narration()`: 参数放在 `provider` 之后、`video_theme` 之前（因为 render_prompt 需要先于可选参数）
- `generate_documentary_script()`: 参数放在最后，即 `(request, vision_provider, text_provider, prompt_manager)`

### 模板变量映射
- **frame_analysis v1.0**: `video_description` ← `request.custom_prompt.unwrap_or("根据视频关键帧生成帧分析报告")`, `language` ← `"zh-CN"`
- **narration_generation v2.0**: `video_title` ← `video_theme.unwrap_or("未命名视频")`, `frame_analysis_json` ← `analysis_markdown`, `language` ← `"zh-CN"`, `style` ← `"正式"`

### 错误转换
- `PromptError` → `PipelineError::Llm { source: LLMError::Validation(e.to_string()) }`（因没有现成的 `From` 转换）

## 安全注意事项
- `narration_generation_v2.0.md` 模板中指定 `"OST": 2`（混合模式），与 `parse_script_clips()` 中缺失 OST 时的默认值 2 一致
- JSON 修复层（`strip_and_repair_json`）保持原样，新增的格式规范可降低 LLM 输出异常概率
