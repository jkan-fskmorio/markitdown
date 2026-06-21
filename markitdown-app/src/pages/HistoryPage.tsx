// 历史记录页面
import { useEffect } from 'react';
import { useHistory } from '../hooks/useHistory';

function HistoryPage() {
  const { history, loadHistory, clearHistory, deleteEntry } = useHistory();

  useEffect(() => {
    loadHistory();
  }, []);

  if (history.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">📋</div>
        <div className="empty-state-text">暂无转换记录</div>
      </div>
    );
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
        <h3 style={{ fontSize: 16 }}>转换历史 ({history.length})</h3>
        <button className="btn btn-secondary" onClick={clearHistory}>
          清空历史
        </button>
      </div>

      <div className="file-list">
        {history.map((entry) => (
          <div key={entry.taskId} className="file-item">
            <span className="file-item-name">{entry.fileName}</span>
            <span className="badge badge-success">
              {entry.format.toUpperCase()}
            </span>
            <span className={`badge ${entry.success ? 'badge-success' : 'badge-error'}`}>
              {entry.success ? '成功' : '失败'}
            </span>
            <span style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>
              {formatSize(entry.fileSize)}
            </span>
            <span style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>
              {entry.durationMs}ms
            </span>
            <span style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>
              {formatDate(entry.convertedAt)}
            </span>
            {entry.error && (
              <span className="badge badge-error" title={entry.error}>
                错误
              </span>
            )}
            <button
              className="file-item-remove"
              onClick={() => deleteEntry(entry.taskId)}
              title="删除"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return `${d.getMonth() + 1}/${d.getDate()} ${d.getHours()}:${String(d.getMinutes()).padStart(2, '0')}`;
}

export default HistoryPage;