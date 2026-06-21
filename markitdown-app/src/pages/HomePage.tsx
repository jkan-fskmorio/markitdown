// 主页 - 文件转换核心界面
import { useState, useCallback, useRef } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useConvert } from '../hooks/useConvert';
import { useAppStore } from '../store/appStore';
import ReactMarkdown from 'react-markdown';
import type { ConvertResult } from '../types';

function HomePage() {
  const { selectedFiles, setSelectedFiles, removeSelectedFile, clearSelectedFiles } =
    useAppStore();
  const { convertFile, cancelConversion, activeTask } = useConvert();
  const [result, setResult] = useState<ConvertResult | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const dropRef = useRef<HTMLDivElement>(null);

  // 选择文件
  const handleSelectFiles = useCallback(async () => {
    const files = await open({
      multiple: true,
      filters: [
        {
          name: '所有支持的文件',
          extensions: [
            'pdf', 'docx', 'pptx', 'xlsx', 'xls', 'html', 'htm',
            'csv', 'json', 'xml', 'txt', 'md', 'epub',
            'jpg', 'jpeg', 'png', 'mp3', 'wav', 'm4a', 'mp4',
            'zip', 'msg', 'ipynb', 'rss',
          ],
        },
      ],
    });
    if (files) {
      const fileList = Array.isArray(files) ? files : [files];
      setSelectedFiles(fileList);
    }
  }, [setSelectedFiles]);

  // 拖拽处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    // Tauri 中用文件路径拖拽
    const files: string[] = [];
    for (let i = 0; i < e.dataTransfer.files.length; i++) {
      const file = e.dataTransfer.files[i];
      // 在 Tauri 中，可以通过 file.path 获取实际路径
      if ('path' in file) {
        files.push((file as any).path);
      }
    }
    if (files.length > 0) {
      setSelectedFiles(files);
    }
  }, [setSelectedFiles]);

  // 开始转换
  const handleConvert = useCallback(async () => {
    if (selectedFiles.length === 0) return;

    for (const filePath of selectedFiles) {
      try {
        const res = await convertFile(filePath);
        if (res.success) {
          setResult(res);
        }
      } catch (err) {
        console.error('转换失败:', err);
      }
    }
  }, [selectedFiles, convertFile]);

  // 取消转换
  const handleCancel = useCallback(() => {
    if (activeTask) {
      cancelConversion(activeTask.taskId);
    }
  }, [activeTask, cancelConversion]);

  // 复制结果
  const handleCopy = useCallback(async () => {
    if (result?.markdown) {
      await navigator.clipboard.writeText(result.markdown);
    }
  }, [result]);

  // 导出文件
  const handleExport = useCallback(async () => {
    if (!result?.markdown) return;
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({
      defaultPath: result.fileName.replace(/\.[^.]+$/, '.md'),
      filters: [{ name: 'Markdown', extensions: ['md'] }],
    });
    if (path) {
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      await writeTextFile(path, result.markdown);
    }
  }, [result]);

  const isConverting = activeTask !== null;

  return (
    <div>
      {/* 拖拽区域 */}
      <div
        ref={dropRef}
        className={`drop-zone ${isDragging ? 'drag-over' : ''}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={handleSelectFiles}
      >
        <div className="drop-zone-icon">📄</div>
        <div className="drop-zone-text">点击选择文件，或将文件拖拽到此处</div>
        <div className="drop-zone-hint">
          支持 PDF, Word, Excel, PowerPoint, HTML, 图片, 音频等 20+ 种格式
        </div>
      </div>

      {/* 文件列表 */}
      {selectedFiles.length > 0 && (
        <div className="card" style={{ marginTop: 16 }}>
          <div className="file-list">
            {selectedFiles.map((file) => (
              <div key={file} className="file-item">
                <span className="file-item-name">
                  {file.split('/').pop() || file.split('\\').pop()}
                </span>
                <button
                  className="file-item-remove"
                  onClick={() => removeSelectedFile(file)}
                  disabled={isConverting}
                >
                  ✕
                </button>
              </div>
            ))}
          </div>
          <div style={{ marginTop: 12, display: 'flex', gap: 8 }}>
            <button
              className="btn btn-primary"
              onClick={handleConvert}
              disabled={isConverting || selectedFiles.length === 0}
            >
              {isConverting ? '转换中...' : `开始转换 (${selectedFiles.length})`}
            </button>
            {isConverting && (
              <button className="btn btn-danger" onClick={handleCancel}>
                取消
              </button>
            )}
            <button
              className="btn btn-secondary"
              onClick={clearSelectedFiles}
              disabled={isConverting}
            >
              清空列表
            </button>
          </div>
        </div>
      )}

      {/* 进度条 */}
      {isConverting && activeTask && (
        <div className="card" style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 8, fontSize: 14 }}>
            {activeTask.message} ({activeTask.percent}%)
          </div>
          <div className="progress-bar">
            <div
              className="progress-bar-fill"
              style={{ width: `${activeTask.percent}%` }}
            />
          </div>
        </div>
      )}

      {/* 转换结果 */}
      {result && result.success && result.markdown && (
        <div className="card" style={{ marginTop: 16 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12 }}>
            <h3 style={{ fontSize: 15 }}>
              转换结果 - {result.fileName}
              <span className="badge badge-success" style={{ marginLeft: 8 }}>
                成功
              </span>
              <span style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginLeft: 8 }}>
                {result.durationMs}ms
              </span>
            </h3>
            <div style={{ display: 'flex', gap: 8 }}>
              <button className="btn btn-secondary" onClick={handleCopy}>
                复制
              </button>
              <button className="btn btn-primary" onClick={handleExport}>
                导出
              </button>
            </div>
          </div>
          <div className="markdown-preview">
            <ReactMarkdown>{result.markdown}</ReactMarkdown>
          </div>
        </div>
      )}

      {/* 错误信息 */}
      {result && !result.success && result.error && (
        <div className="card" style={{ marginTop: 16, borderColor: 'var(--color-error)' }}>
          <h3 style={{ fontSize: 15, color: 'var(--color-error)' }}>
            转换失败 - {result.fileName}
          </h3>
          <p style={{ fontSize: 14, marginTop: 8 }}>{result.error}</p>
        </div>
      )}
    </div>
  );
}

export default HomePage;