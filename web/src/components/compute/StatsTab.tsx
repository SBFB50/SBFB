import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Server, Zap, Clock, CheckCircle, XCircle, Loader2 } from 'lucide-react';
import { TestPanel } from './TestPanel';

interface Props {
  stats: Record<string, any>;
  model: Record<string, any>;
  hybrid: Record<string, any>;
  nodes: any[];
}

export function StatsTab({ stats, model, hybrid, nodes }: Props) {
  const pending = stats.tasks_pending || 0;
  const assigned = stats.tasks_assigned || 0;
  const completed = stats.tasks_completed || 0;
  const failed = stats.tasks_failed || 0;
  const total = pending + assigned + completed + failed;

  return (
    <div className="space-y-4">
      {/* Test panel */}
      <TestPanel />

      {/* Task distribution */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-3">
        <Card>
          <CardHeader className="border-b border-[var(--border)] py-2 px-4">
            <CardTitle className="text-sm">File de taches</CardTitle>
          </CardHeader>
          <CardContent className="p-4 space-y-3">
            <TaskBar label="En attente" count={pending} total={total} color="#f59e0b" icon={<Clock size={14} />} />
            <TaskBar label="En cours" count={assigned} total={total} color="#3b82f6" icon={<Loader2 size={14} className="animate-spin" />} />
            <TaskBar label="Terminees" count={completed} total={total} color="#22c55e" icon={<CheckCircle size={14} />} />
            <TaskBar label="Echouees" count={failed} total={total} color="#ef4444" icon={<XCircle size={14} />} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="border-b border-[var(--border)] py-2 px-4">
            <CardTitle className="text-sm">Modele actif</CardTitle>
          </CardHeader>
          <CardContent className="p-4 space-y-2">
            <InfoRow label="Modele" value={model.target_model || stats.current_model || '-'} />
            <InfoRow label="Tier" value={model.target_tier || stats.model_tier || '-'} />
            <InfoRow label="Mode" value={model.execution_mode || 'local'} />
            <InfoRow label="Transition" value={model.transition_state || 'stable'} />
            <InfoRow label="VRAM totale" value={`${(stats.vram_total_gb || 0).toFixed(0)} GB`} />
            <InfoRow label="Max node" value={`${(model.max_single_node_vram_gb || 0).toFixed(0)} GB`} />
            {model.readiness_pct !== undefined && model.readiness_pct < 100 && (
              <div className="mt-2">
                <div className="flex justify-between text-[10px] text-[var(--text-muted)] mb-1">
                  <span>Readiness</span>
                  <span>{model.readiness_pct?.toFixed(0)}%</span>
                </div>
                <div className="h-1.5 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                  <div
                    className="h-full bg-emerald-500 rounded-full transition-all"
                    style={{ width: `${model.readiness_pct || 0}%` }}
                  />
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* GPU distribution */}
      <Card>
        <CardHeader className="border-b border-[var(--border)] py-2 px-4">
          <CardTitle className="text-sm">Distribution GPU ({nodes.length} nodes)</CardTitle>
        </CardHeader>
        <CardContent className="p-4">
          {nodes.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)] text-center py-4">
              Aucun contributeur connecte. Soyez le premier !
            </p>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-2">
              {nodes.map((n: any) => (
                <div
                  key={n.id}
                  className="bg-[var(--bg-primary)] border border-[var(--border)] rounded px-3 py-2"
                >
                  <div className="flex items-center gap-1.5 mb-1">
                    <div className={`w-1.5 h-1.5 rounded-full ${
                      n.status === 'idle' ? 'bg-emerald-500' :
                      n.status === 'busy' ? 'bg-blue-500 animate-pulse' :
                      'bg-gray-500'
                    }`} />
                    <span className="text-xs font-medium text-[var(--text-primary)] truncate">{n.name}</span>
                  </div>
                  <p className="text-[10px] text-[var(--text-muted)]">
                    {n.gpu_model} ({(n.vram_mb / 1024).toFixed(0)} GB)
                  </p>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function TaskBar({ label, count, total, color, icon }: {
  label: string; count: number; total: number; color: string; icon: React.ReactNode;
}) {
  const pct = total > 0 ? (count / total) * 100 : 0;
  return (
    <div>
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-1.5" style={{ color }}>
          {icon}
          <span className="text-xs">{label}</span>
        </div>
        <span className="text-xs font-mono" style={{ color }}>{count.toLocaleString()}</span>
      </div>
      <div className="h-1 bg-[var(--bg-primary)] rounded-full overflow-hidden">
        <div className="h-full rounded-full transition-all" style={{ width: `${pct}%`, backgroundColor: color }} />
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between text-xs">
      <span className="text-[var(--text-muted)]">{label}</span>
      <span className="text-[var(--text-primary)] font-mono">{value}</span>
    </div>
  );
}
