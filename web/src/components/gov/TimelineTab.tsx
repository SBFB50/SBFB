import { useState, useMemo } from 'react';
import {
  Clock, Newspaper, MessageSquare, AlertTriangle, Bell,
  Search, Filter, ChevronDown,
  Hash, Globe, Camera, Tv,
} from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';

import LoadingSpinner from '../LoadingSpinner';
import {
  useGovPress, useGovAllSocial, useGovAllContradictions, useGovAlerts,
} from '../../hooks/useGovernment';

/* ── Types ── */

interface TimelineEvent {
  id: string;
  type: 'press' | 'social' | 'contradiction' | 'alert';
  date: string;
  title: string;
  description: string;
  meta?: string;
  severity?: string;
  platform?: string;
  url?: string;
}

/* ── Constants ── */

const TYPE_CONFIG: Record<string, { icon: typeof Clock; color: string; bg: string; label: string }> = {
  press:         { icon: Newspaper,      color: 'text-purple-400', bg: 'bg-purple-500/10 border-purple-500/20', label: 'Presse' },
  social:        { icon: MessageSquare,  color: 'text-cyan-400',   bg: 'bg-cyan-500/10 border-cyan-500/20',    label: 'Social' },
  contradiction: { icon: AlertTriangle,  color: 'text-red-400',    bg: 'bg-red-500/10 border-red-500/20',      label: 'Contradiction' },
  alert:         { icon: Bell,           color: 'text-orange-400', bg: 'bg-orange-500/10 border-orange-500/20', label: 'Alerte' },
};

const ITEMS_PER_PAGE = 50;

/* ── Helpers ── */

function platformIcon(p: string) {
  if (p === 'twitter') return <Hash className="size-3" />;
  if (p === 'facebook') return <Globe className="size-3" />;
  if (p === 'instagram') return <Camera className="size-3" />;
  if (p === 'youtube') return <Tv className="size-3" />;
  return <MessageSquare className="size-3" />;
}

function formatDate(d: string | undefined) {
  if (!d) return '';
  try {
    return new Date(d).toLocaleString('fr-FR', {
      day: '2-digit', month: '2-digit', year: 'numeric',
      hour: '2-digit', minute: '2-digit',
    });
  } catch { return d; }
}

/* ── Component ── */

export function TimelineTab() {
  const [searchQuery, setSearchQuery] = useState('');
  const [visibleTypes, setVisibleTypes] = useState<Set<string>>(
    new Set(['press', 'social', 'contradiction', 'alert']),
  );
  const [visibleCount, setVisibleCount] = useState(ITEMS_PER_PAGE);

  // Data hooks
  const pressQ = useGovPress();
  const socialQ = useGovAllSocial();
  const contraQ = useGovAllContradictions();
  const alertsQ = useGovAlerts();

  const isLoading = pressQ.isLoading || socialQ.isLoading || contraQ.isLoading || alertsQ.isLoading;
  const isError = pressQ.isError && socialQ.isError && contraQ.isError && alertsQ.isError;

  const press: any[] = Array.isArray(pressQ.data) ? pressQ.data : [];
  const social: any[] = Array.isArray(socialQ.data) ? socialQ.data : [];
  const contras: any[] = Array.isArray(contraQ.data) ? contraQ.data : [];
  const alerts: any[] = Array.isArray(alertsQ.data) ? alertsQ.data : [];

  // Merge all sources into a unified timeline
  const allEvents = useMemo<TimelineEvent[]>(() => {
    const events: TimelineEvent[] = [];

    for (const a of press) {
      events.push({
        id: `press-${a.id}`,
        type: 'press',
        date: a.published_at || '',
        title: a.title || 'Article sans titre',
        description: a.summary || '',
        meta: a.source_name,
        url: a.url,
      });
    }

    for (const s of social) {
      events.push({
        id: `social-${s.id}`,
        type: 'social',
        date: s.posted_at || '',
        title: s.content?.slice(0, 120) || '',
        description: s.content || '',
        meta: s.platform,
        platform: s.platform,
        url: s.url,
      });
    }

    for (const c of contras) {
      events.push({
        id: `contra-${c.id}`,
        type: 'contradiction',
        date: c.detected_at || '',
        title: c.subject || 'Contradiction',
        description: c.description || '',
        severity: c.severity,
      });
    }

    for (const a of alerts) {
      events.push({
        id: `alert-${a.id}`,
        type: 'alert',
        date: a.created_at || '',
        title: a.title || 'Alerte',
        description: a.description || '',
        severity: a.severity,
      });
    }

    // Sort by date descending
    return events.sort((a, b) => (b.date || '').localeCompare(a.date || ''));
  }, [press, social, contras, alerts]);

  // Filter by type + search
  const filtered = useMemo(() => {
    let items = allEvents.filter(e => visibleTypes.has(e.type));
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      items = items.filter(e =>
        e.title.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        (e.meta || '').toLowerCase().includes(q)
      );
    }
    return items;
  }, [allEvents, visibleTypes, searchQuery]);

  const displayed = filtered.slice(0, visibleCount);
  const hasMore = visibleCount < filtered.length;

  const toggleType = (type: string) => {
    setVisibleTypes(prev => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
    setVisibleCount(ITEMS_PER_PAGE);
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>
          <Clock className="size-4 inline-block mr-1.5 -mt-0.5" />
          Chronologie unifiee
          <span className="text-xs font-normal text-muted-foreground ml-2">
            {filtered.length} evenements
          </span>
        </CardTitle>
      </CardHeader>

      {/* Filter bar */}
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border/50 flex-wrap">
        <Filter className="size-3.5 text-muted-foreground shrink-0" />

        {Object.entries(TYPE_CONFIG).map(([type, cfg]) => {
          const active = visibleTypes.has(type);
          const count = type === 'press' ? press.length
            : type === 'social' ? social.length
            : type === 'contradiction' ? contras.length
            : alerts.length;
          return (
            <Button
              key={type}
              variant={active ? 'default' : 'outline'}
              size="xs"
              onClick={() => toggleType(type)}
              className={active ? '' : 'opacity-50'}
            >
              <cfg.icon className="size-3" />
              {cfg.label} ({count})
            </Button>
          );
        })}

        <Separator orientation="vertical" className="h-5 mx-1" />

        <div className="relative flex-1 max-w-xs">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
          <Input
            placeholder="Filtrer les evenements..."
            value={searchQuery}
            onChange={e => { setSearchQuery(e.target.value); setVisibleCount(ITEMS_PER_PAGE); }}
            className="pl-8 h-7 text-xs"
          />
        </div>
      </div>

      {/* Timeline */}
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {isLoading ? (
            <div className="p-8"><LoadingSpinner text="Chargement de la chronologie..." /></div>
          ) : isError ? (
            <div className="flex flex-col items-center justify-center py-16 text-center gap-3">
              <AlertTriangle size={36} className="text-red-400" />
              <p className="text-sm text-red-400 font-medium">Erreur de chargement</p>
            </div>
          ) : displayed.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Clock size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">
                {allEvents.length === 0
                  ? 'Aucun evenement. Lancez un scan.'
                  : 'Aucun resultat pour ces filtres.'}
              </p>
            </div>
          ) : (
            <div className="relative">
              {/* Vertical line */}
              <div className="absolute left-6 top-0 bottom-0 w-px bg-border/50" />

              {displayed.map((event) => {
                const cfg = TYPE_CONFIG[event.type];
                const Icon = cfg.icon;
                return (
                  <div key={event.id} className="relative flex items-start gap-3 px-4 py-3 border-b border-border/30 hover:bg-muted/20 transition-colors">
                    {/* Icon dot */}
                    <div className={`relative z-10 flex items-center justify-center size-5 rounded-full border ${cfg.bg} shrink-0`}>
                      <Icon className={`size-3 ${cfg.color}`} />
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <Badge variant="outline" className={`text-[10px] ${cfg.color} border-current/30`}>
                          {cfg.label}
                        </Badge>
                        {event.severity && (
                          <Badge variant={event.severity === 'high' ? 'destructive' : 'secondary'}>
                            {event.severity}
                          </Badge>
                        )}
                        {event.platform && (
                          <span className="flex items-center gap-1 text-[10px] text-muted-foreground">
                            {platformIcon(event.platform)} {event.platform}
                          </span>
                        )}
                        {event.meta && event.type === 'press' && (
                          <span className="text-[10px] text-muted-foreground">{event.meta}</span>
                        )}
                        <span className="text-[10px] text-muted-foreground ml-auto shrink-0">
                          {formatDate(event.date)}
                        </span>
                      </div>

                      <p className="text-sm font-medium text-foreground mt-1 line-clamp-1">
                        {event.title}
                      </p>

                      {event.description && event.type !== 'social' && (
                        <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2 leading-relaxed">
                          {event.description}
                        </p>
                      )}
                    </div>
                  </div>
                );
              })}

              {/* Load more */}
              {hasMore && (
                <div className="flex justify-center py-4">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setVisibleCount(c => c + ITEMS_PER_PAGE)}
                  >
                    <ChevronDown className="size-3.5 mr-1" />
                    Charger plus ({filtered.length - visibleCount} restants)
                  </Button>
                </div>
              )}
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
