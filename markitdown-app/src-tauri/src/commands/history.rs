// 转换历史管理命令
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use crate::models::{HistoryEntry, HistoryStats};

/// 获取历史文件路径
fn history_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    Ok(config_dir.join("history.json"))
}

/// 读取历史记录
fn read_history(app: &tauri::AppHandle) -> Result<Vec<HistoryEntry>, String> {
    let path = history_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(&content).map_err(|e| format!("解析历史记录失败: {}", e))
    } else {
        Ok(Vec::new())
    }
}

/// 保存历史记录
fn save_history(app: &tauri::AppHandle, history: &[HistoryEntry]) -> Result<(), String> {
    let path = history_path(app)?;
    let content = serde_json::to_string_pretty(history)
        .map_err(|e| format!("序列化历史记录失败: {}", e))?;
    fs::write(&path, &content).map_err(|e| format!("保存历史记录失败: {}", e))
}

/// 添加历史记录
#[tauri::command]
pub fn add_history_entry(app: tauri::AppHandle, entry: HistoryEntry) -> Result<(), String> {
    let mut history = read_history(&app)?;
    history.insert(0, entry);
    save_history(&app, &history)
}

/// 获取历史记录（分页）
#[tauri::command]
pub fn get_history(
    app: tauri::AppHandle,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<HistoryEntry>, String> {
    let history = read_history(&app)?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50);

    Ok(history
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect())
}

/// 清空历史记录
#[tauri::command]
pub fn clear_history(app: tauri::AppHandle) -> Result<(), String> {
    save_history(&app, &[])
}

/// 删除单条历史记录
#[tauri::command]
pub fn delete_history_entry(app: tauri::AppHandle, task_id: String) -> Result<(), String> {
    let mut history = read_history(&app)?;
    history.retain(|e| e.task_id != task_id);
    save_history(&app, &history)
}

/// 获取历史统计
#[tauri::command]
pub fn get_history_stats(app: tauri::AppHandle) -> Result<HistoryStats, String> {
    let history = read_history(&app)?;
    let total = history.len() as u64;
    let success = history.iter().filter(|e| e.success).count() as u64;
    let fail = total - success;
    let total_size = history.iter().map(|e| e.file_size).sum();

    Ok(HistoryStats {
        total,
        success,
        fail,
        total_size,
    })
}