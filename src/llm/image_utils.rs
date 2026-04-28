use std::io::Cursor;
use std::path::Path;

use base64::Engine;

use crate::error::LLMError;

/// 将图片文件预处理为 base64 data URL（D-20）
///
/// 流水线：
/// image::open() -> thumbnail(1024,1024) -> JPEG quality 85 -> base64 encode -> data URL
pub fn image_to_base64_data_url(path: &Path) -> Result<String, LLMError> {
    let img = image::open(path)
        .map_err(|e| LLMError::Configuration(format!("图片加载失败: {}", e)))?;

    // 缩放到 1024px 保持宽高比，对齐 Python 的 PIL.Image.thumbnail((1024, 1024), LANCZOS)
    let thumb = img.thumbnail(1024, 1024);

    // JPEG 编码 quality=85
    let mut buf = Cursor::new(Vec::new());
    thumb
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| LLMError::Configuration(format!("JPEG 编码失败: {}", e)))?;

    // base64 编码
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.get_ref());

    Ok(format!("data:image/jpeg;base64,{}", b64))
}
