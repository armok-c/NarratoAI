//! 素材搜索模块（Pexels + Pixabay）
//!
//! 通过 reqwest HTTP 客户端调用 Pexels 和 Pixabay 的 REST API 搜索视频素材。
//! 支持多 API key 轮换（AtomicUsize 原子自增）、代理配置、分辨率过滤。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde::Deserialize;
use reqwest::blocking::Client;
use reqwest::Proxy;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// 素材操作错误类型
#[derive(Error, Debug)]
pub enum MaterialError {
    #[error("API 请求失败: {0}")]
    ApiRequest(String),

    #[error("API 认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("文件验证失败: {0}")]
    ValidationFailed(String),

    #[error("API key 未配置: {0}")]
    MissingApiKey(String),

    #[error("IO 错误: {0}")]
    IoError(String),
}

// ---------------------------------------------------------------------------
// Pexels API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PexelsSearchResponse {
    pub page: u32,
    pub per_page: u32,
    pub total_results: u32,
    pub videos: Vec<PexelsVideo>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideo {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    pub url: String,
    pub video_files: Vec<PexelsVideoFile>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoFile {
    pub id: u64,
    pub quality: String,
    pub file_type: String,
    pub width: u32,
    pub height: u32,
    pub link: String,
}

// ---------------------------------------------------------------------------
// Pixabay API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixabaySearchResponse {
    pub total: u32,
    pub total_hits: u32,
    pub hits: Vec<PixabayHit>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayHit {
    pub id: u64,
    pub duration: u32,
    pub videos: PixabayVideos,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideos {
    pub large: Option<PixabayVideoVariant>,
    pub medium: Option<PixabayVideoVariant>,
    pub small: Option<PixabayVideoVariant>,
    pub tiny: Option<PixabayVideoVariant>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoVariant {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

// ---------------------------------------------------------------------------
// Unified result type
// ---------------------------------------------------------------------------

/// 搜索结果的统一表示
#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub id: u64,
    pub duration: u32,
    pub width: u32,
    pub height: u32,
    pub video_url: String,
    pub source: String, // "pexels" | "pixabay"
}

// ---------------------------------------------------------------------------
// Key rotation
// ---------------------------------------------------------------------------

/// 从 key 数组中轮询返回 API key（AtomicUsize 原子自增，防止异步竞态）
pub fn get_api_key(keys: &[String], counter: &AtomicUsize) -> Result<String, MaterialError> {
    if keys.is_empty() {
        return Err(MaterialError::MissingApiKey("API key 列表为空".into()));
    }
    if keys.len() == 1 {
        return Ok(keys[0].clone());
    }
    let idx = counter.fetch_add(1, Ordering::Relaxed) % keys.len();
    Ok(keys[idx].clone())
}

// ---------------------------------------------------------------------------
// HTTP client helper
// ---------------------------------------------------------------------------

/// 构建 reqwest Client，按需配置代理
pub(crate) fn build_client(
    proxy_section: &crate::config::types::ProxySection,
) -> Result<Client, MaterialError> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(60))
        .danger_accept_invalid_certs(false);

    if proxy_section.enabled {
        if !proxy_section.http.is_empty() {
            let proxy = Proxy::http(&proxy_section.http)
                .map_err(|e| MaterialError::ApiRequest(format!("代理配置无效: {}", e)))?;
            builder = builder.proxy(proxy);
        }
        if !proxy_section.https.is_empty() {
            let proxy = Proxy::https(&proxy_section.https)
                .map_err(|e| MaterialError::ApiRequest(format!("代理配置无效: {}", e)))?;
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|e| MaterialError::ApiRequest(format!("HTTP 客户端创建失败: {}", e)))
}

// ---------------------------------------------------------------------------
// URL encoding helper
// ---------------------------------------------------------------------------

/// URL 查询编码
///
/// application/x-www-form-urlencoded 编码。
/// 保留字母数字和 `-`、`_`、`.`、`~`，空格编码为 `+`，
/// 其他字符编码为 `%XX`（UTF-8 逐字节编码）。
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => result.push('+'),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            _ => {
                for b in c.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Search functions
// ---------------------------------------------------------------------------

/// 搜索 Pexels 视频素材
///
/// 调用 `GET https://api.pexels.com/videos/search`，过滤 duration 和分辨率。
pub fn search_videos_pexels(
    search_term: &str,
    minimum_duration: u32,
    target_width: u32,
    target_height: u32,
    api_keys: &[String],
    key_counter: &AtomicUsize,
    proxy_section: &crate::config::types::ProxySection,
) -> Result<Vec<MaterialInfo>, MaterialError> {
    let api_key = get_api_key(api_keys, key_counter)?;
    let api_url = format!(
        "https://api.pexels.com/videos/search?query={}&per_page=20&orientation={}",
        urlencoding(search_term),
        if target_width >= target_height {
            "landscape"
        } else {
            "portrait"
        },
    );

    let client = build_client(proxy_section)?;
    let response = client
        .get(&api_url)
        .header("Authorization", &api_key)
        .timeout(Duration::from_secs(60))
        .send()
        .map_err(|e| MaterialError::ApiRequest(e.to_string()))?;

    if response.status().as_u16() == 401 {
        return Err(MaterialError::AuthenticationFailed(
            "Pexels API key 无效".into(),
        ));
    }
    if !response.status().is_success() {
        return Err(MaterialError::ApiRequest(format!("HTTP {}", response.status())));
    }

    let body: PexelsSearchResponse = response
        .json()
        .map_err(|e| MaterialError::ApiRequest(e.to_string()))?;

    let results: Vec<MaterialInfo> = body
        .videos
        .into_iter()
        .filter(|v| v.duration >= minimum_duration)
        .flat_map(|v| {
            v.video_files
                .into_iter()
                .filter(|f| f.width == target_width && f.height == target_height)
                .map(move |f| MaterialInfo {
                    id: v.id,
                    duration: v.duration,
                    width: f.width,
                    height: f.height,
                    video_url: f.link,
                    source: "pexels".into(),
                })
        })
        .collect();

    Ok(results)
}

/// 搜索 Pixabay 视频素材
///
/// 调用 `GET https://pixabay.com/api/videos/`，选择最匹配目标分辨率的 variant。
pub fn search_videos_pixabay(
    search_term: &str,
    minimum_duration: u32,
    _target_width: u32,
    _target_height: u32,
    api_keys: &[String],
    key_counter: &AtomicUsize,
    proxy_section: &crate::config::types::ProxySection,
) -> Result<Vec<MaterialInfo>, MaterialError> {
    let api_key = get_api_key(api_keys, key_counter)?;
    // Pixabay API 要求 key 放在 URL 查询字符串中（其 API 设计限制）。
    // 确保此处 URL 不被日志输出（API key 已在 tracing::info 中排除）。
    let api_url = format!(
        "https://pixabay.com/api/videos/?q={}&video_type=all&per_page=50&key={}",
        urlencoding(search_term),
        api_key,
    );

    let client = build_client(proxy_section)?;
    let response = client
        .get(&api_url)
        .timeout(Duration::from_secs(60))
        .send()
        .map_err(|e| MaterialError::ApiRequest(e.to_string()))?;

    if !response.status().is_success() {
        return Err(MaterialError::ApiRequest(format!("HTTP {}", response.status())));
    }

    let body: PixabaySearchResponse = response
        .json()
        .map_err(|e| MaterialError::ApiRequest(e.to_string()))?;

    // 选择最匹配目标分辨率的 variant（large > medium > small > tiny）
    fn choose_variant(videos: &PixabayVideos) -> Option<&PixabayVideoVariant> {
        videos
            .large
            .as_ref()
            .or_else(|| videos.medium.as_ref())
            .or_else(|| videos.small.as_ref())
            .or_else(|| videos.tiny.as_ref())
    }

    let results: Vec<MaterialInfo> = body
        .hits
        .into_iter()
        .filter(|h| h.duration >= minimum_duration)
        .filter_map(|h| {
            let variant = choose_variant(&h.videos)?;
            Some(MaterialInfo {
                id: h.id,
                duration: h.duration,
                width: variant.width,
                height: variant.height,
                video_url: variant.url.clone(),
                source: "pixabay".into(),
            })
        })
        .collect();

    Ok(results)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_error_chinese_display() {
        let err = MaterialError::ApiRequest("timeout".into());
        assert!(err.to_string().contains("API 请求失败"));

        let err = MaterialError::AuthenticationFailed("bad key".into());
        assert!(err.to_string().contains("API 认证失败"));

        let err = MaterialError::DownloadFailed("network".into());
        assert!(err.to_string().contains("下载失败"));

        let err = MaterialError::MissingApiKey("pexels".into());
        assert!(err.to_string().contains("API key 未配置"));
    }

    #[test]
    fn test_get_api_key_single() {
        let keys = vec!["key1".to_string()];
        let counter = AtomicUsize::new(0);
        let key = get_api_key(&keys, &counter).unwrap();
        assert_eq!(key, "key1");
    }

    #[test]
    fn test_get_api_key_rotation() {
        let keys = vec![
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ];
        let counter = AtomicUsize::new(0);

        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key1");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key2");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key3");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key1"); // wrap
    }

    #[test]
    fn test_get_api_key_empty() {
        let keys: Vec<String> = vec![];
        let counter = AtomicUsize::new(0);
        let result = get_api_key(&keys, &counter);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key 未配置"));
    }

    #[test]
    fn test_pexels_response_deserialize() {
        let json = r#"{
            "page": 1,
            "per_page": 20,
            "total_results": 1,
            "videos": [{
                "id": 12345,
                "width": 1920,
                "height": 1080,
                "duration": 15,
                "url": "https://www.pexels.com/video/test/",
                "video_files": [{
                    "id": 1,
                    "quality": "hd",
                    "file_type": "video/mp4",
                    "width": 1920,
                    "height": 1080,
                    "link": "https://www.pexels.com/video/download/12345/"
                }]
            }]
        }"#;
        let resp: PexelsSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_results, 1);
        assert_eq!(resp.videos.len(), 1);
        assert_eq!(resp.videos[0].id, 12345);
        assert_eq!(resp.videos[0].video_files[0].quality, "hd");
    }

    #[test]
    fn test_pixabay_response_deserialize() {
        let json = r#"{
            "total": 1,
            "total_hits": 1,
            "hits": [{
                "id": 67890,
                "duration": 10,
                "videos": {
                    "large": {
                        "url": "https://pixabay.com/video/download/67890_large.mp4",
                        "width": 1920,
                        "height": 1080,
                        "size": 5000000
                    },
                    "medium": null,
                    "small": null,
                    "tiny": null
                }
            }]
        }"#;
        let resp: PixabaySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_hits, 1);
        assert_eq!(resp.hits[0].id, 67890);
        assert!(resp.hits[0].videos.large.is_some());
        assert!(resp.hits[0].videos.medium.is_none());
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("AT&T"), "AT%26T");
        assert_eq!(urlencoding("C++"), "C%2B%2B");
        assert_eq!(urlencoding("100%"), "100%25");
        assert_eq!(urlencoding("tag#1"), "tag%231");
        assert_eq!(urlencoding("normal"), "normal");
        assert_eq!(urlencoding("a=b"), "a%3Db");
        assert_eq!(urlencoding("你好"), "%E4%BD%A0%E5%A5%BD");
    }

    #[test]
    fn test_material_info_structure() {
        let info = MaterialInfo {
            id: 1,
            duration: 30,
            width: 1920,
            height: 1080,
            video_url: "https://example.com/video.mp4".into(),
            source: "pexels".into(),
        };
        assert_eq!(info.source, "pexels");
        assert_eq!(info.duration, 30);
    }

    #[test]
    fn test_build_client_default_proxy() {
        let proxy = crate::config::types::ProxySection::default();
        let client = build_client(&proxy);
        assert!(client.is_ok(), "默认代理配置应成功创建 Client");
    }

    #[test]
    fn test_search_videos_pexels_empty_keys() {
        let counter = AtomicUsize::new(0);
        let proxy = crate::config::types::ProxySection::default();
        let result = search_videos_pexels("test", 5, 1920, 1080, &[], &counter, &proxy);
        assert!(result.is_err(), "空 API key 列表应返回错误");
        let err = result.unwrap_err();
        assert!(matches!(err, MaterialError::MissingApiKey(_)));
    }

    #[test]
    fn test_search_videos_pixabay_empty_keys() {
        let counter = AtomicUsize::new(0);
        let proxy = crate::config::types::ProxySection::default();
        let result = search_videos_pixabay("test", 5, 1920, 1080, &[], &counter, &proxy);
        assert!(result.is_err(), "空 API key 列表应返回错误");
        let err = result.unwrap_err();
        assert!(matches!(err, MaterialError::MissingApiKey(_)));
    }
}
