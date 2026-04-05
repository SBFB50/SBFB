import { useState, useEffect, useCallback } from 'react';
import { Play, RefreshCw, Trash2, ChevronDown, CheckCircle, XCircle, Clock, FileText } from 'lucide-react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend,
} from 'recharts';
import Card from '../components/Card';
import MetricCard from '../components/MetricCard';
import ScoreBar from '../components/ScoreBar';
import LoadingSpinner from '../components/LoadingSpinner';
import Badge from '../components/Badge';
import { useCaseStore } from '../stores/caseStore';
import { api } from '../api/client';

interface WaveResult {
  wave: number;
  name: string;
  evidence: number;
  entities: number;
  hypotheses: number;
  alerts: number;
}

export default function Benchmark() {
  const { caseId, caseName } = useCaseStore();
  const [stats, setStats] = useState<any>(null);
  const [hypotheses, setHypotheses] = useState<any[]>([]);
  const [entities, setEntities] = useState<any[]>([]);
  const [evidence, setEvidence] = useState<any[]>([]);
  const [graphStats, setGraphStats] = useState<any>(null);
  const [auditLog, setAuditLog] = useState<any[]>([]);
  const [alerts, setAlerts] = useState<any[]>([]);
  const [evolution, setEvolution] = useState<any[]>([]);
  const [injecting, setInjecting] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'hypotheses' | 'entities' | 'evidence' | 'graph' | 'audit'>('overview');

  const refresh = useCallback(async () => {
    if (!caseId) return;
    try {
      const [s, h, ent, ev, gs, al, aud] = await Promise.all([
        api.get(`/cases/${caseId}/stats`).then(r => r.data).catch(() => null),
        api.get(`/cases/${caseId}/hypotheses`).then(r => r.data).catch(() => []),
        api.get(`/cases/${caseId}/entities`).then(r => r.data).catch(() => []),
        api.get(`/cases/${caseId}/evidence`).then(r => r.data).catch(() => []),
        api.get(`/cases/${caseId}/graph/stats`).then(r => r.data).catch(() => null),
        api.get(`/cases/${caseId}/alerts`).then(r => r.data).catch(() => []),
        api.get(`/cases/${caseId}/audit?limit=20`).then(r => r.data).catch(() => []),
      ]);
      setStats(s);
      setHypotheses(h || []);
      setEntities(ent || []);
      setEvidence(ev || []);
      setGraphStats(gs);
      setAlerts(al || []);
      setAuditLog(aud || []);

      // Fetch evolution for all hypotheses
      if (h && h.length > 0) {
        const evoData: any[] = [];
        for (const hyp of h.slice(0, 5)) {
          try {
            const evo = await api.get(`/hypotheses/${hyp.id}/evolution`).then(r => r.data);
            if (evo) {
              for (const p of evo) {
                evoData.push({ date: p.date?.slice(0, 16), score: p.score, hypothesis: hyp.title?.slice(0, 30) });
              }
            }
          } catch {}
        }
        setEvolution(evoData);
      }
    } catch {}
  }, [caseId]);

  useEffect(() => { refresh(); }, [refresh]);
  useEffect(() => {
    const interval = setInterval(refresh, 15000);
    return () => clearInterval(interval);
  }, [refresh]);

  const triggerAnalysis = async () => {
    if (!caseId) return;
    setAnalyzing(true);
    try {
      await api.post(`/cases/${caseId}/analyze`, { trigger: 'benchmark' });
    } catch {}
    setTimeout(() => { setAnalyzing(false); refresh(); }, 5000);
  };

  const generateHypotheses = async () => {
    if (!caseId) return;
    try {
      await api.post(`/cases/${caseId}/hypotheses/generate`);
    } catch {}
    setTimeout(refresh, 5000);
  };

  const evaluateAll = async () => {
    if (!caseId) return;
    try {
      await api.post(`/cases/${caseId}/evaluate-all`);
    } catch {}
    setTimeout(refresh, 5000);
  };

  if (!caseId) {
    return (
      <Card>
        <div className="flex flex-col items-center py-12">
          <FileText size={48} className="text-[var(--text-muted)] mb-4" />
          <p className="text-[var(--text-muted)]">Selectionnez un dossier dans la sidebar pour voir le benchmark.</p>
        </div>
      </Card>
    );
  }

  const entityTypes: Record<string, number> = {};
  entities.forEach(e => { entityTypes[e.entity_type] = (entityTypes[e.entity_type] || 0) + 1; });

  const tabs = [
    { id: 'overview', label: 'Vue d\'ensemble' },
    { id: 'hypotheses', label: `Hypotheses (${hypotheses.length})` },
    { id: 'entities', label: `Entites (${entities.length})` },
    { id: 'evidence', label: `Preuves (${evidence.length})` },
    { id: 'graph', label: 'Graphe' },
    { id: 'audit', label: 'Audit' },
  ] as const;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Benchmark — {caseName}</h2>
          <p className="text-sm text-[var(--text-muted)]">Evaluation en temps reel</p>
        </div>
        <div className="flex gap-2">
          <button onClick={refresh} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
            <RefreshCw size={12} /> Rafraichir
          </button>
          <button onClick={triggerAnalysis} disabled={analyzing} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50">
            {analyzing ? <LoadingSpinner size={12} /> : <Play size={12} />}
            {analyzing ? 'Analyse...' : 'Analyser'}
          </button>
          <button onClick={generateHypotheses} className="flex items-center gap-1.5 px-3 py-1.5 bg-purple-600 text-white rounded-lg text-xs font-medium hover:bg-purple-700">
            Generer hypotheses
          </button>
          <button onClick={evaluateAll} className="flex items-center gap-1.5 px-3 py-1.5 bg-green-600 text-white rounded-lg text-xs font-medium hover:bg-green-700">
            Re-evaluer
          </button>
        </div>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-6 gap-3">
        <MetricCard label="Preuves" value={stats?.evidence ?? 0} />
        <MetricCard label="Entites" value={stats?.entities ?? 0} />
        <MetricCard label="Hypotheses" value={stats?.hypotheses ?? 0} />
        <MetricCard label="Alertes" value={stats?.alerts ?? 0} />
        <MetricCard label="Monitoring" value={stats?.monitoring_jobs ?? 0} />
        <MetricCard label="Noeuds Neo4j" value={graphStats ? Object.values(graphStats).reduce((a: number, b: any) => a + (typeof b === 'number' ? b : 0), 0) : 0} />
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-[var(--border)]">
        {tabs.map(t => (
          <button
            key={t.id}
            onClick={() => setActiveTab(t.id as any)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeTab === t.id
                ? 'border-[var(--accent)] text-[var(--accent)]'
                : 'border-transparent text-[var(--text-muted)] hover:text-[var(--text-secondary)]'
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* TAB: Overview */}
      {activeTab === 'overview' && (
        <div className="grid grid-cols-2 gap-4">
          {/* Hypotheses score bars */}
          <Card title="Hypotheses">
            {hypotheses.length > 0 ? (
              <div className="space-y-3">
                {hypotheses.sort((a, b) => (b.current_score || 0) - (a.current_score || 0)).map(h => (
                  <div key={h.id}>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-[var(--text-primary)] font-medium truncate mr-2">{h.title}</span>
                      <span className="text-[var(--text-muted)]">{h.current_score?.toFixed(0)}%</span>
                    </div>
                    <ScoreBar score={h.current_score || 0} />
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-[var(--text-muted)] py-4 text-center">Aucune hypothese</p>
            )}
          </Card>

          {/* Entity breakdown */}
          <Card title="Entites par type">
            <div className="space-y-2">
              {Object.entries(entityTypes).sort((a, b) => b[1] - a[1]).map(([type, count]) => (
                <div key={type} className="flex justify-between items-center">
                  <Badge variant={type === 'person' ? 'blue' : type === 'location' ? 'green' : type === 'vehicle' ? 'red' : 'gray'}>
                    {type}
                  </Badge>
                  <span className="text-sm font-mono text-[var(--text-secondary)]">{count}</span>
                </div>
              ))}
            </div>
          </Card>

          {/* Evolution chart */}
          {evolution.length > 0 && (
            <Card title="Evolution des scores" className="col-span-2">
              <ResponsiveContainer width="100%" height={250}>
                <LineChart data={evolution}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis dataKey="date" tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                  <YAxis domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                  <Tooltip contentStyle={{ backgroundColor: 'var(--bg-card)', border: '1px solid var(--border)', borderRadius: '8px', color: 'var(--text-primary)' }} />
                  <Legend />
                  {[...new Set(evolution.map(e => e.hypothesis))].map((name, i) => (
                    <Line key={name} type="monotone" dataKey="score" data={evolution.filter(e => e.hypothesis === name)} name={name} stroke={['#3b82f6', '#22c55e', '#a855f7', '#eab308', '#ef4444'][i % 5]} dot={false} />
                  ))}
                </LineChart>
              </ResponsiveContainer>
            </Card>
          )}

          {/* Recent alerts */}
          <Card title="Alertes recentes" className="col-span-2">
            {alerts.length > 0 ? (
              <div className="space-y-1 max-h-48 overflow-auto">
                {alerts.slice(0, 10).map((a, i) => (
                  <div key={i} className="flex items-center gap-2 text-xs py-1 border-b border-[var(--border)]/30">
                    <Badge variant={a.severity === 'critical' ? 'red' : a.severity === 'warning' ? 'yellow' : 'blue'}>
                      {a.severity}
                    </Badge>
                    <span className="text-[var(--text-secondary)] truncate">{a.title}</span>
                    <span className="text-[var(--text-muted)] ml-auto text-[10px]">{a.created_at?.slice(11, 19)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-[var(--text-muted)] py-2 text-center">Aucune alerte</p>
            )}
          </Card>
        </div>
      )}

      {/* TAB: Hypotheses */}
      {activeTab === 'hypotheses' && (
        <div className="space-y-3">
          {hypotheses.length > 0 ? hypotheses.sort((a, b) => (b.current_score || 0) - (a.current_score || 0)).map(h => (
            <Card key={h.id}>
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-sm font-semibold text-[var(--text-primary)]">{h.title}</h3>
                    <Badge variant={h.status === 'active' ? 'green' : h.status === 'refuted' ? 'red' : 'gray'}>{h.status}</Badge>
                  </div>
                  <p className="text-xs text-[var(--text-muted)] mb-2">{h.description?.slice(0, 200)}</p>
                  <ScoreBar score={h.current_score || 0} height={8} />
                </div>
                <span className="text-2xl font-bold text-[var(--text-primary)] ml-4">{h.current_score?.toFixed(0)}%</span>
              </div>
            </Card>
          )) : (
            <Card><p className="text-sm text-[var(--text-muted)] text-center py-8">Aucune hypothese — cliquez "Generer hypotheses"</p></Card>
          )}
        </div>
      )}

      {/* TAB: Entities */}
      {activeTab === 'entities' && (
        <div className="space-y-4">
          {['person', 'location', 'vehicle', 'phone', 'organization', 'date', 'other'].map(type => {
            const ofType = entities.filter(e => e.entity_type === type);
            if (ofType.length === 0) return null;
            return (
              <Card key={type} title={`${type.charAt(0).toUpperCase() + type.slice(1)} (${ofType.length})`}>
                <div className="flex flex-wrap gap-2">
                  {ofType.map((e, i) => (
                    <span key={i} className="px-2 py-1 bg-[var(--bg-primary)] rounded text-xs text-[var(--text-secondary)] border border-[var(--border)]">
                      {e.name}
                    </span>
                  ))}
                </div>
              </Card>
            );
          })}
        </div>
      )}

      {/* TAB: Evidence */}
      {activeTab === 'evidence' && (
        <div className="space-y-2">
          {evidence.map((e, i) => (
            <Card key={i}>
              <div className="flex items-center gap-3">
                <Badge variant={e.status === 'processed' ? 'green' : e.status === 'processing' ? 'yellow' : 'gray'}>{e.status}</Badge>
                <div className="flex-1">
                  <p className="text-sm font-medium text-[var(--text-primary)]">{e.title}</p>
                  <p className="text-xs text-[var(--text-muted)]">{e.source} — Fiabilite: {e.reliability}/100</p>
                </div>
                {e.summary && (
                  <p className="text-xs text-[var(--text-secondary)] max-w-md truncate">{e.summary}</p>
                )}
              </div>
            </Card>
          ))}
          {evidence.length === 0 && <Card><p className="text-sm text-[var(--text-muted)] text-center py-8">Aucune preuve</p></Card>}
        </div>
      )}

      {/* TAB: Graph */}
      {activeTab === 'graph' && (
        <Card title="Statistiques du graphe Neo4j">
          {graphStats ? (
            <div className="grid grid-cols-4 gap-4">
              {Object.entries(graphStats).map(([label, count]) => (
                <div key={label} className="bg-[var(--bg-primary)] rounded-lg p-4 text-center border border-[var(--border)]">
                  <p className="text-2xl font-bold text-[var(--text-primary)]">{count as number}</p>
                  <p className="text-xs text-[var(--text-muted)] mt-1">{label}</p>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-[var(--text-muted)] text-center py-8">Graphe vide</p>
          )}
        </Card>
      )}

      {/* TAB: Audit */}
      {activeTab === 'audit' && (
        <Card title="Journal d'audit">
          <div className="space-y-1 max-h-96 overflow-auto">
            {auditLog.map((e, i) => (
              <div key={i} className="flex items-center gap-2 text-xs py-1.5 border-b border-[var(--border)]/30">
                <span className="text-[var(--text-muted)] font-mono w-16">{e.timestamp?.slice(11, 19)}</span>
                <Badge variant={e.actor === 'autonomous_loop' ? 'red' : e.actor === 'system' ? 'blue' : 'green'}>
                  {e.actor}
                </Badge>
                <span className="text-[var(--text-secondary)] truncate">{e.summary}</span>
              </div>
            ))}
            {auditLog.length === 0 && <p className="text-sm text-[var(--text-muted)] text-center py-4">Aucune entree</p>}
          </div>
        </Card>
      )}
    </div>
  );
}
