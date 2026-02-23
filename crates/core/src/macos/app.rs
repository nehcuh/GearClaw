//! macOS application management
//!
//! Provides functions to launch, quit, and control applications.

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const OSASCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct AppManager;

impl AppManager {
    pub fn new() -> Self {
        Self
    }

    /// Launch an application by name
    pub async fn launch(&self, app_name: &str) -> Result<String, GearClawError> {
        let fut = Command::new("open").arg("-a").arg(app_name).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(format!(
                    "ERROR:TIMEOUT: 启动应用 {} 超时",
                    app_name
                ))
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("启动应用失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "启动应用 {} 失败: {}",
                app_name, stderr
            )));
        }

        Ok(format!("✓ 已启动应用: {}", app_name))
    }

    /// Quit an application by name
    pub async fn quit(&self, app_name: &str) -> Result<String, GearClawError> {
        let script = format!("tell application \"{}\" to quit", app_name);
        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(format!(
                    "ERROR:TIMEOUT: 退出应用 {} 超时",
                    app_name
                ))
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("退出应用失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "退出应用 {} 失败: {}",
                app_name, stderr
            )));
        }

        Ok(format!("✓ 已退出应用: {}", app_name))
    }

    /// Bring application to front
    pub async fn bring_to_front(&self, app_name: &str) -> Result<String, GearClawError> {
        let script = format!("tell application \"{}\" to activate", app_name);
        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(format!(
                    "ERROR:TIMEOUT: 切换应用 {} 超时",
                    app_name
                ))
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("切换应用失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "切换应用 {} 失败: {}",
                app_name, stderr
            )));
        }

        Ok(format!("✓ 已切换到前台: {}", app_name))
    }

    /// Check if application is running
    pub async fn is_running(&self, app_name: &str) -> Result<String, GearClawError> {
        let script = format!(
            "tell application \"System Events\" to return (name of processes) contains \"{}\"",
            app_name
        );
        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(format!(
                    "ERROR:TIMEOUT: 检查应用 {} 状态超时",
                    app_name
                ))
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("检查应用状态失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "检查应用 {} 失败: {}",
                app_name, stderr
            )));
        }

        let result_str = String::from_utf8_lossy(&output.stdout);
        let result = result_str.trim();
        let is_running = result == "true";

        Ok(if is_running {
            format!("✓ 应用 {} 正在运行", app_name)
        } else {
            format!("✗ 应用 {} 未运行", app_name)
        })
    }

    /// Get list of running applications
    pub async fn list_running(&self) -> Result<String, GearClawError> {
        let script = "tell application \"System Events\" to return name of (processes whose background only is false)";
        let fut = Command::new("osascript").arg("-e").arg(script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 获取应用列表超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("获取应用列表失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "获取应用列表失败: {}",
                stderr
            )));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        Ok(result.to_string())
    }
}
