pub mod audio;
pub mod config;
pub mod documentary;
pub mod error;
pub mod ffmpeg;
pub mod jianying;
pub mod llm;
pub mod material;
pub mod prompt;
pub mod script;
pub mod sde;
pub mod sdp;
pub mod subtitle;
pub mod text_utils;
pub mod tts;
pub mod visual;
pub mod youtube;

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
