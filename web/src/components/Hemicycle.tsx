import { useMemo, useState, useCallback } from 'react';
import { getParliamentPoints } from 'd3-parliament-chart';

// ── Types ─────────────────────────────────────────────────────

interface Politician {
  id: string;
  name: string;
  party?: string;
  group?: string;
  chamber?: string;
  role?: string;
  photo_url?: string;
}

interface HemicycleProps {
  politicians: Politician[];
  /** Width of the SVG (height = width/2 + padding) */
  width?: number;
  /** Filter by chamber: 'an' | 'senat' | 'all' */
  chamber?: 'an' | 'senat' | 'all';
  /** Callback when a seat is clicked */
  onSeatClick?: (politician: Politician) => void;
  /** Optional: highlight a specific party */
  highlightParty?: string | null;
}

// ── Official French political party colors ────────────────────
// Source: fr.wikipedia.org/wiki/Modèle:Infobox_Parti_politique_français/couleurs

const PARTY_COLORS: Record<string, string> = {
  // Gauche
  'LFI': '#cc2443', 'FI': '#cc2443', 'NFP': '#e4032e',
  'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080',
  'EELV': '#00c000', 'ECO': '#00c000', 'ECOS': '#00c000',
  'DVG': '#ffc0c0',
  // Centre
  'RE': '#ffeb00', 'REN': '#ffeb00', 'ENS': '#FED700',
  'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#0001B8',
  'LIOT': '#87ceeb',
  'UDI': '#00cccc',
  // Droite
  'LR': '#0066cc', 'DR': '#adc1fd',
  'RN': '#0D378A', 'UDR': '#162561',
  'DVD': '#adc1fd',
  // Autres
  'SE': '#888888', 'NI': '#888888',
};

const DEFAULT_SEAT_COLOR = '#4a4a5a';

function getPartyColor(party?: string): string {
  if (!party) return DEFAULT_SEAT_COLOR;
  const upper = party.toUpperCase().trim();
  // Try exact match first
  if (PARTY_COLORS[upper]) return PARTY_COLORS[upper];
  // Try partial match
  for (const [key, color] of Object.entries(PARTY_COLORS)) {
    if (upper.includes(key) || key.includes(upper)) return color;
  }
  return DEFAULT_SEAT_COLOR;
}

// ── Political ordering (left to right) ────────────────────────

const PARTY_ORDER: Record<string, number> = {
  'LFI': 0, 'FI': 0, 'NFP': 1,
  'PCF': 2, 'GDR': 2,
  'PS': 3, 'SOC': 3,
  'EELV': 4, 'ECO': 4, 'ECOS': 4,
  'DVG': 5,
  'LIOT': 6,
  'RE': 7, 'REN': 7, 'ENS': 7,
  'DEM': 8, 'MODEM': 8,
  'HOR': 9,
  'UDI': 10,
  'LR': 11, 'DR': 12,
  'UDR': 13,
  'RN': 14,
  'DVD': 15,
  'SE': 16, 'NI': 16,
};

function getPartyOrder(party?: string): number {
  if (!party) return 99;
  const upper = party.toUpperCase().trim();
  if (PARTY_ORDER[upper] !== undefined) return PARTY_ORDER[upper];
  for (const [key, order] of Object.entries(PARTY_ORDER)) {
    if (upper.includes(key)) return order;
  }
  return 99;
}

// ── Hemicycle component ───────────────────────────────────────

export function Hemicycle({
  politicians,
  width = 800,
  chamber = 'all',
  onSeatClick,
  highlightParty = null,
}: HemicycleProps) {
  const [hoveredSeat, setHoveredSeat] = useState<number | null>(null);
  const [tooltipPos, setTooltipPos] = useState({ x: 0, y: 0 });

  // Filter by chamber
  const filtered = useMemo(() => {
    if (chamber === 'all') return politicians;
    return politicians.filter(p => {
      const ch = (p.chamber || '').toLowerCase();
      if (chamber === 'an') return ch.includes('assembl') || ch === 'an' || ch === '' || !p.chamber;
      if (chamber === 'senat') return ch.includes('sénat') || ch.includes('senat');
      return true;
    });
  }, [politicians, chamber]);

  // Sort left-to-right politically
  const sorted = useMemo(() =>
    [...filtered].sort((a, b) => getPartyOrder(a.party || a.group) - getPartyOrder(b.party || b.group)),
    [filtered]
  );

  // Compute seat positions via d3-parliament-chart
  const seatRadius = Math.max(3, Math.min(6, 400 / Math.sqrt(sorted.length || 1)));
  const rowHeight = seatRadius * 3;
  const height = width / 2 + 40;

  const seats = useMemo(() => {
    if (sorted.length === 0) return [];
    const points = getParliamentPoints(
      sorted.length,
      { sections: 1, sectionGap: 0, seatRadius, rowHeight },
      width - 20, // small padding
    );
    return points.map((pt, i) => ({
      ...pt,
      x: pt.x + 10, // offset for padding
      y: pt.y + 10,
      politician: sorted[i],
      color: getPartyColor(sorted[i]?.party || sorted[i]?.group),
    }));
  }, [sorted, seatRadius, rowHeight, width]);

  // Legend: aggregate by party
  const legend = useMemo(() => {
    const counts: Record<string, { color: string; count: number; label: string }> = {};
    for (const s of seats) {
      const key = s.politician?.party || s.politician?.group || 'Autres';
      if (!counts[key]) counts[key] = { color: s.color, count: 0, label: key };
      counts[key].count++;
    }
    return Object.values(counts).sort((a, b) =>
      getPartyOrder(a.label) - getPartyOrder(b.label)
    );
  }, [seats]);

  const handleMouseMove = useCallback((e: React.MouseEvent, idx: number) => {
    setHoveredSeat(idx);
    const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
    if (rect) {
      setTooltipPos({ x: e.clientX - rect.left, y: e.clientY - rect.top - 10 });
    }
  }, []);

  if (sorted.length === 0) {
    return (
      <div className="flex items-center justify-center h-48 text-[var(--text-muted)] text-sm">
        Aucun politicien a afficher
      </div>
    );
  }

  const hoveredPol = hoveredSeat !== null ? seats[hoveredSeat]?.politician : null;

  return (
    <div className="relative">
      {/* Title bar */}
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-[var(--text-primary)]">
          {chamber === 'senat' ? 'Senat' : chamber === 'an' ? 'Assemblee Nationale' : 'Hemicycle'}
          <span className="ml-2 text-[var(--text-muted)] font-normal">{sorted.length} sieges</span>
        </h3>
      </div>

      {/* SVG Hemicycle */}
      <svg
        viewBox={`0 0 ${width} ${height}`}
        width="100%"
        style={{ maxWidth: width }}
        className="mx-auto"
      >
        {/* Background arc guides */}
        <defs>
          <radialGradient id="hemicycle-bg" cx="50%" cy="100%" r="50%">
            <stop offset="0%" stopColor="rgba(59,130,246,0.03)" />
            <stop offset="100%" stopColor="transparent" />
          </radialGradient>
        </defs>
        <rect x="0" y="0" width={width} height={height} fill="transparent" />
        <ellipse
          cx={width / 2}
          cy={height - 10}
          rx={width / 2 - 15}
          ry={height - 25}
          fill="url(#hemicycle-bg)"
          stroke="rgba(255,255,255,0.04)"
          strokeWidth="1"
        />

        {/* Speaker podium */}
        <rect
          x={width / 2 - 20}
          y={height - 16}
          width={40}
          height={12}
          rx={3}
          fill="rgba(59,130,246,0.15)"
          stroke="rgba(59,130,246,0.3)"
          strokeWidth="1"
        />
        <text
          x={width / 2}
          y={height - 7}
          textAnchor="middle"
          fontSize="6"
          fill="rgba(59,130,246,0.6)"
        >
          President
        </text>

        {/* Seats */}
        {seats.map((seat, i) => {
          const isHighlighted = highlightParty
            ? (seat.politician?.party || seat.politician?.group || '').toUpperCase().includes(highlightParty.toUpperCase())
            : true;
          const isHovered = hoveredSeat === i;

          return (
            <circle
              key={i}
              cx={seat.x}
              cy={seat.y}
              r={isHovered ? seatRadius * 1.4 : seatRadius}
              fill={seat.color}
              opacity={isHighlighted ? (isHovered ? 1 : 0.85) : 0.15}
              stroke={isHovered ? '#fff' : 'rgba(0,0,0,0.3)'}
              strokeWidth={isHovered ? 1.5 : 0.5}
              className="cursor-pointer transition-all duration-150"
              onMouseMove={(e) => handleMouseMove(e, i)}
              onMouseLeave={() => setHoveredSeat(null)}
              onClick={() => onSeatClick?.(seat.politician)}
            />
          );
        })}
      </svg>

      {/* Tooltip */}
      {hoveredPol && (
        <div
          className="absolute pointer-events-none bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 shadow-xl z-50"
          style={{
            left: tooltipPos.x,
            top: tooltipPos.y,
            transform: 'translate(-50%, -100%)',
          }}
        >
          <p className="text-xs font-semibold text-[var(--text-primary)]">{hoveredPol.name}</p>
          <p className="text-[10px] text-[var(--text-muted)]">
            {hoveredPol.party || hoveredPol.group || 'Sans etiquette'}
            {hoveredPol.role && ` — ${hoveredPol.role}`}
          </p>
        </div>
      )}

      {/* Legend */}
      <div className="flex flex-wrap gap-x-3 gap-y-1 mt-3 justify-center">
        {legend.map(({ label, color, count }) => (
          <button
            key={label}
            className="flex items-center gap-1.5 text-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            onClick={() => {/* could trigger highlightParty */}}
          >
            <div className="w-2.5 h-2.5 rounded-full shrink-0" style={{ backgroundColor: color }} />
            <span>{label}</span>
            <span className="text-[var(--text-muted)]">({count})</span>
          </button>
        ))}
      </div>
    </div>
  );
}
