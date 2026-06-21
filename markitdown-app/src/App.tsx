// MARKITDOWN 桌面应用 - 主入口
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { useEffect } from 'react';
import { useAppStore } from './store/appStore';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from './types';
import HomePage from './pages/HomePage';
import SettingsPage from './pages/SettingsPage';
import HistoryPage from './pages/HistoryPage';
import './App.css';

function App() {
  const setConfig = useAppStore((s) => s.setConfig);

  useEffect(() => {
    // 加载配置
    invoke<AppConfig>('get_config')
      .then(setConfig)
      .catch((err) => console.error('加载配置失败:', err));
  }, []);

  return (
    <BrowserRouter>
      <div className="app-container">
        <header className="app-header">
          <h1 className="app-title">MarkItDown</h1>
          <span className="app-subtitle">多格式文件转换器</span>
          <nav className="app-nav">
            <a href="/">转换</a>
            <a href="/history">历史</a>
            <a href="/settings">设置</a>
          </nav>
        </header>
        <main className="app-main">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/history" element={<HistoryPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}

export default App;