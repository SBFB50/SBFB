import type { LayoutTypes } from 'reagraph';

/* ═══════════════════════════════════════════════════════════════════
   SHARED TYPES — Government Module
   ═══════════════════════════════════════════════════════════════════ */

export interface Pol {
  id: string; name: string; party?: string; chamber?: string; role?: string;
  constituency?: string; official_url?: string; hatvp_url?: string;
  position_count?: number; contradiction_count?: number; [k: string]: unknown;
}

export interface Pos {
  id: string; date: string; position_type: string; stance?: string;
  subject: string; position_text: string; source_url?: string; [k: string]: unknown;
}

export interface Contra {
  id: string; politician_id: string; subject: string; description: string;
  severity: string; position_a_id: string; position_b_id: string; [k: string]: unknown;
}

export interface ApiNode {
  id: string; label: string; party?: string; chamber?: string; role?: string;
  position_count?: number; contradiction_count?: number; constituency?: string;
  official_url?: string; hatvp_url?: string; [k: string]: unknown;
}

export interface ApiEdge {
  id: string; source: string; target: string; type?: string; label?: string;
  subject?: string; stance_a?: string; stance_b?: string; [k: string]: unknown;
}

export interface PressArticle {
  id: string; title: string; source_name?: string; published_at?: string;
  sentiment?: string; summary?: string; url?: string; [k: string]: unknown;
}

export interface SocialPost {
  id: string; platform: string; content: string; posted_at?: string;
  url?: string; [k: string]: unknown;
}

export interface Transcription {
  id: string; title?: string; transcription?: string;
  duration_seconds?: number; source_url?: string; model_used?: string; [k: string]: unknown;
}

export interface GovAlert {
  id: string; title: string; description?: string; alert_type?: string;
  severity?: string; is_read?: boolean; created_at?: string; [k: string]: unknown;
}

export interface GovWorkerStatus {
  name: string; status?: string; events_processed?: number;
  events_errored?: number; [k: string]: unknown;
}

export interface Affair {
  id: string; title: string; status?: string; description?: string;
  start_date?: string; severity?: string; [k: string]: unknown;
}

export interface Declaration {
  id: string; type?: string; year?: number; patrimony_total?: number;
  interests?: string; url?: string; [k: string]: unknown;
}

/* ═══════════════════════════════════════════════════════════════════
   SHARED CONSTANTS
   ═══════════════════════════════════════════════════════════════════ */

export const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080', 'EELV': '#00c000', 'ECO': '#00c000',
  'RE': '#ffcc00', 'REN': '#ffcc00', 'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#00bfff', 'LR': '#0066cc', 'UDI': '#00cccc', 'LIOT': '#87ceeb',
  'RN': '#0d2244', 'SE': '#64748b',
};

export const DEFAULT_COLOR = '#64748b';

export const LEGEND_PARTIES = [
  { key: 'LFI', label: 'LFI', color: '#cc2443' },
  { key: 'PCF', label: 'PCF / GDR', color: '#dd0000' },
  { key: 'PS', label: 'PS', color: '#ff8080' },
  { key: 'EELV', label: 'EELV', color: '#00c000' },
  { key: 'RE', label: 'Renaissance', color: '#ffcc00' },
  { key: 'DEM', label: 'MoDem', color: '#ff9900' },
  { key: 'HOR', label: 'Horizons', color: '#00bfff' },
  { key: 'LR', label: 'LR', color: '#0066cc' },
  { key: 'RN', label: 'RN', color: '#0d2244' },
  { key: 'SE', label: 'Autres', color: '#64748b' },
];

export const EDGE_COLORS: Record<string, string> = {
  opposition: '#ef4444', agreement: '#22c55e', party: '#374151',
};

export const LAYOUTS: { value: LayoutTypes; label: string }[] = [
  { value: 'forceDirected2d', label: 'Force' },
  { value: 'circular2d', label: 'Circulaire' },
  { value: 'hierarchicalTd', label: 'Hierarchique' },
  { value: 'radialOut2d', label: 'Radial' },
];

export const REAGRAPH_THEME = {
  canvas: { background: '#0a0a0f' },
  node: { fill: '#3b82f6', activeFill: '#06b6d4', opacity: 1, selectedOpacity: 1, inactiveOpacity: 0.2,
    label: { color: '#e4e4e7', activeColor: '#fff', stroke: '#0a0a0f' },
    subLabel: { color: '#a1a1aa', activeColor: '#e4e4e7' } },
  ring: { fill: '#06b6d4', activeFill: '#22d3ee' },
  edge: { fill: '#27272a', activeFill: '#06b6d4', opacity: 1, selectedOpacity: 1, inactiveOpacity: 0.1,
    label: { color: '#71717a', activeColor: '#a1a1aa', stroke: '#0a0a0f' } },
  arrow: { fill: '#52525b', activeFill: '#06b6d4' },
  lasso: { border: '1px solid #06b6d4', background: 'rgba(6,182,212,0.1)' },
  cluster: { stroke: '#374151', fill: '#1e1e2e', opacity: 0.15, selectedOpacity: 0.3, inactiveOpacity: 0.05,
    label: { color: '#a1a1aa' } },
};

export const STANCE_COLOR: Record<string, string> = {
  pour: 'text-green-400', contre: 'text-red-400', abstention: 'text-yellow-400',
};
