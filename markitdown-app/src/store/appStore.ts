// 全局状态管理 - 使用 Zustand
import { create } from 'zustand';
import type { AppConfig, TaskState, HistoryEntry, DependencyStatus } from '../types';

interface AppState {
  // 配置
  config: AppConfig | null;
  setConfig: (config: AppConfig) => void;

  // 当前任务
  activeTask: TaskState | null;
  setActiveTask: (task: TaskState | null) => void;
  updateTaskProgress: (percent: number, message: string) => void;

  // 任务队列
  taskQueue: TaskState[];
  addToQueue: (task: TaskState) => void;
  removeFromQueue: (taskId: string) => void;
  clearQueue: () => void;

  // 历史记录
  history: HistoryEntry[];
  setHistory: (history: HistoryEntry[]) => void;

  // 依赖状态
  dependencies: DependencyStatus[];
  setDependencies: (deps: DependencyStatus[]) => void;

  // UI 状态
  selectedFiles: string[];
  setSelectedFiles: (files: string[]) => void;
  addSelectedFile: (file: string) => void;
  removeSelectedFile: (file: string) => void;
  clearSelectedFiles: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  config: null,
  setConfig: (config) => set({ config }),

  activeTask: null,
  setActiveTask: (task) => set({ activeTask: task }),
  updateTaskProgress: (percent, message) =>
    set((state) => ({
      activeTask: state.activeTask
        ? { ...state.activeTask, percent, message }
        : null,
    })),

  taskQueue: [],
  addToQueue: (task) =>
    set((state) => ({ taskQueue: [...state.taskQueue, task] })),
  removeFromQueue: (taskId) =>
    set((state) => ({
      taskQueue: state.taskQueue.filter((t) => t.taskId !== taskId),
    })),
  clearQueue: () => set({ taskQueue: [] }),

  history: [],
  setHistory: (history) => set({ history }),

  dependencies: [],
  setDependencies: (deps) => set({ dependencies: deps }),

  selectedFiles: [],
  setSelectedFiles: (files) => set({ selectedFiles: files }),
  addSelectedFile: (file) =>
    set((state) => ({
      selectedFiles: [...state.selectedFiles, file],
    })),
  removeSelectedFile: (file) =>
    set((state) => ({
      selectedFiles: state.selectedFiles.filter((f) => f !== file),
    })),
  clearSelectedFiles: () => set({ selectedFiles: [] }),
}));