use std::io::Cursor;
use std::io::Read;
use std::path::Path;

use base64::Engine;
use image::imageops::FilterType::Lanczos3;

use crate::error::LLMError;

/// 将图片文件预处理为 base64 data URL（D-20）
///
/// 流水线：
/// image::open() -> resize(1024,Lanczos3) -> JPEG quality 85 -> base64 encode -> data URL
pub fn image_to_base64_data_url(path: &Path) -> Result<String, LLMError> {
    // 文件大小检查（防御性限制，防止 OOM）
    let metadata = std::fs::metadata(path)
        .map_err(|e| LLMError::General(format!("无法读取文件元数据: {}", e)))?;

    const MAX_IMAGE_SIZE: u64 = 50 * 1024 * 1024; // 50 MB 限制
    if metadata.len() > MAX_IMAGE_SIZE {
        return Err(LLMError::General(format!(
            "图片文件过大: {} bytes (最大允许 {} bytes)",
            metadata.len(),
            MAX_IMAGE_SIZE
        )));
    }

    // JPEG 直通优化：仅读取前 4 字节检测 JPEG 魔术字节，避免全文件读取
    let mut file = std::fs::File::open(path)
        .map_err(|e| LLMError::General(format!("文件打开失败: {}", e)))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|e| LLMError::General(format!("文件读取失败: {}", e)))?;
    drop(file); // 尽早释放文件句柄

    if magic.starts_with(&[0xFF, 0xD8, 0xFF]) {
        // JPEG 直通：重新读取完整文件
        let raw_bytes = std::fs::read(path)
            .map_err(|e| LLMError::General(format!("文件读取失败: {}", e)))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&raw_bytes);
        return Ok(format!("data:image/jpeg;base64,{}", b64));
    }

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
