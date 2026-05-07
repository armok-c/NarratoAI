//! 文本处理工具函数

/// 从 JSON 字符串剥离 markdown 代码块包裹
///
/// 某些 LLM 即使设置 response_format=JsonObject 仍返回 ` ```json ` 包裹，
/// 此函数提取代码块内的纯净 JSON。
///
/// 仅在首行为 ``` 开头且末行为独立的 ``` 时才剥离，避免误切内容中包含的反引号序列。
pub fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() < 2
        || !lines[0].starts_with("```")
        || lines.last().map_or(true, |l| l.trim() != "```")
    {
        return trimmed;
    }
    let first_line_end = trimmed.find('\n').unwrap_or(trimmed.len());
    let last_line_start = trimmed.len() - lines.last().unwrap().len();
    trimmed[first_line_end..last_line_start].trim()
}
