use std::path::{Path, PathBuf};

/// 生成一个 32x32 的测试 JPEG 图片并写入指定路径
/// 图片内容为纯红色（RGB 255,0,0），用于验证图片预处理流水线
pub fn write_test_jpeg(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::DynamicImage::new_rgba8(32, 32);
    // fill with red pixels
    for pixel in img.as_mut_rgba8().unwrap().pixels_mut() {
        *pixel = image::Rgba([255u8, 0u8, 0u8, 255u8]);
    }
    img.save_with_format(path, image::ImageFormat::Jpeg)?;
    Ok(())
}

/// 创建测试 JPEG 图片的路径并写入文件，返回路径
/// `dir` 通常来自 tempfile::TempDir 的 .path()
pub fn create_test_jpeg_path(dir: &Path) -> PathBuf {
    let path = dir.join("test_image.jpg");
    write_test_jpeg(&path).expect("测试图片写入失败");
    path
}

/// 构建一个模拟的 CreateChatCompletionResponse（供测试 extract_text 和消息构建使用）
/// 使用 async_openai 的公开类型构建
pub fn mock_chat_response(text: &str) -> async_openai::types::CreateChatCompletionResponse {
    use async_openai::types::*;
    CreateChatCompletionResponse {
        id: "test-response-id".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "test-model".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatCompletionResponseMessage {
                content: Some(text.to_string()),
                ..Default::default()
            },
            finish_reason: Some("stop".to_string()),
            ..Default::default()
        }],
        usage: None,
        ..Default::default()
    }
}
