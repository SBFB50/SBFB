import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Play, RefreshCw, Trash2, Users, FileText, Brain, AlertTriangle,
  Network, Clock, Activity, Target, Shield, Eye, Search, Zap,
  ChevronRight, Radio, Crosshair, Globe, Database, ChevronDown,
} from 'lucide-react';
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
} from 'recharts';
import Card from '../components/Card';
import ScoreBar from '../components/ScoreBar';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import PipelineTools from '../components/PipelineTools';
import InvestigationTimeline from '../components/InvestigationTimeline';
import InvestigationMap from '../components/InvestigationMap';
import { api } from '../api/client';
import { showToast } from '../components/Toast';

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

interface CaseStats {
  evidence: number;
  entities: number;
  hypotheses: number;
  alerts: number;
  monitoring_jobs: number;
}

interface Suspect {
  id: string;
  entity_name?: string;
  name?: string;
  suspicion_score: number;
  graph_score?: number;
  evidence_score?: number;
  contradiction_score?: number;
  profile_score?: number;
  hypothesis_score?: number;
}

interface Hypothesis {
  id: string;
  title: string;
  status: string;
  current_score: number;
  description?: string;
}

interface CaseData {
  id: string;
  name: string;
  reference: string;
  status: string;
  stats: CaseStats;
  hypotheses: Hypothesis[];
  suspects: Suspect[];
  graphStats: Record<string, number> | null;
  entityTypes: Record<string, number>;
  busStats?: { total_published: number; total_queues: number; total_pending: number };
}

interface AuditEntry {
  timestamp: string;
  action: string;
  actor: string;
  summary: string;
}

interface BenchProgress {
  caseId: string;
  lastAction: string;
  status: string;
  elapsed: number;
  wave?: number;
  totalWaves?: number;
  evidenceIndex?: number;
  totalEvidence?: number;
  step?: string;
  percent?: number;
  workers?: Record<string, { status: string; events_processed: number }>;
}

/* ------------------------------------------------------------------ */
/*  Constants                                                          */
/* ------------------------------------------------------------------ */

const ACTION_ICONS: Record<string, typeof Activity> = {
  evidence_added: FileText,
  evidence_ingested_auto: FileText,
  entity_discovered: Users,
  hypothesis_created: Brain,
  hypothesis_scored: Target,
  contradiction_found: Zap,
  monitoring_result: Search,
  query_generated: Search,
  self_questioning: Brain,
  analysis_completed: Eye,
  analysis_running: Activity,
  investigation_started: Play,
  investigation_stopped: Radio,
  geocode: Globe,
  osint_social: Globe,
  osint_enrichment: Search,
};

const ACTION_COLORS: Record<string, string> = {
  evidence_added: '#3b82f6',
  evidence_ingested_auto: '#3b82f6',
  entity_discovered: '#22c55e',
  hypothesis_created: '#a855f7',
  hypothesis_scored: '#a855f7',
  contradiction_found: '#ef4444',
  monitoring_result: '#06b6d4',
  query_generated: '#06b6d4',
  self_questioning: '#eab308',
  analysis_completed: '#22c55e',
  analysis_running: '#eab308',
  investigation_started: '#22c55e',
  investigation_stopped: '#ef4444',
  geocode: '#22c55e',
  osint_social: '#06b6d4',
  osint_enrichment: '#06b6d4',
};

const BENCH_MODES: Record<string, { label: string; color: string }> = {
  jubillar: { label: 'OSINT', color: '#06b6d4' },
  kulik: { label: 'Evidence', color: '#3b82f6' },
  'golden-state-killer': { label: 'Evidence', color: '#3b82f6' },
  'affaire-moreau': { label: 'Evidence', color: '#a855f7' },
};

const FACTOR_LABELS: { key: string; label: string; color: string }[] = [
  { key: 'graph_score', label: 'Graph', color: '#3b82f6' },
  { key: 'evidence_score', label: 'Evidence', color: '#22c55e' },
  { key: 'contradiction_score', label: 'Contradiction', color: '#ef4444' },
  { key: 'profile_score', label: 'Profile', color: '#a855f7' },
  { key: 'hypothesis_score', label: 'Hypothesis', color: '#eab308' },
];

/* ------------------------------------------------------------------ */
/*  Sub-components                                                     */
/* ------------------------------------------------------------------ */

function StatBox({ value, label, color, icon: Icon }: {
  value: number | string; label: string; color: string; icon: typeof Activity;
}) {
  return (
    <div className="flex items-center gap-3 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl px-4 py-3 min-w-0">
      <div className="p-2 rounded-lg shrink-0" style={{ backgroundColor: `${color}15` }}>
        <Icon size={18} style={{ color }} />
      </div>
      <div className="min-w-0">
        <p className="text-2xl font-bold text-[var(--text-primary)] leading-tight">{value}</p>
        <p className="text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wider">{label}</p>
      </div>
    </div>
  );
}

function SuspectCard({ suspect, rank }: { suspect: Suspect; rank: number }) {
  const name = suspect.entity_name || suspect.name || `Suspect #${suspect.id?.slice(0, 6)}`;
  const score = suspect.suspicion_score || 0;
  const isTop = rank === 1;
  const factors: Record<string, number> = {
    graph_score: suspect.graph_score || 0,
    evidence_score: suspect.evidence_score || 0,
    contradiction_score: suspect.contradiction_score || 0,
    profile_score: suspect.profile_score || 0,
    hypothesis_score: suspect.hypothesis_score || 0,
  };

  const radarData = FACTOR_LABELS.map(f => ({
    factor: f.label,
    value: (factors as Record<string, number>)[f.key] || 0,
  }));

  return (
    <div className={`bg-[var(--bg-card)] border rounded-xl p-4 transition-all ${
      isTop ? 'border-red-500/50 ring-1 ring-red-500/20' : 'border-[var(--border)]'
    }`}>
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2 min-w-0">
          <div className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold shrink-0 ${
            isTop ? 'bg-red-500/20 text-red-400' : 'bg-zinc-700 text-zinc-400'
          }`}>
            #{rank}
          </div>
          <div className="min-w-0">
            <p className={`text-sm font-semibold truncate ${isTop ? 'text-red-400' : 'text-[var(--text-primary)]'}`}>
              {name}
            </p>
          </div>
        </div>
        <div className="text-right shrink-0 ml-2">
          <p className={`text-lg font-bold ${
            score > 60 ? 'text-red-400' : score > 30 ? 'text-yellow-400' : 'text-[var(--text-muted)]'
          }`}>
            {score.toFixed(0)}
          </p>
          <p className="text-[9px] text-[var(--text-muted)]">suspicion</p>
        </div>
      </div>

      {/* Factor bars */}
      <div className="space-y-1.5 mb-3">
        {FACTOR_LABELS.map(f => {
          const val = (factors as Record<string, number>)[f.key] || 0;
          return (
            <div key={f.key} className="flex items-center gap-2">
              <span className="text-[9px] text-[var(--text-muted)] w-20 shrink-0 text-right">{f.label}</span>
              <div className="flex-1 h-1.5 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-500"
                  style={{ width: `${Math.min(val, 100)}%`, backgroundColor: f.color }}
                />
              </div>
              <span className="text-[9px] font-mono text-[var(--text-muted)] w-7 shrink-0">{val.toFixed(0)}</span>
            </div>
          );
        })}
      </div>

      {/* Mini radar */}
      {Object.keys(factors).length > 0 && (
        <div className="flex justify-center -mb-2">
          <ResponsiveContainer width={140} height={100}>
            <RadarChart data={radarData} cx="50%" cy="50%" outerRadius="70%">
              <PolarGrid stroke="var(--border)" />
              <PolarAngleAxis dataKey="factor" tick={{ fill: 'var(--text-muted)', fontSize: 7 }} />
              <Radar
                dataKey="value"
                stroke={isTop ? '#ef4444' : '#3b82f6'}
                fill={isTop ? '#ef4444' : '#3b82f6'}
                fillOpacity={0.15}
                strokeWidth={1.5}
              />
            </RadarChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}

function ActivityItem({ entry }: { entry: AuditEntry }) {
  const IconComp = ACTION_ICONS[entry.action] || Activity;
  const color = ACTION_COLORS[entry.action] || '#6b7280';
  const time = entry.timestamp?.slice(11, 19) || '';

  return (
    <div className="flex items-start gap-2.5 py-2 border-b border-[var(--border)]/20 last:border-0">
      <div className="p-1 rounded shrink-0 mt-0.5" style={{ backgroundColor: `${color}15` }}>
        <IconComp size={12} style={{ color }} />
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-xs text-[var(--text-secondary)] leading-snug truncate">{entry.summary}</p>
        <div className="flex items-center gap-2 mt-0.5">
          <span className="text-[9px] font-mono text-[var(--text-muted)]">{time}</span>
          <Badge variant={
            entry.action?.includes('evidence') ? 'blue' :
            entry.action?.includes('entity') || entry.action?.includes('geocode') ? 'green' :
            entry.action?.includes('hypothesis') ? 'purple' :
            entry.action?.includes('contradiction') ? 'red' :
            entry.action?.includes('monitoring') || entry.action?.includes('osint') ? 'info' :
            'gray'
          }>{entry.action?.replace(/_/g, ' ')}</Badge>
        </div>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Database Explorer                                                  */
/* ------------------------------------------------------------------ */

type DbTab = 'evidence' | 'entities' | 'monitoring' | 'events' | 'chroma';

const DB_TABS: { key: DbTab; label: string }[] = [
  { key: 'evidence', label: 'Evidence' },
  { key: 'entities', label: 'Entites' },
  { key: 'monitoring', label: 'Monitoring' },
  { key: 'events', label: 'Events' },
  { key: 'chroma', label: 'ChromaDB' },
];

function DatabaseExplorer({ caseId }: { caseId: string }) {
  const [open, setOpen] = useState(false);
  const [tab, setTab] = useState<DbTab>('evidence');
  const [data, setData] = useState<Record<DbTab, unknown[]>>({
    evidence: [], entities: [], monitoring: [], events: [], chroma: [],
  });
  const [monResults, setMonResults] = useState<unknown[]>([]);
  const [counts, setCounts] = useState<Record<DbTab, number>>({
    evidence: 0, entities: 0, monitoring: 0, events: 0, chroma: 0,
  });

  useEffect(() => {
    if (!open || !caseId) return;
    let active = true;

    const fetchAll = async () => {
      try {
        const [ev, ent, mon, monRes, aud, chroma] = await Promise.all([
          api.get(`/cases/${caseId}/evidence`).then(r => r.data).catch(() => []),
          api.get(`/cases/${caseId}/entities`).then(r => r.data).catch(() => []),
          api.get(`/cases/${caseId}/monitoring`).then(r => r.data).catch(() => []),
          api.get(`/cases/${caseId}/monitoring/results?limit=30`).then(r => r.data).catch(() => []),
          api.get(`/cases/${caseId}/audit?limit=50`).then(r => r.data).catch(() => []),
          api.get('/search/stats').then(r => r.data).catch(() => ({})),
        ]);
        if (!active) return;

        const chromaArr = Array.isArray(chroma)
          ? chroma
          : Object.entries(chroma)
              .filter(([k]) => !k.startsWith('_'))
              .map(([name, val]) => ({
                name,
                count: typeof val === 'object' && val !== null ? (val as any).count ?? 0 : val,
              }));

        setData({ evidence: ev || [], entities: ent || [], monitoring: mon || [], events: aud || [], chroma: chromaArr });
        setMonResults(monRes || []);
        setCounts({
          evidence: (ev || []).length,
          entities: (ent || []).length,
          monitoring: (mon || []).length,
          events: (aud || []).length,
          chroma: chromaArr.length,
        });
      } catch (e) { console.warn('[DatabaseExplorer] fetch error:', e); }
    };

    fetchAll();
    const interval = setInterval(fetchAll, 5000);
    return () => { active = false; clearInterval(interval); };
  }, [open, caseId]);

  const truncate = (s: unknown, max: number) => {
    const str = String(s || '');
    return str.length > max ? str.slice(0, max) + '...' : str;
  };

  const fmtDate = (d: unknown) => {
    const s = String(d || '');
    return s.slice(0, 19).replace('T', ' ');
  };

  const thClass = 'px-3 py-2 text-left text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)] bg-[var(--bg-primary)] sticky top-0';
  const tdClass = 'px-3 py-1.5 text-xs text-[var(--text-secondary)] font-mono whitespace-nowrap';

  const renderTable = () => {
    switch (tab) {
      case 'evidence':
        return (
          <table className="w-full">
            <thead><tr>
              <th className={thClass}>Status</th><th className={thClass}>Title</th><th className={thClass}>Source</th>
              <th className={thClass}>Type</th><th className={thClass}>Summary</th><th className={thClass}>Created</th>
            </tr></thead>
            <tbody>
              {(data.evidence as Record<string, unknown>[]).map((e, i) => (
                <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                  <td className={tdClass}><Badge variant={String(e.status || 'pending')}>{String(e.status || 'pending')}</Badge></td>
                  <td className={`${tdClass} max-w-[200px] truncate`}>{truncate(e.title, 60)}</td>
                  <td className={tdClass}>{truncate(e.source, 30)}</td>
                  <td className={tdClass}>{String(e.evidence_type || e.type || '-')}</td>
                  <td className={`${tdClass} max-w-[300px] truncate`}>{truncate(e.summary, 100)}</td>
                  <td className={tdClass}>{fmtDate(e.created_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        );

      case 'entities':
        return (
          <table className="w-full">
            <thead><tr>
              <th className={thClass}>Type</th><th className={thClass}>Name</th>
              <th className={thClass}>Description</th><th className={thClass}>Created</th>
            </tr></thead>
            <tbody>
              {(data.entities as Record<string, unknown>[]).map((e, i) => (
                <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                  <td className={tdClass}><Badge type={String(e.entity_type || 'other')}>{String(e.entity_type || 'other')}</Badge></td>
                  <td className={`${tdClass} font-semibold text-[var(--text-primary)]`}>{String(e.name || e.entity_name || '-')}</td>
                  <td className={`${tdClass} max-w-[300px] truncate`}>{truncate(e.description, 100)}</td>
                  <td className={tdClass}>{fmtDate(e.created_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        );

      case 'monitoring': {
        const jobs = data.monitoring as Record<string, unknown>[];
        const results = monResults as Record<string, unknown>[];
        return (
          <div className="space-y-4">
            <div>
              <p className="text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-2 px-3 font-semibold">Jobs ({jobs.length})</p>
              <table className="w-full">
                <thead><tr>
                  <th className={thClass}>Query</th><th className={thClass}>Type</th><th className={thClass}>Results</th>
                  <th className={thClass}>Last Run</th><th className={thClass}>Active</th>
                </tr></thead>
                <tbody>
                  {jobs.map((j, i) => (
                    <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                      <td className={`${tdClass} max-w-[250px] truncate`}>{truncate(j.query, 60)}</td>
                      <td className={tdClass}>{String(j.job_type || j.type || '-')}</td>
                      <td className={tdClass}>{String(j.results_count ?? '-')}</td>
                      <td className={tdClass}>{fmtDate(j.last_run)}</td>
                      <td className={tdClass}>{j.is_active ? '✓' : '✗'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div>
              <p className="text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-2 px-3 font-semibold">Results ({results.length})</p>
              <table className="w-full">
                <thead><tr>
                  <th className={thClass}>Title</th><th className={thClass}>URL</th><th className={thClass}>Score</th>
                  <th className={thClass}>Dup</th><th className={thClass}>Found</th>
                </tr></thead>
                <tbody>
                  {results.map((r, i) => (
                    <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                      <td className={`${tdClass} max-w-[250px] truncate`}>{truncate(r.title, 60)}</td>
                      <td className={tdClass}>
                        {r.url ? <a href={String(r.url)} target="_blank" rel="noreferrer" className="text-blue-400 hover:underline">{truncate(r.url, 40)}</a> : '-'}
                      </td>
                      <td className={tdClass}>{typeof r.relevance_score === 'number' ? (r.relevance_score as number).toFixed(2) : '-'}</td>
                      <td className={tdClass}>{r.is_duplicate ? 'Yes' : 'No'}</td>
                      <td className={tdClass}>{fmtDate(r.found_at)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        );
      }

      case 'events':
        return (
          <table className="w-full">
            <thead><tr>
              <th className={thClass}>Timestamp</th><th className={thClass}>Actor</th>
              <th className={thClass}>Action</th><th className={thClass}>Summary</th>
            </tr></thead>
            <tbody>
              {(data.events as Record<string, unknown>[]).map((e, i) => (
                <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                  <td className={`${tdClass} text-[var(--text-muted)]`}>{fmtDate(e.timestamp)}</td>
                  <td className={tdClass}><Badge variant="blue">{String(e.actor || '-')}</Badge></td>
                  <td className={tdClass}><Badge variant={
                    String(e.action || '').includes('evidence') ? 'blue' :
                    String(e.action || '').includes('entity') ? 'green' :
                    String(e.action || '').includes('hypothesis') ? 'purple' :
                    String(e.action || '').includes('contradiction') ? 'red' : 'gray'
                  }>{String(e.action || '-').replace(/_/g, ' ')}</Badge></td>
                  <td className={`${tdClass} max-w-[400px] truncate`}>{truncate(e.summary, 120)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        );

      case 'chroma':
        return (
          <table className="w-full">
            <thead><tr>
              <th className={thClass}>Collection</th><th className={thClass}>Items</th>
            </tr></thead>
            <tbody>
              {(data.chroma as Record<string, unknown>[]).map((c, i) => (
                <tr key={i} className="border-b border-[var(--border)]/30 hover:bg-[var(--bg-hover)]">
                  <td className={`${tdClass} font-semibold text-[var(--text-primary)]`}>{String(c.name || c.collection || '-')}</td>
                  <td className={tdClass}>{String(c.count ?? c.items ?? c.num_items ?? '-')}</td>
                </tr>
              ))}
            </tbody>
          </table>
        );
    }
  };

  return (
    <div className="border border-[var(--border)] rounded-xl overflow-hidden">
      <button
        onClick={() => setOpen(o => !o)}
        className="w-full flex items-center gap-2 px-4 py-3 bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] transition-colors text-left"
      >
        <Database size={16} className="text-[var(--text-muted)]" />
        <span className="text-sm font-semibold text-[var(--text-primary)] flex-1">Base de donnees</span>
        <span className="text-[10px] text-[var(--text-muted)] font-mono">
          {open ? Object.values(counts).reduce((a, b) => a + b, 0) + ' rows' : 'cliquer pour ouvrir'}
        </span>
        <ChevronDown size={14} className={`text-[var(--text-muted)] transition-transform ${open ? 'rotate-180' : ''}`} />
      </button>

      {open && (
        <div className="border-t border-[var(--border)]">
          {/* Tab bar */}
          <div className="flex gap-0 bg-[var(--bg-primary)] border-b border-[var(--border)] overflow-x-auto">
            {DB_TABS.map(t => (
              <button
                key={t.key}
                onClick={() => setTab(t.key)}
                className={`px-4 py-2 text-xs font-medium whitespace-nowrap border-b-2 transition-colors ${
                  tab === t.key
                    ? 'border-[var(--accent)] text-[var(--accent)] bg-[var(--bg-card)]'
                    : 'border-transparent text-[var(--text-muted)] hover:text-[var(--text-secondary)]'
                }`}
              >
                {t.label}
                <span className="ml-1.5 text-[10px] font-mono opacity-60">{counts[t.key]}</span>
              </button>
            ))}
          </div>

          {/* Table content */}
          <div className="max-h-96 overflow-auto bg-[var(--bg-card)]">
            {counts[tab] === 0 && tab !== 'monitoring' ? (
              <div className="text-center py-8 text-[var(--text-muted)]">
                <Database size={24} className="mx-auto mb-2 opacity-30" />
                <p className="text-xs">Aucune donnee</p>
              </div>
            ) : (
              renderTable()
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Main Component                                                     */
/* ------------------------------------------------------------------ */

export default function Benchmark() {
  const [cases, setCases] = useState<CaseData[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedCase, setSelectedCase] = useState<string | null>(null);
  const [auditLog, setAuditLog] = useState<AuditEntry[]>([]);
  const [availableBenches, setAvailableBenches] = useState<any[]>([]);
  const [launching, setLaunching] = useState<string | null>(null);
  const [benchProgress, setBenchProgress] = useState<BenchProgress | null>(null);
  const benchStartRef = useRef<number | null>(null);

  /* ---- Data fetching ---- */

  const refresh = useCallback(async () => {
    try {
      const rawCases = await api.get('/cases').then(r => r.data);
      const enriched: CaseData[] = [];

      for (const c of rawCases) {
        const [stats, hyps, suspects, ents, gs, invStatus] = await Promise.all([
          api.get(`/cases/${c.id}/stats`).then(r => r.data).catch(() => ({})),
          api.get(`/cases/${c.id}/hypotheses`).then(r => r.data).catch(() => []),
          api.get(`/cases/${c.id}/suspects`).then(r => r.data).catch(() => []),
          api.get(`/cases/${c.id}/entities`).then(r => r.data).catch(() => []),
          api.get(`/cases/${c.id}/graph/stats`).then(r => r.data).catch(() => null),
          api.get(`/cases/${c.id}/investigation/status`).then(r => r.data).catch(() => null),
        ]);

        const entityTypes: Record<string, number> = {};
        (ents || []).forEach((e: any) => {
          entityTypes[e.entity_type] = (entityTypes[e.entity_type] || 0) + 1;
        });

        enriched.push({
          id: c.id,
          name: c.name,
          reference: c.reference || '',
          status: c.status,
          stats: stats || {},
          hypotheses: (hyps || []).sort((a: any, b: any) => (b.current_score || 0) - (a.current_score || 0)),
          suspects: (suspects || []).sort((a: any, b: any) => (b.suspicion_score || 0) - (a.suspicion_score || 0)),
          graphStats: gs,
          entityTypes,
          busStats: invStatus?.bus_stats || undefined,
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

  // Auto-refresh every 4s
  useEffect(() => {
    const interval = setInterval(refresh, 4000);
    return () => clearInterval(interval);
  }, [refresh]);

  // Load available benchmarks
  useEffect(() => {
    api.get('/benchmark/available').then(r => setAvailableBenches(r.data || [])).catch(e => console.warn('[Benchmark] failed to load available benchmarks:', e));
  }, []);

  // Load audit log for selected case
  useEffect(() => {
    if (!selectedCase) return;
    let active = true;
    const fetchAudit = async () => {
      try {
        const aud = await api.get(`/cases/${selectedCase}/audit?limit=20`).then(r => r.data).catch(() => []);
        if (active) setAuditLog(aud || []);
      } catch (e) { console.warn('[Benchmark] audit fetch error:', e); }
    };
    fetchAudit();
    const interval = setInterval(fetchAudit, 5000);
    return () => { active = false; clearInterval(interval); };
  }, [selectedCase]);

  // Poll benchmark progress
  useEffect(() => {
    if (!benchProgress?.caseId) return;
    const caseId = benchProgress.caseId;
    let active = true;

    const poll = async () => {
      try {
        let data: Record<string, unknown> | null = null;
        try {
          data = await api.get(`/benchmark/progress/${caseId}`).then(r => r.data);
        } catch (e) {
          console.warn('[Benchmark] progress endpoint unavailable, falling back:', e);
          data = await api.get(`/cases/${caseId}/investigation/status`).then(r => r.data);
        }
        if (!active || !data) return;

        const elapsed = benchStartRef.current ? Math.floor((Date.now() - benchStartRef.current) / 1000) : 0;
        const invStatus = String(data.status || data.state || 'idle');
        const lastAction = String(data.last_action || data.current_task || data.step || '');

        if ((invStatus === 'idle' || invStatus === 'completed') && elapsed > 10) {
          setBenchProgress(null);
          benchStartRef.current = null;
          refresh();
          return;
        }

        setBenchProgress(prev => prev ? {
          ...prev,
          lastAction,
          status: invStatus,
          elapsed,
          wave: data!.wave as number | undefined,
          totalWaves: data!.total_waves as number | undefined,
          evidenceIndex: data!.evidence_index as number | undefined,
          totalEvidence: data!.total_evidence as number | undefined,
          step: data!.step as string | undefined,
          percent: data!.percent as number | undefined,
          workers: data!.workers as Record<string, { status: string; events_processed: number }> | undefined,
        } : null);
      } catch (e) { console.warn('[Benchmark] progress poll error:', e); }
    };

    const interval = setInterval(poll, 2000);
    poll();
    return () => { active = false; clearInterval(interval); };
  }, [benchProgress?.caseId, refresh]);

  /* ---- Actions ---- */

  const launchBench = async (key: string) => {
    setLaunching(key);
    try {
      const resp = await api.post(`/benchmark/launch/${key}`, {}, { timeout: 15000 });
      const caseId = resp.data?.case_id;
      if (caseId) {
        benchStartRef.current = Date.now();
        setBenchProgress({ caseId, lastAction: 'Demarrage...', status: 'starting', elapsed: 0 });
      }
    } catch (e: any) {
      if (e?.code === 'ECONNABORTED' || e?.message?.includes('timeout')) {
        showToast('info', `Benchmark ${key} lance en arriere-plan`);
      } else {
        showToast('error', `Echec: ${e?.response?.data?.detail || e?.message || 'Erreur'}`);
      }
    }
    setLaunching(null);
    setTimeout(refresh, 3000);
  };

  const deleteCase = async (caseId: string) => {
    try {
      await api.delete(`/cases/${caseId}`);
      setCases(prev => prev.filter(c => c.id !== caseId));
      if (selectedCase === caseId) setSelectedCase(null);
      showToast('info', 'Dossier supprime');
    } catch (e: any) {
      showToast('error', `Suppression echouee: ${e?.response?.data?.detail || e?.message || 'Erreur'}`);
    }
  };

  /* ---- Render: Loading ---- */
  if (loading) return <LoadingSpinner text="Chargement des dossiers..." />;

  /* ---- Render: No cases — Launcher ---- */
  if (cases.length === 0) {
    return (
      <div className="space-y-8 max-w-5xl mx-auto">
        <div className="text-center pt-8">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-[var(--accent)]/10 mb-4">
            <Crosshair size={32} className="text-[var(--accent)]" />
          </div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)] mb-2">Benchmark NEXUS</h1>
          <p className="text-sm text-[var(--text-muted)] max-w-lg mx-auto">
            Evaluez le systeme sur des cold cases reels. NEXUS recoit les preuves brutes sans la solution
            et doit converger vers la verite de maniere autonome.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {availableBenches.map(b => {
            const mode = BENCH_MODES[b.key] || { label: 'Evidence', color: '#3b82f6' };
            return (
              <div key={b.key} className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 hover:border-[var(--accent)]/50 transition-all group">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <h3 className="text-base font-bold text-[var(--text-primary)] group-hover:text-[var(--accent)] transition-colors">
                      {b.name}
                    </h3>
                    <div className="flex items-center gap-2 mt-1.5">
                      <span
                        className="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded"
                        style={{ color: mode.color, backgroundColor: `${mode.color}15` }}
                      >
                        {mode.label}
                      </span>
                      <span className="text-xs text-[var(--text-muted)]">
                        {b.evidence_count} preuves | {b.waves} vagues
                      </span>
                    </div>
                  </div>
                  {b.has_ground_truth && (
                    <Shield size={16} className="text-green-400 shrink-0 mt-1" title="Verite terrain disponible" />
                  )}
                </div>

                {b.has_ground_truth && (
                  <p className="text-xs text-green-400/80 mb-4">
                    Verite connue -- scoring automatique
                  </p>
                )}

                <button
                  onClick={() => launchBench(b.key)}
                  disabled={launching === b.key}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-[var(--accent)] text-white rounded-lg text-sm font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50 transition-colors"
                >
                  {launching === b.key ? <LoadingSpinner size={14} /> : <Play size={14} />}
                  {launching === b.key ? 'Creation...' : 'Lancer le benchmark'}
                </button>
              </div>
            );
          })}
        </div>

        {availableBenches.length === 0 && (
          <div className="text-center py-12 text-[var(--text-muted)]">
            <FileText size={48} className="mx-auto mb-3 opacity-30" />
            <p className="text-sm">Aucun benchmark disponible dans data/benchmark/</p>
          </div>
        )}

        <div className="flex justify-center">
          <button onClick={refresh} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
            <RefreshCw size={12} /> Rafraichir
          </button>
        </div>
      </div>
    );
  }

  /* ---- Render: Dashboard ---- */
  const selected = cases.find(c => c.id === selectedCase) || cases[0];
  const hypothesisWorkerBusy = selected.busStats?.total_pending
    ? selected.busStats.total_pending > 0
    : false;

  const sortedAudit = [...auditLog].sort((a, b) =>
    (b.timestamp || '').localeCompare(a.timestamp || '')
  );

  return (
    <div className="space-y-4">
      {/* ============ HEADER ============ */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-[var(--text-primary)]">
            Investigation: {selected.name}
          </h2>
          <p className="text-xs text-[var(--text-muted)]">
            {cases.length} dossier{cases.length > 1 ? 's' : ''} | ref: {selected.reference || selected.id.slice(0, 8)}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {/* Case selector if multiple */}
          {cases.length > 1 && (
            <select
              value={selectedCase || ''}
              onChange={e => setSelectedCase(e.target.value)}
              className="bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-primary)] text-xs rounded-lg px-3 py-1.5 focus:outline-none focus:border-[var(--accent)]"
            >
              {cases.map(c => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          )}
          <button onClick={refresh} className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
            <RefreshCw size={12} />
          </button>
          <button
            onClick={() => deleteCase(selected.id)}
            className="p-1.5 text-[var(--text-muted)] hover:text-red-400 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg transition-colors"
            title="Supprimer ce dossier"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {/* ============ BENCHMARK PROGRESS BANNER ============ */}
      {benchProgress && (
        <div className="bg-[var(--bg-card)] border border-[var(--accent)]/30 rounded-xl p-4 space-y-3">
          <div className="flex items-center gap-3">
            <LoadingSpinner size={16} />
            <div className="flex-1 min-w-0">
              <p className="text-sm text-[var(--text-primary)] truncate">
                {benchProgress.step || benchProgress.lastAction || 'Traitement en cours...'}
              </p>
              <p className="text-[10px] text-[var(--text-muted)]">
                Statut: {benchProgress.status}
                {benchProgress.wave !== undefined && benchProgress.totalWaves !== undefined && (
                  <> | Vague {benchProgress.wave}/{benchProgress.totalWaves}</>
                )}
                {benchProgress.evidenceIndex !== undefined && benchProgress.totalEvidence !== undefined && (
                  <> | Preuve {benchProgress.evidenceIndex}/{benchProgress.totalEvidence}</>
                )}
              </p>
            </div>
            <div className="flex items-center gap-1 text-xs text-[var(--text-muted)] font-mono shrink-0">
              <Clock size={11} />
              {Math.floor(benchProgress.elapsed / 60)}:{String(benchProgress.elapsed % 60).padStart(2, '0')}
            </div>
          </div>
          {(() => {
            const pct = benchProgress.percent ??
              (benchProgress.totalEvidence && benchProgress.evidenceIndex
                ? Math.round((benchProgress.evidenceIndex / benchProgress.totalEvidence) * 100)
                : null);
            if (pct === null) return null;
            return (
              <div>
                <div className="flex justify-between text-[10px] text-[var(--text-muted)] mb-1">
                  <span>Progression</span>
                  <span>{pct}%</span>
                </div>
                <div className="w-full h-2 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                  <div
                    className="h-full bg-[var(--accent)] rounded-full transition-all duration-700 ease-out"
                    style={{ width: `${Math.min(pct, 100)}%` }}
                  />
                </div>
              </div>
            );
          })()}
        </div>
      )}

      {/* ============ TOP STATS BAR ============ */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-2">
        <StatBox value={selected.stats.evidence || 0} label="Preuves" color="#3b82f6" icon={FileText} />
        <StatBox value={selected.stats.entities || 0} label="Entites" color="#22c55e" icon={Users} />
        <StatBox value={selected.stats.hypotheses || 0} label="Hypotheses" color="#a855f7" icon={Brain} />
        <StatBox value={selected.suspects.length} label="Suspects" color="#ef4444" icon={Target} />
        <StatBox value={selected.stats.alerts || 0} label="Alertes" color="#eab308" icon={AlertTriangle} />
        <StatBox
          value={selected.busStats?.total_published || 0}
          label="Events"
          color="#06b6d4"
          icon={Activity}
        />
      </div>

      {/* ============ MAIN 2-COLUMN LAYOUT ============ */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">

        {/* ---- LEFT COLUMN (2/3) ---- */}
        <div className="lg:col-span-2 space-y-4">

          {/* Suspects */}
          <Card title="Suspects" action={
            <span className="text-[10px] text-[var(--text-muted)]">
              {selected.suspects.length} identifie{selected.suspects.length !== 1 ? 's' : ''}
            </span>
          }>
            {selected.suspects.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {selected.suspects.slice(0, 6).map((s, i) => (
                  <SuspectCard key={s.id || i} suspect={s} rank={i + 1} />
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-[var(--text-muted)]">
                <Target size={32} className="mx-auto mb-2 opacity-30" />
                <p className="text-sm">Aucun suspect identifie</p>
                <p className="text-xs mt-1">Les suspects apparaitront apres l'analyse</p>
              </div>
            )}
          </Card>

          {/* Hypotheses */}
          <Card title="Hypotheses" action={
            selected.hypotheses.length > 0 ? (
              <span className="text-[10px] text-[var(--text-muted)]">
                {selected.hypotheses.length} hypothese{selected.hypotheses.length !== 1 ? 's' : ''}
              </span>
            ) : null
          }>
            {selected.hypotheses.length > 0 ? (
              <div className="space-y-3">
                {selected.hypotheses.map((h, i) => (
                  <div key={h.id} className={`p-3 rounded-lg border transition-all ${
                    i === 0
                      ? 'bg-purple-500/5 border-purple-500/20'
                      : 'bg-[var(--bg-primary)] border-[var(--border)]'
                  }`}>
                    <div className="flex items-start justify-between gap-3 mb-2">
                      <div className="flex items-center gap-2 min-w-0">
                        {i === 0 && <ChevronRight size={14} className="text-purple-400 shrink-0" />}
                        <p className="text-sm font-medium text-[var(--text-primary)] truncate">{h.title}</p>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <Badge variant={h.status}>{h.status}</Badge>
                        <span className="text-sm font-bold" style={{
                          color: h.current_score > 50 ? '#22c55e' : h.current_score > 25 ? '#eab308' : '#ef4444'
                        }}>
                          {h.current_score?.toFixed(0)}%
                        </span>
                      </div>
                    </div>
                    <ScoreBar score={h.current_score || 0} height={5} />
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-[var(--text-muted)]">
                {hypothesisWorkerBusy ? (
                  <>
                    <LoadingSpinner size={24} className="mb-2" />
                    <p className="text-sm">Generation en cours...</p>
                    <p className="text-xs mt-1">Le moteur d'hypotheses traite les preuves</p>
                  </>
                ) : (
                  <>
                    <Brain size={32} className="mx-auto mb-2 opacity-30" />
                    <p className="text-sm">Aucune hypothese</p>
                    <p className="text-xs mt-1">Les hypotheses seront generees apres l'analyse des preuves</p>
                  </>
                )}
              </div>
            )}
          </Card>

          {/* Entity type distribution */}
          {Object.keys(selected.entityTypes).length > 0 && (
            <Card title="Distribution des entites">
              <ResponsiveContainer width="100%" height={200}>
                <BarChart
                  data={Object.entries(selected.entityTypes)
                    .sort(([, a], [, b]) => b - a)
                    .map(([type, count]) => ({ type, count }))}
                  layout="vertical"
                  margin={{ left: 80, right: 20 }}
                >
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" horizontal={false} />
                  <XAxis type="number" tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                  <YAxis type="category" dataKey="type" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} width={75} />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'var(--bg-card)',
                      border: '1px solid var(--border)',
                      borderRadius: '8px',
                      color: 'var(--text-primary)',
                    }}
                  />
                  <Bar dataKey="count" fill="#3b82f6" radius={[0, 4, 4, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </Card>
          )}

          {/* Graph stats */}
          {selected.graphStats && Object.keys(selected.graphStats).length > 0 && (
            <Card title="Graphe Neo4j">
              <div className="grid grid-cols-3 md:grid-cols-4 gap-2">
                {Object.entries(selected.graphStats).map(([label, count]) => (
                  <div key={label} className="bg-[var(--bg-primary)] rounded-lg p-3 text-center border border-[var(--border)]">
                    <p className="text-xl font-bold text-[var(--text-primary)]">{count as number}</p>
                    <p className="text-[10px] text-[var(--text-muted)] truncate">{label}</p>
                  </div>
                ))}
              </div>
            </Card>
          )}
        </div>

        {/* ---- RIGHT COLUMN (1/3) ---- */}
        <div className="space-y-4">

          {/* Pipeline Workers */}
          {selectedCase && <PipelineTools caseId={selectedCase} />}

          {/* Activity Feed */}
          <Card title="Activite recente" action={
            <span className="text-[10px] text-[var(--text-muted)]">
              {sortedAudit.length} event{sortedAudit.length !== 1 ? 's' : ''}
            </span>
          }>
            {sortedAudit.length > 0 ? (
              <div className="max-h-96 overflow-auto -mx-1 px-1">
                {sortedAudit.slice(0, 20).map((e, i) => (
                  <ActivityItem key={i} entry={e} />
                ))}
              </div>
            ) : (
              <div className="text-center py-6 text-[var(--text-muted)]">
                <Activity size={24} className="mx-auto mb-2 opacity-30" />
                <p className="text-xs">En attente d'activite...</p>
              </div>
            )}
          </Card>

          {/* Investigation Map */}
          {selectedCase && <InvestigationMap caseId={selectedCase} />}

          {/* Investigation Timeline */}
          {selectedCase && <InvestigationTimeline caseId={selectedCase} />}
        </div>
      </div>

      {/* ============ ACTIONS BAR ============ */}
      <div className="flex items-center gap-2 pt-2 border-t border-[var(--border)]">
        <button onClick={() => { api.post(`/cases/${selected.id}/investigation/start`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-4 py-2 bg-[var(--accent)] text-white rounded-lg text-sm font-medium hover:bg-[var(--accent-hover)]">
          <Network size={14} /> Investigation autonome
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/analyze`, { trigger: 'benchmark' }); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
          <Play size={12} /> Analyser
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/hypotheses/generate`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
          <Brain size={12} /> Generer hypotheses
        </button>
        <button onClick={() => { api.post(`/cases/${selected.id}/evaluate-all`); setTimeout(refresh, 3000); }}
          className="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]">
          <RefreshCw size={12} /> Re-evaluer
        </button>

        {/* New benchmark button */}
        <div className="ml-auto relative group">
          <button
            className="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs hover:bg-[var(--bg-hover)]"
          >
            <Play size={12} /> Nouveau benchmark
          </button>
          {/* Dropdown */}
          <div className="absolute bottom-full right-0 mb-2 w-64 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-50">
            {availableBenches.map(b => (
              <button
                key={b.key}
                onClick={() => launchBench(b.key)}
                disabled={launching === b.key}
                className="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-[var(--bg-hover)] transition-colors first:rounded-t-xl last:rounded-b-xl"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-xs font-medium text-[var(--text-primary)]">{b.name}</p>
                  <p className="text-[10px] text-[var(--text-muted)]">{b.evidence_count} preuves | {b.waves} vagues</p>
                </div>
                {launching === b.key ? <LoadingSpinner size={12} /> : <ChevronRight size={12} className="text-[var(--text-muted)]" />}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* ============ DATABASE EXPLORER ============ */}
      {selectedCase && <DatabaseExplorer caseId={selectedCase} />}
    </div>
  );
}
