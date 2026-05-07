//! 文本处理工具函数

/// 从 JSON 字符串剥离 markdown 代码块包裹
///
/// 某些 LLM 即使设置 response_format=JsonObject 仍返回 ` ```json ` 包裹，
/// 此函数提取代码块内的纯净 JSON。
pub fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let after_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_start());
    let content = after_prefix.unwrap_or(trimmed);
    content
        .strip_suffix("```")
        .map(|s| s.trim_end())
        .unwrap_or(content)
}
