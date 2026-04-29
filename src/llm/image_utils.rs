use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::imageops::FilterType::Lanczos3;

use crate::error::LLMError;

/// 将图片文件预处理为 base64 data URL（D-20）
///
/// 流水线：
/// image::open() -> resize(1024,Lanczos3) -> JPEG quality 85 -> base64 encode -> data URL
pub fn image_to_base64_data_url(path: &Path) -> Result<String, LLMError> {
    let img = image::open(path)
        .map_err(|e| LLMError::General(format!("图片加载失败: {}", e)))?;

    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err(LLMError::General("图片尺寸为零，无法处理".to_string()));
    }

    // 缩放到 1024px 保持宽高比，使用 Lanczos3 滤镜对齐 Python PIL.Image.thumbnail LANCZOS
    let thumb = if w > h {
        img.resize(1024, (1024 * h / w).max(1), Lanczos3)
    } else {
        img.resize((1024 * w / h).max(1), 1024, Lanczos3)
    };

    // JPEG 编码 quality=85
    let mut buf = Cursor::new(Vec::new());
    {
        use image::codecs::jpeg::JpegEncoder;
        let mut encoder = JpegEncoder::new_with_quality(&mut buf, 85);
        encoder
            .encode_image(&thumb)
            .map_err(|e| LLMError::General(format!("JPEG 编码失败: {}", e)))?;
    }

    // base64 编码
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.get_ref());

    Ok(format!("data:image/jpeg;base64,{}", b64))
}
