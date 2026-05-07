use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;

use narratoai_core::config::ConfigManager;
use narratoai_core::llm::registry::Registry;
use narratoai_core::prompt::manager::PromptManager;
use narratoai_core::prompt::registry::PromptRegistry;

pub mod error;
pub mod models;
pub mod commands;

/// Tauri 应用入口
///
/// 构建完整的 Tauri 2.0 桌面应用后端：
/// - 插件：dialog（文件选择对话框）
/// - 配置：启动时多路径加载 config.toml，注入 Tauri State（D-02）
/// - LLM/Prompt 注册表：初始化并注入 Arc/RwLock State（D-06）
/// - 命令注册：register_all_commands!() 展开全部 15 条命令（D-05）
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // ---- 1. 多路径回退加载 config.toml ----
            let config_path = find_config(app)?;
            let config_manager = ConfigManager::load(&config_path)
                .map_err(|e| format!("配置加载失败: {}", e))?;

            // ---- 2. 注入 AppConfig 到 Tauri State ----
            let app_config = config_manager.get().clone();
            app.manage(app_config);

            // ---- 3. 初始化 LLM Registry ----
            // tokio::sync::RwLock 用于 Tauri State——其 ReadGuard 实现 Send，
            // 允许在 async 命令的 .await 点间持有。
            let registry = Arc::new(tokio::sync::RwLock::new(Registry::new()));
            {
                let mut reg = registry.blocking_write();
                narratoai_core::llm::register::register_all_providers(
                    &config_manager.get(),
                    &mut reg,
                ).map_err(|errors| {
                    format!("LLM provider 注册失败: {:?}", errors)
                })?;
            }
            app.manage::<Arc<tokio::sync::RwLock<Registry>>>(registry);

            // ---- 4. 初始化 Prompt Registry + PromptManager ----
            // PromptRegistry 内部使用 std::sync::RwLock（SharedPromptRegistry 类型要求），
            // 外层用 tokio::sync::RwLock 包装 PromptManager 使 ReadGuard 可 Send。
            let prompt_registry: narratoai_core::prompt::registry::SharedPromptRegistry =
                Arc::new(std::sync::RwLock::new(PromptRegistry::new()));
            {
                let mut reg = prompt_registry.write().map_err(|e| {
                    format!("Prompt 注册表写入锁失败: {}", e)
                })?;
                narratoai_core::prompt::register::register_all_prompts(
                    &mut reg,
                ).map_err(|errors| {
                    format!("Prompt 注册失败: {:?}", errors)
                })?;
            }
            let prompt_manager = Arc::new(tokio::sync::RwLock::new(
                PromptManager::new(prompt_registry)
            ));
            app.manage::<Arc<tokio::sync::RwLock<PromptManager>>>(prompt_manager);

            Ok(())
        })
        .invoke_handler(register_all_commands!())
        .run(tauri::generate_context!())
        .expect("NarratoAI 应用启动失败");
}

/// 多路径回退查找 config.toml
///
/// 搜索顺序：
/// 1. Tauri resource_dir()/config.toml（production bundle 时使用）
/// 2. 当前工作目录/config.toml（development 常用）
/// 3. ./config.toml（fallback，与 Python 版兼容）
fn find_config(app: &tauri::App) -> Result<PathBuf, String> {
    // Path 1: resource_dir
    if let Ok(resource_dir) = app.path().resource_dir() {
        let path = resource_dir.join("config.toml");
        if path.exists() {
            return Ok(path);
        }
    }

    // Path 2: current_dir
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join("config.toml");
        if path.exists() {
            return Ok(path);
        }
    }

    // Path 3: plain fallback
    let path = PathBuf::from("config.toml");
    if path.exists() {
        return Ok(path);
    }

    Err("未找到 config.toml 配置文件。请将 config.toml 放置在应用资源目录、当前工作目录或应用根目录。".into())
}
