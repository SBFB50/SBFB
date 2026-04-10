import { useState } from 'react';
import {
  Search, MessageCircleQuestion, Loader2,
  ExternalLink, Sparkles, FileText, AlertTriangle,
} from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';

import LoadingSpinner from '../LoadingSpinner';
import { useGovSearch, useGovAsk } from '../../hooks/useGovernment';

/* ── Constants ── */

const SUGGESTIONS = [
  'Macron sur l\'immigration',
  'Votes sur le climat',
  'Contradictions recentes',
  'Reforme des retraites',
  'Budget education nationale',
  'Politique energetique nucleaire',
];

/* ── Component ── */

export function SearchTab() {
  const [input, setInput] = useState('');
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState<'search' | 'ask'>('search');

  const searchQ = useGovSearch(mode === 'search' ? query : '');
  const askQ = useGovAsk(mode === 'ask' ? query : '');

  const results: any[] = Array.isArray(searchQ.data?.results)
    ? searchQ.data.results
    : Array.isArray(searchQ.data)
      ? searchQ.data
      : [];
  const answer = askQ.data?.answer || askQ.data?.response || '';
  const answerSources: any[] = Array.isArray(askQ.data?.sources) ? askQ.data.sources : [];

  const isLoading = (mode === 'search' && searchQ.isLoading && query.length >= 2)
    || (mode === 'ask' && askQ.isLoading && query.length >= 2);

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault();
    if (input.trim().length >= 2) {
      setQuery(input.trim());
    }
  };

  const handleSuggestion = (s: string) => {
    setInput(s);
    setQuery(s);
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>
          <Search className="size-4 inline-block mr-1.5 -mt-0.5" />
          Recherche intelligente
        </CardTitle>
      </CardHeader>

      {/* Search input + mode toggle */}
      <div className="px-4 pt-4 pb-3 space-y-3 border-b border-border/50">
        <form onSubmit={handleSubmit} className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
            <input
              type="text"
              value={input}
              onChange={e => setInput(e.target.value)}
              placeholder="Posez une question politique..."
              className="w-full h-10 pl-10 pr-4 rounded-lg border border-border bg-muted/30 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500/40"
            />
          </div>
          <Button type="submit" disabled={input.trim().length < 2}>
            {isLoading ? <Loader2 className="size-4 animate-spin" /> : <Search className="size-4" />}
            Rechercher
          </Button>
        </form>

        {/* Mode toggle */}
        <div className="flex items-center gap-1">
          <Button
            variant={mode === 'search' ? 'default' : 'outline'}
            size="xs"
            onClick={() => { setMode('search'); if (query) setQuery(query); }}
          >
            <Search className="size-3" /> Recherche
          </Button>
          <Button
            variant={mode === 'ask' ? 'default' : 'outline'}
            size="xs"
            onClick={() => { setMode('ask'); if (query) setQuery(query); }}
          >
            <MessageCircleQuestion className="size-3" /> Question
          </Button>
          <span className="text-[10px] text-muted-foreground ml-2">
            {mode === 'search' ? 'Recherche vectorielle dans les donnees' : 'Reponse generee par IA avec sources'}
          </span>
        </div>
      </div>

      {/* Results area */}
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {/* No query yet — show suggestions */}
          {!query && (
            <div className="flex flex-col items-center justify-center py-12 px-4">
              <Sparkles size={40} className="text-cyan-400 mb-4" />
              <p className="text-sm text-muted-foreground mb-6">
                Recherchez dans les positions, articles, transcriptions et contradictions
              </p>
              <div className="flex flex-wrap gap-2 justify-center max-w-lg">
                {SUGGESTIONS.map(s => (
                  <button
                    key={s}
                    onClick={() => handleSuggestion(s)}
                    className="px-3 py-1.5 rounded-full border border-border/60 text-xs text-foreground hover:bg-cyan-500/10 hover:border-cyan-500/30 transition-colors"
                  >
                    {s}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Loading */}
          {query && isLoading && (
            <div className="p-8">
              <LoadingSpinner text={mode === 'ask' ? 'Generation de la reponse...' : 'Recherche en cours...'} />
            </div>
          )}

          {/* Error */}
          {query && ((mode === 'search' && searchQ.isError) || (mode === 'ask' && askQ.isError)) && (
            <div className="flex flex-col items-center justify-center py-16 text-center gap-3">
              <AlertTriangle size={36} className="text-red-400" />
              <p className="text-sm text-red-400 font-medium">Erreur de recherche</p>
              <p className="text-xs text-muted-foreground">
                {((mode === 'search' ? searchQ.error : askQ.error) as Error)?.message || 'Erreur inconnue'}
              </p>
            </div>
          )}

          {/* Search mode results */}
          {query && mode === 'search' && !searchQ.isLoading && !searchQ.isError && (
            <>
              {results.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-16 text-center">
                  <Search size={36} className="text-muted-foreground mb-3" />
                  <p className="text-sm text-muted-foreground">Aucun resultat pour "{query}"</p>
                </div>
              ) : (
                <div>
                  <div className="px-4 py-2 border-b border-border/30">
                    <p className="text-xs text-muted-foreground">{results.length} resultats</p>
                  </div>
                  {results.map((r: any, i: number) => (
                    <div key={r.id || i} className="flex items-start gap-3 px-4 py-3 border-b border-border/30 hover:bg-muted/20 transition-colors">
                      <div className="shrink-0 pt-0.5">
                        {r.type === 'position' ? <FileText className="size-3.5 text-blue-400" />
                        : r.type === 'press' || r.type === 'article' ? <FileText className="size-3.5 text-purple-400" />
                        : r.type === 'contradiction' ? <AlertTriangle className="size-3.5 text-red-400" />
                        : <FileText className="size-3.5 text-muted-foreground" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <Badge variant="outline" className="text-[10px]">{r.type || 'document'}</Badge>
                          {r.politician_name && (
                            <span className="text-xs font-medium text-cyan-400">{r.politician_name}</span>
                          )}
                          {r.date && (
                            <span className="text-[10px] text-muted-foreground ml-auto">
                              {new Date(r.date).toLocaleDateString('fr-FR')}
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-foreground mt-1 line-clamp-2">
                          {r.text || r.content || r.title || r.description || ''}
                        </p>
                        {r.score != null && (
                          <span className="text-[10px] text-muted-foreground">
                            Pertinence: {(r.score * 100).toFixed(0)}%
                          </span>
                        )}
                      </div>
                      {(r.source_url || r.url) && (
                        <a href={r.source_url || r.url} target="_blank" rel="noopener noreferrer" className="shrink-0">
                          <ExternalLink className="size-3.5 text-muted-foreground hover:text-cyan-400 transition-colors" />
                        </a>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </>
          )}

          {/* Ask mode answer */}
          {query && mode === 'ask' && !askQ.isLoading && !askQ.isError && (
            <div className="p-4 space-y-4">
              {!answer ? (
                <div className="flex flex-col items-center justify-center py-16 text-center">
                  <MessageCircleQuestion size={36} className="text-muted-foreground mb-3" />
                  <p className="text-sm text-muted-foreground">Pas de reponse disponible pour "{query}"</p>
                </div>
              ) : (
                <>
                  {/* Answer card */}
                  <Card className="border-cyan-500/20 bg-cyan-500/5">
                    <CardContent className="p-4">
                      <div className="flex items-center gap-2 mb-3">
                        <Sparkles className="size-4 text-cyan-400" />
                        <span className="text-xs font-semibold text-cyan-400 uppercase tracking-wider">
                          Reponse IA
                        </span>
                      </div>
                      <p className="text-sm text-foreground leading-relaxed whitespace-pre-wrap">
                        {answer}
                      </p>
                    </CardContent>
                  </Card>

                  {/* Sources */}
                  {answerSources.length > 0 && (
                    <div>
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                        Sources ({answerSources.length})
                      </p>
                      {answerSources.map((s: any, i: number) => (
                        <div key={s.id || i} className="flex items-start gap-2 py-2 border-b border-border/30">
                          <FileText className="size-3 text-muted-foreground shrink-0 mt-0.5" />
                          <div className="flex-1 min-w-0">
                            <p className="text-xs text-foreground line-clamp-2">
                              {s.text || s.content || s.title || ''}
                            </p>
                            {s.politician_name && (
                              <span className="text-[10px] text-cyan-400">{s.politician_name}</span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </>
              )}
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
