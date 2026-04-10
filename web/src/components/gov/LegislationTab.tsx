import { useState } from 'react';
import { Scroll, ExternalLink, Search, ChevronDown, ChevronRight, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';

import LoadingSpinner from '../LoadingSpinner';
import { useGovLaws } from '../../hooks/useGovernment';

/* ── Types ── */

interface Law {
  id: string;
  title: string;
  short_title?: string;
  procedure?: string;
  status?: string;
  legislature?: string;
  date_depot?: string;
  date_promulgation?: string;
  initiator?: string;
  jo_url?: string;
  source_url?: string;
  amendments_count?: number;
  amendments_adopted?: number;
  articles_initial?: number;
  articles_final?: number;
  duration_days?: number;
  [k: string]: unknown;
}

/* ── Constants ── */

const STATUS_STYLE: Record<string, { variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  en_cours:    { variant: 'secondary' },
  promulgue:   { variant: 'default' },
  rejete:      { variant: 'destructive' },
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

export function LegislationTab() {
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const lawsQ = useGovLaws();
  const rawLaws: Law[] = Array.isArray(lawsQ.data) ? lawsQ.data : [];

  // Filter by search
  const filtered = searchQuery
    ? rawLaws.filter(l =>
        (l.title || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
        (l.short_title || '').toLowerCase().includes(searchQuery.toLowerCase())
      )
    : rawLaws;

  const toggle = (id: string) => setExpandedId(prev => prev === id ? null : id);

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Legislation ({rawLaws.length})</CardTitle>
        <CardAction>
          <div className="relative">
            <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
            <Input
              placeholder="Rechercher une loi..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className="pl-8 h-7 w-64 text-xs"
            />
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {lawsQ.isLoading ? (
            <div className="p-8"><LoadingSpinner text="Chargement des lois..." /></div>
          ) : lawsQ.isError ? (
            <div className="p-8"><ErrorBanner message={(lawsQ.error as Error)?.message || 'Impossible de charger les lois'} /></div>
          ) : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Scroll size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">
                {searchQuery ? 'Aucune loi ne correspond a la recherche.' : 'Aucune loi.'}
              </p>
            </div>
          ) : filtered.map((l: Law) => {
            const expanded = expandedId === l.id;
            const statusKey = (l.status || '').toLowerCase().replace(/[éè]/g, 'e');
            const style = STATUS_STYLE[statusKey] || { variant: 'outline' as const };

            return (
              <div key={l.id} className="border-b border-border/50 hover:bg-muted/30">
                <button
                  onClick={() => toggle(l.id)}
                  className="w-full flex items-start gap-3 px-4 py-3 text-left"
                >
                  <div className="pt-0.5 shrink-0">
                    {expanded
                      ? <ChevronDown className="size-3.5 text-muted-foreground" />
                      : <ChevronRight className="size-3.5 text-muted-foreground" />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <p className="text-sm font-medium text-foreground">
                        {l.short_title || l.title}
                      </p>
                      <Badge variant={style.variant}>
                        {l.status || 'inconnu'}
                      </Badge>
                      {l.procedure && (
                        <Badge variant="outline" className="text-[10px]">{l.procedure}</Badge>
                      )}
                    </div>
                    <div className="flex items-center gap-3 mt-0.5 text-xs text-muted-foreground flex-wrap">
                      {l.legislature && <span>Legislature {l.legislature}</span>}
                      {l.date_depot && (
                        <span>Depose le {new Date(l.date_depot).toLocaleDateString('fr-FR')}</span>
                      )}
                      {l.date_promulgation && (
                        <span>Promulgue le {new Date(l.date_promulgation).toLocaleDateString('fr-FR')}</span>
                      )}
                    </div>
                    {/* Stats row */}
                    <div className="flex items-center gap-3 mt-1 text-xs">
                      {l.amendments_count != null && (
                        <span className="text-muted-foreground">
                          {l.amendments_count} amendement(s)
                          {l.amendments_adopted != null && (
                            <span className="text-green-400"> ({l.amendments_adopted} adoptes)</span>
                          )}
                        </span>
                      )}
                      {l.articles_initial != null && l.articles_final != null && (
                        <span className="text-muted-foreground">
                          {l.articles_initial} \u2192 {l.articles_final} articles
                        </span>
                      )}
                      {l.duration_days != null && (
                        <span className="text-muted-foreground">{l.duration_days}j</span>
                      )}
                    </div>
                  </div>
                </button>

                {expanded && (
                  <div className="px-4 pb-3 pl-11 space-y-2">
                    {l.title !== l.short_title && l.short_title && (
                      <p className="text-xs text-foreground">{l.title}</p>
                    )}
                    {l.initiator && (
                      <p className="text-xs text-muted-foreground">
                        Initiateur : <span className="text-foreground">{l.initiator}</span>
                      </p>
                    )}
                    <div className="flex items-center gap-3">
                      {l.jo_url && (
                        <a href={l.jo_url} target="_blank" rel="noopener noreferrer"
                          className="text-xs text-cyan-400 hover:underline flex items-center gap-1">
                          <ExternalLink className="size-2.5" /> Journal Officiel
                        </a>
                      )}
                      {l.source_url && (
                        <a href={l.source_url} target="_blank" rel="noopener noreferrer"
                          className="text-xs text-cyan-400 hover:underline flex items-center gap-1">
                          <ExternalLink className="size-2.5" /> Source
                        </a>
                      )}
                    </div>
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
