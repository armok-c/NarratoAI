pub mod config;
pub mod error;

/// 库版本号，编译时从 Cargo.toml 注入
pub fn version() -> &'static str {
    "" // RED phase: wrong value, will fail test
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_returns_0_1_0() {
        assert_eq!(version(), "0.1.0");
    }
}
