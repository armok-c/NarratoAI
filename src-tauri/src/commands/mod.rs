pub mod config;
pub mod export;
pub mod pipeline;
pub mod script;
pub mod system;

pub use config::*;
pub use export::*;
pub use pipeline::*;
pub use script::*;
pub use system::*;

/// 集中注册所有 Tauri 命令（D-05）
///
/// 对齐 Phase 2/3/4 的 register_all_providers()/register_all_prompts() 模式。
/// 在 Tauri builder 的 invoke_handler 中展开：
/// `.invoke_handler(register_all_commands!())`
///
/// 新增命令时需同步更新两处：
/// 1. 顶部 `pub mod` + `pub use` 列表
/// 2. 宏内的命令全路径列表
#[macro_export]
macro_rules! register_all_commands {
    () => {
        tauri::generate_handler![
            crate::commands::pipeline::run_documentary,
            crate::commands::pipeline::run_sde,
            crate::commands::pipeline::run_sdp,
            crate::commands::pipeline::generate_documentary_script,
            crate::commands::pipeline::generate_sdp_script,
            crate::commands::script::load_script,
            crate::commands::script::save_script,
            crate::commands::script::validate_script,
            crate::commands::script::get_script_info,
            crate::commands::script::update_narration,
            crate::commands::script::set_ost,
            crate::commands::script::update_timestamp,
            crate::commands::export::export_jianying_draft,
            crate::commands::system::get_system_stats,
            crate::commands::config::get_config,
            crate::commands::config::save_config,
            crate::commands::config::test_llm_connection,
            crate::commands::config::get_edge_tts_voices,
            crate::commands::config::get_system_proxy,
        ]
    };
}
