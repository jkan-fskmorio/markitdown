// 文件操作命令
use std::fs;
use tauri::Manager;
use crate::models::FileInfo;

/// 获取文件信息
#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    let metadata = fs::metadata(&path).map_err(|e| format!("无法读取文件: {}", e))?;
    let path = std::path::Path::new(&path);

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let mime_type = mime_guess::from_path(path)
        .first_raw()
        .map(|s| s.to_string());

    Ok(FileInfo {
        name,
        path: path.to_string_lossy().to_string(),
        size: metadata.len(),
        extension,
        mime_type,
    })
}

/// 批量获取文件信息
#[tauri::command]
pub fn get_files_info(paths: Vec<String>) -> Result<Vec<FileInfo>, String> {
    paths
        .into_iter()
        .map(|p| get_file_info(p))
        .collect::<Result<Vec<_>, _>>()
}

/// 读取文本文件内容
#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 保存文本到文件
#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, &content).map_err(|e| format!("保存文件失败: {}", e))
}

/// 在系统默认程序中打开文件
#[tauri::command]
#[allow(deprecated)]
pub async fn open_in_os(app: tauri::AppHandle, path: String) -> Result<(), String> {
    tauri_plugin_shell::ShellExt::shell(&app)
        .open(path, None)
        .map_err(|e| format!("打开文件失败: {}", e))
}