import { FileText, Users, Lightbulb, Bell, Activity, AlertTriangle } from 'lucide-react';
import MetricCard from '../components/MetricCard';
import Card from '../components/Card';
import ScoreBar from '../components/ScoreBar';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useActiveCase } from '../hooks/useCase';
import { useHypotheses, useAlerts, useAuditLog, useInvestigationStatus } from '../hooks/useApi';

export default function Dashboard() {
  const { caseId } = useCaseStore();
  const { stats } = useActiveCase();
  const hypothesesQuery = useHypotheses();
  const alertsQuery = useAlerts();
  const auditQuery = useAuditLog(20);
  const investigationQuery = useInvestigationStatus();

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Activity size={48} className="text-[var(--text-muted)] mb-4" />
        <h2 className="text-xl font-semibold text-[var(--text-primary)] mb-2">Welcome to NEXUS</h2>
        <p className="text-[var(--text-secondary)] max-w-md">
          Select an existing case or create a new one from the sidebar to begin your investigation.
        </p>
      </div>
    );
  }

  const evidenceCount = stats?.evidence_count ?? stats?.total_evidence ?? 0;
  const entityCount = stats?.entity_count ?? stats?.total_entities ?? 0;
  const hypothesisCount = stats?.hypothesis_count ?? stats?.total_hypotheses ?? 0;
  const alertCount = stats?.alert_count ?? stats?.unread_alerts ?? 0;

  const hypotheses = Array.isArray(hypothesesQuery.data) ? hypothesesQuery.data : [];
  const topHypotheses = hypotheses
    .sort((a: { score?: number }, b: { score?: number }) => (b.score ?? 0) - (a.score ?? 0))
    .slice(0, 5);

  const alerts = Array.isArray(alertsQuery.data) ? alertsQuery.data : [];
  const recentAlerts = alerts.slice(0, 5);

  const auditItems = Array.isArray(auditQuery.data) ? auditQuery.data : [];
  const recentAudit = auditItems.slice(0, 8);

  const invStatus = investigationQuery.data;

  return (
    <div className="space-y-6">
      {/* Metric cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
        <MetricCard label="Evidence" value={evidenceCount} icon={FileText} color="var(--accent)" />
        <MetricCard label="Entities" value={entityCount} icon={Users} color="var(--accent-purple)" />
        <MetricCard label="Hypotheses" value={hypothesisCount} icon={Lightbulb} color="var(--accent-yellow)" />
        <MetricCard label="Alerts" value={alertCount} icon={Bell} color="var(--accent-red)"
          subtitle={alertCount > 0 ? 'unread' : 'all clear'} />
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Top Hypotheses */}
        <Card title="Top Hypotheses" className="xl:col-span-2">
          {hypothesesQuery.isLoading ? (
            <LoadingSpinner size={24} />
          ) : topHypotheses.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">No hypotheses generated yet. Run an analysis to generate them.</p>
          ) : (
            <div className="space-y-4">
              {topHypotheses.map((h: { id?: string; hypothesis_id?: string; title?: string; description?: string; score?: number }) => (
                <ScoreBar
                  key={h.id || h.hypothesis_id}
                  label={h.title || h.description || 'Unnamed'}
                  score={(h.score ?? 0) * (h.score && h.score <= 1 ? 100 : 1)}
                />
              ))}
            </div>
          )}
        </Card>

        {/* Investigation Status */}
        <Card title="Investigation">
          {invStatus ? (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <Badge type={invStatus.status || invStatus.state || 'idle'} />
                <span className="text-sm text-[var(--text-secondary)]">
                  {invStatus.status || invStatus.state || 'idle'}
                </span>
              </div>
              {invStatus.current_task && (
                <p className="text-xs text-[var(--text-muted)]">
                  Current: {invStatus.current_task}
                </p>
              )}
              {invStatus.progress !== undefined && (
                <div className="w-full h-2 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                  <div
                    className="h-full bg-[var(--accent)] rounded-full transition-all"
                    style={{ width: `${invStatus.progress}%` }}
                  />
                </div>
              )}
            </div>
          ) : (
            <p className="text-sm text-[var(--text-muted)]">No active investigation</p>
          )}
        </Card>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* Recent Alerts */}
        <Card title="Recent Alerts">
          {recentAlerts.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">No alerts</p>
          ) : (
            <div className="space-y-2">
              {recentAlerts.map((a: { id?: string; alert_id?: string; severity?: string; message?: string; title?: string; created_at?: string }, i: number) => (
                <div key={a.id || a.alert_id || i} className="flex items-start gap-2 py-2 border-b border-[var(--border)]/50 last:border-0">
                  <AlertTriangle size={14} className={`shrink-0 mt-0.5 ${
                    a.severity === 'high' || a.severity === 'critical' ? 'text-[var(--accent-red)]' : 'text-[var(--accent-yellow)]'
                  }`} />
                  <div className="min-w-0">
                    <p className="text-sm text-[var(--text-primary)] truncate">{a.message || a.title}</p>
                    {a.created_at && (
                      <p className="text-[10px] text-[var(--text-muted)]">
                        {new Date(a.created_at).toLocaleString()}
                      </p>
                    )}
                  </div>
                  {a.severity && <Badge type={a.severity} className="shrink-0" />}
                </div>
              ))}
            </div>
          )}
        </Card>

        {/* Audit Log */}
        <Card title="Audit Log">
          {recentAudit.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">No activity yet</p>
          ) : (
            <div className="space-y-2">
              {recentAudit.map((a: { id?: string; action?: string; details?: string; timestamp?: string; created_at?: string }, i: number) => (
                <div key={a.id || i} className="flex items-start gap-2 py-1.5 border-b border-[var(--border)]/50 last:border-0">
                  <div className="w-1.5 h-1.5 rounded-full bg-[var(--accent)] mt-1.5 shrink-0" />
                  <div className="min-w-0">
                    <p className="text-sm text-[var(--text-secondary)]">{a.action || a.details}</p>
                    {(a.timestamp || a.created_at) && (
                      <p className="text-[10px] text-[var(--text-muted)]">
                        {new Date(a.timestamp || a.created_at!).toLocaleString()}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
