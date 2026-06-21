// 配置管理 Hook
import { useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store/appStore';
import type { AppConfig } from '../types';

export function useConfig() {
  const { config, setConfig } = useAppStore();

  // 初始化时加载配置
  useEffect(() => {
    invoke<AppConfig>('get_config')
      .then(setConfig)
      .catch(console.error);
  }, []);

  // 保存配置
  const saveConfig = useCallback(
    async (newConfig: AppConfig) => {
      await invoke('set_config', { config: newConfig });
      setConfig(newConfig);
    },
    [setConfig]
  );

  // 重置配置
  const resetConfig = useCallback(async () => {
    const defaultConfig = await invoke<AppConfig>('reset_config');
    setConfig(defaultConfig);
  }, [setConfig]);

  return {
    config,
    saveConfig,
    resetConfig,
  };
}