// 设置页面 - AI 模型、外部工具、界面配置
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfig } from '../hooks/useConfig';
import type { AppConfig, DependencyStatus } from '../types';

function SettingsPage() {
  const { config, saveConfig, resetConfig } = useConfig();
  const [deps, setDeps] = useState<DependencyStatus[]>([]);
  const [checking, setChecking] = useState(false);

  // 检查依赖
  const checkDeps = async () => {
    setChecking(true);
    try {
      const result = await invoke<DependencyStatus[]>('check_all_dependencies');
      setDeps(result);
    } catch (err) {
      console.error('检查依赖失败:', err);
    }
    setChecking(false);
  };

  useEffect(() => {
    checkDeps();
  }, []);

  if (!config) return <div className="empty-state"><div className="empty-state-text">加载中...</div></div>;

  const updateConfig = (partial: Partial<AppConfig>) => {
    saveConfig({ ...config, ...partial });
  };

  return (
    <div style={{ maxWidth: 700 }}>
      {/* AI 设置 */}
      <div className="settings-section">
        <h3>AI 模型设置</h3>

        <div className="settings-row">
          <label>AI 提供商</label>
          <select
            value={config.ai.provider}
            onChange={(e) =>
              updateConfig({
                ai: { ...config.ai, provider: e.target.value as any },
              })
            }
          >
            <option value="none">不使用 AI</option>
            <option value="ollama">Ollama (本地)</option>
            <option value="openai">OpenAI</option>
            <option value="custom">自定义 API</option>
          </select>
        </div>

        {config.ai.provider === 'ollama' && (
          <div className="settings-row">
            <label>Ollama 地址</label>
            <input
              type="text"
              value={config.ai.ollamaUrl || 'http://localhost:11434'}
              onChange={(e) =>
                updateConfig({ ai: { ...config.ai, ollamaUrl: e.target.value } })
              }
            />
          </div>
        )}

        {config.ai.provider === 'openai' && (
          <div className="settings-row">
            <label>API Key</label>
            <input
              type="password"
              value={config.ai.openaiKey || ''}
              placeholder="sk-..."
              onChange={(e) =>
                updateConfig({ ai: { ...config.ai, openaiKey: e.target.value } })
              }
            />
          </div>
        )}

        {config.ai.provider === 'custom' && (
          <>
            <div className="settings-row">
              <label>API 地址</label>
              <input
                type="text"
                value={config.ai.customUrl || ''}
                placeholder="http://localhost:8080/v1"
                onChange={(e) =>
                  updateConfig({ ai: { ...config.ai, customUrl: e.target.value } })
                }
              />
            </div>
            <div className="settings-row">
              <label>API Key</label>
              <input
                type="password"
                value={config.ai.customKey || ''}
                onChange={(e) =>
                  updateConfig({ ai: { ...config.ai, customKey: e.target.value } })
                }
              />
            </div>
          </>
        )}

        {config.ai.provider !== 'none' && (
          <>
            <div className="settings-row">
              <label>视觉模型</label>
              <input
                type="text"
                value={config.ai.visionModel}
                onChange={(e) =>
                  updateConfig({ ai: { ...config.ai, visionModel: e.target.value } })
                }
              />
            </div>
            <div className="settings-row">
              <label>语音模型</label>
              <select
                value={config.ai.whisperModel}
                onChange={(e) =>
                  updateConfig({ ai: { ...config.ai, whisperModel: e.target.value } })
                }
              >
                <option value="tiny">tiny (75MB)</option>
                <option value="base">base (145MB)</option>
                <option value="small">small (488MB)</option>
                <option value="medium">medium (1.5GB)</option>
                <option value="large">large (3GB)</option>
              </select>
            </div>
          </>
        )}
      </div>

      {/* 输出设置 */}
      <div className="settings-section">
        <h3>输出设置</h3>
        <div className="settings-row">
          <label>转换后自动打开</label>
          <input
            type="checkbox"
            checked={config.output.autoOpenAfterConvert}
            onChange={(e) =>
              updateConfig({
                output: { ...config.output, autoOpenAfterConvert: e.target.checked },
              })
            }
          />
        </div>
        <div className="settings-row">
          <label>保留 base64 图片</label>
          <input
            type="checkbox"
            checked={config.output.keepDataUris}
            onChange={(e) =>
              updateConfig({
                output: { ...config.output, keepDataUris: e.target.checked },
              })
            }
          />
        </div>
      </div>

      {/* 界面设置 */}
      <div className="settings-section">
        <h3>界面设置</h3>
        <div className="settings-row">
          <label>主题</label>
          <select
            value={config.ui.theme}
            onChange={(e) =>
              updateConfig({ ui: { ...config.ui, theme: e.target.value as any } })
            }
          >
            <option value="system">跟随系统</option>
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </div>
        <div className="settings-row">
          <label>语言</label>
          <select
            value={config.ui.language}
            onChange={(e) =>
              updateConfig({ ui: { ...config.ui, language: e.target.value as any } })
            }
          >
            <option value="zh-CN">中文</option>
            <option value="en-US">English</option>
          </select>
        </div>
      </div>

      {/* 依赖状态 */}
      <div className="settings-section">
        <h3>运行环境检测</h3>
        <button
          className="btn btn-secondary"
          onClick={checkDeps}
          disabled={checking}
          style={{ marginBottom: 12 }}
        >
          {checking ? '检测中...' : '重新检测'}
        </button>

        <div className="file-list">
          {deps.map((dep) => (
            <div key={dep.name} className="file-item">
              <span className="file-item-name">{dep.name}</span>
              <span className={`badge ${dep.installed ? 'badge-success' : 'badge-warning'}`}>
                {dep.installed ? '已安装' : '未安装'}
              </span>
              {dep.version && (
                <span style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>
                  {dep.version}
                </span>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* 重置 */}
      <div style={{ marginTop: 24 }}>
        <button className="btn btn-secondary" onClick={resetConfig}>
          恢复默认设置
        </button>
      </div>
    </div>
  );
}

export default SettingsPage;