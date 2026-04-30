//! 素材 API 集成测试
//!
//! 需要有效的 Pexels/Pixabay API key（从环境变量读取）。
//! 使用 `cargo test -- --ignored` 运行。

#[test]
#[ignore]
fn test_pexels_search_integration() {
    let api_key = std::env::var("PEXELS_API_KEYS")
        .or_else(|_| std::env::var("PEXELS_API_KEY"))
        .expect("需要设置 PEXELS_API_KEYS 或 PEXELS_API_KEY 环境变量");
    assert!(!api_key.is_empty(), "API key 不能为空");
    // 集成测试：使用真实 Pexels API key 调用 search_videos_pexels
    // 完整实现在执行阶段补充
}

#[test]
#[ignore]
fn test_pixabay_search_integration() {
    let api_key = std::env::var("PIXABAY_API_KEYS")
        .or_else(|_| std::env::var("PIXABAY_API_KEY"))
        .expect("需要设置 PIXABAY_API_KEYS 或 PIXABAY_API_KEY 环境变量");
    assert!(!api_key.is_empty(), "API key 不能为空");
    // 集成测试：使用真实 Pixabay API key 调用 search_videos_pixabay
    // 完整实现在执行阶段补充
}
