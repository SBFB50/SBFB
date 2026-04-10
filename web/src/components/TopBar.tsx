import { useLocation } from 'react-router-dom';
import { useCaseStore } from '../stores/caseStore';
import { Activity, Zap, Monitor } from 'lucide-react';
import { useTriggerAnalysis } from '../hooks/useApi';
import { useEffect, useState } from 'react';
import { api } from '../api/client';
import { SidebarTrigger } from '@/components/ui/sidebar';

const routeNames: Record<string, string> = {
  '/': 'Dashboard',
  '/evidence': 'Preuves',
  '/entities': 'Entites',
  '/hypotheses': 'Hypotheses',
  '/graph': 'Graphe',
  '/timeline': 'Chronologie',
  '/investigation': 'Investigation',
  '/benchmark': 'Benchmark',
  '/suspects': 'Suspects',
  '/wiki': 'Wiki',
  '/reports': 'Reports',
  '/images': 'Image Search',
  '/government': 'Gouvernement',
};

interface OllamaStats {
  gpu_name: string;
  gpu_used: number;
  gpu_total: number;
  gpu_pct: number;
}

const OLLAMA_URL = import.meta.env.VITE_OLLAMA_URL || '/ollama';

function useOllamaStats() {
  const [stats, setStats] = useState<OllamaStats | null>(null);

  useEffect(() => {
    let active = true;
    const poll = async () => {
      try {
        const ollama = await fetch(`${OLLAMA_URL}/api/ps`).then(r => r.json()).catch(() => null);
        if (!active) return;
        const models = ollama?.models || [];
        const totalVram = models.reduce((a: number, m: any) => a + (m.size_vram || 0), 0);
        const modelName = models[0]?.name || 'idle';
        setStats({
          gpu_name: modelName,
          gpu_used: Math.round(totalVram / (1024 * 1024)),
          gpu_total: 16384,
          gpu_pct: Math.round((totalVram / (16384 * 1024 * 1024)) * 100),
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
  const stats = useOllamaStats();

  return (
    <header className="flex items-center justify-between px-3 py-2 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
      <div className="flex items-center gap-2">
        <SidebarTrigger className="text-[var(--text-muted)] hover:text-[var(--text-primary)]" />
        <div className="w-px h-4 bg-[var(--border)]" />
        <Activity size={14} className="text-blue-500" />
        <div>
          <h1 className="text-sm font-semibold text-[var(--text-primary)]">{pageName}</h1>
          {caseName && <p className="text-[10px] text-[var(--text-muted)]">Case: {caseName}</p>}
        </div>
      </div>

      <div className="flex items-center gap-3">
        {/* Ollama/GPU stats */}
        {stats && (
          <div className="flex items-center gap-3 text-[10px] text-[var(--text-muted)]">
            {stats.gpu_used > 0 ? (
              <div className="flex items-center gap-1.5">
                <Monitor size={11} />
                <span className="font-mono">{stats.gpu_name.split(':')[0]}</span>
                <MiniBar value={stats.gpu_used} max={stats.gpu_total} color={stats.gpu_pct > 80 ? '#ef4444' : stats.gpu_pct > 50 ? '#eab308' : '#22c55e'} />
                <span className="font-mono">{(stats.gpu_used / 1024).toFixed(1)}/{(stats.gpu_total / 1024).toFixed(0)}G</span>
              </div>
            ) : (
              <div className="flex items-center gap-1.5">
                <Monitor size={11} />
                <span className="text-emerald-400">GPU idle</span>
              </div>
            )}
          </div>
        )}

        {/* Stop All */}
        <button
          onClick={async () => {
            try {
              const cases = await api.get('/cases').then(r => r.data);
              for (const c of cases) {
                await api.post(`/cases/${c.id}/investigation/stop`).catch(() => {});
              }
              const ollamaGen = `${OLLAMA_URL}/api/generate`;
              await fetch(ollamaGen, {
                method: 'POST', body: JSON.stringify({ model: 'nexus', prompt: '', keep_alive: 0 })
              }).catch(() => {});
            } catch {}
          }}
          className="flex items-center gap-1.5 px-2.5 py-1 bg-red-600 hover:bg-red-700 text-white rounded-md text-xs font-medium transition-colors"
        >
          <span className="w-2 h-2 bg-white rounded-sm" />
          Stop
        </button>

        {/* Analyze */}
        <button
          onClick={() => triggerAnalysis.mutate()}
          disabled={triggerAnalysis.isPending}
          className="flex items-center gap-1.5 px-2.5 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-xs font-medium transition-colors disabled:opacity-50"
        >
          <Zap size={12} />
          {triggerAnalysis.isPending ? 'Analyse...' : 'Analyser'}
        </button>
      </div>
    </header>
  );
}
