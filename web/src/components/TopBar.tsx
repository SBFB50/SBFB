import { useLocation } from 'react-router-dom';
import { useCaseStore } from '../stores/caseStore';
import { Activity, Zap } from 'lucide-react';
import { useTriggerAnalysis } from '../hooks/useApi';

const routeNames: Record<string, string> = {
  '/': 'Dashboard',
  '/evidence': 'Evidence',
  '/entities': 'Entities',
  '/hypotheses': 'Hypotheses',
  '/graph': 'Knowledge Graph',
  '/timeline': 'Timeline',
  '/investigation': 'Investigation',
  '/benchmark': 'Benchmark',
  '/monitoring': 'Monitoring',
  '/forensics': 'Forensics',
};

export default function TopBar() {
  const location = useLocation();
  const { caseName } = useCaseStore();
  const triggerAnalysis = useTriggerAnalysis();
  const pageName = routeNames[location.pathname] || 'NEXUS';

  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
      <div className="flex items-center gap-3">
        <Activity size={18} className="text-[var(--accent)]" />
        <div>
          <h1 className="text-base font-semibold text-[var(--text-primary)]">{pageName}</h1>
          {caseName && (
            <p className="text-xs text-[var(--text-muted)]">
              Case: {caseName}
            </p>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={() => triggerAnalysis.mutate()}
          disabled={triggerAnalysis.isPending}
          className="flex items-center gap-2 px-3 py-1.5 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
        >
          <Zap size={14} />
          {triggerAnalysis.isPending ? 'Analyzing...' : 'Analyze'}
        </button>
      </div>
    </header>
  );
}
