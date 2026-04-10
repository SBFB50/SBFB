import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Cpu, Clock, Zap, Target, Trophy, Shield, BarChart3, Heart } from 'lucide-react';

interface Props {
  impact: Record<string, any> | null;
  loading: boolean;
}

export function ContributeTab({ impact, loading }: Props) {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-12 text-[var(--text-muted)]">
        Chargement de votre contribution...
      </div>
    );
  }

  if (!impact || !impact.node_id) {
    return (
      <Card>
        <CardContent className="p-8 text-center">
          <Heart size={32} className="mx-auto mb-3 text-emerald-400 opacity-50" />
          <h3 className="text-sm font-medium text-[var(--text-primary)] mb-1">
            Votre contribution
          </h3>
          <p className="text-xs text-[var(--text-muted)] mb-4">
            Connectez votre worker pour voir votre impact personnel sur la transparence democratique.
          </p>
          <div className="bg-[var(--bg-primary)] rounded-lg p-3 text-left">
            <p className="text-[11px] text-[var(--text-secondary)] font-mono">
              pip install nexus-worker<br />
              nexus-worker register --server nexusgov.fr --name "Pseudo"<br />
              nexus-worker start
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const uptime = impact.uptime || {};
  const uptimeHours = Math.floor((uptime.total_seconds || 0) / 3600);
  const currentHours = Math.floor((uptime.current_session_seconds || 0) / 3600);
  const currentMins = Math.floor(((uptime.current_session_seconds || 0) % 3600) / 60);
  const tasksByType = impact.tasks_by_type || [];

  return (
    <div className="space-y-4">
      {/* Header card */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            <div className="p-3 rounded-xl bg-emerald-500/10">
              <Heart size={24} className="text-emerald-400" />
            </div>
            <div className="flex-1">
              <h3 className="text-base font-semibold text-[var(--text-primary)]">
                {impact.name}
              </h3>
              <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                <span className="flex items-center gap-1">
                  <Cpu size={12} /> {impact.gpu_model} ({(impact.vram_mb / 1024).toFixed(0)} GB)
                </span>
                <span className="flex items-center gap-1">
                  <Shield size={12} /> Trust: {impact.trust_score}
                </span>
              </div>
            </div>
            <div className="text-right">
              <p className="text-2xl font-bold text-emerald-400">{impact.tasks_completed?.toLocaleString()}</p>
              <p className="text-[10px] text-[var(--text-muted)] uppercase">taches completees</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Stats grid */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-3">
        <StatCard
          icon={Trophy}
          label="Classement"
          value={`Top ${100 - impact.percentile}%`}
          color="#fbbf24"
        />
        <StatCard
          icon={Zap}
          label="Tokens cette semaine"
          value={impact.tokens_this_week?.toLocaleString() || '0'}
          color="#06b6d4"
        />
        <StatCard
          icon={Clock}
          label="Uptime total"
          value={`${uptimeHours}h`}
          color="#a855f7"
        />
        <StatCard
          icon={BarChart3}
          label="Session en cours"
          value={currentHours > 0 ? `${currentHours}h ${currentMins}m` : `${currentMins}m`}
          color="#22c55e"
        />
      </div>

      {/* Task breakdown */}
      {tasksByType.length > 0 && (
        <Card>
          <CardHeader className="border-b border-[var(--border)] py-2 px-4">
            <CardTitle className="text-sm flex items-center gap-2">
              <Target size={16} className="text-blue-400" />
              Impact par type de tache
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4">
            <div className="space-y-2">
              {tasksByType.map((t: any) => {
                const total = impact.tasks_completed || 1;
                const pct = (t.count / total) * 100;
                return (
                  <div key={t.task_type}>
                    <div className="flex justify-between text-xs mb-0.5">
                      <span className="text-[var(--text-secondary)]">{t.task_type}</span>
                      <span className="text-[var(--text-muted)] font-mono">{t.count}</span>
                    </div>
                    <div className="h-1 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500 rounded-full"
                        style={{ width: `${Math.min(pct, 100)}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Impact message */}
      <div className="bg-emerald-500/5 border border-emerald-500/20 rounded-lg px-4 py-3 text-center">
        <p className="text-xs text-emerald-400 font-medium">
          Votre GPU a contribue a rendre la democratie plus transparente
        </p>
        <p className="text-[10px] text-[var(--text-muted)] mt-1">
          {impact.tasks_completed?.toLocaleString()} analyses completees — {impact.tokens_this_week?.toLocaleString()} tokens cette semaine
        </p>
      </div>
    </div>
  );
}

function StatCard({ icon: Icon, label, value, color }: {
  icon: typeof Cpu; label: string; value: string; color: string;
}) {
  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-3">
      <div className="flex items-center justify-between mb-1">
        <span className="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{label}</span>
        <Icon size={14} style={{ color }} />
      </div>
      <p className="text-lg font-bold" style={{ color }}>{value}</p>
    </div>
  );
}
