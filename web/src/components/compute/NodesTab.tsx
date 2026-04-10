import { Cpu, Clock, Zap, Shield, CheckCircle } from 'lucide-react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';

interface ComputeNode {
  id: string;
  name: string;
  gpu_model: string;
  vram_mb: number;
  status: string;
  tasks_completed: number;
  tasks_errored: number;
  avg_tokens_per_sec: number;
  trust_score: number;
  connected_at: string | null;
}

interface Props {
  nodes: ComputeNode[];
}

const STATUS_LABELS: Record<string, { label: string; color: string }> = {
  idle: { label: 'En ligne', color: '#22c55e' },
  busy: { label: 'En calcul', color: '#3b82f6' },
  offline: { label: 'Hors ligne', color: '#6b7280' },
  banned: { label: 'Banni', color: '#ef4444' },
};

export function NodesTab({ nodes }: Props) {
  const online = nodes.filter(n => n.status === 'idle' || n.status === 'busy');
  const offline = nodes.filter(n => n.status === 'offline');

  return (
    <div className="space-y-4">
      {/* Online nodes */}
      <Card>
        <CardHeader className="border-b border-[var(--border)] py-2 px-4">
          <CardTitle className="text-sm flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-emerald-500" />
            En ligne ({online.length})
          </CardTitle>
        </CardHeader>
        <CardContent className="p-3">
          {online.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)] text-center py-4">Aucun noeud en ligne</p>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-2">
              {online.map(n => <NodeCard key={n.id} node={n} />)}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Offline nodes */}
      {offline.length > 0 && (
        <Card>
          <CardHeader className="border-b border-[var(--border)] py-2 px-4">
            <CardTitle className="text-sm flex items-center gap-2 text-[var(--text-muted)]">
              <div className="w-2 h-2 rounded-full bg-gray-500" />
              Hors ligne ({offline.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="p-3">
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-2">
              {offline.map(n => <NodeCard key={n.id} node={n} />)}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function NodeCard({ node }: { node: ComputeNode }) {
  const st = STATUS_LABELS[node.status] || STATUS_LABELS.offline;
  const vramGb = (node.vram_mb / 1024).toFixed(0);

  return (
    <div className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 space-y-2">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full" style={{ backgroundColor: st.color }} />
          <span className="text-sm font-medium text-[var(--text-primary)]">{node.name}</span>
        </div>
        <span className="text-[10px] px-1.5 py-0.5 rounded" style={{
          backgroundColor: st.color + '20',
          color: st.color,
        }}>
          {st.label}
        </span>
      </div>

      {/* GPU */}
      <div className="flex items-center gap-1.5 text-xs text-[var(--text-secondary)]">
        <Cpu size={12} />
        <span>{node.gpu_model}</span>
        <span className="text-[var(--text-muted)]">({vramGb} GB)</span>
      </div>

      {/* Stats row */}
      <div className="flex items-center gap-3 text-[10px] text-[var(--text-muted)]">
        <span className="flex items-center gap-1">
          <CheckCircle size={10} className="text-emerald-400" />
          {node.tasks_completed.toLocaleString()}
        </span>
        {node.avg_tokens_per_sec > 0 && (
          <span className="flex items-center gap-1">
            <Zap size={10} className="text-yellow-400" />
            {node.avg_tokens_per_sec.toFixed(1)} t/s
          </span>
        )}
        <span className="flex items-center gap-1">
          <Shield size={10} className={node.trust_score >= 80 ? 'text-emerald-400' : ''} />
          {node.trust_score}
        </span>
      </div>
    </div>
  );
}
