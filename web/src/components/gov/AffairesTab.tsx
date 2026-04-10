import { useState } from 'react';
import { Scale, ExternalLink, ChevronDown, ChevronRight, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select';

import LoadingSpinner from '../LoadingSpinner';
import { useGovAffairs } from '../../hooks/useGovernment';

/* ── Types ── */

interface Affair {
  id: string;
  title: string;
  politician_name?: string;
  status?: string;
  category?: string;
  description?: string;
  source_url?: string;
  date_start?: string;
  date_end?: string;
  [k: string]: unknown;
}

/* ── Constants ── */

const STATUS_STYLE: Record<string, { color: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  enquete:   { color: 'text-yellow-400', variant: 'outline' },
  jugement:  { color: 'text-blue-400',   variant: 'secondary' },
  condamne:  { color: 'text-red-400',    variant: 'destructive' },
  relaxe:    { color: 'text-green-400',   variant: 'default' },
};

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

/* ── Component ── */

export function AffairesTab() {
  const [statusFilter, setStatusFilter] = useState('all');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [sortOrder, setSortOrder] = useState<'desc' | 'asc'>('desc');

  const affairsQ = useGovAffairs();
  const rawAffairs: Affair[] = Array.isArray(affairsQ.data) ? affairsQ.data : [];

  // Filter by status
  const filtered = statusFilter === 'all'
    ? rawAffairs
    : rawAffairs.filter(a => (a.status || '').toLowerCase() === statusFilter);

  // Sort by date
  const sorted = [...filtered].sort((a, b) => {
    const dateA = a.date_start ? new Date(a.date_start).getTime() : 0;
    const dateB = b.date_start ? new Date(b.date_start).getTime() : 0;
    return sortOrder === 'desc' ? dateB - dateA : dateA - dateB;
  });

  const toggle = (id: string) => setExpandedId(prev => prev === id ? null : id);

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Affaires judiciaires</CardTitle>
        <CardAction>
          <div className="flex items-center gap-2">
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger size="sm" className="w-36"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">Tous statuts</SelectItem>
                <SelectItem value="enquete">Enquete</SelectItem>
                <SelectItem value="jugement">Jugement</SelectItem>
                <SelectItem value="condamne">Condamne</SelectItem>
                <SelectItem value="relaxe">Relaxe</SelectItem>
              </SelectContent>
            </Select>
            <Button variant="outline" size="xs"
              onClick={() => setSortOrder(o => o === 'desc' ? 'asc' : 'desc')}>
              Date {sortOrder === 'desc' ? '\u2193' : '\u2191'}
            </Button>
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {affairsQ.isLoading ? (
            <div className="p-8"><LoadingSpinner text="Chargement des affaires..." /></div>
          ) : affairsQ.isError ? (
            <div className="p-8"><ErrorBanner message={(affairsQ.error as Error)?.message || 'Impossible de charger les affaires'} /></div>
          ) : sorted.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Scale size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucune affaire.</p>
            </div>
          ) : sorted.map((a: Affair) => {
            const expanded = expandedId === a.id;
            const style = STATUS_STYLE[(a.status || '').toLowerCase()] || STATUS_STYLE.enquete;

            return (
              <div key={a.id} className="border-b border-border/50 hover:bg-muted/30">
                <button
                  onClick={() => toggle(a.id)}
                  className="w-full flex items-start gap-3 px-4 py-3 text-left"
                >
                  <div className="pt-0.5 shrink-0">
                    {expanded
                      ? <ChevronDown className="size-3.5 text-muted-foreground" />
                      : <ChevronRight className="size-3.5 text-muted-foreground" />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <p className="text-sm font-medium text-foreground">{a.title}</p>
                      <Badge variant={style.variant}>
                        {a.status || 'inconnu'}
                      </Badge>
                      {a.category && (
                        <Badge variant="outline" className="text-[10px]">{a.category}</Badge>
                      )}
                    </div>
                    <div className="flex items-center gap-2 mt-0.5">
                      {a.politician_name && (
                        <span className="text-xs text-cyan-400">{a.politician_name}</span>
                      )}
                      {a.date_start && (
                        <span className="text-xs text-muted-foreground">
                          {new Date(a.date_start).toLocaleDateString('fr-FR')}
                          {a.date_end && ` \u2014 ${new Date(a.date_end).toLocaleDateString('fr-FR')}`}
                        </span>
                      )}
                    </div>
                  </div>
                </button>

                {expanded && (
                  <div className="px-4 pb-3 pl-11">
                    {a.description && (
                      <p className="text-xs text-muted-foreground mb-2">{a.description}</p>
                    )}
                    {a.source_url && (
                      <a href={a.source_url} target="_blank" rel="noopener noreferrer"
                        className="text-xs text-cyan-400 hover:underline flex items-center gap-1">
                        <ExternalLink className="size-2.5" /> Source
                      </a>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
