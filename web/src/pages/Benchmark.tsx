import { useState } from 'react';
import { Gauge, Play, RefreshCw } from 'lucide-react';
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
} from 'recharts';
import Card from '../components/Card';
import LoadingSpinner from '../components/LoadingSpinner';
import { useBenchmarkResults } from '../hooks/useApi';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { runBenchmark } from '../api/client';

interface BenchmarkResult {
  model?: string;
  model_name?: string;
  tokens_per_second?: number;
  latency_ms?: number;
  accuracy?: number;
  reasoning?: number;
  extraction?: number;
  coherence?: number;
  overall_score?: number;
  timestamp?: string;
}

export default function Benchmark() {
  const resultsQuery = useBenchmarkResults();
  const queryClient = useQueryClient();
  const [running, setRunning] = useState(false);

  const runMutation = useMutation({
    mutationFn: () => runBenchmark({}),
    onMutate: () => setRunning(true),
    onSettled: () => {
      setRunning(false);
      queryClient.invalidateQueries({ queryKey: ['benchmarkResults'] });
    },
  });

  const results: BenchmarkResult[] = Array.isArray(resultsQuery.data)
    ? resultsQuery.data
    : (resultsQuery.data?.results || []);

  // Prepare chart data
  const speedData = results.map(r => ({
    name: r.model || r.model_name || 'Unknown',
    'Tokens/s': r.tokens_per_second || 0,
    'Latency (ms)': r.latency_ms || 0,
  }));

  const radarData = results.length > 0 ? [
    { metric: 'Accuracy', ...Object.fromEntries(results.map(r => [r.model || r.model_name, (r.accuracy ?? 0) * 100])) },
    { metric: 'Reasoning', ...Object.fromEntries(results.map(r => [r.model || r.model_name, (r.reasoning ?? 0) * 100])) },
    { metric: 'Extraction', ...Object.fromEntries(results.map(r => [r.model || r.model_name, (r.extraction ?? 0) * 100])) },
    { metric: 'Coherence', ...Object.fromEntries(results.map(r => [r.model || r.model_name, (r.coherence ?? 0) * 100])) },
  ] : [];

  const modelColors = ['#3b82f6', '#22c55e', '#a855f7', '#eab308', '#ef4444', '#06b6d4'];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Model Benchmark</h2>
          <p className="text-sm text-[var(--text-muted)]">Compare LLM performance across tasks</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => resultsQuery.refetch()}
            className="flex items-center gap-2 px-3 py-2 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-sm hover:bg-[var(--bg-hover)] transition-colors"
          >
            <RefreshCw size={14} /> Refresh
          </button>
          <button
            onClick={() => runMutation.mutate()}
            disabled={running}
            className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            <Play size={14} />
            {running ? 'Running...' : 'Run Benchmark'}
          </button>
        </div>
      </div>

      {running && (
        <Card className="border-[var(--accent)]/30">
          <div className="flex items-center gap-3">
            <LoadingSpinner size={20} />
            <p className="text-sm text-[var(--text-secondary)]">
              Benchmark in progress... This may take several minutes.
            </p>
          </div>
        </Card>
      )}

      {resultsQuery.isLoading ? (
        <LoadingSpinner text="Loading results..." />
      ) : results.length === 0 ? (
        <Card>
          <div className="flex flex-col items-center py-12">
            <Gauge size={48} className="text-[var(--text-muted)] mb-4" />
            <p className="text-sm text-[var(--text-muted)]">
              No benchmark results yet. Click "Run Benchmark" to test model performance.
            </p>
          </div>
        </Card>
      ) : (
        <>
          {/* Results table */}
          <Card title="Results">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--border)]">
                    <th className="py-2 px-3 text-left text-xs font-semibold text-[var(--text-muted)] uppercase">Model</th>
                    <th className="py-2 px-3 text-right text-xs font-semibold text-[var(--text-muted)] uppercase">Tokens/s</th>
                    <th className="py-2 px-3 text-right text-xs font-semibold text-[var(--text-muted)] uppercase">Latency</th>
                    <th className="py-2 px-3 text-right text-xs font-semibold text-[var(--text-muted)] uppercase">Accuracy</th>
                    <th className="py-2 px-3 text-right text-xs font-semibold text-[var(--text-muted)] uppercase">Overall</th>
                  </tr>
                </thead>
                <tbody>
                  {results.map((r, i) => (
                    <tr key={i} className="border-b border-[var(--border)]/50">
                      <td className="py-2.5 px-3">
                        <div className="flex items-center gap-2">
                          <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: modelColors[i % modelColors.length] }} />
                          <span className="text-[var(--text-primary)] font-medium">{r.model || r.model_name}</span>
                        </div>
                      </td>
                      <td className="py-2.5 px-3 text-right font-mono text-[var(--text-secondary)]">
                        {r.tokens_per_second?.toFixed(1) || '-'}
                      </td>
                      <td className="py-2.5 px-3 text-right font-mono text-[var(--text-secondary)]">
                        {r.latency_ms ? `${r.latency_ms.toFixed(0)}ms` : '-'}
                      </td>
                      <td className="py-2.5 px-3 text-right font-mono text-[var(--text-secondary)]">
                        {r.accuracy !== undefined ? `${(r.accuracy * 100).toFixed(1)}%` : '-'}
                      </td>
                      <td className="py-2.5 px-3 text-right font-mono font-bold text-[var(--text-primary)]">
                        {r.overall_score !== undefined ? `${(r.overall_score * 100).toFixed(1)}%` : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>

          <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
            {/* Speed comparison */}
            <Card title="Speed Comparison">
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={speedData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis dataKey="name" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
                  <YAxis tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'var(--bg-card)',
                      border: '1px solid var(--border)',
                      borderRadius: '8px',
                      color: 'var(--text-primary)',
                    }}
                  />
                  <Bar dataKey="Tokens/s" fill="#3b82f6" radius={[4, 4, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </Card>

            {/* Radar chart */}
            {radarData.length > 0 && (
              <Card title="Capability Comparison">
                <ResponsiveContainer width="100%" height={300}>
                  <RadarChart data={radarData}>
                    <PolarGrid stroke="var(--border)" />
                    <PolarAngleAxis dataKey="metric" tick={{ fill: 'var(--text-muted)', fontSize: 11 }} />
                    <PolarRadiusAxis angle={30} domain={[0, 100]} tick={{ fill: 'var(--text-muted)', fontSize: 10 }} />
                    {results.map((r, i) => (
                      <Radar
                        key={i}
                        name={r.model || r.model_name || ''}
                        dataKey={r.model || r.model_name || ''}
                        stroke={modelColors[i % modelColors.length]}
                        fill={modelColors[i % modelColors.length]}
                        fillOpacity={0.15}
                      />
                    ))}
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--bg-card)',
                        border: '1px solid var(--border)',
                        borderRadius: '8px',
                        color: 'var(--text-primary)',
                      }}
                    />
                  </RadarChart>
                </ResponsiveContainer>
              </Card>
            )}
          </div>
        </>
      )}
    </div>
  );
}
