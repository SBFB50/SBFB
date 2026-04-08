import { useState, useEffect, useCallback } from 'react';
import { UserSearch, RefreshCw, ChevronRight } from 'lucide-react';
import {
  RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
} from 'recharts';
import Card from '../components/Card';
import ScoreBar from '../components/ScoreBar';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import {
  getSuspects,
  scoreAllSuspects,
  evaluateSuspectProfile,
  getSuspectEvolution,
} from '../api/client';
import { showToast } from '../components/Toast';

interface Suspect {
  id: string;
  case_id: string;
  entity_id: string;
  entity_name?: string;
  suspicion_score: number;
  graph_score: number;
  evidence_score: number;
  contradiction_score: number;
  profile_score: number;
  hypothesis_score: number;
  known_motive: string | null;
  alibi_status: string;
  criminal_record: string | null;
  relationship_to_victim: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

interface EvolutionPoint {
  date: string;
  suspicion_score: number;
  graph_score?: number;
  evidence_score?: number;
  contradiction_score?: number;
  profile_score?: number;
  hypothesis_score?: number;
}

const ALIBI_VARIANTS: Record<string, string> = {
  none: 'red',
  weak: 'warning',
  partial: 'yellow',
  strong: 'green',
  verified: 'blue',
  unknown: 'gray',
};

const FACTOR_LABELS: Record<string, string> = {
  graph: 'Graphe',
  evidence: 'Preuves',
  contradiction: 'Contradictions',
  profile: 'Profil',
  hypothesis: 'Hypotheses',
};

export default function Suspects() {
  const { caseId } = useCaseStore();
  const [suspects, setSuspects] = useState<Suspect[]>([]);
  const [loading, setLoading] = useState(true);
  const [scoring, setScoring] = useState(false);
  const [evaluating, setEvaluating] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [evolution, setEvolution] = useState<EvolutionPoint[]>([]);

  const refresh = useCallback(async () => {
    if (!caseId) return;
    try {
      const data = await getSuspects(caseId);
      const list = Array.isArray(data) ? data : (data?.suspects ?? []);
      setSuspects(list);
      if (list.length > 0 && !selectedId) {
        setSelectedId(list[0].id);
      }
    } catch (e) {
      showToast('error', 'Failed to load suspects');
      console.error('Failed to load suspects:', e);
    }
    setLoading(false);
  }, [caseId, selectedId]);

  useEffect(() => { refresh(); }, [caseId]);

  // Load evolution for selected suspect
  useEffect(() => {
    if (!selectedId) { setEvolution([]); return; }
    (async () => {
      try {
        const data = await getSuspectEvolution(selectedId);
        setEvolution(Array.isArray(data) ? data : []);
      } catch {
        setEvolution([]);
      }
    })();
  }, [selectedId]);

  const handleScoreAll = async () => {
    if (!caseId) return;
    setScoring(true);
    try {
      await scoreAllSuspects(caseId);
      await refresh();
    } catch (e) {
      showToast('error', 'Failed to score suspects');
      console.error('Score all failed:', e);
    }
    setScoring(false);
  };

  const handleEvaluate = async (suspectId: string) => {
    setEvaluating(suspectId);
    try {
      await evaluateSuspectProfile(suspectId);
      await refresh();
    } catch (e) {
      showToast('error', 'Failed to evaluate suspect profile');
      console.error('Evaluate failed:', e);
    }
    setEvaluating(null);
  };

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <UserSearch size={48} className="text-[var(--text-muted)] mb-3" />
        <p className="text-sm text-[var(--text-muted)]">Selectionnez un dossier pour voir les suspects</p>
      </div>
    );
  }

  if (loading) return <LoadingSpinner text="Chargement des suspects..." />;

  const selected = suspects.find(s => s.id === selectedId) ?? null;

  // Radar data for selected suspect
  const radarData = selected ? [
    { factor: 'Graphe', score: selected.graph_score },
    { factor: 'Preuves', score: selected.evidence_score },
    { factor: 'Contradictions', score: selected.contradiction_score },
    { factor: 'Profil', score: selected.profile_score },
    { factor: 'Hypotheses', score: selected.hypothesis_score },
  ] : [];

  // Evolution line chart data
  const evoChartData = evolution.map(p => ({
    date: (p.date || '').slice(0, 16),
    Score: p.suspicion_score,
  }));

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Suspects</h2>
          <p className="text-sm text-[var(--text-muted)]">
            {suspects.length} suspect{suspects.length !== 1 ? 's' : ''} — classes par score de suspicion
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleScoreAll}
            disabled={scoring}
            className="flex items-center gap-1.5 px-4 py-2 bg-[var(--accent)] text-white rounded-lg text-sm font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50 transition-colors"
          >
            {scoring ? <LoadingSpinner size={14} /> : <RefreshCw size={14} />}
            {scoring ? 'Scoring en cours...' : 'Scorer tous les suspects'}
          </button>
        </div>
      </div>

      {/* No suspects */}
      {suspects.length === 0 && (
        <Card>
          <div className="text-center py-12">
            <UserSearch size={48} className="text-[var(--text-muted)] mx-auto mb-3" />
            <p className="text-sm text-[var(--text-muted)]">
              Aucun suspect identifie. Lancez le scoring pour analyser les personnes du dossier.
            </p>
          </div>
        </Card>
      )}

      {/* Suspect cards */}
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-3">
        {suspects.map(s => {
          const name = s.entity_name || s.entity_id;
          const isSelected = s.id === selectedId;
          return (
            <div
              key={s.id}
              onClick={() => setSelectedId(s.id)}
              className={`bg-[var(--bg-card)] border rounded-lg p-4 cursor-pointer transition-all ${
                isSelected
                  ? 'border-[var(--accent)] ring-1 ring-[var(--accent)]/30'
                  : 'border-[var(--border)] hover:border-[var(--text-muted)]'
              }`}
            >
              {/* Name + alibi */}
              <div className="flex items-start justify-between mb-3">
                <h3 className="text-base font-bold text-[var(--text-primary)] truncate mr-2">{name}</h3>
                <Badge variant={ALIBI_VARIANTS[s.alibi_status] || 'gray'}>
                  {s.alibi_status === 'none' ? 'Pas d\'alibi' : s.alibi_status}
                </Badge>
              </div>

              {/* Global score */}
              <ScoreBar label="Score global" score={s.suspicion_score} height={8} />

              {/* Sub-scores */}
              <div className="mt-3 space-y-1.5">
                <ScoreBar label={FACTOR_LABELS.graph} score={s.graph_score} height={4} />
                <ScoreBar label={FACTOR_LABELS.evidence} score={s.evidence_score} height={4} />
                <ScoreBar label={FACTOR_LABELS.contradiction} score={s.contradiction_score} height={4} />
                <ScoreBar label={FACTOR_LABELS.profile} score={s.profile_score} height={4} />
                <ScoreBar label={FACTOR_LABELS.hypothesis} score={s.hypothesis_score} height={4} />
              </div>

              {/* Relationship + notes */}
              {s.relationship_to_victim && (
                <p className="text-xs text-[var(--text-secondary)] mt-3">
                  <span className="text-[var(--text-muted)]">Lien victime:</span> {s.relationship_to_victim}
                </p>
              )}
              {s.notes && (
                <p className="text-xs text-[var(--text-muted)] mt-1 truncate">{s.notes}</p>
              )}

              {/* Evaluate button */}
              <button
                onClick={(e) => { e.stopPropagation(); handleEvaluate(s.id); }}
                disabled={evaluating === s.id}
                className="mt-3 w-full flex items-center justify-center gap-1.5 px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-xs font-medium hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] disabled:opacity-50 transition-colors"
              >
                {evaluating === s.id ? (
                  <><LoadingSpinner size={12} /> Evaluation...</>
                ) : (
                  <><ChevronRight size={12} /> Evaluer profil</>
                )}
              </button>
            </div>
          );
        })}
      </div>

      {/* Detail section */}
      {selected && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Radar chart */}
          <Card title={`Profil — ${selected.entity_name || selected.entity_id}`}>
            {radarData.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <RadarChart data={radarData}>
                  <PolarGrid stroke="var(--border)" />
                  <PolarAngleAxis dataKey="factor" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
                  <PolarRadiusAxis domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 9 }} />
                  <Radar
                    name="Score"
                    dataKey="score"
                    stroke="#ef4444"
                    fill="#ef4444"
                    fillOpacity={0.25}
                  />
                </RadarChart>
              </ResponsiveContainer>
            ) : (
              <p className="text-sm text-[var(--text-muted)] text-center py-8">Aucune donnee</p>
            )}
          </Card>

          {/* Evolution chart */}
          <Card title="Evolution temporelle du score">
            {evoChartData.length > 1 ? (
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={evoChartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis dataKey="date" tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                  <YAxis domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'var(--bg-card)',
                      border: '1px solid var(--border)',
                      borderRadius: '8px',
                      color: 'var(--text-primary)',
                    }}
                  />
                  <Line
                    type="monotone"
                    dataKey="Score"
                    stroke="#ef4444"
                    strokeWidth={2}
                    dot={{ r: 3, fill: '#ef4444' }}
                  />
                </LineChart>
              </ResponsiveContainer>
            ) : (
              <p className="text-sm text-[var(--text-muted)] text-center py-8">
                {evoChartData.length === 1
                  ? 'Un seul snapshot — pas encore d\'evolution'
                  : 'Aucun snapshot disponible'}
              </p>
            )}
          </Card>
        </div>
      )}
    </div>
  );
}
