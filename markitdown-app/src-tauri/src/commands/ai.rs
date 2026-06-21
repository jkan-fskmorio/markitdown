// AI 服务管理命令
use std::process::Command;
use crate::models::DependencyStatus;

/// 检查 Ollama 状态
#[tauri::command]
pub async fn check_ollama() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let models = json["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|m| m["name"].as_str().unwrap_or("").to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            Ok(serde_json::json!({
                "installed": true,
                "running": true,
                "models": models,
            }))
        }
        Ok(_r) => {
            // 检查 Ollama 是否安装但未运行
            let which = which::which("ollama");
            Ok(serde_json::json!({
                "installed": which.is_ok(),
                "running": false,
                "models": [],
            }))
        }
        Err(_) => {
            let which = which::which("ollama");
            Ok(serde_json::json!({
                "installed": which.is_ok(),
                "running": false,
                "models": [],
            }))
        }
    }
}

/// 检查 Whisper 状态
#[tauri::command]
pub fn check_whisper() -> DependencyStatus {
    // 检查 whisper 命令是否可用
    let installed = which::which("whisper").is_ok();

    // 检查缓存目录中的模型
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("whisper"))
        .unwrap_or_default();

    let models = if cache_dir.exists() {
        std::fs::read_dir(&cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    DependencyStatus {
        name: "Whisper (语音识别)".into(),
        dep_type: "model".into(),
        required: false,
        installed: installed || !models.is_empty(),
        version: if !models.is_empty() {
            Some(models.join(", "))
        } else {
            None
        },
        path: Some(cache_dir.to_string_lossy().to_string()),
        message: if installed || !models.is_empty() {
            "Whisper 已安装".into()
        } else {
            "建议安装 Whisper 以支持语音转录".into()
        },
    }
}

/// 检查 exiftool 状态
#[tauri::command]
pub fn check_exiftool() -> DependencyStatus {
    let installed = which::which("exiftool").is_ok();
    let version = if installed {
        Command::new("exiftool")
            .arg("-ver")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    DependencyStatus {
        name: "ExifTool (元数据提取)".into(),
        dep_type: "system".into(),
        required: false,
        installed,
        version,
        path: None,
        message: if installed {
            "ExifTool 已安装".into()
        } else {
            "建议安装 ExifTool 以提取文件元数据".into()
        },
    }
}

/// 检查 ffmpeg 状态
#[tauri::command]
pub fn check_ffmpeg() -> DependencyStatus {
    let installed = which::which("ffmpeg").is_ok();

    DependencyStatus {
        name: "FFmpeg (音视频处理)".into(),
        dep_type: "system".into(),
        required: false,
        installed,
        version: None,
        path: None,
        message: if installed {
            "FFmpeg 已安装".into()
        } else {
            "建议安装 FFmpeg 以支持音视频转换".into()
        },
    }
}

/// 一键检查所有依赖
#[tauri::command]
pub async fn check_all_dependencies() -> Result<Vec<DependencyStatus>, String> {
    let mut deps = Vec::new();

    // Python 环境
    let python_installed = which::which("python3").is_ok() || which::which("python").is_ok();
    deps.push(DependencyStatus {
        name: "Python 3 (转换引擎)".into(),
        dep_type: "python".into(),
        required: true,
        installed: python_installed,
        version: None,
        path: None,
        message: if python_installed {
            "Python 已安装".into()
        } else {
            "Python 3 是必需的运行环境".into()
        },
    });

    // MarkItDown
    let markitdown_installed = which::which("markitdown").is_ok();
    deps.push(DependencyStatus {
        name: "MarkItDown (核心转换库)".into(),
        dep_type: "python".into(),
        required: true,
        installed: markitdown_installed,
        version: None,
        path: None,
        message: if markitdown_installed {
            "MarkItDown 已安装".into()
        } else {
            "需要安装 markitdown 才能使用转换功能".into()
        },
    });

    deps.push(check_ollama().await.map(|j| {
        let installed = j["installed"].as_bool().unwrap_or(false);
        let running = j["running"].as_bool().unwrap_or(false);
        DependencyStatus {
            name: "Ollama (本地 AI)".into(),
            dep_type: "system".into(),
            required: false,
            installed,
            version: Some(if running {
                "运行中".into()
            } else {
                "未运行".into()
            }),
            path: None,
            message: if running {
                "Ollama 运行中，可使用本地 AI 模型".into()
            } else if installed {
                "Ollama 已安装但未运行".into()
            } else {
                "建议安装 Ollama 以使用本地 AI 功能".into()
            },
        }
    }).unwrap_or_else(|_| DependencyStatus {
        name: "Ollama (本地 AI)".into(),
        dep_type: "system".into(),
        required: false,
        installed: false,
        version: None,
        path: None,
        message: "建议安装 Ollama 以使用本地 AI 功能".into(),
    }));

    deps.push(check_whisper());
    deps.push(check_exiftool());
    deps.push(check_ffmpeg());

    Ok(deps)
}