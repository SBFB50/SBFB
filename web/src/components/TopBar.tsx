import { useLocation } from 'react-router-dom';
import { useCaseStore } from '../stores/caseStore';
import { Activity, Zap, Cpu, HardDrive, Monitor } from 'lucide-react';
import { useTriggerAnalysis } from '../hooks/useApi';
import { useEffect, useState } from 'react';
import { api } from '../api/client';

const routeNames: Record<string, string> = {
  '/': 'Dashboard',
  '/evidence': 'Preuves',
  '/entities': 'Entites',
  '/hypotheses': 'Hypotheses',
  '/graph': 'Graphe',
  '/timeline': 'Chronologie',
  '/investigation': 'Investigation',
  '/benchmark': 'Benchmark',
  '/monitoring': 'Surveillance',
  '/forensics': 'Forensique',
  '/suspects': 'Suspects',
};

interface SystemStats {
  cpu: number;
  ram_used: number;
  ram_total: number;
  ram_pct: number;
  gpu_name: string;
  gpu_used: number;
  gpu_total: number;
  gpu_pct: number;
  gpu_temp: number;
}

const OLLAMA_URL = import.meta.env.VITE_OLLAMA_URL || '/ollama';

function useSystemStats() {
  const [stats, setStats] = useState<SystemStats | null>(null);

  useEffect(() => {
    let active = true;
    const poll = async () => {
      try {
        // Use Ollama ps as a proxy for GPU usage
        const [health, ollama] = await Promise.all([
          api.get('/health').then(r => r.data).catch(() => null),
          fetch(`${OLLAMA_URL}/api/ps`).then(r => r.json()).catch(() => null),
        ]);

        if (!active) return;

        const models = ollama?.models || [];
        const totalVram = models.reduce((a: number, m: any) => a + (m.size_vram || 0), 0);
        const totalSize = models.reduce((a: number, m: any) => a + (m.size || 0), 0);
        const modelName = models[0]?.name || 'idle';

        setStats({
          cpu: 0, // Can't get from browser
          ram_used: 0,
          ram_total: 0,
          ram_pct: 0,
          gpu_name: modelName,
          gpu_used: Math.round(totalVram / (1024 * 1024)),
          gpu_total: 16384, // RTX 5080
          gpu_pct: Math.round((totalVram / (16384 * 1024 * 1024)) * 100),
          gpu_temp: 0,
        });
      } catch {}
    };

    poll();
    const interval = setInterval(poll, 5000);
    return () => { active = false; clearInterval(interval); };
  }, []);

  return stats;
}

function MiniBar({ value, max, color }: { value: number; max: number; color: string }) {
  const pct = Math.min(100, (value / max) * 100);
  return (
    <div className="w-16 h-1.5 bg-[var(--bg-primary)] rounded-full overflow-hidden">
      <div className="h-full rounded-full transition-all" style={{ width: `${pct}%`, backgroundColor: color }} />
    </div>
  );
}

export default function TopBar() {
  const location = useLocation();
  const { caseName } = useCaseStore();
  const triggerAnalysis = useTriggerAnalysis();
  const pageName = routeNames[location.pathname] || 'NEXUS';
  const stats = useSystemStats();

  return (
    <header className="flex items-center justify-between px-4 py-2 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
      <div className="flex items-center gap-3">
        <Activity size={16} className="text-[var(--accent)]" />
        <div>
          <h1 className="text-sm font-semibold text-[var(--text-primary)]">{pageName}</h1>
          {caseName && <p className="text-[10px] text-[var(--text-muted)]">Case: {caseName}</p>}
        </div>
      </div>

      {/* System stats bar */}
      <div className="flex items-center gap-4">
        {stats && (
          <div className="flex items-center gap-4 text-[10px] text-[var(--text-muted)]">
            {stats.gpu_used > 0 && (
              <div className="flex items-center gap-1.5">
                <Monitor size={11} />
                <span className="font-mono">{stats.gpu_name.split(':')[0]}</span>
                <MiniBar value={stats.gpu_used} max={stats.gpu_total} color={stats.gpu_pct > 80 ? '#ef4444' : stats.gpu_pct > 50 ? '#eab308' : '#22c55e'} />
                <span className="font-mono">{(stats.gpu_used / 1024).toFixed(1)}/{(stats.gpu_total / 1024).toFixed(0)}G</span>
              </div>
            )}
            {stats.gpu_used === 0 && (
              <div className="flex items-center gap-1.5">
                <Monitor size={11} />
                <span className="text-green-400">GPU idle</span>
              </div>
            )}
          </div>
        )}

        <button
          onClick={async () => {
            try {
              // Stop all investigations
              const cases = await api.get('/cases').then(r => r.data);
              for (const c of cases) {
                await api.post(`/cases/${c.id}/investigation/stop`).catch(() => {});
              }
              // Unload all LLMs
              const ollamaGen = `${OLLAMA_URL}/api/generate`;
              await fetch(ollamaGen, {
                method: 'POST', body: JSON.stringify({ model: 'nexus', prompt: '', keep_alive: 0 })
              }).catch(() => {});
              await fetch(ollamaGen, {
                method: 'POST', body: JSON.stringify({ model: 'gemma4:e4b', prompt: '', keep_alive: 0 })
              }).catch(() => {});
              await fetch(ollamaGen, {
                method: 'POST', body: JSON.stringify({ model: 'huihui_ai/deepseek-r1-abliterated:14b', prompt: '', keep_alive: 0 })
              }).catch(() => {});
            } catch {}
          }}
          className="flex items-center gap-1.5 px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded-lg text-xs font-medium transition-colors"
        >
          <span className="w-2 h-2 bg-white rounded-sm" />
          Stop All
        </button>
        <button
          onClick={() => triggerAnalysis.mutate()}
          disabled={triggerAnalysis.isPending}
          className="flex items-center gap-1.5 px-3 py-1 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-xs font-medium transition-colors disabled:opacity-50"
        >
          <Zap size={12} />
          {triggerAnalysis.isPending ? 'Analyse...' : 'Analyser'}
        </button>
      </div>
    </header>
  );
}
