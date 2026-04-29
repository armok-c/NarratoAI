use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use base64::Engine;
use image::GenericImageView;  // required for .dimensions()
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

    if magic.starts_with(&[0xFF, 0xD8, 0xFF]) {
        // JPEG 直通：回寻到文件开头再完整读取，避免 TOCTOU 竞态
        file.seek(SeekFrom::Start(0))
            .map_err(|e| LLMError::General(format!("文件寻址失败: {}", e)))?;
        let mut raw_bytes = Vec::new();
        file.read_to_end(&mut raw_bytes)
            .map_err(|e| LLMError::General(format!("文件读取失败: {}", e)))?;
        // 验证 JPEG 完整性（通过完整解码）。避免重新编码导致的画质损失，
        // 但解码后内存占用可能较大（50 MB JPEG 解码约 200 MB）。
        if let Err(e) = image::load_from_memory(&raw_bytes) {
            return Err(LLMError::General(format!("JPEG 图片损坏: {}", e)));
        }
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
    // 使用 u64 运算避免 u32 乘法溢出（极端宽高比下 h > 4,194,303 时会溢出）
    let thumb = if w > h {
        let new_h = ((1024u64 * h as u64) / w as u64) as u32;
        img.resize(1024, new_h.max(1), Lanczos3)
    } else {
        let new_w = ((1024u64 * w as u64) / h as u64) as u32;
        img.resize(new_w.max(1), 1024, Lanczos3)
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
