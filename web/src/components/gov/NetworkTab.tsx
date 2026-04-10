import { useState, useRef, useCallback, useMemo } from 'react';
import {
  Network, Maximize2, Download, Group, AlertTriangle,
} from 'lucide-react';
import { GraphCanvas } from 'reagraph';
import type { GraphCanvasRef, LayoutTypes, InternalGraphNode, InternalGraphEdge } from 'reagraph';

import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select';

import LoadingSpinner from '../LoadingSpinner';
import { PoliticalGraph } from './PoliticalGraph';
import { AnalyticalGraph } from './AnalyticalGraph';
import { useGovGraph } from '../../hooks/useGovernment';
import {
  PARTY_COLORS, DEFAULT_COLOR, LEGEND_PARTIES, EDGE_COLORS,
  LAYOUTS, REAGRAPH_THEME,
} from './types';
import type { ApiNode, ApiEdge } from './types';

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

/* ── Network Tab ── */

export function NetworkTab({ chamberFilter, selectedId, onSelectPolitician, onSwitchTab }: {
  chamberFilter: string; selectedId: string | null;
  onSelectPolitician: (id: string) => void; onSwitchTab: (tab: string) => void;
}) {
  const graphRef = useRef<GraphCanvasRef | null>(null);
  const [engine, setEngine] = useState<'g6' | 'sigma' | 'reagraph'>('g6');
  const [relationFilter, setRelationFilter] = useState('all');
  const [groupByParty, setGroupByParty] = useState(false);
  const [sizeMetric, setSizeMetric] = useState<'degree' | 'betweenness' | 'positions'>('degree');
  const [gSelectedData, setGSelectedData] = useState<ApiNode | ApiEdge | null>(null);
  const [gSelectedType, setGSelectedType] = useState<'node' | 'edge' | null>(null);

  // Reagraph state (kept for legacy 3D mode)
  const [layout, setLayout] = useState<LayoutTypes>('forceDirected2d');
  const [gSelectedId, setGSelectedId] = useState<string | null>(null);

  const graphQ = useGovGraph(chamberFilter !== 'all' ? { chamber: chamberFilter } : undefined);
  const apiData = graphQ.data || { nodes: [], edges: [] };
  const apiNodes: ApiNode[] = apiData.nodes || [];
  const apiEdges: ApiEdge[] = apiData.edges || [];

  // Reagraph data (only computed when needed)
  const reagraphNodes = useMemo(() => engine !== 'reagraph' ? [] : apiNodes.map((n: ApiNode) => ({
    id: n.id, label: n.label || n.id,
    subLabel: `${n.party || 'SE'} — ${n.chamber === 'assemblee' ? 'AN' : 'Senat'}`,
    fill: PARTY_COLORS[n.party || ''] || DEFAULT_COLOR,
    size: Math.max(3, Math.min(20, (n.position_count || 0) + 3)),
    cluster: n.party || 'Autres', data: n,
  })), [apiNodes, engine]);

  const reagraphEdges = useMemo(() => engine !== 'reagraph' ? [] : apiEdges
    .filter(e => relationFilter === 'all' || e.type === relationFilter)
    .map((e: ApiEdge) => ({
      id: e.id, source: e.source, target: e.target,
      label: e.type === 'party' ? '' : (e.label || ''),
      fill: EDGE_COLORS[e.type || ''] || '#374151',
      size: e.type === 'opposition' ? 2 : 1, data: e,
    })), [apiEdges, relationFilter, engine]);

  const handleNodeClick = useCallback((node: InternalGraphNode) => {
    setGSelectedId(node.id); setGSelectedType('node'); setGSelectedData(node.data as ApiNode);
    onSelectPolitician(node.id);
  }, [onSelectPolitician]);
  const handleEdgeClick = useCallback((edge: InternalGraphEdge) => {
    setGSelectedId(edge.id); setGSelectedType('edge'); setGSelectedData(edge.data as ApiEdge);
  }, []);

  const handleG6NodeClick = useCallback((node: ApiNode) => {
    setGSelectedType('node'); setGSelectedData(node);
    onSelectPolitician(node.id);
  }, [onSelectPolitician]);
  const handleG6EdgeClick = useCallback((edge: ApiEdge) => {
    setGSelectedType('edge'); setGSelectedData(edge);
  }, []);

  return (
    <div className="flex flex-col gap-2 h-[calc(100vh-380px)]">
      {/* Controls */}
      <div className="flex items-center gap-2 flex-wrap">
        {/* Engine selector */}
        <div className="flex border border-border rounded-md overflow-hidden">
          {(['g6', 'sigma', 'reagraph'] as const).map(e => (
            <button key={e} onClick={() => setEngine(e)}
              className={`px-2.5 py-1 text-xs font-medium transition-colors ${engine === e ? 'bg-blue-600 text-white' : 'bg-transparent text-muted-foreground hover:text-foreground'}`}>
              {e === 'g6' ? 'G6 Explorer' : e === 'sigma' ? 'Sigma Analyse' : '3D Reagraph'}
            </button>
          ))}
        </div>
        <Separator orientation="vertical" className="h-5" />

        {/* Relation filter */}
        {['all', 'opposition', 'agreement', 'party'].map(v => (
          <Button key={v} variant={relationFilter === v ? 'default' : 'outline'} size="xs"
            onClick={() => setRelationFilter(v)}>
            {v === 'all' ? 'Tous' : v === 'opposition' ? 'Oppositions' : v === 'agreement' ? 'Accords' : 'Parti'}
          </Button>
        ))}
        <Separator orientation="vertical" className="h-5" />

        {/* Engine-specific controls */}
        {engine === 'g6' && (
          <Button variant={groupByParty ? 'default' : 'outline'} size="xs" onClick={() => setGroupByParty(c => !c)}>
            <Group className="size-3" /> Combos parti
          </Button>
        )}
        {engine === 'sigma' && (
          <Select value={sizeMetric} onValueChange={v => setSizeMetric(v as 'degree' | 'betweenness' | 'positions')}>
            <SelectTrigger size="sm" className="w-36"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="degree">Taille: Degre</SelectItem>
              <SelectItem value="betweenness">Taille: Centralite</SelectItem>
              <SelectItem value="positions">Taille: Positions</SelectItem>
            </SelectContent>
          </Select>
        )}
        {engine === 'reagraph' && (
          <>
            <Select value={layout} onValueChange={v => setLayout(v as LayoutTypes)}>
              <SelectTrigger size="sm" className="w-32"><SelectValue /></SelectTrigger>
              <SelectContent>
                {LAYOUTS.map(l => <SelectItem key={l.value} value={l.value}>{l.label}</SelectItem>)}
              </SelectContent>
            </Select>
            <div className="ml-auto flex gap-1">
              <Button variant="outline" size="icon-xs" onClick={() => graphRef.current?.centerGraph()}><Maximize2 className="size-3" /></Button>
              <Button variant="outline" size="icon-xs" onClick={() => {
                const url = graphRef.current?.exportCanvas();
                if (url) { const a = document.createElement('a'); a.download = 'reseau-politique.png'; a.href = url; a.click(); }
              }}><Download className="size-3" /></Button>
            </div>
          </>
        )}
      </div>

      {/* Main */}
      <div className="flex gap-2 flex-1 min-h-0">
        {/* Legend */}
        <Card className="w-[150px] shrink-0">
          <CardHeader><CardTitle>Legende</CardTitle></CardHeader>
          <CardContent>
            <ScrollArea className="h-[calc(100vh-500px)]">
              <div className="space-y-3">
                <div className="space-y-1">
                  {LEGEND_PARTIES.map(p => (
                    <div key={p.key} className="flex items-center gap-2">
                      <div className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: p.color }} />
                      <span className="text-[10px] text-muted-foreground">{p.label}</span>
                    </div>
                  ))}
                </div>
                <Separator />
                <div className="space-y-1">
                  <div className="flex items-center gap-2"><div className="w-3 h-0.5 bg-red-500 rounded" /><span className="text-[10px] text-muted-foreground">Opposition</span></div>
                  <div className="flex items-center gap-2"><div className="w-3 h-0.5 bg-green-500 rounded" /><span className="text-[10px] text-muted-foreground">Accord</span></div>
                  <div className="flex items-center gap-2"><div className="w-3 h-0.5 bg-zinc-600 rounded" /><span className="text-[10px] text-muted-foreground">Parti</span></div>
                </div>
                <Separator />
                <div className="space-y-1 text-xs">
                  <p className="text-muted-foreground">{apiNodes.length} noeuds</p>
                  <p className="text-muted-foreground">{apiEdges.length} liens</p>
                </div>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* Graph */}
        <Card className="flex-1 overflow-hidden p-0">
          {graphQ.isLoading ? <div className="flex items-center justify-center h-full"><LoadingSpinner text="Chargement du graphe..." /></div>
          : graphQ.isError ? <div className="flex items-center justify-center h-full"><ErrorBanner message={(graphQ.error as Error)?.message || 'Impossible de charger le graphe'} /></div>
          : apiNodes.length === 0 ? <div className="flex flex-col items-center justify-center h-full"><Network size={40} className="text-muted-foreground mb-3" /><p className="text-sm text-muted-foreground">Aucune donnee. Lancez un scan.</p></div>
          : engine === 'g6' ? (
            <PoliticalGraph
              nodes={apiNodes}
              edges={apiEdges}
              groupByParty={groupByParty}
              relationFilter={relationFilter}
              selectedId={selectedId}
              onNodeClick={handleG6NodeClick}
              onEdgeClick={handleG6EdgeClick}
            />
          ) : engine === 'sigma' ? (
            <AnalyticalGraph
              nodes={apiNodes}
              edges={apiEdges.filter(e => relationFilter === 'all' || e.type === relationFilter)}
              sizeMetric={sizeMetric}
              detectCommunities
              onNodeClick={handleG6NodeClick}
            />
          ) : (
            <GraphCanvas ref={graphRef} nodes={reagraphNodes} edges={reagraphEdges} layoutType={layout} theme={REAGRAPH_THEME}
              cameraMode="pan" draggable clusterAttribute="cluster"
              sizingType="attribute" sizingAttribute="size" labelType="all" edgeArrowPosition="none"
              selections={gSelectedId ? [gSelectedId] : []}
              onNodeClick={handleNodeClick} onEdgeClick={handleEdgeClick} onCanvasClick={() => { setGSelectedId(null); setGSelectedType(null); setGSelectedData(null); }} />
          )}
        </Card>

        {/* Detail */}
        <Card className="w-[220px] shrink-0">
          <CardHeader><CardTitle>Detail</CardTitle></CardHeader>
          <CardContent>
            <ScrollArea className="h-[calc(100vh-500px)]">
              {!gSelectedData ? (
                <p className="text-xs text-muted-foreground text-center py-8">Cliquez sur un noeud ou un lien</p>
              ) : gSelectedType === 'node' ? (
                <div className="space-y-2.5">
                  <p className="text-sm font-semibold text-foreground">{(gSelectedData as ApiNode).label}</p>
                  <p className="text-xs text-muted-foreground">{(gSelectedData as ApiNode).party} — {(gSelectedData as ApiNode).chamber === 'assemblee' ? 'AN' : 'Senat'}</p>
                  {(gSelectedData as ApiNode).role && <p className="text-xs text-muted-foreground">{(gSelectedData as ApiNode).role}</p>}
                  <Separator />
                  <div className="flex gap-3 text-xs">
                    <div><p className="text-muted-foreground">Positions</p><p className="font-medium text-foreground">{(gSelectedData as ApiNode).position_count || 0}</p></div>
                    <div><p className="text-muted-foreground">Contr.</p><p className="font-medium text-red-400">{(gSelectedData as ApiNode).contradiction_count || 0}</p></div>
                  </div>
                  <Button variant="ghost" size="xs" className="w-full" onClick={() => { onSelectPolitician((gSelectedData as ApiNode).id); onSwitchTab('politicians'); }}>
                    Voir les positions
                  </Button>
                </div>
              ) : (
                <div className="space-y-2.5">
                  <p className="text-sm font-semibold" style={{ color: EDGE_COLORS[(gSelectedData as ApiEdge).type || ''] || '#fff' }}>
                    {(gSelectedData as ApiEdge).type === 'opposition' ? 'Opposition' : (gSelectedData as ApiEdge).type === 'agreement' ? 'Accord' : 'Parti'}
                  </p>
                  {(gSelectedData as ApiEdge).subject && <p className="text-xs text-foreground">{(gSelectedData as ApiEdge).subject}</p>}
                  {(gSelectedData as ApiEdge).label && <p className="text-xs text-muted-foreground">{(gSelectedData as ApiEdge).label}</p>}
                  {(gSelectedData as ApiEdge).stance_a && <div className="text-xs"><span className="text-muted-foreground">A: </span><span>{(gSelectedData as ApiEdge).stance_a}</span></div>}
                  {(gSelectedData as ApiEdge).stance_b && <div className="text-xs"><span className="text-muted-foreground">B: </span><span>{(gSelectedData as ApiEdge).stance_b}</span></div>}
                </div>
              )}
            </ScrollArea>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
