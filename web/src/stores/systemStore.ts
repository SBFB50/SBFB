import { create } from 'zustand';

interface SystemStats {
  cpu_percent: number;
  ram_percent: number;
  ram_used_gb: number;
  ram_total_gb: number;
  gpu_percent: number;
  gpu_memory_used_gb: number;
  gpu_memory_total_gb: number;
  gpu_temp: number;
}

interface SystemStore {
  stats: SystemStats | null;
  healthy: boolean;
  setStats: (stats: SystemStats) => void;
  setHealthy: (healthy: boolean) => void;
}

export const useSystemStore = create<SystemStore>((set) => ({
  stats: null,
  healthy: false,
  setStats: (stats) => set({ stats }),
  setHealthy: (healthy) => set({ healthy }),
}));
