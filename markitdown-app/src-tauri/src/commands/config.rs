// 配置管理命令
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use crate::models::AppConfig;

/// 获取配置目录
fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    Ok(config_dir.join("config.json"))
}

/// 获取全局配置
#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let path = config_path(&app)?;

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))
    } else {
        let config = AppConfig::default();
        // 保存默认配置
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, &content)
            .map_err(|e| format!("保存配置失败: {}", e))?;
        Ok(config)
    }
}

/// 保存全局配置
#[tauri::command]
pub fn set_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&path, &content)
        .map_err(|e| format!("保存配置失败: {}", e))
}

/// 重置配置为默认值
#[tauri::command]
pub fn reset_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let config = AppConfig::default();
    set_config(app.clone(), config.clone())?;
    Ok(config)
}