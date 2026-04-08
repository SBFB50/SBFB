import { useState, useEffect } from 'react';
import { api } from '../api/client';

interface WorkerStatus {
  status: 'idle' | 'processing' | 'done' | 'error';
  events_processed: number;
  queue_size: number;
  last_event: string;
  detail: string;
}

interface BusStats {
  total_published: number;
  total_queues: number;
  total_pending: number;
}

interface PipelineStatus {
  running: boolean;
  mode: string;
  started_at: string;
  total_events: number;
  bus_stats: BusStats;
  workers: Record<string, WorkerStatus>;
  // Legacy fields (backward compat)
  cycle_count?: number;
  last_action?: string;
  tools?: Record<string, unknown>;
}

const TOOLS_CONFIG = [
  // INGEST
  { key: 'evidence_ingest', name: 'Evidence Ingest', phase: 'INGEST', icon: '\u{1F4E5}' },
  { key: 'entity_extractor', name: 'Entity Extractor', phase: 'INGEST', icon: '\u{1F464}' },
  { key: 'summarizer', name: 'Summarizer', phase: 'INGEST', icon: '\u{1F4DD}' },
  { key: 'chunker_embed', name: 'Chunker + RAG', phase: 'INGEST', icon: '\u{1F9E9}' },
  // ENRICH
  { key: 'neo4j_sync', name: 'Neo4j Graph', phase: 'ENRICH', icon: '\u{1F578}\uFE0F' },
  { key: 'osint_recon', name: 'OSINT Recon', phase: 'ENRICH', icon: '\u{1F310}' },
  { key: 'geo_mapper', name: 'GeoMapper', phase: 'ENRICH', icon: '\u{1F5FA}\uFE0F' },
  { key: 'forensics', name: 'Forensics', phase: 'ENRICH', icon: '\u{1F52C}' },
  // ANALYZE
  { key: 'analysis', name: 'Deep Analysis', phase: 'ANALYZE', icon: '\u{1F9E0}' },
  { key: 'hypothesis', name: 'Hypothesis Engine', phase: 'ANALYZE', icon: '\u{1F4A1}' },
  { key: 'contradiction', name: 'Contradictions', phase: 'ANALYZE', icon: '\u26A1' },
  // SCORE
  { key: 'suspect_scorer', name: 'Suspect Scorer', phase: 'SCORE', icon: '\u{1F3AF}' },
  { key: 'query_generator', name: 'Query Generator', phase: 'SCORE', icon: '\u{1F50D}' },
];

const PHASES = ['INGEST', 'ENRICH', 'ANALYZE', 'SCORE'] as const;

const PHASE_COLORS: Record<string, string> = {
  INGEST: '#3b82f6',
  ENRICH: '#22c55e',
  ANALYZE: '#a855f7',
  SCORE: '#eab308',
};

const STATUS_STYLES: Record<string, { bg: string; ring: string; pulse: boolean; dot: string }> = {
  idle: { bg: 'bg-zinc-800', ring: 'ring-zinc-700', pulse: false, dot: 'bg-zinc-500' },
  processing: { bg: 'bg-blue-900/40', ring: 'ring-blue-500', pulse: true, dot: 'bg-blue-400' },
  done: { bg: 'bg-green-900/30', ring: 'ring-green-600', pulse: false, dot: 'bg-green-400' },
  error: { bg: 'bg-red-900/30', ring: 'ring-red-500', pulse: false, dot: 'bg-red-400' },
};

export default function PipelineTools({ caseId }: { caseId: string }) {
  const [pipeline, setPipeline] = useState<PipelineStatus | null>(null);

  useEffect(() => {
    if (!caseId) return;
    const fetchStatus = async () => {
      try {
        const resp = await api.get(`/cases/${caseId}/investigation/status`);
        setPipeline(resp.data);
      } catch {
        setPipeline(null);
      }
    };
    fetchStatus();
    // SSE events invalidate React Query caches which trigger re-renders.
    // This interval is a slow fallback for resilience (was 2s, now 30s).
    const interval = setInterval(fetchStatus, 30000);
    return () => clearInterval(interval);
  }, [caseId]);

  if (!pipeline) return null;

  const workers = pipeline.workers || {};
  const busStats = pipeline.bus_stats;
  const totalEvents = pipeline.total_events || busStats?.total_published || 0;

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-semibold text-[var(--text-primary)]">Pipeline Event-Driven</h3>
          {pipeline.running && (
            <span className="flex items-center gap-1.5 text-[10px] text-green-400 font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
              Active
            </span>
          )}
          {!pipeline.running && totalEvents > 0 && (
            <span className="text-[10px] text-[var(--text-muted)]">
              Arrete
            </span>
          )}
        </div>
        {pipeline.started_at && (
          <span className="text-[10px] text-[var(--text-muted)] font-mono">
            Demarre {pipeline.started_at?.slice(11, 19)}
          </span>
        )}
      </div>

      {/* EventBus stats bar */}
      {busStats && (
        <div className="flex items-center gap-4 mb-4 px-3 py-2 bg-[var(--bg-primary)] rounded-lg border border-[var(--border)]">
          <div className="flex items-center gap-1.5">
            <span className="text-[10px] text-[var(--text-muted)]">Events publies:</span>
            <span className="text-xs font-mono font-bold text-[var(--text-primary)]">
              {busStats.total_published}
            </span>
          </div>
          <div className="w-px h-3 bg-[var(--border)]" />
          <div className="flex items-center gap-1.5">
            <span className="text-[10px] text-[var(--text-muted)]">Files actives:</span>
            <span className="text-xs font-mono font-bold text-[var(--text-primary)]">
              {busStats.total_queues}
            </span>
          </div>
          <div className="w-px h-3 bg-[var(--border)]" />
          <div className="flex items-center gap-1.5">
            <span className="text-[10px] text-[var(--text-muted)]">En attente:</span>
            <span className={`text-xs font-mono font-bold ${busStats.total_pending > 0 ? 'text-yellow-400' : 'text-[var(--text-primary)]'}`}>
              {busStats.total_pending}
            </span>
          </div>
        </div>
      )}

      {/* Workers grouped by phase */}
      {PHASES.map(phase => {
        const phaseTools = TOOLS_CONFIG.filter(t => t.phase === phase);
        return (
          <div key={phase} className="mb-3">
            <div className="flex items-center gap-2 mb-2">
              <span
                className="w-2 h-2 rounded-full"
                style={{ backgroundColor: PHASE_COLORS[phase] }}
              />
              <span className="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
                {phase}
              </span>
            </div>
            <div className="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
              {phaseTools.map(tool => {
                const ws = workers[tool.key];
                const status = ws?.status || 'idle';
                const style = STATUS_STYLES[status] || STATUS_STYLES.idle;
                const eventsProcessed = ws?.events_processed || 0;
                const queueSize = ws?.queue_size || 0;

                return (
                  <div
                    key={tool.key}
                    className={`${style.bg} ring-1 ${style.ring} rounded-lg p-2.5 transition-all ${
                      style.pulse ? 'animate-pulse' : ''
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-1.5">
                      <span className="text-sm">{tool.icon}</span>
                      <span className="text-[11px] font-medium text-[var(--text-primary)] truncate flex-1">
                        {tool.name}
                      </span>
                      <span className={`w-1.5 h-1.5 rounded-full ${style.dot}`} />
                    </div>

                    {/* Status badge */}
                    <div className="flex items-center gap-1.5 mb-1">
                      <span className={`text-[9px] font-medium uppercase tracking-wide ${
                        status === 'idle' ? 'text-zinc-500' :
                        status === 'processing' ? 'text-blue-400' :
                        status === 'done' ? 'text-green-400' :
                        'text-red-400'
                      }`}>
                        {status}
                      </span>
                      {status === 'error' && ws?.detail && (
                        <span className="text-[8px] text-red-400/70 truncate" title={ws.detail}>
                          {ws.detail.slice(0, 20)}
                        </span>
                      )}
                    </div>

                    {/* Metrics row */}
                    <div className="flex items-center gap-3 text-[9px] text-[var(--text-muted)] font-mono">
                      <span title="Events traites">
                        {eventsProcessed} evt
                      </span>
                      {queueSize > 0 && (
                        <span className="text-yellow-400" title="Events en attente">
                          +{queueSize} pending
                        </span>
                      )}
                    </div>

                    {/* Last event timestamp */}
                    {ws?.last_event && (
                      <p className="text-[8px] text-[var(--text-muted)] mt-1 font-mono">
                        {ws.last_event.slice(11, 19)}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
