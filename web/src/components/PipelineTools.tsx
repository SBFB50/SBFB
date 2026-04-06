import { useState, useEffect } from 'react';
import { api } from '../api/client';

interface ToolStatus {
  status: 'idle' | 'running' | 'done' | 'error';
  detail: string;
  file: string;
  updated_at: string;
  cycle: number;
}

interface PipelineStatus {
  running: boolean;
  cycle_count: number;
  last_action: string;
  started_at: string;
  tools: Record<string, ToolStatus>;
}

const TOOLS_CONFIG = [
  { key: 'monitoring', name: 'Monitoring', phase: 'OBSERVE', icon: '🔍', file: 'monitoring/scheduler.py' },
  { key: 'evidence_processor', name: 'Evidence Processor', phase: 'ORIENT', icon: '📄', file: 'core/evidence_processor.py' },
  { key: 'neo4j_sync', name: 'Neo4j Sync', phase: 'ORIENT', icon: '🕸️', file: 'db/neo4j_db.py' },
  { key: 'osint_recon', name: 'OSINT Recon', phase: 'ORIENT', icon: '🌐', file: 'recon/' },
  { key: 'geo_mapper', name: 'GeoMapper', phase: 'ORIENT', icon: '🗺️', file: 'core/geo_mapper.py' },
  { key: 'image_analyzer', name: 'Image Analyzer', phase: 'ORIENT', icon: '👁️', file: 'core/image_analyzer.py' },
  { key: 'visual_embedder', name: 'Visual Embedder', phase: 'ORIENT', icon: '🖼️', file: 'vision/embeddings.py' },
  { key: 'analysis_pipeline', name: 'Analysis Pipeline', phase: 'DECIDE', icon: '🧠', file: 'core/analysis_pipeline.py' },
  { key: 'hypothesis_engine', name: 'Hypothesis Engine', phase: 'DECIDE', icon: '💡', file: 'core/hypothesis_engine.py' },
  { key: 'contradiction_detector', name: 'Contradiction Detector', phase: 'DECIDE', icon: '⚡', file: 'core/contradiction_detector.py' },
  { key: 'suspect_scorer', name: 'Suspect Scorer', phase: 'DECIDE', icon: '🎯', file: 'core/suspect_scorer.py' },
  { key: 'forensics', name: 'Forensics (BPA/Trace/Audio)', phase: 'DECIDE', icon: '🔬', file: 'forensics/' },
  { key: 'timeline_builder', name: 'Timeline Builder', phase: 'DECIDE', icon: '📅', file: 'core/timeline_builder.py' },
];

const PHASE_COLORS: Record<string, string> = {
  OBSERVE: '#3b82f6',
  ORIENT: '#22c55e',
  DECIDE: '#a855f7',
  ACT: '#eab308',
  QUESTION: '#ef4444',
};

const STATUS_STYLES: Record<string, { bg: string; ring: string; pulse: boolean }> = {
  idle: { bg: 'bg-zinc-800', ring: 'ring-zinc-700', pulse: false },
  running: { bg: 'bg-blue-900/40', ring: 'ring-blue-500', pulse: true },
  done: { bg: 'bg-green-900/30', ring: 'ring-green-600', pulse: false },
  error: { bg: 'bg-red-900/30', ring: 'ring-red-500', pulse: false },
};

export default function PipelineTools({ caseId }: { caseId: string }) {
  const [pipeline, setPipeline] = useState<PipelineStatus | null>(null);

  useEffect(() => {
    if (!caseId) return;
    const fetch = async () => {
      try {
        const resp = await api.get(`/cases/${caseId}/investigation/status`);
        setPipeline(resp.data);
      } catch {
        setPipeline(null);
      }
    };
    fetch();
    const interval = setInterval(fetch, 2000);
    return () => clearInterval(interval);
  }, [caseId]);

  if (!pipeline) return null;

  const tools = pipeline.tools || {};
  const phases = ['OBSERVE', 'ORIENT', 'DECIDE'];

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-semibold text-[var(--text-primary)]">Pipeline autonome</h3>
          {pipeline.running && (
            <span className="flex items-center gap-1.5 text-[10px] text-green-400 font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
              Cycle {pipeline.cycle_count} — {pipeline.last_action}
            </span>
          )}
          {!pipeline.running && pipeline.cycle_count > 0 && (
            <span className="text-[10px] text-[var(--text-muted)]">
              Arrete apres {pipeline.cycle_count} cycles
            </span>
          )}
        </div>
        {pipeline.started_at && (
          <span className="text-[10px] text-[var(--text-muted)] font-mono">
            Demarre {pipeline.started_at?.slice(11, 19)}
          </span>
        )}
      </div>

      {phases.map(phase => {
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
                const ts = tools[tool.key];
                const status = ts?.status || 'idle';
                const style = STATUS_STYLES[status];
                return (
                  <div
                    key={tool.key}
                    className={`${style.bg} ring-1 ${style.ring} rounded-lg p-2.5 transition-all ${
                      style.pulse ? 'animate-pulse' : ''
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-sm">{tool.icon}</span>
                      <span className="text-[11px] font-medium text-[var(--text-primary)] truncate">
                        {tool.name}
                      </span>
                    </div>
                    <p className="text-[9px] text-[var(--text-muted)] font-mono truncate">
                      {tool.file}
                    </p>
                    {ts?.detail && (
                      <p className="text-[10px] text-[var(--text-secondary)] mt-1 truncate">
                        {ts.detail}
                      </p>
                    )}
                    {ts?.updated_at && (
                      <p className="text-[9px] text-[var(--text-muted)] mt-0.5 font-mono">
                        {ts.updated_at.slice(11, 19)}
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
