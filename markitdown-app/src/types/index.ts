// MARKITDOWN 桌面应用 - 前端类型定义
// 与 Rust 后端 models.rs 保持一致

/** 转换选项 */
export interface ConvertOptions {
  keepDataUris?: boolean;
  usePlugins?: boolean;
  llmClient?: string;
  llmModel?: string;
  llmApiKey?: string;
  llmApiBase?: string;
  llmPrompt?: string;
  useWhisper?: boolean;
  whisperModel?: string;
  useExiftool?: boolean;
}

/** 转换结果 */
export interface ConvertResult {
  taskId: string;
  fileName: string;
  filePath: string;
  fileSize: number;
  success: boolean;
  markdown?: string;
  title?: string;
  error?: string;
  durationMs: number;
  format: string;
  convertedAt: string;
}

/** 转换进度 */
export interface ConvertProgress {
  taskId: string;
  fileName: string;
  percent: number;
  stage: 'preparing' | 'converting' | 'post-processing' | 'done';
  message: string;
}

/** 文件信息 */
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  extension: string;
  mimeType?: string;
}

/** 应用配置 */
export interface AppConfig {
  output: OutputConfig;
  ai: AIConfig;
  tools: ToolsConfig;
  ui: UIConfig;
}

export interface OutputConfig {
  defaultDir?: string;
  autoOpenAfterConvert: boolean;
  keepDataUris: boolean;
}

export interface AIConfig {
  provider: 'none' | 'ollama' | 'openai' | 'custom';
  ollamaUrl?: string;
  openaiKey?: string;
  customUrl?: string;
  customKey?: string;
  visionModel: string;
  whisperModel: string;
  visionPrompt: string;
}

export interface ToolsConfig {
  exiftoolPath?: string;
  ffmpegPath?: string;
}

export interface UIConfig {
  theme: 'light' | 'dark' | 'system';
  language: 'zh-CN' | 'en-US';
  maxHistory: number;
}

/** 依赖状态 */
export interface DependencyStatus {
  name: string;
  depType: 'python' | 'system' | 'model' | 'plugin';
  required: boolean;
  installed: boolean;
  version?: string;
  path?: string;
  message: string;
}

/** 历史记录 */
export interface HistoryEntry {
  taskId: string;
  fileName: string;
  filePath: string;
  fileSize: number;
  outputPath?: string;
  success: boolean;
  error?: string;
  durationMs: number;
  convertedAt: string;
  format: string;
}

/** 历史统计 */
export interface HistoryStats {
  total: number;
  success: number;
  fail: number;
  totalSize: number;
}

/** 支持的格式 */
export interface FormatInfo {
  extension: string;
  name: string;
  category: string;
  requiresAi: boolean;
  requiresExternalTool: boolean;
}

/** 批量转换进度 */
export interface BatchProgress {
  total: number;
  completed: number;
  current: string;
}

/** 转换任务状态 */
export type TaskStatus = 'idle' | 'preparing' | 'converting' | 'done' | 'error' | 'cancelled';

/** 任务状态映射 */
export interface TaskState {
  taskId: string;
  fileName: string;
  status: TaskStatus;
  percent: number;
  message: string;
  result?: ConvertResult;
}