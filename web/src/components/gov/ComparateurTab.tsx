import { useState, useMemo } from 'react';
import { Search, ArrowLeftRight, ExternalLink, AlertTriangle } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';

import {
  useGovPoliticians,
  useGovPositions,
  useGovPoliticianContradictions,
} from '../../hooks/useGovernment';
import { RadarProfile } from './PoliticalCharts';
import LoadingSpinner from '../LoadingSpinner';

/* ═══════════════════════════════════════════════════════════════════
   CONSTANTS
   ═══════════════════════════════════════════════════════════════════ */

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080', 'EELV': '#00c000', 'ECO': '#00c000',
  'RE': '#ffcc00', 'REN': '#ffcc00', 'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#00bfff', 'LR': '#0066cc', 'UDI': '#00cccc', 'LIOT': '#87ceeb',
  'RN': '#0d2244', 'SE': '#64748b',
};
const DEFAULT_COLOR = '#64748b';

const STANCE_COLOR: Record<string, string> = {
  pour: 'text-green-400',
  contre: 'text-red-400',
  abstention: 'text-yellow-400',
};

/* ═══════════════════════════════════════════════════════════════════
   TYPES
   ═══════════════════════════════════════════════════════════════════ */

interface Pol {
  id: string;
  name: string;
  party?: string;
  chamber?: string;
  role?: string;
  constituency?: string;
  position_count?: number;
  contradiction_count?: number;
  metadata?: Record<string, unknown>;
  [k: string]: unknown;
}

interface Pos {
  id: string;
  date: string;
  position_type: string;
  stance?: string;
  subject: string;
  position_text: string;
  source_url?: string;
  [k: string]: unknown;
}

interface Contra {
  id: string;
  politician_id: string;
  subject: string;
  description: string;
  severity: string;
  [k: string]: unknown;
}

/* ═══════════════════════════════════════════════════════════════════
   POLITICIAN SELECTOR (search dropdown)
   ═══════════════════════════════════════════════════════════════════ */

function PoliticianSelector({
  label,
  selectedId,
  onSelect,
  politicians,
  color,
}: {
  label: string;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  politicians: Pol[];
  color: string;
}) {
  const [searchQ, setSearchQ] = useState('');
  const [open, setOpen] = useState(false);

  const filtered = searchQ
    ? politicians.filter(
        p =>
          p.name.toLowerCase().includes(searchQ.toLowerCase()) ||
          (p.party || '').toLowerCase().includes(searchQ.toLowerCase())
      ).slice(0, 12)
    : [];

  const selected = politicians.find(p => p.id === selectedId);

  return (
    <div className="relative">
      <p className="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5">{label}</p>
      {selected ? (
        <div
          className="flex items-center gap-2 px-3 py-2 rounded-lg border border-border bg-muted/20 cursor-pointer hover:bg-muted/40"
          onClick={() => { onSelect(null); setOpen(true); }}
        >
          <div className="w-3 h-3 rounded-full shrink-0" style={{ backgroundColor: color }} />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-foreground truncate">{selected.name}</p>
            <p className="text-[10px] text-muted-foreground">{selected.party || 'SE'} — {selected.chamber === 'assemblee' ? 'AN' : 'Senat'}</p>
          </div>
          <Button variant="ghost" size="xs" onClick={(e) => { e.stopPropagation(); onSelect(null); }}>
            Changer
          </Button>
        </div>
      ) : (
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
          <Input
            placeholder="Rechercher un politicien..."
            value={searchQ}
            onChange={e => { setSearchQ(e.target.value); setOpen(true); }}
            onFocus={() => setOpen(true)}
            className="pl-8 h-9 text-xs"
          />
          {open && searchQ && filtered.length > 0 && (
            <div className="absolute z-50 top-full mt-1 left-0 right-0 bg-popover border border-border rounded-lg shadow-xl max-h-48 overflow-auto">
              {filtered.map(p => (
                <button
                  key={p.id}
                  className="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-muted/50 transition-colors"
                  onClick={() => { onSelect(p.id); setSearchQ(''); setOpen(false); }}
                >
                  <div className="w-2 h-2 rounded-full" style={{ backgroundColor: PARTY_COLORS[p.party || ''] || DEFAULT_COLOR }} />
                  <span className="text-xs font-medium text-foreground">{p.name}</span>
                  <span className="text-[10px] text-muted-foreground ml-auto">{p.party || 'SE'}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   COMPARISON COLUMN
   ═══════════════════════════════════════════════════════════════════ */

function ComparisonColumn({
  pol,
  positions,
  contradictions,
  posLoading,
  contraLoading,
  color,
}: {
  pol: Pol;
  positions: Pos[];
  contradictions: Contra[];
  posLoading: boolean;
  contraLoading: boolean;
  color: string;
}) {
  const coherenceScore = typeof pol.metadata === 'object' && pol.metadata
    ? (pol.metadata as Record<string, unknown>).coherence_score
    : null;

  return (
    <div className="space-y-4">
      {/* Identity card */}
      <div className="p-3 rounded-lg border border-border/50 bg-muted/10" style={{ borderTopColor: color, borderTopWidth: 3 }}>
        <div className="flex items-center gap-2 mb-2">
          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: color }} />
          <p className="text-sm font-semibold text-foreground">{pol.name}</p>
        </div>
        <div className="grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
          <span className="text-muted-foreground">Parti</span>
          <span className="text-foreground font-medium">{pol.party || 'SE'}</span>
          <span className="text-muted-foreground">Chambre</span>
          <span className="text-foreground">{pol.chamber === 'assemblee' ? 'Assemblee Nationale' : 'Senat'}</span>
          <span className="text-muted-foreground">Role</span>
          <span className="text-foreground truncate">{pol.role || '-'}</span>
          <span className="text-muted-foreground">Circonscription</span>
          <span className="text-foreground truncate">{pol.constituency || '-'}</span>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-2">
        <div className="p-2 rounded-lg bg-muted/20 text-center">
          <p className="text-lg font-bold text-foreground tabular-nums">{pol.position_count ?? 0}</p>
          <p className="text-[10px] text-muted-foreground">Positions</p>
        </div>
        <div className="p-2 rounded-lg bg-red-500/5 text-center">
          <p className="text-lg font-bold text-red-400 tabular-nums">{pol.contradiction_count ?? 0}</p>
          <p className="text-[10px] text-muted-foreground">Contradictions</p>
        </div>
        <div className="p-2 rounded-lg bg-cyan-500/5 text-center">
          <p className="text-lg font-bold text-cyan-400 tabular-nums">
            {coherenceScore != null ? `${Math.round(Number(coherenceScore) * 100)}%` : '-'}
          </p>
          <p className="text-[10px] text-muted-foreground">Coherence</p>
        </div>
      </div>

      {/* Contradictions */}
      {contraLoading ? (
        <LoadingSpinner text="..." />
      ) : contradictions.length > 0 ? (
        <div>
          <p className="text-xs font-semibold text-red-400 uppercase tracking-wider mb-2">
            Contradictions ({contradictions.length})
          </p>
          {contradictions.slice(0, 5).map(c => (
            <div key={c.id} className="mb-2 p-2 rounded-lg bg-red-500/5 border border-red-500/20">
              <div className="flex items-center gap-2 mb-0.5">
                <Badge variant="destructive" className="text-[9px]">{c.severity}</Badge>
                <span className="text-xs font-medium text-foreground truncate">{c.subject}</span>
              </div>
              <p className="text-[10px] text-muted-foreground line-clamp-2">{c.description}</p>
            </div>
          ))}
          {contradictions.length > 5 && (
            <p className="text-[10px] text-muted-foreground italic">+{contradictions.length - 5} autres</p>
          )}
        </div>
      ) : null}

      <Separator />

      {/* Positions */}
      <div>
        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
          Positions recentes ({positions.length})
        </p>
        {posLoading ? (
          <LoadingSpinner text="..." />
        ) : positions.length === 0 ? (
          <p className="text-xs text-muted-foreground">Aucune position.</p>
        ) : (
          positions.slice(0, 10).map(pos => (
            <div key={pos.id} className="mb-2 p-2 rounded-lg border border-border/50 bg-card/50">
              <div className="flex items-center gap-2 mb-0.5">
                <span className="text-[9px] font-mono text-muted-foreground">{pos.date}</span>
                <Badge variant="secondary" className="text-[9px]">{pos.position_type}</Badge>
                {pos.stance && (
                  <span className={`text-[9px] font-semibold ${STANCE_COLOR[pos.stance] || ''}`}>
                    {pos.stance.toUpperCase()}
                  </span>
                )}
              </div>
              <p className="text-[10px] font-medium text-foreground">{pos.subject}</p>
              <p className="text-[10px] text-muted-foreground line-clamp-2 mt-0.5">{pos.position_text}</p>
              {pos.source_url && (
                <a href={pos.source_url} target="_blank" rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-[9px] text-cyan-400 hover:underline mt-0.5">
                  <ExternalLink size={8} /> Source
                </a>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   VOTING ALIGNMENT
   ═══════════════════════════════════════════════════════════════════ */

function VotingAlignment({ posA, posB }: { posA: Pos[]; posB: Pos[] }) {
  const alignment = useMemo(() => {
    // Find common subjects
    const subjectsB = new Map<string, Pos>();
    for (const p of posB) subjectsB.set(p.subject.toLowerCase(), p);

    let same = 0;
    let opposed = 0;
    let total = 0;
    const details: { subject: string; stanceA: string; stanceB: string; match: boolean }[] = [];

    for (const pA of posA) {
      const pB = subjectsB.get(pA.subject.toLowerCase());
      if (pB && pA.stance && pB.stance) {
        total++;
        const match = pA.stance === pB.stance;
        if (match) same++;
        else opposed++;
        details.push({
          subject: pA.subject,
          stanceA: pA.stance,
          stanceB: pB.stance,
          match,
        });
      }
    }

    return { same, opposed, total, details: details.slice(0, 10) };
  }, [posA, posB]);

  if (alignment.total === 0) {
    return (
      <div className="text-center py-4">
        <p className="text-xs text-muted-foreground">Aucun sujet commun avec votes compares.</p>
      </div>
    );
  }

  const pct = Math.round((alignment.same / alignment.total) * 100);

  return (
    <div className="space-y-3">
      {/* Score bar */}
      <div className="flex items-center gap-3">
        <div className="flex-1 h-3 bg-muted rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all"
            style={{
              width: `${pct}%`,
              background: pct > 66 ? '#22c55e' : pct > 33 ? '#eab308' : '#ef4444',
            }}
          />
        </div>
        <span className="text-sm font-bold tabular-nums" style={{
          color: pct > 66 ? '#22c55e' : pct > 33 ? '#eab308' : '#ef4444',
        }}>
          {pct}%
        </span>
      </div>
      <div className="flex justify-between text-[10px] text-muted-foreground">
        <span>{alignment.same} accords</span>
        <span>{alignment.opposed} desaccords</span>
        <span>{alignment.total} sujets communs</span>
      </div>

      {/* Detail table */}
      {alignment.details.length > 0 && (
        <div className="space-y-1">
          {alignment.details.map((d, i) => (
            <div key={i} className={`flex items-center gap-2 px-2 py-1 rounded text-[10px] ${d.match ? 'bg-green-500/5' : 'bg-red-500/5'}`}>
              <span className={`font-medium ${d.match ? 'text-green-400' : 'text-red-400'}`}>
                {d.match ? '=' : '\u2260'}
              </span>
              <span className="flex-1 truncate text-foreground">{d.subject}</span>
              <span className={STANCE_COLOR[d.stanceA] || 'text-muted-foreground'}>{d.stanceA}</span>
              <span className="text-muted-foreground">vs</span>
              <span className={STANCE_COLOR[d.stanceB] || 'text-muted-foreground'}>{d.stanceB}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   MAIN COMPONENT
   ═══════════════════════════════════════════════════════════════════ */

export function ComparateurTab() {
  const [idA, setIdA] = useState<string | null>(null);
  const [idB, setIdB] = useState<string | null>(null);

  const polsQ = useGovPoliticians();
  const pols: Pol[] = Array.isArray(polsQ.data) ? polsQ.data : [];

  // Data for politician A
  const posAQ = useGovPositions(idA);
  const contraAQ = useGovPoliticianContradictions(idA);
  const positionsA: Pos[] = Array.isArray(posAQ.data) ? posAQ.data : [];
  const contradictionsA: Contra[] = Array.isArray(contraAQ.data) ? contraAQ.data : [];

  // Data for politician B
  const posBQ = useGovPositions(idB);
  const contraBQ = useGovPoliticianContradictions(idB);
  const positionsB: Pos[] = Array.isArray(posBQ.data) ? posBQ.data : [];
  const contradictionsB: Contra[] = Array.isArray(contraBQ.data) ? contraBQ.data : [];

  const polA = pols.find(p => p.id === idA);
  const polB = pols.find(p => p.id === idB);

  const colorA = PARTY_COLORS[polA?.party || ''] || '#3b82f6';
  const colorB = PARTY_COLORS[polB?.party || ''] || '#ef4444';

  // Radar comparison
  const compareIds = useMemo(() => {
    const ids: string[] = [];
    if (idA) ids.push(idA);
    if (idB) ids.push(idB);
    return ids;
  }, [idA, idB]);

  return (
    <div className="space-y-3 h-[calc(100vh-380px)] overflow-auto">
      {/* Selectors */}
      <Card>
        <CardContent className="p-4">
          <div className="grid grid-cols-[1fr_auto_1fr] gap-4 items-end">
            <PoliticianSelector
              label="Politicien A"
              selectedId={idA}
              onSelect={setIdA}
              politicians={pols}
              color={colorA}
            />
            <div className="flex items-center justify-center pb-2">
              <ArrowLeftRight className="size-5 text-muted-foreground" />
            </div>
            <PoliticianSelector
              label="Politicien B"
              selectedId={idB}
              onSelect={setIdB}
              politicians={pols}
              color={colorB}
            />
          </div>
        </CardContent>
      </Card>

      {/* Comparison content */}
      {(!polA && !polB) ? (
        <Card className="flex items-center justify-center py-16">
          <div className="text-center">
            <ArrowLeftRight size={40} className="text-muted-foreground mx-auto mb-3" />
            <p className="text-sm text-muted-foreground">Selectionnez deux politiciens a comparer</p>
          </div>
        </Card>
      ) : (
        <>
          {/* Side by side details */}
          <div className="grid grid-cols-2 gap-3">
            {/* Column A */}
            <Card>
              <CardContent className="p-4">
                <ScrollArea className="max-h-[400px]">
                  {polA ? (
                    <ComparisonColumn
                      pol={polA}
                      positions={positionsA}
                      contradictions={contradictionsA}
                      posLoading={posAQ.isLoading}
                      contraLoading={contraAQ.isLoading}
                      color={colorA}
                    />
                  ) : (
                    <div className="flex items-center justify-center py-8">
                      <p className="text-xs text-muted-foreground">Selectionnez un politicien</p>
                    </div>
                  )}
                </ScrollArea>
              </CardContent>
            </Card>

            {/* Column B */}
            <Card>
              <CardContent className="p-4">
                <ScrollArea className="max-h-[400px]">
                  {polB ? (
                    <ComparisonColumn
                      pol={polB}
                      positions={positionsB}
                      contradictions={contradictionsB}
                      posLoading={posBQ.isLoading}
                      contraLoading={contraBQ.isLoading}
                      color={colorB}
                    />
                  ) : (
                    <div className="flex items-center justify-center py-8">
                      <p className="text-xs text-muted-foreground">Selectionnez un politicien</p>
                    </div>
                  )}
                </ScrollArea>
              </CardContent>
            </Card>
          </div>

          {/* Voting alignment */}
          {polA && polB && (
            <Card>
              <CardHeader className="border-b">
                <CardTitle className="flex items-center gap-2">
                  <ArrowLeftRight className="size-4" />
                  Alignement des votes
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4">
                <VotingAlignment posA={positionsA} posB={positionsB} />
              </CardContent>
            </Card>
          )}

          {/* Radar comparison */}
          {compareIds.length > 0 && (
            <Card>
              <CardHeader className="border-b">
                <CardTitle>Profil Radar — Comparaison</CardTitle>
              </CardHeader>
              <CardContent className="p-4">
                <RadarProfile politicians={pols} compareIds={compareIds} />
              </CardContent>
            </Card>
          )}
        </>
      )}
    </div>
  );
}
