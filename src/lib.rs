pub mod config;
pub mod ffmpeg;
pub mod error;
pub mod tts;
pub mod llm;

/// 库版本号，编译时从 Cargo.toml 注入
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_nonempty_semver() {
        assert!(!version().is_empty(), "版本号不应为空");
        assert!(version().contains('.'), "版本号应包含点号 (semver 格式)");
    }
}
