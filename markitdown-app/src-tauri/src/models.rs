// MARKITDOWN 桌面应用 - 数据模型定义
use serde::{Deserialize, Serialize};

/// 转换选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub keep_data_uris: Option<bool>,
    pub use_plugins: Option<bool>,
    pub llm_client: Option<String>,
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_api_base: Option<String>,
    pub llm_prompt: Option<String>,
    pub use_whisper: Option<bool>,
    pub whisper_model: Option<String>,
    pub use_exiftool: Option<bool>,
}

/// 转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResult {
    pub task_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub success: bool,
    pub markdown: Option<String>,
    pub title: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub format: String,
    pub converted_at: String,
}

/// 转换进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertProgress {
    pub task_id: String,
    pub file_name: String,
    pub percent: u32,
    pub stage: String,
    pub message: String,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub extension: String,
    pub mime_type: Option<String>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub output: OutputConfig,
    pub ai: AIConfig,
    pub tools: ToolsConfig,
    pub ui: UIConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output: OutputConfig::default(),
            ai: AIConfig::default(),
            tools: ToolsConfig::default(),
            ui: UIConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    pub default_dir: Option<String>,
    pub auto_open_after_convert: bool,
    pub keep_data_uris: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_dir: None,
            auto_open_after_convert: false,
            keep_data_uris: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIConfig {
    pub provider: String, // "none" | "ollama" | "openai" | "custom"
    pub ollama_url: Option<String>,
    pub openai_key: Option<String>,
    pub custom_url: Option<String>,
    pub custom_key: Option<String>,
    pub vision_model: String,
    pub whisper_model: String,
    pub vision_prompt: String,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: "none".into(),
            ollama_url: Some("http://localhost:11434".into()),
            openai_key: None,
            custom_url: None,
            custom_key: None,
            vision_model: "gpt-4o".into(),
            whisper_model: "base".into(),
            vision_prompt: "Write a detailed caption for this image.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    pub exiftool_path: Option<String>,
    pub ffmpeg_path: Option<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            exiftool_path: None,
            ffmpeg_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIConfig {
    pub theme: String,
    pub language: String,
    pub max_history: u32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "zh-CN".into(),
            max_history: 100,
        }
    }
}

/// 依赖状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub name: String,
    pub dep_type: String,
    pub required: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

/// 历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub task_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub output_path: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub converted_at: String,
    pub format: String,
}

/// 历史统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStats {
    pub total: u64,
    pub success: u64,
    pub fail: u64,
    pub total_size: u64,
}

/// 支持的格式信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatInfo {
    pub extension: String,
    pub name: String,
    pub category: String,
    pub requires_ai: bool,
    pub requires_external_tool: bool,
}