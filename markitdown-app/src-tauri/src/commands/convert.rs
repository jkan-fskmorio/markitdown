// 转换操作命令 - Sidecar 管理 (Tauri 2.x Shell Plugin)
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use chrono::Utc;
use uuid::Uuid;
use crate::models::{ConvertOptions, ConvertResult, ConvertProgress};

/// 活跃的转换任务
pub struct ConversionState {
    pub active_tasks: Mutex<HashMap<String, tauri_plugin_shell::process::CommandChild>>,
}

/// 单文件转换
#[tauri::command]
pub async fn convert(
    app: AppHandle,
    path: String,
    options: Option<ConvertOptions>,
) -> Result<ConvertResult, String> {
    let task_id = Uuid::new_v4().to_string();
    let file_name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_size = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);
    let format = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let start_time = std::time::Instant::now();

    log::info!("开始转换: {} (task: {})", file_name, task_id);

    // 发送初始进度
    let _ = app.emit("convert:progress", ConvertProgress {
        task_id: task_id.clone(),
        file_name: file_name.clone(),
        percent: 0,
        stage: "preparing".into(),
        message: "准备转换...".into(),
    });

    // 获取 sidecar 路径
    let sidecar_path = resolve_sidecar_path(&app)?;

    // 使用 tauri-plugin-shell 启动 sidecar
    let sidecar_command = app.shell()
        .command(&sidecar_path)
        .args([&path]);

    let (mut rx, child) = sidecar_command
        .spawn()
        .map_err(|e| format!("启动转换引擎失败: {}", e))?;

    // 注册活跃任务（用于取消）
    {
        let state = app.state::<ConversionState>();
        state.active_tasks.lock().unwrap().insert(task_id.clone(), child);
    }

    let _ = app.emit("convert:progress", ConvertProgress {
        task_id: task_id.clone(),
        file_name: file_name.clone(),
        percent: 10,
        stage: "converting".into(),
        message: "正在转换...".into(),
    });

    // 收集输出
    let mut output = String::new();
    let mut error_msg: Option<String> = None;

    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                output.push_str(&String::from_utf8_lossy(&line));
                output.push('\n');
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                let line_str = String::from_utf8_lossy(&line);
                if line_str.starts_with("PROGRESS:") {
                    if let Some(rest) = line_str.strip_prefix("PROGRESS:") {
                        let parts: Vec<&str> = rest.splitn(2, ':').collect();
                        if parts.len() >= 2 {
                            let percent: u32 = parts[0].parse().unwrap_or(0);
                            let _ = app.emit("convert:progress", ConvertProgress {
                                task_id: task_id.clone(),
                                file_name: file_name.clone(),
                                percent,
                                stage: "converting".into(),
                                message: parts[1].to_string(),
                            });
                        }
                    }
                } else {
                    log::warn!("Sidecar stderr: {}", line_str);
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(status) => {
                if status.code != Some(0) {
                    error_msg = Some(format!("转换失败，退出码: {:?}", status.code));
                }
                break;
            }
            tauri_plugin_shell::process::CommandEvent::Error(err) => {
                error_msg = Some(format!("转换引擎错误: {}", err));
                break;
            }
            _ => {}
        }
    }

    // 清理活跃任务
    {
        let state = app.state::<ConversionState>();
        state.active_tasks.lock().unwrap().remove(&task_id);
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    if let Some(err) = error_msg {
        let _ = app.emit("convert:error", serde_json::json!({
            "taskId": task_id,
            "fileName": file_name,
            "error": err,
        }));

        return Ok(ConvertResult {
            task_id: task_id.clone(),
            file_name: file_name.clone(),
            file_path: path,
            file_size,
            success: false,
            markdown: None,
            title: None,
            error: Some(err),
            duration_ms,
            format,
            converted_at: Utc::now().to_rfc3339(),
        });
    }

    let markdown = output.trim().to_string();

    let _ = app.emit("convert:progress", ConvertProgress {
        task_id: task_id.clone(),
        file_name: file_name.clone(),
        percent: 100,
        stage: "done".into(),
        message: "转换完成".into(),
    });

    let _ = app.emit("convert:complete", serde_json::json!({
        "taskId": task_id,
        "fileName": file_name,
        "success": true,
    }));

    Ok(ConvertResult {
        task_id,
        file_name,
        file_path: path,
        file_size,
        success: true,
        markdown: Some(markdown),
        title: None,
        error: None,
        duration_ms,
        format,
        converted_at: Utc::now().to_rfc3339(),
    })
}

/// 取消转换
#[tauri::command]
pub async fn cancel(app: AppHandle, task_id: String) -> Result<(), String> {
    let state = app.state::<ConversionState>();
    let mut tasks = state.active_tasks.lock().unwrap();

    if let Some(child) = tasks.remove(&task_id) {
        child.kill().map_err(|e| format!("取消转换失败: {}", e))?;
        let _ = app.emit("convert:cancelled", serde_json::json!({
            "taskId": task_id,
        }));
        log::info!("已取消转换: {}", task_id);
    }

    Ok(())
}

/// 批量转换
#[tauri::command]
pub async fn convert_batch(
    app: AppHandle,
    paths: Vec<String>,
    options: Option<ConvertOptions>,
) -> Result<Vec<ConvertResult>, String> {
    let total = paths.len();
    let mut results = Vec::with_capacity(total);

    for (i, path) in paths.into_iter().enumerate() {
        let completed = i as u32 + 1;
        let _ = app.emit("convert:batch-progress", serde_json::json!({
            "total": total,
            "completed": completed,
            "current": path,
        }));

        let result = convert(app.clone(), path, options.clone()).await?;
        results.push(result);
    }

    Ok(results)
}

/// 解析 sidecar 路径
fn resolve_sidecar_path(app: &AppHandle) -> Result<String, String> {
    // 优先级:
    // 1. 环境变量 MARKITDOWN_SIDECAR_PATH
    // 2. 开发环境: workspace 下的 Python 脚本
    // 3. 生产环境: 打包的 sidecar 二进制

    if let Ok(path) = std::env::var("MARKITDOWN_SIDECAR_PATH") {
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }

    // 开发环境: 使用 Python 直接运行 markitdown CLI
    let dev_script = app
        .path()
        .resource_dir()
        .map(|d| d.join("sidecar").join("convert.py"))
        .unwrap_or_default();

    if dev_script.exists() {
        return Ok(format!("python3 {}", dev_script.display()));
    }

    // 回退: 尝试直接调用 markitdown 命令
    Ok("markitdown".into())
}