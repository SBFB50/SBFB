import { useState, useMemo } from 'react';
import { Search } from 'lucide-react';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

import { useGovPositions } from '../../hooks/useGovernment';
import { RadarProfile, PartyChord, PartyTreemap } from './PoliticalCharts';
import type { Pol, Pos } from './types';

/* ── Stats Tab ── */

export function StatsTab({ politicians }: { politicians: Pol[] }) {
  const [compareIds, setCompareIds] = useState<string[]>([]);
  const [searchQ, setSearchQ] = useState('');
  const _posQ = useGovPositions(null);
  const allPositions: Pos[] = [];

  // Get all positions for the chord/heatmap (use first 3 politicians' positions as sample)
  const topPols = useMemo(() =>
    [...politicians].sort((a, b) => (b.position_count || 0) - (a.position_count || 0)).slice(0, 3),
    [politicians]
  );

  // Auto-select top 3 for radar comparison
  const effectiveCompare = compareIds.length > 0 ? compareIds : topPols.map(p => p.id);

  const filteredForSearch = searchQ
    ? politicians.filter(p => p.name.toLowerCase().includes(searchQ.toLowerCase()))
    : [];

  return (
    <div className="space-y-4">
      {/* Row 1: Radar + Treemap */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-3">
        <Card>
          <CardHeader className="border-b">
            <CardTitle>Profil Radar — Comparaison</CardTitle>
            <CardAction>
              <div className="flex items-center gap-2">
                <div className="relative">
                  <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3 text-muted-foreground" />
                  <Input placeholder="Ajouter un politicien..." value={searchQ}
                    onChange={e => setSearchQ(e.target.value)} className="pl-7 h-6 w-40 text-[10px]" />
                </div>
                {compareIds.length > 0 && (
                  <Button variant="ghost" size="xs" onClick={() => setCompareIds([])}>Reset</Button>
                )}
              </div>
            </CardAction>
          </CardHeader>
          {searchQ && filteredForSearch.length > 0 && (
            <div className="border-b border-border max-h-32 overflow-auto">
              {filteredForSearch.slice(0, 8).map(p => (
                <button key={p.id} onClick={() => {
                  if (!compareIds.includes(p.id) && compareIds.length < 3) {
                    setCompareIds([...compareIds, p.id]);
                  }
                  setSearchQ('');
                }}
                  className="w-full text-left px-3 py-1 text-xs hover:bg-muted/50 text-foreground">
                  {p.name} <span className="text-muted-foreground">({p.party || 'SE'})</span>
                </button>
              ))}
            </div>
          )}
          <CardContent>
            <RadarProfile politicians={politicians} compareIds={effectiveCompare} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="border-b">
            <CardTitle>Repartition par Parti</CardTitle>
          </CardHeader>
          <CardContent>
            <PartyTreemap politicians={politicians} />
          </CardContent>
        </Card>
      </div>

      {/* Row 2: Chord */}
      <Card>
        <CardHeader className="border-b">
          <CardTitle>Interactions entre Partis — Diagramme de Cordes</CardTitle>
          <CardAction>
            <p className="text-[10px] text-muted-foreground">Sujets communs entre partis</p>
          </CardAction>
        </CardHeader>
        <CardContent>
          <PartyChord politicians={politicians} positions={allPositions} />
        </CardContent>
      </Card>
    </div>
  );
}
