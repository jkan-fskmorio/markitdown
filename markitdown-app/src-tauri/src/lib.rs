// MarkItDown 桌面应用 - Rust 后端入口
// 注册所有 Tauri 插件和命令

mod commands;
mod models;

use commands::convert::ConversionState;
use std::collections::HashMap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 注册 Tauri 官方插件
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::default().build())
        // 管理转换状态
        .manage(ConversionState {
            active_tasks: std::sync::Mutex::new(HashMap::new()),
        })
        // 注册所有命令
        .invoke_handler(tauri::generate_handler![
            // 文件操作
            commands::file::get_file_info,
            commands::file::get_files_info,
            commands::file::read_text_file,
            commands::file::write_text_file,
            commands::file::open_in_os,
            // 转换操作
            commands::convert::convert,
            commands::convert::cancel,
            commands::convert::convert_batch,
            // 配置管理
            commands::config::get_config,
            commands::config::set_config,
            commands::config::reset_config,
            // AI 服务管理
            commands::ai::check_ollama,
            commands::ai::check_whisper,
            commands::ai::check_exiftool,
            commands::ai::check_ffmpeg,
            commands::ai::check_all_dependencies,
            // 历史记录
            commands::history::add_history_entry,
            commands::history::get_history,
            commands::history::clear_history,
            commands::history::delete_history_entry,
            commands::history::get_history_stats,
        ])
        .run(tauri::generate_context!())
        .expect("启动 MarkItDown 桌面应用时发生错误");
}