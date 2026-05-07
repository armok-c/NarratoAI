use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::prompt::error::PromptError;

/// 过滤器函数类型
type FilterFn = fn(&str) -> String;

use super::template_var_regex;

/// 内置过滤器映射缓存
static BUILTIN_FILTERS: OnceLock<HashMap<&'static str, FilterFn>> = OnceLock::new();

/// 返回内置过滤器映射表（6 个过滤器，缓存到 OnceLock）
fn builtin_filters() -> &'static HashMap<&'static str, FilterFn> {
    BUILTIN_FILTERS.get_or_init(|| {
        let mut m: HashMap<&'static str, FilterFn> = HashMap::new();

    // upper——转换为大写
    m.insert("upper", |s: &str| s.to_uppercase());

    // lower——转换为小写
    m.insert("lower", |s: &str| s.to_lowercase());

    // title——单词首字母大写（对 ASCII 字母生效）
    m.insert("title", |s: &str| {
        let mut out = String::with_capacity(s.len());
        let mut new_word = true;
        for c in s.chars() {
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
        serde_json::to_string(s).expect("serializing &str to JSON string cannot fail")
    });

        m
    })
}

/// 渲染模板——单遍正则替换（变量 + 过滤器）
///
/// 使用单一正则 `r"\$\{(\w+)(?:\|(\w+))?\}"` 同时匹配 `${variable}` 和
/// `${variable|filter_name}` 两种格式，在原始模板上完成所有替换。
///
/// 单一遍处理的优势：
/// - 变量值中的 `${...|...}` 文本不会被重新解释为过滤器表达式（防御模板注入）
/// - 变量和过滤器共享同一套校验逻辑，不存在验证路径分裂
/// - 消除多遍替换中重复的正则编译和遍历开销
///
/// # 流程
///
/// 第 1 步：正则捕获所有占位符，校验变量全部存在，校验过滤器名有效
/// 第 2 步：在原始模板上一次性替换所有占位符
///
/// # 注意
///
/// - 仅支持 `${variable}` 和 `${variable|filter}` 语法，不支持裸 `$variable`
/// - 不支持链式过滤器（如 `${var|f1|f2}`），添加链式支持需同时更新正则
pub fn render(template: &str, vars: &HashMap<&str, &str>) -> Result<String, PromptError> {
    // 同时匹配 ${variable} 和 ${variable|filter_name}
    // group 1 = variable name, group 2 = filter name (optional)
    let re = template_var_regex();

    let filters = builtin_filters();

    // 第 1 步：校验变量全部存在
    let mut missing: HashSet<String> = HashSet::new();
    for caps in re.captures_iter(template) {
        let name = caps
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or("");
        if !name.is_empty() && !vars.contains_key(name) {
            missing.insert(name.to_string());
        }
    }
    if !missing.is_empty() {
        let mut missing_list: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        missing_list.sort();
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}",
            missing_list.join(", ")
        )));
    }

    // 校验过滤器名全部有效
    for caps in re.captures_iter(template) {
        if let Some(filter_name) = caps.get(2).map(|m| m.as_str()) {
            if !filter_name.is_empty() && !filters.contains_key(filter_name) {
                return Err(PromptError::TemplateRender(format!(
                    "未知过滤器: {}", filter_name
                )));
            }
        }
    }

    // 第 2 步：在原始模板上一次性替换所有占位符
    let result = re.replace_all(template, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let value = vars.get(var_name).copied().unwrap_or("");

        match caps.get(2).map(|m| m.as_str()) {
            Some(filter_name) if !filter_name.is_empty() => {
                let filter_fn = filters.get(filter_name)
                    .expect("filter passed validation but not found");
                filter_fn(value)
            }
            _ => value.to_string(),
        }
    });

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 基本变量替换：${variable} 语法
    #[test]
    fn test_basic_variable_substitution() {
        let template = "Hello, ${name}! Welcome to ${place}.";
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
        assert_eq!(result.chars().take(97).collect::<String>(), long_text.chars().take(97).collect::<String>());
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
        let template = "${subject} is ${adjective|upper} and ${result}.";
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
