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
    fn test_version_matches_cargo_toml() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
