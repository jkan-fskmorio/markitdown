// 历史记录 Hook
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store/appStore';
import type { HistoryEntry, HistoryStats } from '../types';

export function useHistory() {
  const { history, setHistory } = useAppStore();

  // 加载历史记录
  const loadHistory = useCallback(async (limit?: number, offset?: number) => {
    const entries = await invoke<HistoryEntry[]>('get_history', { limit, offset });
    setHistory(entries);
    return entries;
  }, [setHistory]);

  // 清空历史
  const clearHistory = useCallback(async () => {
    await invoke('clear_history');
    setHistory([]);
  }, [setHistory]);

  // 删除单条记录
  const deleteEntry = useCallback(async (taskId: string) => {
    await invoke('delete_history_entry', { taskId });
    setHistory(history.filter((e) => e.taskId !== taskId));
  }, [history, setHistory]);

  // 获取统计
  const getStats = useCallback(async () => {
    return await invoke<HistoryStats>('get_history_stats');
  }, []);

  return {
    history,
    loadHistory,
    clearHistory,
    deleteEntry,
    getStats,
  };
}