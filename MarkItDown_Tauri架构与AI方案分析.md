# MarkItDown 桌面化 — Tauri 2.0 架构与云端服务替代方案深度分析

> 针对性分析：Tauri 2.0 + Sidecar 可行性 & Azure 云端服务是否必须

---

## 一、Tauri 2.0 + Sidecar 方案深度分析

### 1.1 核心问题：Sidecar 能否完美适配？

**结论先行：可以完美适配，而且是 MarkItDown 桌面化的最优方案。**

理由如下：

#### 1.1.1 MarkItDown 的天然特性与 Sidecar 高度匹配

MarkItDown 本质上是一个**无状态、输入→输出**的转换工具：

```
输入: 文件路径/文件流  →  输出: Markdown 文本
```

这恰好是 Sidecar 最擅长处理的场景。Sidecar 的通信模型是：

```
Tauri Rust Backend → spawn Sidecar 进程 → stdin 传参 → stdout 读结果
```

MarkItDown 现有的 CLI 接口（`__main__.py`）已经完美支持这种模式：

```bash
# 现有 CLI 用法
markitdown input.pdf -o output.md
markitdown input.pdf > output.md
cat input.pdf | markitdown
```

#### 1.1.2 推荐的 Sidecar 架构方案

**方案 A：CLI 直调模式（推荐，最简单）**

```
┌──────────────────────────────────────────────────────┐
│  Tauri 桌面应用                                       │
│  ┌──────────────────┐    ┌─────────────────────────┐ │
│  │  Web 前端 (React) │◄──►│  Rust 后端 (Tauri Core)  │ │
│  │  - 文件拖拽/选择   │    │  - 文件系统操作          │ │
│  │  - 进度显示       │    │  - Sidecar 生命周期管理   │ │
│  │  - 结果预览       │    │  - 进程间通信            │ │
│  └──────────────────┘    └──────┬──────────────────┘ │
│                                  │ spawn sidecar      │
│                                  ▼                    │
│                          ┌──────────────┐            │
│                          │ markitdown    │            │
│                          │ (PyInstaller  │            │
│                          │  独立二进制)   │            │
│                          └──────────────┘            │
└──────────────────────────────────────────────────────┘
```

**通信协议**：
- Rust 调用 Sidecar 时传入文件路径
- Sidecar 输出 Markdown 到 stdout
- Rust 捕获 stdout 返回给前端

**方案 B：HTTP API 服务模式（更灵活，适合复杂场景）**

```
┌──────────────────────────────────────────────────────┐
│  Tauri 桌面应用                                       │
│  ┌──────────────────┐    ┌─────────────────────────┐ │
│  │  Web 前端 (React) │◄──►│  Rust 后端 (Tauri Core)  │ │
│  └────────┬─────────┘    └──────────┬──────────────┘ │
│           │ HTTP localhost:9876     │ spawn sidecar  │
│           └─────────────────────────┤                │
│                                     ▼                │
│                          ┌──────────────────┐        │
│                          │ markitdown-server│        │
│                          │ (FastAPI +       │        │
│                          │  PyInstaller)    │        │
│                          │ POST /convert    │        │
│                          │ GET  /health     │        │
│                          │ GET  /progress   │        │
│                          └──────────────────┘        │
└──────────────────────────────────────────────────────┘
```

**对比分析**：

| 维度 | 方案 A (CLI 直调) | 方案 B (HTTP 服务) |
|------|-------------------|---------------------|
| 复杂度 | 低 | 中 |
| Sidecar 二进制大小 | 300-500MB | 300-500MB + FastAPI 依赖 |
| 启动速度 | 每次转换冷启动 | 一次启动，持续运行 |
| 并发处理 | 每个文件一个进程 | 单进程处理请求队列 |
| 进度反馈 | 无（仅成功/失败） | 可实时推送进度 |
| 取消转换 | kill 进程 | HTTP 请求取消 |
| 内存占用 | 按需启动，用完即释放 | 常驻 ~200MB |
| 适合场景 | 单文件快速转换 | 批量转换、需要进度反馈 |

**建议：两阶段实施**
- **MVP 阶段**：方案 A，快速验证
- **正式版**：方案 B，更好用户体验

#### 1.1.3 Tauri 2.0 Sidecar 的具体实现细节

**Step 1: 打包 MarkItDown 为独立二进制**

```toml
# src-tauri/Cargo.toml 中配置 sidecar
[tauri]
# 声明依赖的外部二进制
[tauri.bundle]
externalBin = [
  "binaries/markitdown-sidecar",  # macOS/Linux 名称
]
```

```json
// src-tauri/tauri.conf.json
{
  "bundle": {
    "externalBin": [
      "binaries/markitdown-sidecar"
    ]
  }
}
```

**Step 2: Rust 端调用 Sidecar**

```rust
// src-tauri/src/main.rs
use tauri::api::process::Command;
use tauri::Manager;

#[tauri::command]
async fn convert_file(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    let sidecar = app.shell()
        .sidecar("markitdown-sidecar")
        .map_err(|e| e.to_string())?;
    
    let (mut rx, _child) = sidecar
        .args([&file_path, "--output-format", "markdown"])
        .spawn()
        .map_err(|e| e.to_string())?;
    
    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            tauri::api::process::CommandEvent::Stdout(line) => {
                output.push_str(&line);
            }
            tauri::api::process::CommandEvent::Stderr(line) => {
                eprintln!("Sidecar error: {}", line);
            }
            tauri::api::process::CommandEvent::Terminated(status) => {
                if status.code != Some(0) {
                    return Err(format!("转换失败，退出码: {:?}", status.code));
                }
            }
            _ => {}
        }
    }
    
    Ok(output)
}
```

**Step 3: 前端调用**

```typescript
// src/App.tsx
import { invoke } from '@tauri-apps/api/core';

async function convertFile(filePath: string) {
  try {
    const markdown = await invoke<string>('convert_file', {
      filePath: filePath
    });
    // 显示转换结果
    setMarkdownContent(markdown);
  } catch (error) {
    console.error('转换失败:', error);
  }
}
```

#### 1.1.4 Sidecar 打包的二进制体积问题

这是最需要关注的问题。MarkItDown 完整依赖的 PyInstaller 打包预估：

| 组件 | 体积 |
|------|------|
| Python 运行时 | ~30MB |
| markitdown 核心 | ~5MB |
| pdfminer + pdfplumber | ~15MB |
| mammoth + lxml | ~10MB |
| python-pptx | ~5MB |
| pandas + openpyxl | ~40MB |
| pydub + SpeechRecognition | ~10MB |
| 其他依赖 | ~20MB |
| **总计（full 模式）** | **~135MB** |

> 注意：这是 Python 侧依赖的体积，不是整个应用的体积。Tauri 应用本身只有 ~5MB。

**体积优化策略**：

1. **按需打包**：默认只打包核心格式（PDF/Office），多媒体格式可选下载
2. **UPX 压缩**：PyInstaller 支持 UPX 压缩，可减少 30-50%
3. **分拆多个 Sidecar**：
   ```
   markitdown-core    (PDF, DOCX, PPTX, HTML, 文本)    ~80MB
   markitdown-media   (图片, 音频, 视频)              ~60MB (可选)
   ```
4. **最终体积预估**：压缩后 80-120MB，在可接受范围内

#### 1.1.5 与 Electron 的对比

| 维度 | Tauri 2.0 + Sidecar | Electron + Python |
|------|---------------------|-------------------|
| 应用基础体积 | ~5MB | ~100MB |
| Python 运行时 | 嵌入 Sidecar | 嵌入主进程或子进程 |
| 总包体积 | ~130MB | ~250MB+ |
| 内存占用 | 按需启动 | 常驻 Chromium |
| 启动速度 | 快 (无浏览器引擎) | 慢 (需启动 Chromium) |
| 安全性 | Rust 内存安全 | Node.js 风险 |
| 前端技术栈 | 任意 Web 框架 | 任意 Web 框架 |
| 开发体验 | Rust 学习曲线 | JS/TS 更易上手 |

**结论：Tauri 在体积、性能、安全性上全面优于 Electron。** 对于你熟悉 Rust 的情况，Tauri 是明确的最优选择。

---

## 二、云端 AI 服务替代方案分析

### 2.1 核心结论：Azure 完全不是必须的

我来逐层拆解项目中的云端服务依赖：

#### 2.1.1 项目中的云端服务全景图

```
MarkItDown 转换能力
├── 离线内置转换器（核心，零依赖云服务）
│   ├── PDF  →  pdfminer + pdfplumber           ← 纯本地
│   ├── DOCX →  mammoth                          ← 纯本地
│   ├── PPTX →  python-pptx                      ← 纯本地
│   ├── XLSX →  pandas + openpyxl               ← 纯本地
│   ├── HTML →  BeautifulSoup + markdownify      ← 纯本地
│   ├── CSV/JSON/TXT →  charset_normalizer      ← 纯本地
│   ├── EPUB →  ebooklib                          ← 纯本地
│   ├── ZIP  →  zipfile                           ← 纯本地
│   └── 图片  →  exiftool（可选，本地工具）       ← 纯本地
│
├── 可选云端增强（非必须，可替换）
│   ├── 图片描述  →  LLM Vision API (OpenAI 兼容)  ← 可替换为任何兼容 API
│   ├── 音频转录  →  Google Speech Recognition    ← 可替换为本地 Whisper
│   ├── 图片 OCR  →  LLM Vision API (OCR 插件)     ← 可替换为任何兼容 API
│   │
│   └── Azure 专有服务（仅 Microsoft 云）
│       ├── Azure Document Intelligence  → 仅 PDF/图片 OCR 增强
│       └── Azure Content Understanding  → 多模态增强 + 视频
│
└── 外部工具（本地，非云服务）
    ├── exiftool  →  元数据提取（本地命令行工具）
    └── ffmpeg    →  音频格式转换（本地命令行工具）
```

#### 2.1.2 Azure 服务的具体作用与替代方案

**Azure Document Intelligence** (`_doc_intel_converter.py`)

| 特性 | 它做了什么 | 离线替代方案 |
|------|-----------|------------|
| PDF 文本提取 | 云端 OCR + 布局分析 | 内置 pdfminer + pdfplumber（已足够好） |
| 扫描 PDF OCR | 高精度 OCR | markitdown-ocr 插件 + 本地 LLM Vision |
| 表格识别 | 结构化表格提取 | pdfplumber 表格提取（已有） |
| 公式识别 | 数学公式提取 | 内置 docx math 模块 |

**结论**：Azure Doc Intel 是"锦上添花"而非"雪中送炭"。内置转换器已经覆盖了 90% 的场景，仅在极端复杂的扫描文档上 Azure 有明显优势。

**Azure Content Understanding** (`_cu_converter.py`)

| 特性 | 它做了什么 | 替代方案 |
|------|-----------|---------|
| 文档处理 | 云端多模态提取 | 内置转换器（已覆盖） |
| 图片处理 | 图片内容理解 | LLM Vision API（OpenAI 兼容） |
| 音频处理 | 语音转文字 | 本地 Whisper / Google Speech API |
| **视频处理** | **唯一支持视频的选项** | **ffmpeg 提取音频 + Whisper 转录** |
| 结构化字段 | YAML 字段提取 | 自定义后处理 |

**关键发现**：Azure CU 是项目中**唯一**支持视频直接转换的选项。但这不是不可替代的——视频处理完全可以拆解为：

```
视频文件 → ffmpeg 提取音频 → Whisper 转录 → 文本文档
```

#### 2.1.3 本地替代方案详细设计

**方案总览**：

```
┌─────────────────────────────────────────────────────────┐
│              MarkItDown 桌面版 AI 能力架构                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  层级 1: 纯离线（默认，零配置）                           │
│  ├── PDF/Office 文档转换  →  内置转换器                  │
│  ├── 图片 EXIF 元数据    →  exiftool (可选)             │
│  └── 音频元数据          →  exiftool (可选)             │
│                                                         │
│  层级 2: 本地 AI（推荐，完全离线）                        │
│  ├── 图片描述/OCR        →  Ollama + LLaVA/llama-vision │
│  ├── 音频转录            →  faster-whisper (本地)       │
│  └── 视频转录            →  ffmpeg + faster-whisper     │
│                                                         │
│  层级 3: 云端 API（可选，高质量）                         │
│  ├── 图片描述/OCR        →  OpenAI / Groq / 任何兼容API │
│  ├── 音频转录            →  OpenAI Whisper API          │
│  └── 视频处理            →  Google Video AI (可选)      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

#### 2.1.4 各 AI 能力的替代实现

**A. 图片描述/OCR — 替代 LLM Vision**

当前实现：通过 `llm_client` 参数注入 OpenAI 兼容客户端

```python
# 现有代码中已经是 OpenAI 兼容的（非 Azure 专有）
md = MarkItDown(
    llm_client=OpenAI(),       # 任何 OpenAI 兼容客户端
    llm_model="gpt-4o",        # 任何模型名
)
```

**替换为本地 Ollama 的示例**：

```python
from openai import OpenAI

# 指向本地 Ollama 服务
local_client = OpenAI(
    base_url="http://localhost:11434/v1",
    api_key="ollama"
)

md = MarkItDown(
    llm_client=local_client,
    llm_model="llava:13b",  # 或 minicpm-v, llama3.2-vision 等
)
```

**推荐的本地模型**：

| 模型 | 大小 | 硬件要求 | 适用场景 |
|------|------|---------|---------|
| llava:7b | ~4GB | 8GB RAM | 基本图片描述 |
| llava:13b | ~8GB | 16GB RAM | 较好的图片理解 |
| minicpm-v:8b | ~5GB | 8GB RAM | 中文 OCR 优秀 |
| llama3.2-vision:11b | ~7GB | 16GB RAM | 通用视觉理解 |

**B. 音频转录 — 替代 Google Speech Recognition**

当前实现：`_transcribe_audio.py` 使用 Google Speech Recognition API

```python
# 当前代码：需要网络
recognizer = sr.Recognizer()
transcript = recognizer.recognize_google(audio)
```

**替换为本地 Whisper 的方案**：

```python
# 新建 _transcribe_audio_local.py
import whisper
import tempfile
import os

# 全局加载模型（只加载一次）
_model = None

def get_model(model_size="base"):
    global _model
    if _model is None:
        _model = whisper.load_model(model_size)
    return _model

def transcribe_audio_local(file_stream, audio_format="wav"):
    model = get_model("base")  # tiny/base/small/medium/large
    
    # 写入临时文件
    with tempfile.NamedTemporaryFile(suffix=f".{audio_format}", delete=False) as tmp:
        tmp.write(file_stream.read())
        tmp_path = tmp.name
    
    try:
        result = model.transcribe(tmp_path, language="zh")  # 可指定语言
        return result["text"]
    finally:
        os.unlink(tmp_path)
```

**Whisper 模型体积与性能**：

| 模型 | 大小 | VRAM | 相对速度 | 中文准确率 |
|------|------|------|---------|-----------|
| tiny | ~75MB | ~1GB | 10x | 一般 |
| base | ~145MB | ~1GB | 7x | 可接受 |
| small | ~488MB | ~2GB | 4x | 较好 |
| medium | ~1.5GB | ~5GB | 2x | 好 |
| large-v3 | ~3GB | ~10GB | 1x | 优秀 |

**C. 视频处理 — 替代 Azure CU**

当前状态：项目**不支持本地视频处理**，仅 Azure CU 支持视频。

**本地实现方案**：

```python
# 新建 _video_converter.py
import subprocess
import tempfile
import os
from markitdown._base_converter import DocumentConverter, DocumentConverterResult
from markitdown._stream_info import StreamInfo

class VideoConverter(DocumentConverter):
    """本地视频转换器：提取音频 + 转录"""
    
    ACCEPTED_EXTENSIONS = [".mp4", ".mov", ".avi", ".mkv", ".webm"]
    
    def accepts(self, file_stream, stream_info, **kwargs):
        return (stream_info.extension or "").lower() in self.ACCEPTED_EXTENSIONS
    
    def convert(self, file_stream, stream_info, **kwargs):
        # 1. 保存视频到临时文件
        with tempfile.NamedTemporaryFile(suffix=".mp4", delete=False) as tmp:
            tmp.write(file_stream.read())
            video_path = tmp.name
        
        try:
            # 2. 提取音频
            audio_path = video_path + ".wav"
            subprocess.run([
                "ffmpeg", "-i", video_path,
                "-vn", "-acodec", "pcm_s16le",
                "-ar", "16000", "-ac", "1",
                "-y", audio_path
            ], check=True, capture_output=True)
            
            # 3. 转录音频
            result = whisper.load_model("base").transcribe(audio_path)
            
            return DocumentConverterResult(
                markdown=f"### Video Transcript\n\n{result['text']}"
            )
        finally:
            os.unlink(video_path)
            if os.path.exists(audio_path):
                os.unlink(audio_path)
```

#### 2.1.5 最终推荐：四层 AI 能力架构

```
用户配置界面：
┌──────────────────────────────────────────────────┐
│  AI 功能设置                                      │
│                                                  │
│  ○ 纯离线模式（仅使用内置转换器，无需任何 AI）      │
│                                                  │
│  ● 本地 AI 模式（推荐）                           │
│    ├── 视觉模型: [Ollama + llava:13b    ▼]       │
│    ├── 语音模型: [faster-whisper base  ▼]       │
│    └── 自动下载管理: ☑                           │
│                                                  │
│  ○ 云端 API 模式                                 │
│    ├── API 提供商: [OpenAI ▼]                     │
│    ├── API Key: [sk-••••••••••]                  │
│    ├── 视觉模型: [gpt-4o ▼]                      │
│    └── 语音模型: [whisper-1 ▼]                   │
│                                                  │
│  ○ 自定义模式（高级）                              │
│    └── API 地址: [http://localhost:11434/v1]      │
└──────────────────────────────────────────────────┘
```

---

## 三、综合推荐方案

### 3.1 最终推荐架构

```
┌─────────────────────────────────────────────────────────┐
│                  MarkItDown 桌面版                        │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │  Tauri 2.0 Shell (约 5MB)                         │ │
│  │                                                    │ │
│  │  ┌─────────────┐  ┌──────────────────────────────┐│ │
│  │  │ React 前端   │  │  Rust 后端 (Tauri Core)      ││ │
│  │  │ - 文件管理   │  │  - 窗口管理                  ││ │
│  │  │ - 转换界面   │  │  - 文件系统                  ││ │
│  │  │ - 结果预览   │  │  - Sidecar 管理              ││ │
│  │  │ - AI 设置    │  │  - 本地 AI 启动/停止         ││ │
│  │  └─────────────┘  └──────┬───────────┬───────────┘│ │
│  │                           │ spawn     │ spawn      │ │
│  └───────────────────────────┼───────────┼────────────┘ │
│                               ▼           ▼              │
│  ┌──────────────────┐  ┌─────────────────────────────┐  │
│  │ markitdown       │  │ AI 本地服务 (可选)           │  │
│  │ (PyInstaller)    │  │ - Ollama (视觉)              │  │
│  │ Sidecar #1       │  │ - faster-whisper (语音)      │  │
│  │ - PDF/Office     │  │                              │  │
│  │ - HTML/文本      │  │ 或云端 API:                  │  │
│  │ - 图片/音频元数据│  │ - OpenAI / Groq / 自定义     │  │
│  └──────────────────┘  └─────────────────────────────┘  │
│                                                         │
│  ┌──────────────────┐                                   │
│  │ 外部工具 (可选)   │                                   │
│  │ - ffmpeg          │                                   │
│  │ - exiftool        │                                   │
│  └──────────────────┘                                   │
└─────────────────────────────────────────────────────────┘
```

### 3.2 为什么 Tauri 是最佳选择

1. **体积极小**：Tauri 基础应用 ~5MB，对比 Electron ~100MB+
2. **性能出色**：Rust 后端，无 Chromium 开销
3. **Sidecar 机制**：天然支持外部进程管理，不需要 hack
4. **安全性**：Rust 内存安全，进程隔离
5. **你懂 Rust**：学习成本低，开发效率高
6. **跨平台一致**：Windows/macOS/Linux 一套代码

### 3.3 Sidecar 的完美适配性

MarkItDown 的 Sidecar 适配不存在任何技术障碍：

- MarkItDown 已经是无状态的 CLI 工具，天然适合作为 Sidecar
- 通信模型简单：文件路径进，Markdown 出
- PyInstaller 打包成熟，可生成独立二进制
- Sidecar 二进制可独立于 Tauri 应用更新

### 3.4 关于 Azure 的最终结论

**Azure 完全不是必须的。** 理由：

1. 内置转换器覆盖了 90% 的文档转换需求（完全离线）
2. 图片描述/OCR 使用 OpenAI 兼容 API，支持任何本地或云端模型
3. 音频转录可替换为本地 Whisper
4. 视频处理可通过 ffmpeg + Whisper 实现
5. Azure 服务仅作为"额外的高质量选项"提供给有 Azure 订阅的用户

**建议的默认配置**：完全离线 + 本地 AI（Ollama + Whisper），云端 API 作为可选配置。

---

## 四、实施路线图建议

### 第一阶段：MVP（1-2 周）
- Tauri 2.0 项目搭建
- MarkItDown PyInstaller Sidecar 打包
- 基础 UI：文件选择 + 转换 + 结果预览
- 支持格式：PDF、DOCX、PPTX、XLSX、HTML、纯文本

### 第二阶段：AI 集成（1 周）
- 本地 AI 服务管理（Ollama 检测/启动）
- 图片描述/OCR 功能
- 音频转录（Whisper 集成）
- AI 设置界面

### 第三阶段：完善（1 周）
- 视频处理（ffmpeg + Whisper）
- 批量转换
- 拖拽支持
- 转换历史
- 安装包制作（.msi / .dmg / .AppImage）

---

**总结**：Tauri 2.0 + Sidecar 是最佳方案，Azure 完全可选。建议立即开始 MVP 开发。