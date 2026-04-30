use regex::Regex;
use std::collections::HashMap;

use crate::prompt::error::PromptError;

/// 过滤器函数类型
type FilterFn = fn(&str) -> String;

/// 返回内置过滤器映射表（6 个过滤器）
fn builtin_filters() -> HashMap<&'static str, FilterFn> {
    let mut m: HashMap<&'static str, FilterFn> = HashMap::new();

    // upper——转换为大写
    m.insert("upper", |s: &str| s.to_uppercase());

    // lower——转换为小写
    m.insert("lower", |s: &str| s.to_lowercase());

    // title——单词首字母大写（对 ASCII 字母生效）
    m.insert("title", |s: &str| {
        let mut chars = s.chars().peekable();
        let mut out = String::with_capacity(s.len());
        let mut new_word = true;
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                out.push(c);
                new_word = true;
            } else if new_word {
                out.extend(c.to_uppercase());
                new_word = false;
            } else {
                out.push(c);
            }
        }
        out
    });

    // strip——去除首尾空白
    m.insert("strip", |s: &str| s.trim().to_string());

    // truncate——截断（长度 <= 100 字符返回原串，否则截断前 97 字符 + "..."）
    m.insert("truncate", |s: &str| {
        let char_count = s.chars().count();
        if char_count <= 100 {
            s.to_string()
        } else {
            let truncated: String = s.chars().take(97).collect();
            format!("{}...", truncated)
        }
    });

    // json——JSON 转义字符串
    m.insert("json", |s: &str| {
        serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
    });

    m
}

/// 渲染模板——两遍正则替换 + 过滤器应用
///
/// # 流程
///
/// 第 1 遍（变量提取和校验）：使用正则 `r"\$\{(\w+)\}|\$(\w+)"`
/// 通过 `captures_iter` 提取所有变量名并与上下文比对，缺失变量返回错误。
///
/// 第 2 遍（变量替换）：使用 `replace_all` 将 `${variable}` 和 `$variable`
/// 替换为上下文中对应的值。
///
/// 第 3 遍（过滤器应用）：使用正则 `r"\$\{(\w+)\|(\w+)\}"` 处理过滤器语法
/// `${variable|filter_name}`，应用内置过滤器函数。
///
/// # 注意
///
/// - `${variable|filter}` 格式不会被第 1/2 遍的正则匹配（因为 `|` 不是 `\w`）
pub fn render(template: &str, vars: &HashMap<&str, &str>) -> Result<String, PromptError> {
    // 编译变量正则
    let var_re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").map_err(|e| {
        PromptError::TemplateRender(format!("正则编译失败: {}", e))
    })?;

    // 第 1 遍：提取所有变量名，校验全部存在
    let mut missing: Vec<String> = Vec::new();
    for caps in var_re.captures_iter(template) {
        let name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        if !name.is_empty() && !vars.contains_key(name) && !missing.contains(&name.to_string()) {
            missing.push(name.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}",
            missing.join(", ")
        )));
    }

    // 第 2 遍：变量替换（所有变量已在第 1 遍预校验，此处 safe）
    let result = var_re.replace_all(template, |caps: &regex::Captures| {
        let name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        vars.get(name).copied().unwrap_or("")
    });

    // 第 3 遍：过滤器应用——处理 ${variable|filter_name} 格式
    let filter_re = Regex::new(r"\$\{(\w+)\|(\w+)\}").map_err(|e| {
        PromptError::TemplateRender(format!("正则编译失败: {}", e))
    })?;
    let filters = builtin_filters();

    // 校验过滤器引用的变量是否全部存在
    let mut missing_filter_vars: Vec<String> = Vec::new();
    for caps in filter_re.captures_iter(&result) {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if !var_name.is_empty() && !vars.contains_key(var_name) && !missing_filter_vars.contains(&var_name.to_string()) {
            missing_filter_vars.push(var_name.to_string());
        }
    }
    if !missing_filter_vars.is_empty() {
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}",
            missing_filter_vars.join(", ")
        )));
    }

    // Validate filter names before applying
    for caps in filter_re.captures_iter(&result) {
        let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if !filter_name.is_empty() && !filters.contains_key(filter_name) {
            return Err(PromptError::TemplateRender(format!(
                "未知过滤器: {}", filter_name
            )));
        }
    }

    let result = filter_re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if let (Some(filter_fn), Some(value)) = (filters.get(filter_name), vars.get(var_name)) {
            filter_fn(value)
        } else {
            // 过滤器未找到或变量不存在，保留原始文本
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    });

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 基本变量替换：${variable} 和 $variable 两种语法
    #[test]
    fn test_basic_variable_substitution() {
        let template = "Hello, ${name}! Welcome to $place.";
        let mut vars = HashMap::new();
        vars.insert("name", "World");
        vars.insert("place", "NarratoAI");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "Hello, World! Welcome to NarratoAI.");
    }

    /// 缺失变量返回 PromptError::TemplateRender
    #[test]
    fn test_missing_variable_error() {
        let template = "Hello, ${name}! Your ${role} is ready.";
        let mut vars = HashMap::new();
        vars.insert("name", "User");

        let err = render(template, &vars).expect_err("缺少变量应返回错误");
        match err {
            PromptError::TemplateRender(msg) => {
                assert!(msg.contains("role"), "应指出缺失变量名: {}", msg);
            }
            _ => panic!("应为 TemplateRender 错误"),
        }
    }

    /// upper 过滤器正确大写
    #[test]
    fn test_upper_filter() {
        let template = "${name|upper}";
        let mut vars = HashMap::new();
        vars.insert("name", "hello world");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "HELLO WORLD");
    }

    /// lower 过滤器正确小写
    #[test]
    fn test_lower_filter() {
        let template = "${name|lower}";
        let mut vars = HashMap::new();
        vars.insert("name", "HELLO WORLD");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "hello world");
    }

    /// title 过滤器正确首字母大写
    #[test]
    fn test_title_filter() {
        let template = "${text|title}";
        let mut vars = HashMap::new();
        vars.insert("text", "hello world from narratoai");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "Hello World From Narratoai");
    }

    /// strip 过滤器正确去空白
    #[test]
    fn test_strip_filter() {
        let template = "--${text|strip}--";
        let mut vars = HashMap::new();
        vars.insert("text", "  hello world  ");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "--hello world--");
    }

    /// truncate 过滤器正确截断
    #[test]
    fn test_truncate_filter_short() {
        let template = "${text|truncate}";
        let mut vars = HashMap::new();
        vars.insert("text", "short text");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "short text");
    }

    #[test]
    fn test_truncate_filter_long() {
        let template = "${text|truncate}";
        let long_text = "A".repeat(150);
        let mut vars = HashMap::new();
        vars.insert("text", long_text.as_str());

        let result = render(template, &vars).expect("渲染应成功");
        assert!(result.ends_with("..."), "应包含省略号");
        assert_eq!(result.chars().count(), 100);
        assert_eq!(&result[..97], &long_text[..97]);
    }

    /// 中文截断测试——验证多字节 UTF-8 字符不会 panic
    #[test]
    fn test_truncate_filter_chinese() {
        let template = "${text|truncate}";
        let long_chinese: String = "你好世界".repeat(30); // 120 个中文字符
        let mut vars = HashMap::new();
        vars.insert("text", long_chinese.as_str());

        let result = render(template, &vars).expect("渲染应成功");
        assert!(result.ends_with("..."), "应包含省略号");
        assert_eq!(result.chars().count(), 100);
    }

    /// json 过滤器正确转义
    #[test]
    fn test_json_filter() {
        let template = "${text|json}";
        let mut vars = HashMap::new();
        vars.insert("text", "hello \"world\" & <test>");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "\"hello \\\"world\\\" & <test>\"");
    }

    /// 多变量混合替换
    #[test]
    fn test_multi_variable_mixed() {
        let template = "${subject} is ${adjective|upper} and $result.";
        let mut vars = HashMap::new();
        vars.insert("subject", "This");
        vars.insert("adjective", "awesome");
        vars.insert("result", "working");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "This is AWESOME and working.");
    }

    /// 特殊字符（中文、标点）在占位符值中的处理
    #[test]
    fn test_chinese_characters() {
        let template = "${title}: ${content}";
        let mut vars = HashMap::new();
        vars.insert("title", "视频解说");
        vars.insert("content", "这是一个关于自然的纪录片");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "视频解说: 这是一个关于自然的纪录片");
    }

    /// 过滤器与普通变量混合
    #[test]
    fn test_filter_with_plain_variable() {
        let template = "${name|upper} says: ${message}";
        let mut vars = HashMap::new();
        vars.insert("name", "alice");
        vars.insert("message", "hello");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "ALICE says: hello");
    }

    /// 多个缺失变量应全部报告
    #[test]
    fn test_multiple_missing_variables() {
        let template = "${a} + ${b} = ${c}";
        let mut vars = HashMap::new();
        vars.insert("a", "1");

        let err = render(template, &vars).expect_err("应报告缺失变量");
        match err {
            PromptError::TemplateRender(msg) => {
                assert!(msg.contains("b"), "应包含 b: {}", msg);
                assert!(msg.contains("c"), "应包含 c: {}", msg);
            }
            _ => panic!("应为 TemplateRender 错误"),
        }
    }

    /// 变量名只包含字母数字和下划线，不含 `|` 等特殊字符
    #[test]
    fn test_variable_name_only_word_chars() {
        let template = "${valid_name} and ${another_var}";
        let mut vars = HashMap::new();
        vars.insert("valid_name", "first");
        vars.insert("another_var", "second");

        let result = render(template, &vars).expect("渲染应成功");
        assert_eq!(result, "first and second");
    }
}
