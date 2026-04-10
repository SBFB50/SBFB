import { useEffect, useMemo, useRef, useState } from 'react';
import { SigmaContainer, useLoadGraph, useSigma, useRegisterEvents } from '@react-sigma/core';
import '@react-sigma/core/lib/style.css';
import Graph from 'graphology';
import louvain from 'graphology-communities-louvain';
import { degreeCentrality, betweennessCentrality } from 'graphology-metrics/centrality';

// ── Types ─────────────────────────────────────────────────────

interface ApiNode {
  id: string;
  label: string;
  party?: string;
  chamber?: string;
  position_count?: number;
  contradiction_count?: number;
  [k: string]: unknown;
}

interface ApiEdge {
  id: string;
  source: string;
  target: string;
  type?: string;
  [k: string]: unknown;
}

interface AnalyticalGraphProps {
  nodes: ApiNode[];
  edges: ApiEdge[];
  /** Metric to size nodes by */
  sizeMetric?: 'degree' | 'betweenness' | 'positions';
  /** Whether to run Louvain community detection for coloring */
  detectCommunities?: boolean;
  onNodeClick?: (node: ApiNode) => void;
}

// ── Party colors ──────────────────────────────────────────────

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080', 'EELV': '#00c000', 'ECO': '#00c000',
  'RE': '#ffeb00', 'REN': '#ffeb00', 'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#0001B8', 'LR': '#0066cc', 'RN': '#0D378A',
  'LIOT': '#87ceeb', 'SE': '#64748b',
};

const COMMUNITY_PALETTE = [
  '#3b82f6', '#ef4444', '#22c55e', '#eab308', '#a855f7',
  '#06b6d4', '#f97316', '#ec4899', '#14b8a6', '#8b5cf6',
  '#f43f5e', '#0ea5e9', '#84cc16', '#d946ef', '#fbbf24',
];

function getPartyColor(party?: string): string {
  if (!party) return '#64748b';
  const upper = party.toUpperCase().trim();
  return PARTY_COLORS[upper] || '#64748b';
}

// ── Graph loader component ────────────────────────────────────

function GraphLoader({
  nodes,
  edges,
  sizeMetric,
  detectCommunities,
  onNodeClick,
  onMetrics,
}: AnalyticalGraphProps & { onMetrics?: (m: MetricsData) => void }) {
  const loadGraph = useLoadGraph();
  const sigma = useSigma();
  const registerEvents = useRegisterEvents();

  useEffect(() => {
    const graph = new Graph();

    // Add nodes
    const nodeSet = new Set<string>();
    for (const n of nodes) {
      if (nodeSet.has(n.id)) continue;
      nodeSet.add(n.id);
      graph.addNode(n.id, {
        label: n.label || n.id,
        x: Math.random() * 1000,
        y: Math.random() * 1000,
        size: 5,
        color: getPartyColor(n.party),
        party: n.party,
        data: n,
      });
    }

    // Add edges
    for (const e of edges) {
      if (!nodeSet.has(e.source) || !nodeSet.has(e.target)) continue;
      if (e.source === e.target) continue;
      try {
        graph.addEdge(e.source, e.target, {
          color: e.type === 'opposition' ? '#ef444466' : e.type === 'agreement' ? '#22c55e66' : '#37415133',
          size: e.type === 'opposition' ? 1.5 : 0.5,
          type: e.type,
        });
      } catch {
        // Skip duplicate edges
      }
    }

    // Community detection (Louvain)
    if (detectCommunities && graph.order > 0) {
      try {
        const communities = louvain(graph);
        const communitySet = new Set(Object.values(communities));
        const communityColors: Record<string, string> = {};
        let idx = 0;
        for (const c of communitySet) {
          communityColors[String(c)] = COMMUNITY_PALETTE[idx % COMMUNITY_PALETTE.length];
          idx++;
        }
        graph.forEachNode((node) => {
          const community = String(communities[node]);
          graph.setNodeAttribute(node, 'color', communityColors[community]);
          graph.setNodeAttribute(node, 'community', community);
        });

        if (onMetrics) {
          onMetrics({ communities: communitySet.size, modularity: null });
        }
      } catch {
        // Louvain can fail on disconnected graphs
      }
    }

    // Centrality-based sizing
    if (graph.order > 0) {
      try {
        if (sizeMetric === 'degree') {
          const degrees = degreeCentrality(graph);
          const maxDeg = Math.max(...Object.values(degrees), 0.001);
          graph.forEachNode((node) => {
            graph.setNodeAttribute(node, 'size', 4 + (degrees[node] / maxDeg) * 20);
          });
        } else if (sizeMetric === 'betweenness') {
          const bc = betweennessCentrality(graph);
          const maxBc = Math.max(...Object.values(bc), 0.001);
          graph.forEachNode((node) => {
            graph.setNodeAttribute(node, 'size', 4 + (bc[node] / maxBc) * 20);
          });
        } else {
          // positions count
          graph.forEachNode((node) => {
            const data = graph.getNodeAttribute(node, 'data') as ApiNode;
            graph.setNodeAttribute(node, 'size', Math.max(4, Math.min(20, (data?.position_count || 0) * 1.5 + 4)));
          });
        }
      } catch {
        // metrics can fail on empty graphs
      }
    }

    loadGraph(graph);
  }, [nodes, edges, sizeMetric, detectCommunities, loadGraph]);

  // Register click events
  useEffect(() => {
    registerEvents({
      clickNode: (event) => {
        const nodeData = nodes.find(n => n.id === event.node);
        if (nodeData && onNodeClick) onNodeClick(nodeData);
      },
    });
  }, [registerEvents, nodes, onNodeClick]);

  return null;
}

// ── Metrics display ───────────────────────────────────────────

interface MetricsData {
  communities: number;
  modularity: number | null;
}

// ── Main component ────────────────────────────────────────────

export function AnalyticalGraph({
  nodes,
  edges,
  sizeMetric = 'degree',
  detectCommunities = true,
  onNodeClick,
}: AnalyticalGraphProps) {
  const [metrics, setMetrics] = useState<MetricsData | null>(null);

  if (nodes.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-[var(--text-muted)]">
        Aucune donnee pour l'analyse
      </div>
    );
  }

  return (
    <div className="relative w-full h-full">
      <SigmaContainer
        style={{ width: '100%', height: '100%', background: '#0a0a0f' }}
        settings={{
          allowInvalidContainer: true,
          renderEdgeLabels: false,
          defaultEdgeType: 'line',
          labelColor: { color: '#e2e4f0' },
          labelSize: 10,
          labelRenderedSizeThreshold: 8,
          edgeLabelColor: { color: '#565973' },
          edgeLabelSize: 8,
          zoomToSizeRatioFunction: (ratio) => Math.min(1, ratio),
          itemSizesReference: 'positions',
          zoomDuration: 200,
          enableEdgeEvents: true,
        }}
      >
        <GraphLoader
          nodes={nodes}
          edges={edges}
          sizeMetric={sizeMetric}
          detectCommunities={detectCommunities}
          onNodeClick={onNodeClick}
          onMetrics={setMetrics}
        />
      </SigmaContainer>

      {/* Metrics overlay */}
      {metrics && (
        <div className="absolute top-3 left-3 bg-[var(--bg-card)]/90 border border-[var(--border)] rounded-lg px-3 py-2 z-10">
          <p className="text-[10px] text-[var(--text-muted)] uppercase tracking-wider mb-1">Analyse</p>
          <div className="flex gap-4 text-xs">
            <div>
              <span className="text-[var(--text-muted)]">Communautes: </span>
              <span className="text-[var(--text-primary)] font-medium">{metrics.communities}</span>
            </div>
            <div>
              <span className="text-[var(--text-muted)]">Noeuds: </span>
              <span className="text-[var(--text-primary)] font-medium">{nodes.length}</span>
            </div>
            <div>
              <span className="text-[var(--text-muted)]">Liens: </span>
              <span className="text-[var(--text-primary)] font-medium">{edges.length}</span>
            </div>
          </div>
        </div>
      )}

      {/* Legend */}
      <div className="absolute bottom-3 left-3 text-[10px] text-[var(--text-muted)] z-10">
        Taille: {sizeMetric === 'degree' ? 'degre' : sizeMetric === 'betweenness' ? 'centralite' : 'positions'}
        {detectCommunities && ' · Couleur: communautes Louvain'}
      </div>
    </div>
  );
}
