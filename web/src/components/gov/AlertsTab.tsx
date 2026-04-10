import { Bell, CheckCircle, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';

import LoadingSpinner from '../LoadingSpinner';
import { useGovAlerts, useMarkAlertRead } from '../../hooks/useGovernment';
import type { GovAlert } from './types';

/* ── Error Banner ── */

function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center gap-3">
      <AlertTriangle size={36} className="text-red-400" />
      <p className="text-sm text-red-400 font-medium">Erreur de chargement</p>
      <p className="text-xs text-muted-foreground max-w-md">{message}</p>
    </div>
  );
}

/* ── Alerts Tab ── */

export function AlertsTab() {
  const alertsQ = useGovAlerts();
  const markRead = useMarkAlertRead();
  const alerts: GovAlert[] = Array.isArray(alertsQ.data) ? alertsQ.data : [];
  const unread = alerts.filter((a: GovAlert) => !a.is_read);
  const read = alerts.filter((a: GovAlert) => a.is_read);

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Alertes ({unread.length} non lues)</CardTitle>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {alertsQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : alertsQ.isError ? <div className="p-8"><ErrorBanner message={(alertsQ.error as Error)?.message || 'Impossible de charger les alertes'} /></div>
          : alerts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Bell size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucune alerte.</p>
            </div>
          ) : [...unread, ...read].map((a: GovAlert) => (
            <div key={a.id} className={`flex items-start gap-3 px-4 py-3 border-b border-border/50 transition-colors ${a.is_read ? 'opacity-50' : 'hover:bg-muted/30'}`}>
              <Badge variant={a.severity === 'high' ? 'destructive' : a.severity === 'medium' ? 'secondary' : 'outline'}>
                {a.alert_type}
              </Badge>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground">{a.title}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.description}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.created_at ? new Date(a.created_at).toLocaleString('fr-FR') : ''}</p>
              </div>
              {!a.is_read && (
                <Button variant="ghost" size="xs" onClick={() => markRead.mutate(a.id)}>
                  <CheckCircle className="size-3" /> Lu
                </Button>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
