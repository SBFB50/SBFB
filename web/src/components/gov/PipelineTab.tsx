import { Activity, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';

import LoadingSpinner from '../LoadingSpinner';
import { useGovWorkers } from '../../hooks/useGovernment';
import type { GovWorkerStatus } from './types';

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

/* ── Pipeline Tab ── */

export function PipelineTab() {
  const workersQ = useGovWorkers();
  const data = workersQ.data || { running: false, workers: 0, worker_status: [] };
  const workers: GovWorkerStatus[] = Array.isArray(data.worker_status) ? data.worker_status : [];

  const statusColor = (s: string) => {
    if (s === 'processing') return 'text-green-400';
    if (s === 'error' || s === 'circuit_open') return 'text-red-400';
    if (s === 'idle') return 'text-muted-foreground';
    return 'text-yellow-400';
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Pipeline ({workers.length} workers)</CardTitle>
        <CardAction>
          <Badge variant={data.running ? 'default' : 'destructive'}>
            {data.running ? 'Actif' : 'Arrete'}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[calc(100vh-460px)]">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
            {workersQ.isError ? <div className="col-span-full p-8"><ErrorBanner message={(workersQ.error as Error)?.message || 'Impossible de charger le pipeline'} /></div>
            : workersQ.isLoading ? <div className="col-span-full p-8"><LoadingSpinner text="Chargement..." /></div>
            : null}
            {workers.map((w: GovWorkerStatus) => (
              <div key={w.name} className="rounded-lg border border-border/50 bg-muted/20 px-3 py-2.5">
                <div className="flex items-center gap-2 mb-1">
                  <div className={`size-1.5 rounded-full ${w.status === 'processing' ? 'bg-green-400' : w.status === 'error' ? 'bg-red-400' : 'bg-zinc-500'}`} />
                  <p className="text-xs font-medium text-foreground truncate">{w.name?.replace('gov_', '')}</p>
                </div>
                <p className={`text-xs ${statusColor(w.status || 'idle')}`}>{w.status || 'idle'}</p>
                {w.events_processed !== undefined && (
                  <p className="text-xs text-muted-foreground mt-0.5">{w.events_processed} traites</p>
                )}
                {(w.events_errored ?? 0) > 0 && (
                  <p className="text-xs text-red-400">{w.events_errored} erreurs</p>
                )}
              </div>
            ))}
          </div>
          {workers.length === 0 && !workersQ.isLoading && !workersQ.isError && (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Activity size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Pipeline non demarre.</p>
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
