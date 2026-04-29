pub mod config;
pub mod error;
pub mod ffmpeg;
pub mod jianying;
pub mod script;

/// 库版本号，编译时从 Cargo.toml 注入
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_returns_0_1_0() {
        assert_eq!(version(), "0.1.0");
    }
}
