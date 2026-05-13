use std::time::{Duration, UNIX_EPOCH};

use narratoai_core::config::types::AppConfig;
use narratoai_core::config::ConfigManager;
use narratoai_core::tts::common::ProxyConfig;
use serde_json::{json, Value};
use tauri::State;

use crate::error::CommandError;

const EDGE_VOICE_LIST_URL: &str =
    "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list";
const EDGE_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
    "AppleWebKit/537.36 (KHTML, like Gecko) ",
    "Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0"
);

#[tauri::command]
pub fn get_config(config_manager: State<'_, ConfigManager>) -> Result<Value, CommandError> {
    let config = config_manager.get();
    config_to_json(&config, true)
}

#[tauri::command]
pub async fn save_config(
    updates: Value,
    config_manager: State<'_, ConfigManager>,
    expected_mtime: Option<i64>,
) -> Result<Value, CommandError> {
    let config_path = config_manager.config_path().to_path_buf();

    if let Some(expected_mtime) = expected_mtime {
        let actual_mtime = config_file_mtime(&config_path)?;
        if actual_mtime != expected_mtime {
            let current_config = config_to_json(&config_manager.get(), true)?;
            return Ok(json!({
                "status": "conflict",
                "current_config": current_config
            }));
        }
    }

    if !updates.is_object() {
        return Err(command_error(
            "INVALID_CONFIG_UPDATE",
            "配置更新必须是 JSON 对象",
        ));
    }
    let updates = strip_masked_secrets(updates);

    let config_arc = config_manager.config();
    let current_config = config_arc
        .read()
        .map_err(|e| command_error("CONFIG_LOCK_ERROR", format!("读取配置锁失败: {}", e)))?
        .clone();

    let mut merged_value = config_to_json(&current_config, false)?;
    merge_json(&mut merged_value, &updates);

    let merged_config: AppConfig = serde_json::from_value(merged_value.clone())
        .map_err(|e| command_error("CONFIG_PARSE_ERROR", format!("配置更新格式错误: {}", e)))?;

    let toml_value = toml::Value::try_from(merged_value)
        .map_err(|e| command_error("CONFIG_SERIALIZE_ERROR", format!("配置转换 TOML 失败: {}", e)))?;
    let toml_content = toml::to_string_pretty(&toml_value)
        .map_err(|e| command_error("CONFIG_SERIALIZE_ERROR", format!("配置序列化失败: {}", e)))?;

    std::fs::write(&config_path, toml_content)
        .map_err(|e| command_error("CONFIG_WRITE_ERROR", format!("写入配置失败: {}", e)))?;

    let mut guard = config_arc
        .write()
        .map_err(|e| command_error("CONFIG_LOCK_ERROR", format!("写入配置锁失败: {}", e)))?;
    *guard = merged_config;

    Ok(json!({ "status": "ok" }))
}

#[tauri::command]
pub async fn test_llm_connection(
    provider: String,
    model: String,
    api_key: String,
    base_url: String,
) -> Result<String, CommandError> {
    let endpoint = format!(
        "{}/v1/chat/completions",
        base_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| command_error("LLM_ERROR", format!("构建 HTTP 客户端失败: {}", e)))?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [{ "role": "user", "content": "Hi" }],
            "max_tokens": 5
        }))
        .send()
        .await
        .map_err(|e| command_error("LLM_ERROR", format!("{} 连接失败: {}", provider, e)))?;

    let status = response.status();
    if status.is_success() {
        Ok("连接成功".to_string())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(command_error(
            "LLM_ERROR",
            format!("{} 连接失败: HTTP {} {}", provider, status, body),
        ))
    }
}

#[tauri::command]
pub async fn get_edge_tts_voices(
    config_manager: State<'_, ConfigManager>,
) -> Result<Value, CommandError> {
    let config = config_manager.get();
    let proxy_config = ProxyConfig::from_proxy(Some(&config.proxy));
    let builder = reqwest::Client::builder().timeout(Duration::from_secs(30));
    let client = proxy_config
        .apply_to_client(builder)
        .build()
        .map_err(|e| command_error("TTS_ERROR", format!("构建 Edge-TTS 客户端失败: {}", e)))?;

    let response = client
        .get(EDGE_VOICE_LIST_URL)
        .header("User-Agent", EDGE_USER_AGENT)
        .send()
        .await
        .map_err(|e| command_error("TTS_ERROR", format!("获取 Edge-TTS 音色失败: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(command_error(
            "TTS_ERROR",
            format!("获取 Edge-TTS 音色失败: HTTP {} {}", status, body),
        ));
    }

    response
        .json::<Value>()
        .await
        .map_err(|e| command_error("TTS_ERROR", format!("解析 Edge-TTS 音色失败: {}", e)))
}

#[tauri::command]
pub fn get_system_proxy() -> Result<Value, CommandError> {
    let env_https = env_any(&["HTTPS_PROXY", "https_proxy"]);
    let env_http = env_any(&["HTTP_PROXY", "http_proxy"]);
    let env_all = env_any(&["ALL_PROXY", "all_proxy"]);
    let https = env_https.clone().or_else(|| env_all.clone()).unwrap_or_default();
    let http = env_http.clone().or_else(|| env_all.clone()).unwrap_or_default();

    Ok(json!({
        "https": https,
        "http": http,
        "env_https": env_https,
        "env_http": env_http,
        "env_all": env_all
    }))
}

fn config_to_json(config: &AppConfig, mask_secrets: bool) -> Result<Value, CommandError> {
    let mut value = serde_json::to_value(config)
        .map_err(|e| command_error("CONFIG_SERIALIZE_ERROR", format!("配置序列化失败: {}", e)))?;

    set_secret(&mut value, &["app", "vision_openai_api_key"], &config.app.vision_openai_api_key, mask_secrets)?;
    set_secret(&mut value, &["app", "text_openai_api_key"], &config.app.text_openai_api_key, mask_secrets)?;
    set_secret(&mut value, &["azure", "speech_key"], &config.azure.speech_key, mask_secrets)?;
    set_secret(&mut value, &["tencent", "secret_id"], &config.tencent.secret_id, mask_secrets)?;
    set_secret(&mut value, &["tencent", "secret_key"], &config.tencent.secret_key, mask_secrets)?;
    set_secret(&mut value, &["soulvoice", "api_key"], &config.soulvoice.api_key, mask_secrets)?;
    set_secret(&mut value, &["tts_qwen", "api_key"], &config.tts_qwen.api_key, mask_secrets)?;
    set_secret(&mut value, &["doubaotts", "ak"], &config.doubaotts.ak, mask_secrets)?;
    set_secret(&mut value, &["doubaotts", "sk"], &config.doubaotts.sk, mask_secrets)?;
    set_secret(&mut value, &["doubaotts", "token"], &config.doubaotts.token, mask_secrets)?;

    Ok(value)
}

fn set_secret(
    root: &mut Value,
    path: &[&str],
    raw: &str,
    mask: bool,
) -> Result<(), CommandError> {
    let value = if mask {
        Value::String(mask_secret(raw))
    } else {
        Value::String(raw.to_string())
    };
    set_path(root, path, value)
}

fn set_path(root: &mut Value, path: &[&str], value: Value) -> Result<(), CommandError> {
    let mut current = root;
    for key in &path[..path.len() - 1] {
        current = current
            .as_object_mut()
            .and_then(|obj| obj.get_mut(*key))
            .ok_or_else(|| command_error("CONFIG_SERIALIZE_ERROR", format!("缺少配置段: {}", key)))?;
    }

    let last = path[path.len() - 1];
    current
        .as_object_mut()
        .ok_or_else(|| command_error("CONFIG_SERIALIZE_ERROR", "配置段不是对象"))?
        .insert(last.to_string(), value);
    Ok(())
}

fn mask_secret(secret: &str) -> String {
    if secret.is_empty() {
        String::new()
    } else if secret.chars().count() <= 8 {
        "****".to_string()
    } else {
        let prefix: String = secret.chars().take(3).collect();
        let suffix: String = secret
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{}...****...{}", prefix, suffix)
    }
}

fn merge_json(base: &mut Value, updates: &Value) {
    match (base, updates) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_json(base_value, update_value),
                    None => {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                }
            }
        }
        (base_value, update_value) => {
            *base_value = update_value.clone();
        }
    }
}

fn strip_masked_secrets(mut updates: Value) -> Value {
    remove_masked_secret(&mut updates, &["app", "vision_openai_api_key"]);
    remove_masked_secret(&mut updates, &["app", "text_openai_api_key"]);
    remove_masked_secret(&mut updates, &["azure", "speech_key"]);
    remove_masked_secret(&mut updates, &["tencent", "secret_id"]);
    remove_masked_secret(&mut updates, &["tencent", "secret_key"]);
    remove_masked_secret(&mut updates, &["soulvoice", "api_key"]);
    remove_masked_secret(&mut updates, &["tts_qwen", "api_key"]);
    remove_masked_secret(&mut updates, &["doubaotts", "ak"]);
    remove_masked_secret(&mut updates, &["doubaotts", "sk"]);
    remove_masked_secret(&mut updates, &["doubaotts", "token"]);
    updates
}

fn remove_masked_secret(root: &mut Value, path: &[&str]) {
    let mut current = root;
    for key in &path[..path.len() - 1] {
        match current.as_object_mut().and_then(|obj| obj.get_mut(*key)) {
            Some(next) => current = next,
            None => return,
        }
    }

    let Some(obj) = current.as_object_mut() else {
        return;
    };
    let last = path[path.len() - 1];
    let should_remove = obj
        .get(last)
        .and_then(Value::as_str)
        .map(|value| value.contains("...****...") || value == "****")
        .unwrap_or(false);
    if should_remove {
        obj.remove(last);
    }
}

fn config_file_mtime(path: &std::path::Path) -> Result<i64, CommandError> {
    let modified = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|e| command_error("CONFIG_IO_ERROR", format!("读取配置修改时间失败: {}", e)))?;
    let seconds = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|e| command_error("CONFIG_IO_ERROR", format!("配置修改时间无效: {}", e)))?
        .as_secs() as i64;
    Ok(seconds)
}

fn env_any(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| std::env::var(key).ok())
        .filter(|value| !value.trim().is_empty())
}

fn command_error(code: impl Into<String>, message: impl Into<String>) -> CommandError {
    CommandError {
        code: code.into(),
        message: message.into(),
    }
}
