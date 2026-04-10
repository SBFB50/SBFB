import { useState } from 'react';
import { MessageSquare, Hash, Globe, Camera, Tv, ExternalLink, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';

import LoadingSpinner from '../LoadingSpinner';
import { useGovAllSocial } from '../../hooks/useGovernment';
import type { SocialPost } from './types';

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

/* ── Social Tab ── */

export function SocialTab() {
  const [platform, setPlatform] = useState('');
  const socialQ = useGovAllSocial(platform || undefined);
  const posts: SocialPost[] = Array.isArray(socialQ.data) ? socialQ.data : [];

  const platformIcon = (p: string) => {
    if (p === 'twitter') return <Hash className="size-3" />;
    if (p === 'facebook') return <Globe className="size-3" />;
    if (p === 'instagram') return <Camera className="size-3" />;
    if (p === 'youtube') return <Tv className="size-3" />;
    return <MessageSquare className="size-3" />;
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Reseaux sociaux</CardTitle>
        <CardAction>
          <div className="flex gap-1">
            {['', 'twitter', 'facebook', 'instagram'].map(p => (
              <Button key={p} variant={platform === p ? 'default' : 'outline'} size="xs"
                onClick={() => setPlatform(p)}>
                {p ? platformIcon(p) : null} {p || 'Tous'}
              </Button>
            ))}
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {socialQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : socialQ.isError ? <div className="p-8"><ErrorBanner message={(socialQ.error as Error)?.message || 'Impossible de charger les posts'} /></div>
          : posts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <MessageSquare size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucun post.</p>
            </div>
          ) : posts.map((p: SocialPost) => (
            <div key={p.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <div className="pt-0.5">{platformIcon(p.platform)}</div>
              <div className="flex-1 min-w-0">
                <p className="text-xs text-muted-foreground">{p.platform} — {p.posted_at ? new Date(p.posted_at).toLocaleDateString('fr-FR') : ''}</p>
                <p className="text-sm text-foreground mt-0.5 line-clamp-3">{p.content}</p>
              </div>
              {p.url && (
                <a href={p.url} target="_blank" rel="noopener noreferrer">
                  <Button variant="ghost" size="icon-xs" aria-label="Ouvrir le post"><ExternalLink className="size-3" /></Button>
                </a>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
