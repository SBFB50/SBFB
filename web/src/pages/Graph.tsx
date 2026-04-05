import { useState, useCallback, useRef, useEffect } from 'react';
import { Network, Maximize2, RotateCcw } from 'lucide-react';
import Card from '../components/Card';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useGraph } from '../hooks/useApi';

// Lazy-load the graph component to handle SSR / missing canvas gracefully
import ForceGraph2D from 'react-force-graph-2d';

const nodeColorMap: Record<string, string> = {
  person: '#3b82f6',
  location: '#22c55e',
  organization: '#a855f7',
  event: '#eab308',
  vehicle: '#f97316',
  weapon: '#ef4444',
  phone: '#06b6d4',
  email: '#6366f1',
  evidence: '#64748b',
  hypothesis: '#f59e0b',
  document: '#d97706',
};

interface GraphNode {
  id: string;
  name?: string;
  label?: string;
  type?: string;
  group?: string;
  val?: number;
  x?: number;
  y?: number;
}

interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  label?: string;
  type?: string;
  weight?: number;
}

interface SelectedNode extends GraphNode {}

export default function Graph() {
  const { caseId } = useCaseStore();
  const graphQuery = useGraph();
  const graphRef = useRef<{ zoomToFit: (ms?: number) => void } | null>(null);
  const [selectedNode, setSelectedNode] = useState<SelectedNode | null>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        setDimensions({
          width: containerRef.current.offsetWidth,
          height: containerRef.current.offsetHeight,
        });
      }
    };
    updateSize();
    window.addEventListener('resize', updateSize);
    return () => window.removeEventListener('resize', updateSize);
  }, []);

  const handleNodeClick = useCallback((node: GraphNode) => {
    setSelectedNode(node);
  }, []);

  const handleZoomFit = useCallback(() => {
    if (graphRef.current) {
      graphRef.current.zoomToFit(400);
    }
  }, []);

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Network size={48} className="text-[var(--text-muted)] mb-4" />
        <p className="text-[var(--text-secondary)]">Select a case to view the knowledge graph.</p>
      </div>
    );
  }

  const rawData = graphQuery.data;
  const nodes: GraphNode[] = rawData?.nodes || rawData?.vertices || [];
  const links: GraphLink[] = (rawData?.links || rawData?.edges || []).map((l: GraphLink & { from?: string; to?: string }) => ({
    ...l,
    source: l.source || l.from || '',
    target: l.target || l.to || '',
  }));

  const graphData = { nodes, links };

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Knowledge Graph</h2>
          <p className="text-sm text-[var(--text-muted)]">
            {nodes.length} nodes, {links.length} relationships
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleZoomFit}
            className="flex items-center gap-2 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-sm hover:bg-[var(--bg-hover)] transition-colors"
          >
            <Maximize2 size={14} /> Fit
          </button>
          <button
            onClick={() => graphQuery.refetch()}
            className="flex items-center gap-2 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] rounded-lg text-sm hover:bg-[var(--bg-hover)] transition-colors"
          >
            <RotateCcw size={14} /> Refresh
          </button>
        </div>
      </div>

      <div className="flex-1 flex gap-4 min-h-0">
        {/* Graph canvas */}
        <div
          ref={containerRef}
          className="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg overflow-hidden relative"
        >
          {graphQuery.isLoading ? (
            <div className="absolute inset-0 flex items-center justify-center">
              <LoadingSpinner text="Loading graph..." />
            </div>
          ) : nodes.length === 0 ? (
            <div className="absolute inset-0 flex items-center justify-center">
              <p className="text-sm text-[var(--text-muted)]">No graph data yet. Add evidence and run analysis.</p>
            </div>
          ) : (
            <ForceGraph2D
              ref={graphRef as React.MutableRefObject<never>}
              graphData={graphData}
              width={dimensions.width}
              height={dimensions.height}
              backgroundColor="transparent"
              nodeLabel={(node: GraphNode) => node.name || node.label || node.id}
              nodeColor={(node: GraphNode) => {
                const type = (node.type || node.group || '').toLowerCase();
                return nodeColorMap[type] || '#64748b';
              }}
              nodeVal={(node: GraphNode) => node.val || 4}
              nodeCanvasObject={(node: GraphNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
                const label = node.name || node.label || node.id;
                const type = (node.type || node.group || '').toLowerCase();
                const color = nodeColorMap[type] || '#64748b';
                const size = (node.val || 4) * 1.5;
                const x = node.x || 0;
                const y = node.y || 0;

                // Draw node
                ctx.beginPath();
                ctx.arc(x, y, size, 0, 2 * Math.PI);
                ctx.fillStyle = color;
                ctx.fill();

                // Draw selected ring
                if (selectedNode && selectedNode.id === node.id) {
                  ctx.strokeStyle = '#ffffff';
                  ctx.lineWidth = 2 / globalScale;
                  ctx.stroke();
                }

                // Draw label
                if (globalScale > 0.7) {
                  const fontSize = Math.max(10 / globalScale, 3);
                  ctx.font = `${fontSize}px Inter, sans-serif`;
                  ctx.textAlign = 'center';
                  ctx.textBaseline = 'top';
                  ctx.fillStyle = '#e4e4e7';
                  ctx.fillText(label, x, y + size + 2);
                }
              }}
              linkColor={() => 'rgba(100, 116, 139, 0.3)'}
              linkWidth={(link: GraphLink) => (link.weight || 1) * 0.5}
              linkDirectionalArrowLength={4}
              linkDirectionalArrowRelPos={1}
              onNodeClick={handleNodeClick as (node: object) => void}
              cooldownTicks={100}
              onEngineStop={handleZoomFit}
            />
          )}
        </div>

        {/* Detail panel */}
        {selectedNode && (
          <Card className="w-72 shrink-0 overflow-y-auto" title="Node Details">
            <div className="space-y-3">
              <div>
                <p className="text-xs text-[var(--text-muted)] mb-1">Name</p>
                <p className="text-sm font-medium text-[var(--text-primary)]">
                  {selectedNode.name || selectedNode.label || selectedNode.id}
                </p>
              </div>
              <div>
                <p className="text-xs text-[var(--text-muted)] mb-1">Type</p>
                <Badge type={selectedNode.type || selectedNode.group || 'unknown'} />
              </div>
              <div>
                <p className="text-xs text-[var(--text-muted)] mb-1">ID</p>
                <p className="text-xs font-mono text-[var(--text-secondary)] break-all">{selectedNode.id}</p>
              </div>
              <div>
                <p className="text-xs text-[var(--text-muted)] mb-1">Connections</p>
                <p className="text-sm text-[var(--text-secondary)]">
                  {links.filter(l => {
                    const src = typeof l.source === 'object' ? l.source.id : l.source;
                    const tgt = typeof l.target === 'object' ? l.target.id : l.target;
                    return src === selectedNode.id || tgt === selectedNode.id;
                  }).length}
                </p>
              </div>
              <button
                onClick={() => setSelectedNode(null)}
                className="w-full py-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
              >
                Close
              </button>
            </div>
          </Card>
        )}
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-3">
        {Object.entries(nodeColorMap).map(([type, color]) => (
          <div key={type} className="flex items-center gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: color }} />
            <span className="text-xs text-[var(--text-muted)] capitalize">{type}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
