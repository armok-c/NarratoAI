use regex::Regex;
use std::sync::OnceLock;

static TEMPLATE_VAR_REGEX: OnceLock<Regex> = OnceLock::new();

pub(crate) fn template_var_regex() -> &'static Regex {
    TEMPLATE_VAR_REGEX.get_or_init(|| {
        Regex::new(r"\$\{(\w+)(?:\|(\w+))?\}")
            .expect("TEMPLATE_VAR_REGEX 编译失败")
    })
}

pub mod types;
pub mod error;
pub mod registry;
pub mod template;
pub mod manager;
pub mod validators;
pub mod register;
