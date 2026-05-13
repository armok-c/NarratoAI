use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;
use tauri::State;

/// 系统资源使用数据（D-32）
///
/// get_system_stats 命令的返回值类型。所有字段为 f64 百分比，
/// 供前端 SystemMonitorBar 组件展示。
#[derive(Serialize)]
pub struct SystemStats {
    pub cpu_percent: f64,
    pub ram_percent: f64,
}

/// Tauri State 持有的 System 实例（D-32）
///
/// 使用 std::sync::Mutex 而非 tokio::sync::Mutex，因为 sysinfo 操作
/// 是同步的（无 .await 点），且每次调用只锁定几毫秒。
/// CPU 百分比依赖 delta 计算，因此必须复用同一个 System 实例。
pub struct SystemState(pub Mutex<System>);

/// 获取系统 CPU 和 RAM 使用率（D-32）
///
/// 从 Tauri State 中取出 SystemState，刷新系统信息后返回百分比值。
/// CPU 百分比由 sysinfo 内部的 delta 计算提供，RAM 百分比为
/// used_memory / total_memory * 100。
///
/// # Errors
///
/// - Mutex 锁定失败时返回 String 错误信息
/// - total_memory 为 0 时 ram_percent 返回 0.0
#[tauri::command]
pub async fn get_system_stats(state: State<'_, SystemState>) -> Result<SystemStats, String> {
    let mut system = state
        .0
        .lock()
        .map_err(|e| format!("无法锁定系统状态: {}", e))?;

    system.refresh_all();

    let cpu_percent = system.global_cpu_usage() as f64;

    let total_mem = system.total_memory();
    let ram_percent = if total_mem > 0 {
        (system.used_memory() as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };

    Ok(SystemStats {
        cpu_percent,
        ram_percent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::System;

    /// 验证 SystemStats 结构体可以构造
    #[test]
    fn test_system_stats_construction() {
        let stats = SystemStats {
            cpu_percent: 50.0,
            ram_percent: 75.5,
        };
        assert_eq!(stats.cpu_percent, 50.0);
        assert_eq!(stats.ram_percent, 75.5);
    }

    /// 验证 SystemState 可以包装 System 实例
    #[test]
    fn test_system_state_construction() {
        let system = System::new_all();
        let state = SystemState(Mutex::new(system));
        assert!(state.0.lock().is_ok());
    }

    /// 验证 total_memory 为 0 时 ram_percent 返回 0.0
    #[test]
    fn test_ram_percent_zero_total() {
        // 使用一个空的 System 实例（memory 字段为 0）
        let system = System::new();
        let cpu = system.global_cpu_usage() as f64;
        let total_mem = system.total_memory();
        let ram = if total_mem > 0 {
            (system.used_memory() as f64 / total_mem as f64) * 100.0
        } else {
            0.0
        };
        assert_eq!(ram, 0.0);
        // CPU 在新 System 实例上应返回合理值（可能为 0 或任意浮点数）
        assert!(cpu >= 0.0);
    }
}
