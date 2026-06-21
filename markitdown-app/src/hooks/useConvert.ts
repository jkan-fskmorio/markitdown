// 转换操作 Hook - 封装 Tauri invoke 调用和事件监听
import { useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../store/appStore';
import type { ConvertOptions, ConvertResult, ConvertProgress } from '../types';

export function useConvert() {
  const { activeTask, setActiveTask, updateTaskProgress } =
    useAppStore();

  // 监听转换进度事件
  useEffect(() => {
    const unlistenProgress = listen<ConvertProgress>('convert:progress', (e) => {
      updateTaskProgress(e.payload.percent, e.payload.message);
    });

    const unlistenComplete = listen<{ taskId: string; fileName: string; success: boolean }>(
      'convert:complete',
      () => {
        setActiveTask(null);
      }
    );

    const unlistenError = listen<{ taskId: string; fileName: string; error: string }>(
      'convert:error',
      () => {
        setActiveTask(null);
      }
    );

    const unlistenCancelled = listen<{ taskId: string }>('convert:cancelled', () => {
      setActiveTask(null);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenError.then((fn) => fn());
      unlistenCancelled.then((fn) => fn());
    };
  }, []);

  // 单文件转换
  const convertFile = useCallback(
    async (path: string, options?: ConvertOptions): Promise<ConvertResult> => {
      return await invoke<ConvertResult>('convert', { path, options });
    },
    []
  );

  // 批量转换
  const convertBatch = useCallback(
    async (paths: string[], options?: ConvertOptions): Promise<ConvertResult[]> => {
      return await invoke<ConvertResult[]>('convert_batch', { paths, options });
    },
    []
  );

  // 取消转换
  const cancelConversion = useCallback(async (taskId: string) => {
    await invoke('cancel', { taskId });
  }, []);

  return {
    activeTask,
    convertFile,
    convertBatch,
    cancelConversion,
  };
}