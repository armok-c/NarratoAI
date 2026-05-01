//! 素材下载与缓存模块
//!
//! 下载素材视频到本地缓存，使用 MD5 URL 去重（去掉 query string），
//! 写入时使用 temp 文件 + 原子重命名，下载后通过 ffprobe 验证。

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;

use uuid::Uuid;

use md5::{Md5, Digest};

use crate::config::types::ProxySection;
use crate::ffmpeg::probe::probe_audio;
use crate::material::searcher::{
    MaterialError, MaterialInfo, search_videos_pexels, search_videos_pixabay,
    build_client,
};

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

/// 计算 URL 的 MD5 缓存 key（去掉 query string，与 Python 版行为一致）
pub fn compute_cache_key(url: &str) -> String {
    let url_without_query = url.split('?').next().unwrap_or(url);
    let mut hasher = Md5::new();
    hasher.update(url_without_query.as_bytes());
    let result = hasher.finalize();
    format!("vid-{:x}", result)
}

// ---------------------------------------------------------------------------
// Single video download
// ---------------------------------------------------------------------------

/// 下载并保存单个视频到本地缓存
///
/// 流程：
/// 1. 创建保存目录
/// 2. 计算 MD5 缓存 key
/// 3. 缓存命中则跳过下载
/// 4. HTTP 下载视频到 temp 文件
/// 5. 原子重命名为最终路径
/// 6. ffprobe 验证（duration > 0）
pub fn save_video(
    video_url: &str,
    save_dir: &Path,
    proxy_section: &ProxySection,
) -> Result<PathBuf, MaterialError> {
    // 1. 创建保存目录
    std::fs::create_dir_all(save_dir)
        .map_err(|e| MaterialError::IoError(e.to_string()))?;

    // 2. 计算缓存 key
    let cache_key = compute_cache_key(video_url);
    let video_path = save_dir.join(format!("{}.mp4", cache_key));

    // 3. 缓存命中检查
    if video_path.exists() {
        let file_size = std::fs::metadata(&video_path)
            .map(|m| m.len())
            .unwrap_or(0);
        if file_size > 0 {
            tracing::info!("素材缓存命中: {}", video_path.display());
            return Ok(video_path);
        }
    }

    // 4. HTTP 下载
    tracing::info!("下载素材: {}", video_url);
    let client = build_client(proxy_section)?;
    let mut response = client
        .get(video_url)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .map_err(|e| MaterialError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(MaterialError::DownloadFailed(format!(
            "HTTP {}",
            response.status()
        )));
    }

    // 5. 流式写入临时文件（唯一后缀防止并发冲突）
    let temp_path = save_dir.join(format!("{}.{}.tmp", cache_key, Uuid::new_v4()));
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| MaterialError::IoError(e.to_string()))?;
    std::io::copy(&mut response, &mut file)
        .map_err(|e| MaterialError::DownloadFailed(format!("流式写入失败: {}", e)))?;
    drop(file);

    // 6. 原子重命名
    std::fs::rename(&temp_path, &video_path)
        .map_err(|e| MaterialError::IoError(e.to_string()))?;

    // 7. ffprobe 验证
    match probe_audio(&video_path) {
        Ok(duration) => {
            if duration <= 0.0 {
                let _ = std::fs::remove_file(&video_path);
                return Err(MaterialError::ValidationFailed(
                    "下载的视频时长为 0".into(),
                ));
            }
        }
        Err(e) => {
            let _ = std::fs::remove_file(&video_path);
            return Err(MaterialError::ValidationFailed(format!(
                "ffprobe 验证失败: {}",
                e
            )));
        }
    }

    tracing::info!("素材下载完成: {}", video_path.display());
    Ok(video_path)
}

// ---------------------------------------------------------------------------
// Orchestrated download
// ---------------------------------------------------------------------------

/// 编排搜索 + 下载 + 时长汇总（Python 版 1:1）
///
/// 1. 对每个 search_term 调用搜索函数获取所有匹配素材
/// 2. 去重（按 URL）
/// 3. 逐个下载
/// 4. 累积总时长，超过 audio_duration 后停止
pub fn download_videos(
    task_id: &str,
    search_terms: &[String],
    source: &str,
    target_width: u32,
    target_height: u32,
    audio_duration: f64,
    max_clip_duration: u32,
    api_keys_pexels: &[String],
    api_keys_pixabay: &[String],
    key_counter_pexels: &AtomicUsize,
    key_counter_pixabay: &AtomicUsize,
    proxy_section: &ProxySection,
) -> Result<Vec<PathBuf>, MaterialError> {
    // 使用固定相对路径（与 Python 版行为一致）。库模式调用者应确保 CWD 正确。
    let save_dir = PathBuf::from("storage").join("temp").join(task_id);

    // 1. 收集所有搜索结果
    let mut all_items: Vec<MaterialInfo> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for term in search_terms {
        let items = match source {
            "pexels" => search_videos_pexels(
                term,
                max_clip_duration,
                target_width,
                target_height,
                api_keys_pexels,
                key_counter_pexels,
                proxy_section,
            )?,
            "pixabay" => search_videos_pixabay(
                term,
                max_clip_duration,
                target_width,
                target_height,
                api_keys_pixabay,
                key_counter_pixabay,
                proxy_section,
            )?,
            _ => {
                return Err(MaterialError::ApiRequest(format!(
                    "未知素材源: {}",
                    source
                )))
            }
        };

        for item in items {
            if seen_urls.insert(item.video_url.clone()) {
                all_items.push(item);
            }
        }
    }

    // 2. 下载，直到累积时长达到 audio_duration
    let mut downloaded_paths: Vec<PathBuf> = Vec::new();
    let mut total_duration: f64 = 0.0;

    for item in &all_items {
        if total_duration >= audio_duration {
            break;
        }

        match save_video(&item.video_url, &save_dir, proxy_section) {
            Ok(path) => {
                // 使用搜索 API 返回的时长而非 ffprobe 实际时长（与 Python 版行为一致）
                total_duration += item.duration as f64;
                downloaded_paths.push(path);
            }
            Err(e) => {
                tracing::warn!("素材下载失败 (跳过): {}", e);
                continue;
            }
        }
    }

    Ok(downloaded_paths)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_compute_cache_key() {
        // MD5("https://example.com/video.mp4") = 已知值
        let key = compute_cache_key("https://example.com/video.mp4");
        assert!(
            key.starts_with("vid-"),
            "缓存 key 应以 vid- 开头: {}",
            key
        );
        assert_eq!(key.len(), 36, "vid- + 32 hex chars = 36: {}", key);
    }

    #[test]
    fn test_compute_cache_key_removes_query_string() {
        // 两个 URL 只有 query 不同，缓存 key 应相同
        let key1 = compute_cache_key("https://example.com/video.mp4?token=abc");
        let key2 = compute_cache_key("https://example.com/video.mp4?token=xyz");
        assert_eq!(key1, key2, "query string 不应影响缓存 key");
    }

    #[test]
    fn test_compute_cache_key_different_urls() {
        let key1 = compute_cache_key("https://example.com/video1.mp4");
        let key2 = compute_cache_key("https://example.com/video2.mp4");
        assert_ne!(key1, key2, "不同 URL 应产生不同缓存 key");
    }

    #[test]
    fn test_save_video_invalid_url() {
        // 无效 URL 下载应返回错误
        let save_dir = std::env::temp_dir().join("narratoai_test_material");
        let proxy = ProxySection::default();

        let result = save_video(
            "https://invalid.example.com/nonexistent.mp4",
            &save_dir,
            &proxy,
        );
        // 预期出错（网络不通或 404）
        assert!(result.is_err());
    }

    #[test]
    fn test_download_videos_no_terms() {
        let result = download_videos(
            "test",
            &[],
            "pexels",
            1920,
            1080,
            30.0,
            10,
            &[],
            &[],
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
            &ProxySection::default(),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty(), "无搜索词应返回空列表");
    }
}
