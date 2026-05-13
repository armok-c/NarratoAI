use serde::Serialize;
use std::sync::{Mutex, MutexGuard, PoisonError};
use sysinfo::System;
use tauri::State;

use crate::error::CommandError;

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
pub struct SystemState {
    inner: Mutex<System>,
}

impl SystemState {
    pub fn new(system: System) -> Self {
        Self { inner: Mutex::new(system) }
    }

    pub fn lock(&self) -> Result<MutexGuard<'_, System>, PoisonError<MutexGuard<'_, System>>> {
        self.inner.lock()
    }
}

/// 获取系统 CPU 和 RAM 使用率（D-32）
///
/// 从 Tauri State 中取出 SystemState，刷新系统信息后返回百分比值。
/// CPU 百分比由 sysinfo 内部的 delta 计算提供，RAM 百分比为
/// used_memory / total_memory * 100。
///
/// # Errors
///
/// - Mutex 锁定失败时返回 CommandError (INTERNAL_ERROR)
/// - total_memory 为 0 时 ram_percent 返回 0.0
#[tauri::command]
pub fn get_system_stats(state: State<'_, SystemState>) -> Result<SystemStats, CommandError> {
    let mut system = state
        .lock()
        .map_err(|e| CommandError {
            code: "INTERNAL_ERROR".into(),
            message: format!("无法锁定系统状态: {}", e),
        })?;

    system.refresh_all();

    let cpu_percent = system.global_cpu_usage() as f64;
    let ram_percent = calc_ram_percent(system.used_memory(), system.total_memory());

    Ok(SystemStats {
        cpu_percent,
        ram_percent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let state = SystemState::new(system);
        assert!(state.lock().is_ok());
    }

    /// 验证 calc_ram_percent 在 total 为 0 时返回 0.0
    #[test]
    fn test_ram_percent_zero_total() {
        assert_eq!(calc_ram_percent(0, 0), 0.0);
        assert_eq!(calc_ram_percent(100, 0), 0.0);
        assert_eq!(calc_ram_percent(100, 200), 50.0);
    }
}

/// 纯函数：计算 RAM 使用百分比
fn calc_ram_percent(used: u64, total: u64) -> f64 {
    if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}
