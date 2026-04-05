import { useState, useEffect, useCallback } from 'react';
import { Play, RefreshCw, Trash2, BarChart3, Users, FileText, Brain, AlertTriangle, Network } from 'lucide-react';
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, Cell,
  RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
} from 'recharts';
import Card from '../components/Card';
import ScoreBar from '../components/ScoreBar';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { api } from '../api/client';

interface CaseData {
  id: string;
  name: string;
  reference: string;
  status: string;
  stats: { evidence: number; entities: number; hypotheses: number; alerts: number; monitoring_jobs: number };
  hypotheses: any[];
  graphStats: any;
  entityTypes: Record<string, number>;
}

const COLORS = ['#3b82f6', '#22c55e', '#a855f7', '#eab308', '#ef4444', '#06b6d4', '#f97316', '#ec4899'];

export default function Benchmark() {
  const [cases, setCases] = useState<CaseData[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedCase, setSelectedCase] = useState<string | null>(null);
  const [auditLog, setAuditLog] = useState<any[]>([]);
  const [evolution, setEvolution] = useState<any[]>([]);
  const [availableBenches, setAvailableBenches] = useState<any[]>([]);
  const [launching, setLaunching] = useState<string | null>(null);
  const [showLauncher, setShowLauncher] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const rawCases = await api.get('/cases').then(r => r.data);
      const enriched: CaseData[] = [];

      for (const c of rawCases) {
        const [stats, hyps, ents, gs] = await Promise.all([
          api.get(`/cases/${c.id}/stats`).then(r => r.data).catch(() => ({})),
          api.get(`/cases/${c.id}/hypotheses`).then(r => r.data).catch(() => []),
          api.get(`/cases/${c.id}/entities`).then(r => r.data).catch(() => []),
          api.get(`/cases/${c.id}/graph/stats`).then(r => r.data).catch(() => null),
        ]);

        const entityTypes: Record<string, number> = {};
        (ents || []).forEach((e: any) => { entityTypes[e.entity_type] = (entityTypes[e.entity_type] || 0) + 1; });

        enriched.push({
          id: c.id,
          name: c.name,
          reference: c.reference || '',
          status: c.status,
          stats: stats || {},
          hypotheses: (hyps || []).sort((a: any, b: any) => (b.current_score || 0) - (a.current_score || 0)),
          graphStats: gs,
          entityTypes,
        });
      }

      setCases(enriched);
      if (enriched.length > 0 && !selectedCase) {
        setSelectedCase(enriched[0].id);
      }
    } catch (e) {
      console.error('Failed to load cases:', e);
    }
    setLoading(false);
  }, [selectedCase]);

  useEffect(() => { refresh(); }, []);

  // Load available benchmarks
  useEffect(() => {
    api.get('/benchmark/available').then(r => setAvailableBenches(r.data || [])).catch(() => {});
  }, []);

  const launchBench = async (key: string) => {
    setLaunching(key);
    try {
      const resp = await api.post(`/benchmark/launch/${key}`);
      if (resp.data?.case_id) {
        setShowLauncher(false);
        setTimeout(refresh, 3000);
      }
    } catch (e) {
      console.error('Launch failed:', e);
    }
    setLaunching(null);
  };

  const injectWave = async (caseId: string, benchKey: string, wave: number) => {
    try {
      await api.post(`/benchmark/inject/${caseId}/${benchKey}/wave/${wave}`);
      setTimeout(refresh, 3000);
    } catch {}
  };

  const deleteCase = async (caseId: string) => {
    try {
      await api.delete(`/cases/${caseId}`);
      setCases(prev => prev.filter(c => c.id !== caseId));
      if (selectedCase === caseId) setSelectedCase(null);
    } catch {}
  };

  // Load details for selected case
  useEffect(() => {
    if (!selectedCase) return;
    (async () => {
      try {
        const [aud] = await Promise.all([
          api.get(`/cases/${selectedCase}/audit?limit=30`).then(r => r.data).catch(() => []),
        ]);
        setAuditLog(aud || []);

        // Evolution
        const caseData = cases.find(c => c.id === selectedCase);
        if (caseData && caseData.hypotheses.length > 0) {
          const evoData: any[] = [];
          for (const hyp of caseData.hypotheses.slice(0, 5)) {
            try {
              const evo = await api.get(`/hypotheses/${hyp.id}/evolution`).then(r => r.data);
              if (evo) {
                for (const p of evo) {
                  evoData.push({ date: p.date?.slice(0, 16), score: p.score, hypothesis: hyp.title?.slice(0, 25) });
                }
              }
            } catch {}
          }
          setEvolution(evoData);
        } else {
          setEvolution([]);
        }
      } catch {}
    })();
  }, [selectedCase, cases]);

  if (loading) return <LoadingSpinner text="Chargement des dossiers..." />;

  if (cases.length === 0) {
    return (
      <Card>
        <div className="flex flex-col items-center py-16">
          <BarChart3 size={56} className="text-[var(--text-muted)] mb-4" />
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">Aucun dossier</h3>
          <p className="text-sm text-[var(--text-muted)]">Creez un dossier et injectez des preuves pour commencer le benchmark.</p>
        </div>
      </Card>
    );
  }

  const selected = cases.find(c => c.id === selectedCase) || cases[0];

  // Comparison data for bar chart
  const comparisonData = cases.map(c => ({
    name: c.name.slice(0, 20),
    Preuves: c.stats.evidence || 0,
    Entites: c.stats.entities || 0,
    Hypotheses: c.stats.hypotheses || 0,
    Alertes: c.stats.alerts || 0,
  }));

  // Entity type radar for selected case
  const radarData = Object.entries(selected.entityTypes).map(([type, count]) => ({
    type: type.charAt(0).toUpperCase() + type.slice(1),
    count,
  }));

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Benchmark global</h2>
          <p className="text-sm text-[var(--text-muted)]">{cases.length} dossier{cases.length > 1 ? 's' : ''} — vue comparative</p>
        </div>
        <div className="flex gap-2">
          <button onClick={() => setShowLauncher(!showLauncher)} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--accent-hover)]">
            <Play size={12} /> Nouveau benchmark
          </button>
          <button onClick={refresh} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
            <RefreshCw size={12} /> Rafraichir
          </button>
        </div>
      </div>

      {/* Launch new benchmark */}
      {showLauncher && (
        <Card title="Lancer un nouveau benchmark">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {availableBenches.map(b => (
              <div key={b.key} className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4">
                <h4 className="text-sm font-semibold text-[var(--text-primary)] mb-1">{b.name}</h4>
                <p className="text-xs text-[var(--text-muted)] mb-3">
                  {b.evidence_count} preuves — {b.waves} vagues
                  {b.has_ground_truth && ' — verite connue'}
                </p>
                <button
                  onClick={() => launchBench(b.key)}
                  disabled={launching === b.key}
                  className="w-full flex items-center justify-center gap-1.5 px-3 py-2 bg-[var(--accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50"
                >
                  {launching === b.key ? <LoadingSpinner size={12} /> : <Play size={12} />}
                  {launching === b.key ? 'Lancement...' : 'Lancer'}
                </button>
              </div>
            ))}
            {availableBenches.length === 0 && (
              <p className="text-sm text-[var(--text-muted)] col-span-3 text-center py-4">
                Aucun benchmark disponible dans data/benchmark/
              </p>
            )}
          </div>
        </Card>
      )}

      {/* Cases overview cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
        {cases.map((c, i) => (
          <div
            key={c.id}
            onClick={() => setSelectedCase(c.id)}
            className={`bg-[var(--bg-card)] border rounded-lg p-4 cursor-pointer transition-all ${
              c.id === selectedCase ? 'border-[var(--accent)] ring-1 ring-[var(--accent)]/30' : 'border-[var(--border)] hover:border-[var(--text-muted)]'
            }`}
          >
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-semibold text-[var(--text-primary)] truncate">{c.name}</h3>
              <div className="flex items-center gap-1.5">
                <Badge variant={c.status}>{c.status}</Badge>
                <button
                  onClick={(e) => { e.stopPropagation(); deleteCase(c.id); }}
                  className="p-1 text-[var(--text-muted)] hover:text-red-400 transition-colors"
                  title="Supprimer"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            </div>
            <div className="grid grid-cols-4 gap-2 text-center">
              <div>
                <p className="text-lg font-bold text-[var(--text-primary)]">{c.stats.evidence || 0}</p>
                <p className="text-[10px] text-[var(--text-muted)]">Preuves</p>
              </div>
              <div>
                <p className="text-lg font-bold text-[var(--text-primary)]">{c.stats.entities || 0}</p>
                <p className="text-[10px] text-[var(--text-muted)]">Entites</p>
              </div>
              <div>
                <p className="text-lg font-bold text-[var(--text-primary)]">{c.stats.hypotheses || 0}</p>
                <p className="text-[10px] text-[var(--text-muted)]">Hyp.</p>
              </div>
              <div>
                <p className="text-lg font-bold text-[var(--text-primary)]">{c.stats.alerts || 0}</p>
                <p className="text-[10px] text-[var(--text-muted)]">Alertes</p>
              </div>
            </div>
            {c.hypotheses.length > 0 && (
              <div className="mt-3">
                <ScoreBar score={c.hypotheses[0].current_score || 0} height={4} />
                <p className="text-[10px] text-[var(--text-muted)] mt-1 truncate">{c.hypotheses[0].title}</p>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Comparison chart */}
      {cases.length > 1 && (
        <Card title="Comparaison des dossiers">
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={comparisonData}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
              <XAxis dataKey="name" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
              <YAxis tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
              <Tooltip contentStyle={{ backgroundColor: 'var(--bg-card)', border: '1px solid var(--border)', borderRadius: '8px', color: 'var(--text-primary)' }} />
              <Legend />
              <Bar dataKey="Preuves" fill="#3b82f6" radius={[3, 3, 0, 0]} />
              <Bar dataKey="Entites" fill="#22c55e" radius={[3, 3, 0, 0]} />
              <Bar dataKey="Hypotheses" fill="#a855f7" radius={[3, 3, 0, 0]} />
              <Bar dataKey="Alertes" fill="#eab308" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </Card>
      )}

      {/* Selected case detail */}
      <div className="grid grid-cols-2 gap-4">
        {/* Hypotheses */}
        <Card title={`Hypotheses — ${selected.name.slice(0, 25)}`}>
          {selected.hypotheses.length > 0 ? (
            <div className="space-y-3">
              {selected.hypotheses.map((h: any) => (
                <div key={h.id}>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-[var(--text-primary)] font-medium truncate mr-2">{h.title}</span>
                    <div className="flex items-center gap-2 shrink-0">
                      <Badge variant={h.status}>{h.status}</Badge>
                      <span className="font-bold" style={{ color: h.current_score > 50 ? '#22c55e' : h.current_score > 25 ? '#eab308' : '#ef4444' }}>
                        {h.current_score?.toFixed(0)}%
                      </span>
                    </div>
                  </div>
                  <ScoreBar score={h.current_score || 0} height={6} />
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-[var(--text-muted)] text-center py-6">Aucune hypothese</p>
          )}
        </Card>

        {/* Entity radar */}
        <Card title="Repartition des entites">
          {radarData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <RadarChart data={radarData}>
                <PolarGrid stroke="var(--border)" />
                <PolarAngleAxis dataKey="type" tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                <PolarRadiusAxis tick={{ fill: 'var(--text-muted)', fontSize: 9 }} />
                <Radar name="Entites" dataKey="count" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.2} />
              </RadarChart>
            </ResponsiveContainer>
          ) : (
            <p className="text-sm text-[var(--text-muted)] text-center py-6">Aucune entite</p>
          )}
        </Card>

        {/* Evolution chart */}
        {evolution.length > 0 && (
          <Card title="Evolution des scores" className="col-span-2">
            <ResponsiveContainer width="100%" height={220}>
              <LineChart data={evolution}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="date" tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                <YAxis domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                <Tooltip contentStyle={{ backgroundColor: 'var(--bg-card)', border: '1px solid var(--border)', borderRadius: '8px', color: 'var(--text-primary)' }} />
                <Legend />
                {[...new Set(evolution.map(e => e.hypothesis))].map((name, i) => (
                  <Line key={name} type="monotone" dataKey="score" data={evolution.filter(e => e.hypothesis === name)} name={name} stroke={COLORS[i % COLORS.length]} dot={false} strokeWidth={2} />
                ))}
              </LineChart>
            </ResponsiveContainer>
          </Card>
        )}

        {/* Graph stats */}
        {selected.graphStats && (
          <Card title="Graphe Neo4j">
            <div className="grid grid-cols-3 gap-3">
              {Object.entries(selected.graphStats).map(([label, count]) => (
                <div key={label} className="bg-[var(--bg-primary)] rounded-lg p-3 text-center border border-[var(--border)]">
                  <p className="text-xl font-bold text-[var(--text-primary)]">{count as number}</p>
                  <p className="text-[10px] text-[var(--text-muted)]">{label}</p>
                </div>
              ))}
            </div>
          </Card>
        )}

        {/* Audit log */}
        <Card title="Journal d'audit">
          <div className="space-y-1 max-h-64 overflow-auto">
            {auditLog.length > 0 ? auditLog.map((e: any, i: number) => (
              <div key={i} className="flex items-center gap-2 text-xs py-1 border-b border-[var(--border)]/30">
                <span className="text-[var(--text-muted)] font-mono w-14 shrink-0">{e.timestamp?.slice(11, 19)}</span>
                <Badge variant={e.actor === 'autonomous_loop' ? 'red' : e.actor === 'system' ? 'blue' : 'green'}>{e.actor}</Badge>
                <span className="text-[var(--text-secondary)] truncate">{e.summary}</span>
              </div>
            )) : (
              <p className="text-sm text-[var(--text-muted)] text-center py-4">Aucune entree</p>
            )}
          </div>
        </Card>
      </div>

      {/* Actions */}
      <div className="flex gap-2">
        <button onClick={() => { api.post(`/cases/${selected.id}/analyze`, { trigger: 'benchmark' }); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-4 py-2 bg-[var(--accent)] text-white rounded-lg text-sm font-medium hover:bg-[var(--accent-hover)]">
          <Play size={14} /> Analyser
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/hypotheses/generate`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-4 py-2 bg-purple-600 text-white rounded-lg text-sm font-medium hover:bg-purple-700">
          <Brain size={14} /> Generer hypotheses
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/evaluate-all`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-4 py-2 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700">
          <RefreshCw size={14} /> Re-evaluer
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/investigation/start`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-4 py-2 bg-orange-600 text-white rounded-lg text-sm font-medium hover:bg-orange-700">
          <Network size={14} /> Investigation autonome
        </button>
      </div>
    </div>
  );
}
