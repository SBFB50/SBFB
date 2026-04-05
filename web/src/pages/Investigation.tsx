import { Play, Square, RefreshCw, Search, AlertCircle } from 'lucide-react';
import Card from '../components/Card';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import {
  useInvestigationStatus,
  useStartInvestigation,
  useStopInvestigation,
  useAlerts,
  useMonitoringJobs,
} from '../hooks/useApi';

export default function Investigation() {
  const { caseId } = useCaseStore();
  const statusQuery = useInvestigationStatus();
  const startInv = useStartInvestigation();
  const stopInv = useStopInvestigation();
  const alertsQuery = useAlerts();
  const monitoringQuery = useMonitoringJobs();

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Search size={48} className="text-[var(--text-muted)] mb-4" />
        <p className="text-[var(--text-secondary)]">Select a case to manage investigations.</p>
      </div>
    );
  }

  const status = statusQuery.data;
  const isRunning = status?.status === 'running' || status?.state === 'running';
  const alerts = Array.isArray(alertsQuery.data) ? alertsQuery.data : [];
  const jobs = Array.isArray(monitoringQuery.data) ? monitoringQuery.data : [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Investigation Center</h2>
          <p className="text-sm text-[var(--text-muted)]">Autonomous investigation management</p>
        </div>
        <div className="flex gap-2">
          {isRunning ? (
            <button
              onClick={() => stopInv.mutate()}
              disabled={stopInv.isPending}
              className="flex items-center gap-2 px-4 py-2 bg-[var(--accent-red)] hover:bg-[var(--accent-red)]/80 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
            >
              <Square size={14} />
              {stopInv.isPending ? 'Stopping...' : 'Stop'}
            </button>
          ) : (
            <button
              onClick={() => startInv.mutate()}
              disabled={startInv.isPending}
              className="flex items-center gap-2 px-4 py-2 bg-[var(--accent-green)] hover:bg-[var(--accent-green)]/80 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
            >
              <Play size={14} />
              {startInv.isPending ? 'Starting...' : 'Start Investigation'}
            </button>
          )}
        </div>
      </div>

      {/* Status Card */}
      <Card title="Status">
        {statusQuery.isLoading ? (
          <LoadingSpinner size={24} />
        ) : status ? (
          <div className="space-y-4">
            <div className="flex items-center gap-3">
              <div className={`w-3 h-3 rounded-full ${isRunning ? 'bg-[var(--accent-green)] animate-pulse' : 'bg-[var(--text-muted)]'}`} />
              <Badge type={status.status || status.state || 'idle'} />
              <span className="text-sm text-[var(--text-secondary)]">
                {status.status || status.state || 'idle'}
              </span>
            </div>

            {status.current_task && (
              <div className="px-3 py-2 bg-[var(--bg-primary)] rounded-lg">
                <p className="text-xs text-[var(--text-muted)] mb-1">Current Task</p>
                <p className="text-sm text-[var(--text-primary)]">{status.current_task}</p>
              </div>
            )}

            {status.progress !== undefined && (
              <div>
                <div className="flex justify-between text-xs text-[var(--text-muted)] mb-1">
                  <span>Progress</span>
                  <span>{Math.round(status.progress)}%</span>
                </div>
                <div className="w-full h-2 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                  <div
                    className="h-full bg-[var(--accent)] rounded-full transition-all duration-500"
                    style={{ width: `${status.progress}%` }}
                  />
                </div>
              </div>
            )}

            {status.steps_completed !== undefined && (
              <p className="text-xs text-[var(--text-muted)]">
                Steps: {status.steps_completed}/{status.total_steps || '?'}
              </p>
            )}

            {status.last_action && (
              <div className="px-3 py-2 bg-[var(--bg-primary)] rounded-lg">
                <p className="text-xs text-[var(--text-muted)] mb-1">Last Action</p>
                <p className="text-sm text-[var(--text-secondary)]">{status.last_action}</p>
              </div>
            )}
          </div>
        ) : (
          <p className="text-sm text-[var(--text-muted)]">No investigation data</p>
        )}
      </Card>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* Monitoring Jobs */}
        <Card title="Monitoring Jobs"
          action={
            <button onClick={() => monitoringQuery.refetch()} className="text-[var(--text-muted)] hover:text-[var(--text-primary)]">
              <RefreshCw size={14} />
            </button>
          }
        >
          {monitoringQuery.isLoading ? (
            <LoadingSpinner size={24} />
          ) : jobs.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">No monitoring jobs</p>
          ) : (
            <div className="space-y-2">
              {jobs.map((job: Record<string, unknown>, i: number) => (
                <div key={String(job.id || job.job_id || i)} className="flex items-center justify-between py-2 border-b border-[var(--border)]/50 last:border-0">
                  <div className="min-w-0">
                    <p className="text-sm text-[var(--text-primary)] truncate">
                      {String(job.name || job.query || job.type || 'Job')}
                    </p>
                    <p className="text-[10px] text-[var(--text-muted)]">
                      {String(job.schedule || job.interval || '')}
                      {job.last_run ? ` | Last: ${new Date(String(job.last_run)).toLocaleString()}` : ''}
                    </p>
                  </div>
                  <Badge type={String(job.status || job.state || 'active')} />
                </div>
              ))}
            </div>
          )}
        </Card>

        {/* Alerts */}
        <Card title="Alerts"
          action={
            <button onClick={() => alertsQuery.refetch()} className="text-[var(--text-muted)] hover:text-[var(--text-primary)]">
              <RefreshCw size={14} />
            </button>
          }
        >
          {alertsQuery.isLoading ? (
            <LoadingSpinner size={24} />
          ) : alerts.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">No alerts</p>
          ) : (
            <div className="space-y-2 max-h-80 overflow-y-auto">
              {alerts.slice(0, 20).map((alert: Record<string, unknown>, i: number) => (
                <div key={String(alert.id || alert.alert_id || i)} className="flex items-start gap-2 py-2 border-b border-[var(--border)]/50 last:border-0">
                  <AlertCircle size={14} className={`shrink-0 mt-0.5 ${
                    alert.severity === 'high' || alert.severity === 'critical'
                      ? 'text-[var(--accent-red)]'
                      : 'text-[var(--accent-yellow)]'
                  }`} />
                  <div className="min-w-0 flex-1">
                    <p className="text-sm text-[var(--text-primary)] truncate">
                      {String(alert.message || alert.title || '')}
                    </p>
                    {alert.created_at ? (
                      <p className="text-[10px] text-[var(--text-muted)]">
                        {new Date(String(alert.created_at)).toLocaleString()}
                      </p>
                    ) : null}
                  </div>
                  {alert.severity ? <Badge type={String(alert.severity)} className="shrink-0" /> : null}
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
