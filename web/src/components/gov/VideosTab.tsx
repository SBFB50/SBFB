import { useState } from 'react';
import { Video, Search, ExternalLink, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';

import LoadingSpinner from '../LoadingSpinner';
import { useGovAllTranscriptions } from '../../hooks/useGovernment';
import type { Transcription } from './types';

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

/* ── Videos Tab ── */

export function VideosTab() {
  const [searchQ, setSearchQ] = useState('');
  const transcriptionsQ = useGovAllTranscriptions();
  const transcriptions: Transcription[] = Array.isArray(transcriptionsQ.data) ? transcriptionsQ.data : [];
  const filtered = searchQ
    ? transcriptions.filter((t: Transcription) =>
        (t.title || '').toLowerCase().includes(searchQ.toLowerCase()) ||
        (t.transcription || '').toLowerCase().includes(searchQ.toLowerCase())
      )
    : transcriptions;

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Transcriptions video</CardTitle>
        <CardAction>
          <div className="relative">
            <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
            <Input placeholder="Rechercher dans les transcriptions..." value={searchQ}
              onChange={e => setSearchQ(e.target.value)} className="pl-8 h-7 w-64 text-xs" />
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {transcriptionsQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : transcriptionsQ.isError ? <div className="p-8"><ErrorBanner message={(transcriptionsQ.error as Error)?.message || 'Impossible de charger les transcriptions'} /></div>
          : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Video size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucune transcription.</p>
            </div>
          ) : filtered.map((t: Transcription) => (
            <div key={t.id} className="px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <div className="flex items-center gap-2 mb-1">
                <Video className="size-3.5 text-cyan-400 shrink-0" />
                <p className="text-sm font-medium text-foreground truncate">{t.title || 'Sans titre'}</p>
                {t.duration_seconds && (
                  <span className="text-xs text-muted-foreground shrink-0">{Math.floor(t.duration_seconds / 60)}min</span>
                )}
              </div>
              <p className="text-xs text-muted-foreground line-clamp-3 mt-1">{(t.transcription || '').slice(0, 300)}...</p>
              <div className="flex gap-2 mt-1.5">
                {t.source_url && (
                  <a href={t.source_url} target="_blank" rel="noopener noreferrer"
                    className="text-xs text-cyan-400 hover:underline flex items-center gap-1">
                    <ExternalLink className="size-2.5" /> Source
                  </a>
                )}
                <span className="text-xs text-muted-foreground">{t.model_used}</span>
              </div>
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
