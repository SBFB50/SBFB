import { useState } from 'react';
import { Lightbulb, RefreshCw, Sparkles } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import Card from '../components/Card';
import ScoreBar from '../components/ScoreBar';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useHypotheses, useGenerateHypotheses, useEvaluateHypotheses } from '../hooks/useApi';
import { useQuery } from '@tanstack/react-query';
import { getHypothesisEvolution } from '../api/client';

interface Hypothesis {
  id?: string;
  hypothesis_id?: string;
  title?: string;
  description?: string;
  score?: number;
  status?: string;
  supporting_evidence?: number;
  contradicting_evidence?: number;
  created_at?: string;
}

export default function Hypotheses() {
  const { caseId } = useCaseStore();
  const hypothesesQuery = useHypotheses();
  const generateHypotheses = useGenerateHypotheses();
  const evaluateHypotheses = useEvaluateHypotheses();
  const [selectedHypId, setSelectedHypId] = useState<string | null>(null);

  const evolutionQuery = useQuery({
    queryKey: ['hypothesisEvolution', selectedHypId],
    queryFn: () => getHypothesisEvolution(selectedHypId!),
    enabled: !!selectedHypId,
  });

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Lightbulb size={48} className="text-[var(--text-muted)] mb-4" />
        <p className="text-[var(--text-secondary)]">Select a case to view hypotheses.</p>
      </div>
    );
  }

  const hypotheses: Hypothesis[] = Array.isArray(hypothesesQuery.data) ? hypothesesQuery.data : [];
  const sorted = [...hypotheses].sort((a, b) => (b.score ?? 0) - (a.score ?? 0));

  const evolutionData = Array.isArray(evolutionQuery.data) ? evolutionQuery.data : [];
  const chartData = evolutionData.map((e: { timestamp?: string; created_at?: string; score?: number }) => ({
    time: e.timestamp || e.created_at
      ? new Date(e.timestamp || e.created_at!).toLocaleDateString()
      : '',
    score: (e.score ?? 0) <= 1 ? (e.score ?? 0) * 100 : (e.score ?? 0),
  }));

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Hypotheses</h2>
          <p className="text-sm text-[var(--text-muted)]">{hypotheses.length} hypotheses</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => generateHypotheses.mutate()}
            disabled={generateHypotheses.isPending}
            className="flex items-center gap-2 px-3 py-2 bg-[var(--accent-purple)] hover:bg-[var(--accent-purple)]/80 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            <Sparkles size={14} />
            {generateHypotheses.isPending ? 'Generating...' : 'Generate'}
          </button>
          <button
            onClick={() => evaluateHypotheses.mutate()}
            disabled={evaluateHypotheses.isPending}
            className="flex items-center gap-2 px-3 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            <RefreshCw size={14} className={evaluateHypotheses.isPending ? 'animate-spin' : ''} />
            {evaluateHypotheses.isPending ? 'Evaluating...' : 'Evaluate All'}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Hypothesis list */}
        <div className="xl:col-span-2 space-y-3">
          {hypothesesQuery.isLoading ? (
            <LoadingSpinner text="Loading hypotheses..." />
          ) : sorted.length === 0 ? (
            <Card>
              <p className="text-sm text-[var(--text-muted)] text-center py-8">
                No hypotheses yet. Add evidence then click "Generate" to create hypotheses.
              </p>
            </Card>
          ) : (
            sorted.map((h) => {
              const id = h.id || h.hypothesis_id || '';
              const score = (h.score ?? 0) <= 1 ? (h.score ?? 0) * 100 : (h.score ?? 0);
              const isSelected = selectedHypId === id;

              return (
                <div
                  key={id}
                  onClick={() => setSelectedHypId(isSelected ? null : id)}
                  className={`bg-[var(--bg-card)] border rounded-lg p-4 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-[var(--accent)] ring-1 ring-[var(--accent)]/30'
                      : 'border-[var(--border)] hover:border-[var(--border)]/80'
                  }`}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="min-w-0 flex-1 mr-3">
                      <h4 className="text-sm font-semibold text-[var(--text-primary)] mb-1">
                        {h.title || h.description || 'Unnamed Hypothesis'}
                      </h4>
                      {h.description && h.title && (
                        <p className="text-xs text-[var(--text-secondary)] line-clamp-2">{h.description}</p>
                      )}
                    </div>
                    {h.status && <Badge type={h.status} />}
                  </div>
                  <ScoreBar label="" score={score} />
                  <div className="flex gap-4 mt-2 text-xs text-[var(--text-muted)]">
                    {h.supporting_evidence !== undefined && (
                      <span className="text-[var(--accent-green)]">
                        +{h.supporting_evidence} supporting
                      </span>
                    )}
                    {h.contradicting_evidence !== undefined && (
                      <span className="text-[var(--accent-red)]">
                        -{h.contradicting_evidence} contradicting
                      </span>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Evolution chart */}
        <Card title="Score Evolution">
          {!selectedHypId ? (
            <p className="text-sm text-[var(--text-muted)] text-center py-8">
              Click a hypothesis to see its score evolution
            </p>
          ) : evolutionQuery.isLoading ? (
            <LoadingSpinner size={24} />
          ) : chartData.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)] text-center py-8">
              No evolution data yet
            </p>
          ) : (
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="time" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
                <YAxis domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
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
                  dataKey="score"
                  stroke="var(--accent)"
                  strokeWidth={2}
                  dot={{ r: 4, fill: 'var(--accent)' }}
                />
              </LineChart>
            </ResponsiveContainer>
          )}
        </Card>
      </div>
    </div>
  );
}
