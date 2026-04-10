import { useMemo } from 'react';
import {
  FileText, AlertTriangle, Bell, Users,
  Newspaper, TrendingUp, Calendar,
} from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';

import LoadingSpinner from '../LoadingSpinner';
import MetricCard from '../MetricCard';
import {
  useGovStats, useGovAlerts, useGovAllContradictions,
} from '../../hooks/useGovernment';

/* ── Helpers ── */

function getWeekRange(): { start: Date; end: Date; label: string } {
  const now = new Date();
  const end = new Date(now);
  const start = new Date(now);
  start.setDate(start.getDate() - 7);

  const fmt = (d: Date) => d.toLocaleDateString('fr-FR', { day: '2-digit', month: 'long' });
  return { start, end, label: `${fmt(start)} — ${fmt(end)}` };
}

function isThisWeek(dateStr: string | undefined, weekStart: Date): boolean {
  if (!dateStr) return false;
  try {
    const d = new Date(dateStr);
    return d >= weekStart;
  } catch { return false; }
}

/* ── Component ── */

export function RecapTab() {
  const statsQ = useGovStats();
  const alertsQ = useGovAlerts();
  const contraQ = useGovAllContradictions();

  const stats = statsQ.data || { politicians: 0, positions: 0, contradictions: 0, press_articles: 0, last_scan: null };
  const alerts: any[] = Array.isArray(alertsQ.data) ? alertsQ.data : [];
  const allContras: any[] = Array.isArray(contraQ.data) ? contraQ.data : [];

  const isLoading = statsQ.isLoading || alertsQ.isLoading || contraQ.isLoading;
  const week = useMemo(() => getWeekRange(), []);

  // Filter items from this week
  const weekContras = useMemo(
    () => allContras.filter(c => isThisWeek(c.detected_at, week.start)),
    [allContras, week.start],
  );
  const weekAlerts = useMemo(
    () => alerts.filter(a => isThisWeek(a.created_at, week.start)),
    [alerts, week.start],
  );
  const weekRecaps = useMemo(
    () => alerts.filter(a => a.alert_type === 'recap' && isThisWeek(a.created_at, week.start)),
    [alerts, week.start],
  );

  // Sort contradictions: high severity first
  const sortedWeekContras = useMemo(
    () => [...weekContras].sort((a, b) => {
      const sev = { high: 3, medium: 2, low: 1 };
      return (sev[b.severity as keyof typeof sev] || 0) - (sev[a.severity as keyof typeof sev] || 0);
    }),
    [weekContras],
  );

  const sortedWeekAlerts = useMemo(
    () => [...weekAlerts].sort((a, b) => {
      const sev = { high: 3, medium: 2, low: 1 };
      return (sev[b.severity as keyof typeof sev] || 0) - (sev[a.severity as keyof typeof sev] || 0);
    }),
    [weekAlerts],
  );

  if (isLoading) {
    return (
      <Card className="h-[calc(100vh-380px)] flex items-center justify-center">
        <LoadingSpinner text="Chargement du recap..." />
      </Card>
    );
  }

  return (
    <div className="space-y-4 h-[calc(100vh-380px)] overflow-auto pr-1">
      {/* Header */}
      <div className="flex items-center gap-3">
        <div className="p-2 rounded-lg bg-purple-500/10">
          <Calendar size={18} className="text-purple-400" />
        </div>
        <div>
          <h3 className="text-sm font-semibold text-foreground">Recap de la semaine</h3>
          <p className="text-xs text-muted-foreground">{week.label}</p>
        </div>
      </div>

      {/* Summary metrics */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-3">
        <MetricCard
          label="Politiciens scannes"
          value={stats.politicians}
          icon={Users}
          color="var(--accent-cyan)"
        />
        <MetricCard
          label="Positions collectees"
          value={stats.positions}
          icon={FileText}
          color="var(--accent-green)"
        />
        <MetricCard
          label="Contradictions (semaine)"
          value={weekContras.length}
          icon={AlertTriangle}
          color="var(--accent-red)"
        />
        <MetricCard
          label="Alertes (semaine)"
          value={weekAlerts.length}
          icon={Bell}
          color="var(--accent-orange)"
        />
      </div>

      {/* Weekly recap text (if any recap alerts) */}
      {weekRecaps.length > 0 && (
        <Card className="border-purple-500/20 bg-purple-500/5">
          <CardHeader className="border-b border-purple-500/10">
            <CardTitle className="text-purple-400">
              <TrendingUp className="size-4 inline-block mr-1.5 -mt-0.5" />
              Resume automatique
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4">
            {weekRecaps.map((r: any) => (
              <div key={r.id} className="mb-3 last:mb-0">
                <p className="text-sm text-foreground leading-relaxed">{r.description || r.title}</p>
                <p className="text-[10px] text-muted-foreground mt-1">
                  {r.created_at ? new Date(r.created_at).toLocaleString('fr-FR') : ''}
                </p>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-3">
        {/* Top contradictions */}
        <Card>
          <CardHeader className="border-b">
            <CardTitle>
              <AlertTriangle className="size-4 inline-block mr-1.5 -mt-0.5 text-red-400" />
              Contradictions de la semaine ({weekContras.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <ScrollArea className="max-h-[350px]">
              {sortedWeekContras.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-10 text-center">
                  <AlertTriangle size={28} className="text-muted-foreground mb-2" />
                  <p className="text-xs text-muted-foreground">Aucune contradiction cette semaine</p>
                </div>
              ) : sortedWeekContras.map((c: any) => (
                <div key={c.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/30 hover:bg-muted/20 transition-colors">
                  <Badge variant={c.severity === 'high' ? 'destructive' : c.severity === 'medium' ? 'secondary' : 'outline'}>
                    {c.severity}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground">{c.subject}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2 leading-relaxed">{c.description}</p>
                    {c.detected_at && (
                      <p className="text-[10px] text-muted-foreground mt-1">
                        {new Date(c.detected_at).toLocaleDateString('fr-FR')}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </ScrollArea>
          </CardContent>
        </Card>

        {/* Top alerts */}
        <Card>
          <CardHeader className="border-b">
            <CardTitle>
              <Bell className="size-4 inline-block mr-1.5 -mt-0.5 text-orange-400" />
              Alertes de la semaine ({weekAlerts.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <ScrollArea className="max-h-[350px]">
              {sortedWeekAlerts.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-10 text-center">
                  <Bell size={28} className="text-muted-foreground mb-2" />
                  <p className="text-xs text-muted-foreground">Aucune alerte cette semaine</p>
                </div>
              ) : sortedWeekAlerts.map((a: any) => (
                <div key={a.id} className={`flex items-start gap-3 px-4 py-3 border-b border-border/30 transition-colors ${a.is_read ? 'opacity-50' : 'hover:bg-muted/20'}`}>
                  <Badge variant={a.severity === 'high' ? 'destructive' : a.severity === 'medium' ? 'secondary' : 'outline'}>
                    {a.alert_type || 'info'}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground">{a.title}</p>
                    {a.description && (
                      <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{a.description}</p>
                    )}
                    {a.created_at && (
                      <p className="text-[10px] text-muted-foreground mt-1">
                        {new Date(a.created_at).toLocaleString('fr-FR')}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </ScrollArea>
          </CardContent>
        </Card>
      </div>

      {/* All-time stats summary */}
      <Card>
        <CardHeader className="border-b">
          <CardTitle>
            <TrendingUp className="size-4 inline-block mr-1.5 -mt-0.5" />
            Vue d'ensemble
          </CardTitle>
        </CardHeader>
        <CardContent className="p-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
            <div>
              <p className="text-2xl font-bold text-foreground tabular-nums">{stats.politicians}</p>
              <p className="text-xs text-muted-foreground mt-0.5">Politiciens total</p>
            </div>
            <div>
              <p className="text-2xl font-bold text-foreground tabular-nums">{stats.positions}</p>
              <p className="text-xs text-muted-foreground mt-0.5">Positions total</p>
            </div>
            <div>
              <p className="text-2xl font-bold text-red-400 tabular-nums">{allContras.length}</p>
              <p className="text-xs text-muted-foreground mt-0.5">Contradictions total</p>
            </div>
            <div>
              <p className="text-2xl font-bold text-orange-400 tabular-nums">{alerts.length}</p>
              <p className="text-xs text-muted-foreground mt-0.5">Alertes total</p>
            </div>
          </div>
          {stats.last_scan && (
            <>
              <Separator className="my-3" />
              <p className="text-xs text-muted-foreground text-center">
                Dernier scan: {new Date(stats.last_scan).toLocaleString('fr-FR')}
              </p>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
