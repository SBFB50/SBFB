import { useMemo } from 'react';
import { ResponsiveRadar } from '@nivo/radar';
import { ResponsiveChord } from '@nivo/chord';
import { ResponsiveHeatMap } from '@nivo/heatmap';
import { ResponsiveTreeMap } from '@nivo/treemap';

// ── Types ─────────────────────────────────────────────────────

interface Politician {
  id: string;
  name: string;
  party?: string;
  position_count?: number;
  contradiction_count?: number;
  [k: string]: unknown;
}

interface Position {
  id: string;
  politician_id?: string;
  stance?: string;
  subject?: string;
  [k: string]: unknown;
}

// ── Shared theme ──────────────────────────────────────────────

const nivoTheme = {
  background: 'transparent',
  text: { fill: '#8b8fa8', fontSize: 10 },
  axis: {
    ticks: { text: { fill: '#565973', fontSize: 9 } },
    legend: { text: { fill: '#8b8fa8', fontSize: 11 } },
  },
  grid: { line: { stroke: 'rgba(255,255,255,0.05)' } },
  legends: { text: { fill: '#8b8fa8', fontSize: 10 } },
  labels: { text: { fill: '#e2e4f0', fontSize: 10 } },
  tooltip: {
    container: {
      background: '#1a1d35',
      color: '#e2e4f0',
      fontSize: 12,
      borderRadius: 8,
      border: '1px solid rgba(255,255,255,0.08)',
      boxShadow: '0 4px 20px rgba(0,0,0,0.5)',
    },
  },
};

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'PCF': '#dd0000', 'PS': '#ff8080',
  'EELV': '#00c000', 'RE': '#ffeb00', 'DEM': '#ff9900',
  'HOR': '#0001B8', 'LR': '#0066cc', 'RN': '#0D378A',
  'LIOT': '#87ceeb', 'SE': '#64748b',
};

// ── 1. Politician Radar Profile ───────────────────────────────

interface RadarProfileProps {
  politicians: Politician[];
  /** IDs of politicians to compare (max 3) */
  compareIds: string[];
}

export function RadarProfile({ politicians, compareIds }: RadarProfileProps) {
  const data = useMemo(() => {
    const selected = politicians.filter(p => compareIds.includes(p.id));
    if (selected.length === 0) return [];

    // Compute relative metrics
    const maxPos = Math.max(...politicians.map(p => p.position_count || 0), 1);
    const maxContra = Math.max(...politicians.map(p => p.contradiction_count || 0), 1);

    const axes = ['Activite', 'Positions', 'Contradictions', 'Visibilite', 'Influence'];
    return axes.map(axis => {
      const row: Record<string, unknown> = { metric: axis };
      for (const p of selected) {
        const posNorm = ((p.position_count || 0) / maxPos) * 100;
        const contraNorm = ((p.contradiction_count || 0) / maxContra) * 100;
        switch (axis) {
          case 'Activite': row[p.name] = posNorm; break;
          case 'Positions': row[p.name] = Math.min(100, (p.position_count || 0) * 5); break;
          case 'Contradictions': row[p.name] = contraNorm; break;
          case 'Visibilite': row[p.name] = Math.min(100, posNorm * 0.7 + contraNorm * 0.3); break;
          case 'Influence': row[p.name] = Math.min(100, posNorm * 0.5 + 20); break;
        }
      }
      return row;
    });
  }, [politicians, compareIds]);

  const keys = useMemo(() =>
    politicians.filter(p => compareIds.includes(p.id)).map(p => p.name),
    [politicians, compareIds]
  );

  if (data.length === 0 || keys.length === 0) {
    return <EmptyChart message="Selectionnez des politiciens a comparer" />;
  }

  return (
    <div style={{ height: 350 }}>
      <ResponsiveRadar
        data={data}
        keys={keys}
        indexBy="metric"
        maxValue={100}
        margin={{ top: 40, right: 80, bottom: 40, left: 80 }}
        curve="linearClosed"
        borderWidth={2}
        borderColor={{ from: 'color' }}
        gridLevels={5}
        gridShape="circular"
        gridLabelOffset={16}
        enableDots
        dotSize={6}
        dotColor={{ theme: 'background' }}
        dotBorderWidth={2}
        dotBorderColor={{ from: 'color' }}
        fillOpacity={0.15}
        blendMode="normal"
        colors={['#3b82f6', '#ef4444', '#22c55e']}
        theme={nivoTheme}
        legends={[{
          anchor: 'top-left', direction: 'column', translateX: -60, translateY: -30,
          itemWidth: 100, itemHeight: 16, symbolSize: 10, symbolShape: 'circle',
        }]}
      />
    </div>
  );
}

// ── 2. Cross-Party Chord Diagram ──────────────────────────────

interface ChordProps {
  politicians: Politician[];
  positions: Position[];
}

export function PartyChord({ politicians, positions }: ChordProps) {
  const { matrix, parties } = useMemo(() => {
    // Group politicians by party
    const polByParty: Record<string, Set<string>> = {};
    for (const p of politicians) {
      const party = p.party || 'SE';
      if (!polByParty[party]) polByParty[party] = new Set();
      polByParty[party].add(p.id);
    }

    const parties = Object.keys(polByParty).sort();
    if (parties.length < 2) return { matrix: [], parties: [] };

    // Build matrix: shared subjects between parties
    const subjectParties: Record<string, Set<string>> = {};
    for (const pos of positions) {
      if (!pos.subject || !pos.politician_id) continue;
      const pol = politicians.find(p => p.id === pos.politician_id);
      if (!pol) continue;
      const party = pol.party || 'SE';
      if (!subjectParties[pos.subject]) subjectParties[pos.subject] = new Set();
      subjectParties[pos.subject].add(party);
    }

    // Count co-occurrences
    const matrix = parties.map(() => parties.map(() => 0));
    for (const partiesOnSubject of Object.values(subjectParties)) {
      const arr = [...partiesOnSubject];
      for (let i = 0; i < arr.length; i++) {
        for (let j = i + 1; j < arr.length; j++) {
          const ii = parties.indexOf(arr[i]);
          const jj = parties.indexOf(arr[j]);
          if (ii >= 0 && jj >= 0) {
            matrix[ii][jj]++;
            matrix[jj][ii]++;
          }
        }
      }
    }

    return { matrix, parties };
  }, [politicians, positions]);

  if (parties.length < 2) {
    return <EmptyChart message="Pas assez de partis pour le diagramme" />;
  }

  return (
    <div style={{ height: 400 }}>
      <ResponsiveChord
        data={matrix}
        keys={parties}
        margin={{ top: 30, right: 30, bottom: 30, left: 30 }}
        padAngle={0.04}
        innerRadiusRatio={0.9}
        innerRadiusOffset={0.02}
        arcOpacity={0.85}
        arcBorderWidth={1}
        arcBorderColor={{ from: 'color', modifiers: [['darker', 0.4]] }}
        ribbonOpacity={0.4}
        ribbonBorderWidth={0}
        enableLabel
        label="id"
        labelOffset={14}
        labelRotation={-90}
        labelTextColor={{ from: 'color', modifiers: [['brighter', 1]] }}
        colors={(d: { id: string }) => PARTY_COLORS[d.id] || '#64748b'}
        theme={nivoTheme}
        legends={[{
          anchor: 'bottom', direction: 'row', translateY: 30,
          itemWidth: 60, itemHeight: 14, symbolSize: 10,
        }]}
      />
    </div>
  );
}

// ── 3. Party Treemap ──────────────────────────────────────────

interface TreemapProps {
  politicians: Politician[];
}

export function PartyTreemap({ politicians }: TreemapProps) {
  const data = useMemo(() => {
    const groups: Record<string, number> = {};
    for (const p of politicians) {
      const party = p.party || 'Sans etiquette';
      groups[party] = (groups[party] || 0) + 1;
    }

    return {
      name: 'Parlement',
      children: Object.entries(groups)
        .sort((a, b) => b[1] - a[1])
        .map(([name, count]) => ({
          name,
          count,
          color: PARTY_COLORS[name] || '#64748b',
        })),
    };
  }, [politicians]);

  if (data.children.length === 0) {
    return <EmptyChart message="Aucun politicien" />;
  }

  return (
    <div style={{ height: 300 }}>
      <ResponsiveTreeMap
        data={data}
        identity="name"
        value="count"
        margin={{ top: 0, right: 0, bottom: 0, left: 0 }}
        tile="squarify"
        innerPadding={2}
        outerPadding={2}
        label={(node: any) => `${node.id} (${node.formattedValue})`}
        labelSkipSize={40}
        labelTextColor="#e2e4f0"
        borderWidth={1}
        borderColor="rgba(0,0,0,0.3)"
        colors={(node: any) => node.data.color || '#64748b'}
        nodeOpacity={0.85}
        theme={nivoTheme}
      />
    </div>
  );
}

// ── 4. Participation Heatmap ──────────────────────────────────

interface HeatmapProps {
  politicians: Politician[];
  positions: Position[];
  /** Number of top politicians to show */
  topN?: number;
}

export function ParticipationHeatmap({ politicians, positions, topN = 15 }: HeatmapProps) {
  const data = useMemo(() => {
    // Count positions per politician per month
    const polCounts: Record<string, Record<string, number>> = {};
    const months = new Set<string>();

    for (const pos of positions) {
      if (!pos.politician_id) continue;
      const date = (pos as any).date || (pos as any).created_at;
      if (!date) continue;
      const month = date.substring(0, 7); // YYYY-MM
      months.add(month);
      if (!polCounts[pos.politician_id]) polCounts[pos.politician_id] = {};
      polCounts[pos.politician_id][month] = (polCounts[pos.politician_id][month] || 0) + 1;
    }

    const sortedMonths = [...months].sort();
    if (sortedMonths.length === 0) return [];

    // Top N politicians by total positions
    const topPols = politicians
      .filter(p => polCounts[p.id])
      .sort((a, b) => (Object.values(polCounts[b.id] || {}).reduce((s, v) => s + v, 0)) -
                      (Object.values(polCounts[a.id] || {}).reduce((s, v) => s + v, 0)))
      .slice(0, topN);

    return topPols.map(p => ({
      id: p.name.length > 20 ? p.name.substring(0, 18) + '...' : p.name,
      data: sortedMonths.map(month => ({
        x: month,
        y: polCounts[p.id]?.[month] || 0,
      })),
    }));
  }, [politicians, positions, topN]);

  if (data.length === 0) {
    return <EmptyChart message="Pas assez de donnees temporelles" />;
  }

  return (
    <div style={{ height: Math.max(300, data.length * 28 + 60) }}>
      <ResponsiveHeatMap
        data={data}
        margin={{ top: 40, right: 20, bottom: 20, left: 140 }}
        axisTop={{
          tickSize: 0,
          tickPadding: 5,
          tickRotation: -45,
        }}
        axisLeft={{
          tickSize: 0,
          tickPadding: 8,
        }}
        colors={{
          type: 'sequential',
          scheme: 'blues',
          minValue: 0,
        }}
        emptyColor="#1a1d35"
        borderWidth={1}
        borderColor="rgba(0,0,0,0.2)"
        labelTextColor={{ from: 'color', modifiers: [['darker', 3]] }}
        theme={nivoTheme}
      />
    </div>
  );
}

// ── Empty state ───────────────────────────────────────────────

function EmptyChart({ message }: { message: string }) {
  return (
    <div className="flex items-center justify-center h-48 text-sm text-[var(--text-muted)]">
      {message}
    </div>
  );
}
