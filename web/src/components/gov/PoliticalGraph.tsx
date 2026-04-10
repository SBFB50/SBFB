import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { Graph, type NodeData, type EdgeData, type ComboData } from '@antv/g6';

// ── Types ─────────────────────────────────────────────────────

interface ApiNode {
  id: string;
  label: string;
  party?: string;
  chamber?: string;
  role?: string;
  position_count?: number;
  contradiction_count?: number;
  constituency?: string;
  [k: string]: unknown;
}

interface ApiEdge {
  id: string;
  source: string;
  target: string;
  type?: string;
  label?: string;
  subject?: string;
  stance_a?: string;
  stance_b?: string;
  [k: string]: unknown;
}

interface PoliticalGraphProps {
  nodes: ApiNode[];
  edges: ApiEdge[];
  loading?: boolean;
  /** Group nodes by party into combos */
  groupByParty?: boolean;
  /** Filter edge types: 'all' | 'opposition' | 'agreement' | 'party' */
  relationFilter?: string;
  /** Selected node ID */
  selectedId?: string | null;
  onNodeClick?: (node: ApiNode) => void;
  onEdgeClick?: (edge: ApiEdge) => void;
}

// ── Colors ────────────────────────────────────────────────────

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'NFP': '#e4032e',
  'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080',
  'EELV': '#00c000', 'ECO': '#00c000', 'ECOS': '#00c000',
  'RE': '#ffeb00', 'REN': '#ffeb00', 'ENS': '#FED700',
  'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#0001B8', 'LR': '#0066cc',
  'RN': '#0D378A', 'UDR': '#162561',
  'LIOT': '#87ceeb', 'UDI': '#00cccc',
  'SE': '#64748b', 'NI': '#64748b',
};

const EDGE_COLORS: Record<string, string> = {
  opposition: '#ef4444',
  agreement: '#22c55e',
  party: '#374151',
};

function getPartyColor(party?: string): string {
  if (!party) return '#64748b';
  const upper = party.toUpperCase().trim();
  if (PARTY_COLORS[upper]) return PARTY_COLORS[upper];
  for (const [key, color] of Object.entries(PARTY_COLORS)) {
    if (upper.includes(key)) return color;
  }
  return '#64748b';
}

// ── Component ─────────────────────────────────────────────────

export function PoliticalGraph({
  nodes: apiNodes,
  edges: apiEdges,
  loading = false,
  groupByParty = false,
  relationFilter = 'all',
  selectedId = null,
  onNodeClick,
  onEdgeClick,
}: PoliticalGraphProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);
  const [ready, setReady] = useState(false);

  // Build G6 data
  const { g6Nodes, g6Edges, g6Combos } = useMemo(() => {
    // Nodes
    const nodeSet = new Set(apiNodes.map(n => n.id));
    const g6Nodes: NodeData[] = apiNodes.map(n => {
      const color = getPartyColor(n.party);
      const size = Math.max(16, Math.min(40, (n.position_count || 0) * 2 + 16));
      return {
        id: n.id,
        data: { ...n, color, nodeSize: size },
        combo: groupByParty ? (n.party || 'Autres') : undefined,
        style: {
          size,
          fill: color,
          stroke: selectedId === n.id ? '#fff' : 'rgba(0,0,0,0.3)',
          lineWidth: selectedId === n.id ? 2 : 0.5,
          labelText: n.label || n.id,
          labelFill: '#e2e4f0',
          labelFontSize: 10,
          labelPlacement: 'bottom' as const,
          labelOffsetY: 4,
          iconSrc: undefined,
          shadowColor: selectedId === n.id ? color : undefined,
          shadowBlur: selectedId === n.id ? 12 : 0,
        },
      };
    });

    // Edges (filtered)
    const g6Edges: EdgeData[] = apiEdges
      .filter(e => {
        if (relationFilter !== 'all' && e.type !== relationFilter) return false;
        return nodeSet.has(e.source) && nodeSet.has(e.target);
      })
      .map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        data: { ...e },
        style: {
          stroke: EDGE_COLORS[e.type || ''] || '#374151',
          lineWidth: e.type === 'opposition' ? 1.5 : 0.8,
          opacity: e.type === 'party' ? 0.15 : 0.5,
          labelText: e.type === 'party' ? '' : (e.label || ''),
          labelFill: '#8b8fa8',
          labelFontSize: 8,
          endArrow: false,
        },
      }));

    // Combos (party groups)
    const g6Combos: ComboData[] = [];
    if (groupByParty) {
      const parties = new Set(apiNodes.map(n => n.party || 'Autres'));
      for (const party of parties) {
        const color = getPartyColor(party);
        g6Combos.push({
          id: party,
          data: { label: party },
          style: {
            fill: `${color}10`,
            stroke: `${color}40`,
            lineWidth: 1,
            labelText: party,
            labelFill: color,
            labelFontSize: 12,
            labelFontWeight: 600,
            collapsedSize: 40,
            collapsedFill: color,
            collapsedStroke: `${color}80`,
          },
        });
      }
    }

    return { g6Nodes, g6Edges, g6Combos };
  }, [apiNodes, apiEdges, groupByParty, relationFilter, selectedId]);

  // Initialize graph
  useEffect(() => {
    if (!containerRef.current || apiNodes.length === 0) return;

    const container = containerRef.current;
    const width = container.offsetWidth;
    const height = container.offsetHeight;

    if (graphRef.current) {
      graphRef.current.destroy();
      graphRef.current = null;
    }

    const graph = new Graph({
      container,
      width,
      height,
      autoFit: 'view',
      padding: 20,
      data: {
        nodes: g6Nodes,
        edges: g6Edges,
        combos: g6Combos,
      },
      node: {
        type: 'circle',
        style: {
          size: (d: NodeData) => (d.style?.size as number) || 20,
          fill: (d: NodeData) => (d.style?.fill as string) || '#64748b',
          stroke: (d: NodeData) => (d.style?.stroke as string) || 'rgba(0,0,0,0.3)',
          lineWidth: (d: NodeData) => (d.style?.lineWidth as number) || 0.5,
          labelText: (d: NodeData) => (d.style?.labelText as string) || '',
          labelFill: '#e2e4f0',
          labelFontSize: 10,
          labelPlacement: 'bottom',
          labelOffsetY: 4,
        },
        state: {
          active: { stroke: '#3b82f6', lineWidth: 2, shadowColor: '#3b82f6', shadowBlur: 10 },
          selected: { stroke: '#fff', lineWidth: 2.5, shadowColor: '#fff', shadowBlur: 15 },
          inactive: { opacity: 0.2 },
        },
      },
      edge: {
        type: 'line',
        style: {
          stroke: (d: EdgeData) => (d.style?.stroke as string) || '#374151',
          lineWidth: (d: EdgeData) => (d.style?.lineWidth as number) || 0.8,
          opacity: (d: EdgeData) => (d.style?.opacity as number) || 0.5,
          labelText: (d: EdgeData) => (d.style?.labelText as string) || '',
          labelFill: '#8b8fa8',
          labelFontSize: 8,
          endArrow: false,
        },
        state: {
          active: { stroke: '#06b6d4', lineWidth: 2, opacity: 1 },
          inactive: { opacity: 0.05 },
        },
      },
      combo: {
        type: 'circle',
        style: {
          fill: (d: ComboData) => (d.style?.fill as string) || 'rgba(255,255,255,0.03)',
          stroke: (d: ComboData) => (d.style?.stroke as string) || 'rgba(255,255,255,0.1)',
          lineWidth: 1,
          labelText: (d: ComboData) => (d.style?.labelText as string) || '',
          labelFill: (d: ComboData) => (d.style?.labelFill as string) || '#8b8fa8',
          labelFontSize: 12,
          labelPlacement: 'top',
        },
      },
      layout: groupByParty
        ? {
            type: 'combo-combined',
            comboPadding: 20,
            outerLayout: { type: 'circular' },
            innerLayout: { type: 'd3-force', preventOverlap: true },
          }
        : {
            type: 'd3-force',
            preventOverlap: true,
            nodeSize: 30,
            linkDistance: 80,
            nodeStrength: -200,
            collide: { radius: 20 },
          },
      behaviors: [
        'zoom-canvas',
        'drag-canvas',
        'drag-element',
        { type: 'click-select', multiple: false },
        {
          type: 'hover-activate',
          degree: 1,
          inactiveState: 'inactive',
          state: 'active',
        },
        ...(groupByParty ? [{ type: 'collapse-expand' }] : []),
      ],
      animation: true,
      background: '#0a0a0f',
    });

    graph.on('node:click', (event: any) => {
      const nodeData = apiNodes.find(n => n.id === event.target?.id);
      if (nodeData && onNodeClick) onNodeClick(nodeData);
    });

    graph.on('edge:click', (event: any) => {
      const edgeData = apiEdges.find(e => e.id === event.target?.id);
      if (edgeData && onEdgeClick) onEdgeClick(edgeData);
    });

    graph.render().then(() => {
      setReady(true);
    });

    graphRef.current = graph;

    // Resize observer
    const ro = new ResizeObserver(entries => {
      for (const entry of entries) {
        const { width: w, height: h } = entry.contentRect;
        if (w > 0 && h > 0) graph.resize(w, h);
      }
    });
    ro.observe(container);

    return () => {
      ro.disconnect();
      graph.destroy();
      graphRef.current = null;
      setReady(false);
    };
  }, [g6Nodes, g6Edges, g6Combos, groupByParty]);

  // Methods exposed via ref
  const fitView = useCallback(() => {
    graphRef.current?.fitView();
  }, []);

  const exportPNG = useCallback(async () => {
    if (!graphRef.current) return;
    const url = await graphRef.current.toDataURL('image/png');
    const a = document.createElement('a');
    a.download = 'reseau-politique.png';
    a.href = url;
    a.click();
  }, []);

  return (
    <div className="relative w-full h-full">
      <div ref={containerRef} className="w-full h-full" />

      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-[var(--bg-primary)]/80 z-10">
          <div className="flex items-center gap-2 text-sm text-[var(--text-muted)]">
            <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
            Chargement du graphe...
          </div>
        </div>
      )}

      {/* Floating controls */}
      {ready && (
        <div className="absolute bottom-3 right-3 flex gap-1.5 z-10">
          <button
            onClick={fitView}
            className="px-2 py-1 bg-[var(--bg-card)] border border-[var(--border)] rounded text-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            title="Recentrer"
          >
            Recentrer
          </button>
          <button
            onClick={exportPNG}
            className="px-2 py-1 bg-[var(--bg-card)] border border-[var(--border)] rounded text-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            title="Exporter PNG"
          >
            PNG
          </button>
        </div>
      )}

      {/* Stats */}
      {ready && (
        <div className="absolute top-3 left-3 text-[10px] text-[var(--text-muted)] z-10">
          {apiNodes.length} noeuds · {g6Edges.length} liens
          {groupByParty && ` · ${g6Combos.length} groupes`}
        </div>
      )}
    </div>
  );
}
