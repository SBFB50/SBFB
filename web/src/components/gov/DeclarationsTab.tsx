import { useState } from 'react';
import { FileCheck, ExternalLink, Search, Users, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';

import LoadingSpinner from '../LoadingSpinner';
import { useGovPoliticians, useGovDeclarations } from '../../hooks/useGovernment';

/* ── Types ── */

interface Pol {
  id: string;
  name: string;
  party?: string;
  chamber?: string;
  [k: string]: unknown;
}

interface Declaration {
  id: string;
  type?: string;
  qualite?: string;
  departement?: string;
  date_publication?: string;
  date_depot?: string;
  url?: string;
  [k: string]: unknown;
}

/* ── Constants ── */

const TYPE_STYLE: Record<string, { label: string; variant: 'default' | 'secondary' | 'outline' }> = {
  patrimoine: { label: 'Patrimoine', variant: 'default' },
  interets:   { label: 'Interets',   variant: 'secondary' },
  activites:  { label: 'Activites',  variant: 'outline' },
};

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080', 'EELV': '#00c000', 'ECO': '#00c000',
  'RE': '#ffcc00', 'REN': '#ffcc00', 'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#00bfff', 'LR': '#0066cc', 'UDI': '#00cccc', 'LIOT': '#87ceeb',
  'RN': '#0d2244', 'SE': '#64748b',
};
const DEFAULT_COLOR = '#64748b';

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

export function DeclarationsTab() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const polsQ = useGovPoliticians();
  const declQ = useGovDeclarations(selectedId);

  const pols: Pol[] = Array.isArray(polsQ.data) ? polsQ.data : [];
  const declarations: Declaration[] = Array.isArray(declQ.data) ? declQ.data : [];

  const filtered = searchQuery
    ? pols.filter(p =>
        p.name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.party?.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : pols;

  const selectedPol = pols.find(p => p.id === selectedId);

  return (
    <div className="flex gap-3 h-[calc(100vh-380px)]">
      {/* ── Left panel: politician selector ── */}
      <Card className="w-72 shrink-0 flex flex-col">
        <CardHeader className="border-b">
          <CardTitle className="text-sm">Politicien</CardTitle>
          <CardAction>
            <div className="relative">
              <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
              <Input
                placeholder="Rechercher..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                className="pl-8 h-7 w-full text-xs"
              />
            </div>
          </CardAction>
        </CardHeader>
        <CardContent className="flex-1 p-0">
          <ScrollArea className="h-full">
            {polsQ.isLoading ? (
              <div className="p-6"><LoadingSpinner text="Chargement..." /></div>
            ) : polsQ.isError ? (
              <div className="p-4"><ErrorBanner message={(polsQ.error as Error)?.message || 'Erreur'} /></div>
            ) : filtered.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <Users size={28} className="text-muted-foreground mb-2" />
                <p className="text-xs text-muted-foreground">Aucun politicien.</p>
              </div>
            ) : filtered.map((p: Pol) => (
              <button
                key={p.id}
                onClick={() => setSelectedId(p.id)}
                className={`w-full flex items-center gap-2 px-3 py-2 text-left border-b border-border/50 transition-colors hover:bg-muted/50 ${
                  selectedId === p.id ? 'bg-cyan-500/5 border-l-2 border-l-cyan-500' : ''
                }`}
              >
                <div
                  className="w-2 h-2 rounded-full shrink-0"
                  style={{ backgroundColor: PARTY_COLORS[p.party || ''] || DEFAULT_COLOR }}
                />
                <div className="flex-1 min-w-0">
                  <p className="text-xs font-medium text-foreground truncate">{p.name}</p>
                  <p className="text-[10px] text-muted-foreground">
                    {p.party || 'SE'} — {p.chamber === 'assemblee' ? 'AN' : 'Senat'}
                  </p>
                </div>
              </button>
            ))}
          </ScrollArea>
        </CardContent>
      </Card>

      {/* ── Right panel: declarations ── */}
      <Card className="flex-1 flex flex-col">
        <CardHeader className="border-b">
          <CardTitle>
            {selectedPol
              ? `Declarations — ${selectedPol.name}`
              : 'Declarations HATVP'}
          </CardTitle>
          {selectedPol && declarations.length > 0 && (
            <CardAction>
              <span className="text-xs text-muted-foreground">{declarations.length} declaration(s)</span>
            </CardAction>
          )}
        </CardHeader>
        <CardContent className="flex-1 p-0">
          <ScrollArea className="h-full">
            {!selectedId ? (
              <div className="flex flex-col items-center justify-center py-16 text-center">
                <FileCheck size={36} className="text-muted-foreground mb-3" />
                <p className="text-sm text-muted-foreground">Selectionnez un politicien pour voir ses declarations.</p>
              </div>
            ) : declQ.isLoading ? (
              <div className="p-8"><LoadingSpinner text="Chargement des declarations..." /></div>
            ) : declQ.isError ? (
              <div className="p-8"><ErrorBanner message={(declQ.error as Error)?.message || 'Impossible de charger les declarations'} /></div>
            ) : declarations.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-center">
                <FileCheck size={36} className="text-muted-foreground mb-3" />
                <p className="text-sm text-muted-foreground">Aucune declaration pour ce politicien.</p>
              </div>
            ) : declarations.map((d: Declaration) => {
              const typeKey = (d.type || '').toLowerCase();
              const style = TYPE_STYLE[typeKey] || { label: d.type || 'Autre', variant: 'outline' as const };

              return (
                <div key={d.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30">
                  <Badge variant={style.variant}>
                    {style.label}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      {d.qualite && (
                        <span className="text-sm font-medium text-foreground">{d.qualite}</span>
                      )}
                      {d.departement && (
                        <span className="text-xs text-muted-foreground">({d.departement})</span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 mt-0.5 text-xs text-muted-foreground">
                      {d.date_publication && (
                        <span>Publiee le {new Date(d.date_publication).toLocaleDateString('fr-FR')}</span>
                      )}
                      {d.date_depot && (
                        <span>Deposee le {new Date(d.date_depot).toLocaleDateString('fr-FR')}</span>
                      )}
                    </div>
                  </div>
                  {d.url && (
                    <a href={d.url} target="_blank" rel="noopener noreferrer"
                      className="text-xs text-cyan-400 hover:underline flex items-center gap-1 shrink-0">
                      <ExternalLink className="size-3" /> HATVP
                    </a>
                  )}
                </div>
              );
            })}
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  );
}
