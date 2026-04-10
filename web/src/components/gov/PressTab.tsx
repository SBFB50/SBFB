import { useState } from 'react';
import { Newspaper, ExternalLink, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';

import LoadingSpinner from '../LoadingSpinner';
import { useGovPress } from '../../hooks/useGovernment';
import type { PressArticle } from './types';

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

/* ── Press Tab ── */

export function PressTab() {
  const [sentiment, setSentiment] = useState('');
  const pressQ = useGovPress(sentiment || undefined);
  const articles: PressArticle[] = Array.isArray(pressQ.data) ? pressQ.data : [];

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Revue de presse</CardTitle>
        <CardAction>
          <div className="flex gap-1">
            {['', 'positive', 'neutral', 'negative'].map(s => (
              <Button key={s} variant={sentiment === s ? 'default' : 'outline'} size="xs"
                onClick={() => setSentiment(s)}>
                {s || 'Tous'}
              </Button>
            ))}
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {pressQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : pressQ.isError ? <div className="p-8"><ErrorBanner message={(pressQ.error as Error)?.message || 'Impossible de charger la presse'} /></div>
          : articles.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Newspaper size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucun article.</p>
            </div>
          ) : articles.map((a: PressArticle) => (
            <div key={a.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <Badge variant={a.sentiment === 'positive' ? 'default' : a.sentiment === 'negative' ? 'destructive' : 'outline'}>
                {a.sentiment || '?'}
              </Badge>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground truncate">{a.title}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.source_name} — {a.published_at ? new Date(a.published_at).toLocaleDateString('fr-FR') : ''}</p>
                {a.summary && <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{a.summary}</p>}
              </div>
              {a.url && (
                <a href={a.url} target="_blank" rel="noopener noreferrer">
                  <Button variant="ghost" size="icon-xs" aria-label="Ouvrir l'article"><ExternalLink className="size-3" /></Button>
                </a>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
