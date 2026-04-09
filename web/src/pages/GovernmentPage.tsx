import { useState, useRef, useCallback, useMemo } from 'react';
import {
  Landmark, Users, FileText, AlertTriangle, Clock,
  Play, Square, ExternalLink, Search, Loader2,
  Network, Maximize2, Download, Group,
  Newspaper, MessageSquare, Video, Bell, Activity,
  CheckCircle, Globe, Hash, Camera, Tv,
} from 'lucide-react';
import { GraphCanvas } from 'reagraph';
import type { GraphCanvasRef, LayoutTypes, InternalGraphNode, InternalGraphEdge } from 'reagraph';

import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select';

import { showToast } from '../components/Toast';
import MetricCard from '../components/MetricCard';
import LoadingSpinner from '../components/LoadingSpinner';

import {
  useGovStats, useGovPoliticians, useGovPositions,
  useGovPoliticianContradictions, useGovAllContradictions,
  useTriggerGovScan, useStopGovScan, useGovScanStatus,
  useDetectGovContradictions, useStopGovDetection, useGovDetectionStatus,
  useGovGraph,
  useGovPress, useGovAllSocial, useGovAllTranscriptions,
  useGovAlerts, useMarkAlertRead, useGovWorkers,
} from '../hooks/useGovernment';

/* ═══════════════════════════════════════════════════════════════════
   ERROR BANNER
   ═══════════════════════════════════════════════════════════════════ */

function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center gap-3">
      <AlertTriangle size={36} className="text-red-400" />
      <p className="text-sm text-red-400 font-medium">Erreur de chargement</p>
      <p className="text-xs text-muted-foreground max-w-md">{message}</p>
    </div>
  );
}

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

const LEGEND_PARTIES = [
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

const EDGE_COLORS: Record<string, string> = {
  opposition: '#ef4444', agreement: '#22c55e', party: '#374151',
};

const LAYOUTS: { value: LayoutTypes; label: string }[] = [
  { value: 'forceDirected2d', label: 'Force' },
  { value: 'circular2d', label: 'Circulaire' },
  { value: 'hierarchicalTd', label: 'Hierarchique' },
  { value: 'radialOut2d', label: 'Radial' },
];

const REAGRAPH_THEME = {
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

const STANCE_COLOR: Record<string, string> = {
  pour: 'text-green-400', contre: 'text-red-400', abstention: 'text-yellow-400',
};

/* ═══════════════════════════════════════════════════════════════════
   TYPES
   ═══════════════════════════════════════════════════════════════════ */

interface Pol { id: string; name: string; party?: string; chamber?: string; role?: string;
  constituency?: string; official_url?: string; hatvp_url?: string;
  position_count?: number; contradiction_count?: number; [k: string]: unknown; }
interface Pos { id: string; date: string; position_type: string; stance?: string;
  subject: string; position_text: string; source_url?: string; [k: string]: unknown; }
interface Contra { id: string; politician_id: string; subject: string; description: string;
  severity: string; position_a_id: string; position_b_id: string; [k: string]: unknown; }
interface ApiNode { id: string; label: string; party?: string; chamber?: string; role?: string;
  position_count?: number; contradiction_count?: number; constituency?: string;
  official_url?: string; hatvp_url?: string; [k: string]: unknown; }
interface ApiEdge { id: string; source: string; target: string; type?: string; label?: string;
  subject?: string; stance_a?: string; stance_b?: string; [k: string]: unknown; }
interface PressArticle { id: string; title: string; source_name?: string; published_at?: string;
  sentiment?: string; summary?: string; url?: string; [k: string]: unknown; }
interface SocialPost { id: string; platform: string; content: string; posted_at?: string;
  url?: string; [k: string]: unknown; }
interface Transcription { id: string; title?: string; transcription?: string;
  duration_seconds?: number; source_url?: string; model_used?: string; [k: string]: unknown; }
interface GovAlert { id: string; title: string; description?: string; alert_type?: string;
  severity?: string; is_read?: boolean; created_at?: string; [k: string]: unknown; }
interface GovWorkerStatus { name: string; status?: string; events_processed?: number;
  events_errored?: number; [k: string]: unknown; }

/* ═══════════════════════════════════════════════════════════════════
   MAIN PAGE
   ═══════════════════════════════════════════════════════════════════ */

export default function GovernmentPage() {
  const [activeTab, setActiveTab] = useState('politicians');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [chamberFilter, setChamberFilter] = useState('all');

  // Data
  const statsQ = useGovStats();
  const polsQ = useGovPoliticians(chamberFilter !== 'all' ? { chamber: chamberFilter } : undefined);
  const posQ = useGovPositions(selectedId);
  const polContraQ = useGovPoliticianContradictions(selectedId);
  const allContraQ = useGovAllContradictions();

  // Scan
  const scanStatusQ = useGovScanStatus();
  const triggerScan = useTriggerGovScan();
  const stopScan = useStopGovScan();
  const detectStatusQ = useGovDetectionStatus();
  const triggerDetect = useDetectGovContradictions();
  const stopDetect = useStopGovDetection();

  const stats = statsQ.data || { politicians: 0, positions: 0, contradictions: 0, last_scan: null };
  const pols: Pol[] = Array.isArray(polsQ.data) ? polsQ.data : [];
  const positions: Pos[] = Array.isArray(posQ.data) ? posQ.data : [];
  const polContras: Contra[] = Array.isArray(polContraQ.data) ? polContraQ.data : [];
  const allContras: Contra[] = Array.isArray(allContraQ.data) ? allContraQ.data : [];
  const scanStatus = scanStatusQ.data || { running: false, phase: '', progress: '', politicians_scanned: 0, politicians_total: 0, items_found: 0, items_new: 0 };
  const detectStatus = detectStatusQ.data || { running: false, phase: '', progress: '' };

  const filtered = searchQuery
    ? pols.filter(p => p.name?.toLowerCase().includes(searchQuery.toLowerCase()) || p.party?.toLowerCase().includes(searchQuery.toLowerCase()))
    : pols;
  const selectedPol = pols.find(p => p.id === selectedId);

  const handleScan = () => {
    if (scanStatus.running) stopScan.mutate(undefined, { onSuccess: () => showToast('info', 'Scan arrete') });
    else triggerScan.mutate(undefined, { onSuccess: () => showToast('success', 'Scan lance') });
  };
  const handleDetect = () => {
    if (detectStatus.running) stopDetect.mutate(undefined, { onSuccess: () => showToast('info', 'Detection arretee') });
    else triggerDetect.mutate(undefined, { onSuccess: () => showToast('success', 'Detection lancee') });
  };
  const goToNetwork = (polId: string) => { setSelectedId(polId); setActiveTab('network'); };

  const scanProgress = scanStatus.politicians_total > 0
    ? Math.round((scanStatus.politicians_scanned / scanStatus.politicians_total) * 100) : null;

  return (
    <div className="flex flex-col h-full gap-3">
      {/* ── Header ── */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-lg bg-cyan-500/10"><Landmark size={22} className="text-cyan-400" /></div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">Gouvernement Francais</h2>
            <p className="text-xs text-muted-foreground">Suivi factuel de l'activite parlementaire</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant={scanStatus.running ? 'destructive' : 'default'} size="sm" onClick={handleScan}>
            {scanStatus.running ? <><Square className="size-3.5" /> Arreter</> : <><Play className="size-3.5" /> Scanner</>}
          </Button>
          <Button variant={detectStatus.running ? 'destructive' : 'outline'} size="sm" onClick={handleDetect}>
            {detectStatus.running ? <><Square className="size-3.5" /> Arreter</> : <><AlertTriangle className="size-3.5" /> Contradictions</>}
          </Button>
        </div>
      </div>

      {/* ── Scan Progress ── */}
      {scanStatus.running && (
        <div className="flex items-center gap-3 px-3 py-2 rounded-lg border border-cyan-500/20 bg-cyan-500/5">
          <Loader2 size={14} className="animate-spin text-cyan-400 shrink-0" />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 text-xs">
              <span className="font-medium text-cyan-400">Scan</span>
              <span className="text-foreground">{scanStatus.phase}</span>
              <span className="text-muted-foreground">{scanStatus.progress}</span>
            </div>
            {scanProgress !== null && (
              <Progress value={scanProgress} className="mt-1.5" />
            )}
          </div>
          <span className="text-xs text-muted-foreground tabular-nums shrink-0">
            {scanStatus.items_found} trouves, {scanStatus.items_new} nouveaux
          </span>
        </div>
      )}

      {/* ── Metrics ── */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-3">
        <MetricCard label="Politiciens" value={stats.politicians} icon={Users} color="var(--accent-cyan)" />
        <MetricCard label="Positions" value={stats.positions} icon={FileText} color="var(--accent-green)" />
        <MetricCard label="Contradictions" value={stats.contradictions} icon={AlertTriangle} color="var(--accent-red)" />
        <MetricCard label="Dernier scan" value={stats.last_scan ? new Date(stats.last_scan).toLocaleDateString('fr-FR') : 'Jamais'} icon={Clock} color="var(--accent-purple)" />
      </div>

      {/* ── Tabs ── */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as string)} className="flex-1 flex flex-col min-h-0">
        <TabsList variant="line">
          <TabsTrigger value="politicians">Politiciens ({stats.politicians})</TabsTrigger>
          <TabsTrigger value="network"><Network className="size-3.5" /> Reseau</TabsTrigger>
          <TabsTrigger value="contradictions">Contradictions ({stats.contradictions})</TabsTrigger>
          <TabsTrigger value="press"><Newspaper className="size-3.5" /> Presse</TabsTrigger>
          <TabsTrigger value="social"><MessageSquare className="size-3.5" /> Social</TabsTrigger>
          <TabsTrigger value="videos"><Video className="size-3.5" /> Videos</TabsTrigger>
          <TabsTrigger value="alerts"><Bell className="size-3.5" /> Alertes</TabsTrigger>
          <TabsTrigger value="pipeline"><Activity className="size-3.5" /> Pipeline</TabsTrigger>
        </TabsList>

        {/* ── TAB: Politicians ── */}
        <TabsContent value="politicians" className="flex-1 mt-3">
          <div className="flex gap-3 h-[calc(100vh-380px)]">
            {/* List */}
            <Card className="flex-[2] flex flex-col">
              <CardHeader className="border-b">
                <CardTitle>Politiciens</CardTitle>
                <CardAction>
                  <div className="flex items-center gap-2">
                    <div className="relative">
                      <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
                      <Input placeholder="Rechercher..." value={searchQuery} onChange={e => setSearchQuery(e.target.value)} className="pl-8 h-7 w-48 text-xs" />
                    </div>
                    <Select value={chamberFilter} onValueChange={setChamberFilter}>
                      <SelectTrigger size="sm" className="w-32"><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="all">Toutes chambres</SelectItem>
                        <SelectItem value="assemblee">Assemblee</SelectItem>
                        <SelectItem value="senat">Senat</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </CardAction>
              </CardHeader>
              <CardContent className="flex-1 p-0">
                <ScrollArea className="h-full">
                  {polsQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
                   : polsQ.isError ? <div className="p-8"><ErrorBanner message={(polsQ.error as Error)?.message || 'Impossible de charger les politiciens'} /></div>
                   : filtered.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-16 text-center">
                      <Users size={36} className="text-muted-foreground mb-3" />
                      <p className="text-sm text-muted-foreground">Aucun politicien. Lancez un scan.</p>
                    </div>
                   ) : filtered.map((p: Pol) => (
                    <button key={p.id} onClick={() => setSelectedId(p.id)}
                      className={`w-full flex items-center gap-3 px-4 py-2.5 text-left border-b border-border/50 transition-colors hover:bg-muted/50 ${selectedId === p.id ? 'bg-cyan-500/5 border-l-2 border-l-cyan-500' : ''}`}>
                      <div className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: PARTY_COLORS[p.party || ''] || DEFAULT_COLOR }} />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-foreground truncate">{p.name}</p>
                        <p className="text-xs text-muted-foreground">{p.party || 'SE'} — {p.chamber === 'assemblee' ? 'AN' : 'Senat'}</p>
                      </div>
                      <div className="flex items-center gap-3 shrink-0 text-xs tabular-nums">
                        {(p.position_count ?? 0) > 0 && <span className="text-muted-foreground">{p.position_count} pos.</span>}
                        {(p.contradiction_count ?? 0) > 0 && <span className="text-red-400 font-medium">{p.contradiction_count} contr.</span>}
                      </div>
                    </button>
                  ))}
                </ScrollArea>
              </CardContent>
            </Card>

            {/* Detail */}
            <Card className="flex-1 flex flex-col">
              <CardHeader className="border-b">
                <CardTitle>{selectedPol ? selectedPol.name : 'Detail'}</CardTitle>
                {selectedPol && (
                  <CardAction>
                    <Button variant="ghost" size="xs" onClick={() => goToNetwork(selectedPol.id)}>
                      <Network className="size-3" /> Reseau
                    </Button>
                  </CardAction>
                )}
              </CardHeader>
              <CardContent className="flex-1 p-0">
                <ScrollArea className="h-full">
                  {!selectedPol ? (
                    <div className="flex flex-col items-center justify-center py-16 text-center">
                      <Users size={36} className="text-muted-foreground mb-3" />
                      <p className="text-sm text-muted-foreground">Selectionnez un politicien</p>
                    </div>
                  ) : (
                    <div className="p-4 space-y-4">
                      {/* Info */}
                      <div className="space-y-1">
                        <p className="text-xs text-muted-foreground">{selectedPol.role} — {selectedPol.constituency}</p>
                        <div className="flex gap-1.5">
                          {selectedPol.official_url && <a href={selectedPol.official_url} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-1 text-xs text-cyan-400 hover:underline"><ExternalLink size={10} />Fiche</a>}
                          {selectedPol.hatvp_url && <a href={selectedPol.hatvp_url} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-1 text-xs text-cyan-400 hover:underline"><ExternalLink size={10} />HATVP</a>}
                        </div>
                      </div>
                      <Separator />
                      {/* Contradictions */}
                      {polContras.length > 0 && (<>
                        <div>
                          <p className="text-xs font-semibold text-red-400 uppercase tracking-wider mb-2">Contradictions ({polContras.length})</p>
                          {polContras.map((c: Contra) => (
                            <div key={c.id} className="mb-2 p-2.5 rounded-lg bg-red-500/5 border border-red-500/20">
                              <div className="flex items-center gap-2 mb-1">
                                <Badge variant="destructive">{c.severity}</Badge>
                                <span className="text-xs font-medium text-foreground">{c.subject}</span>
                              </div>
                              <p className="text-xs text-muted-foreground leading-relaxed">{c.description}</p>
                            </div>
                          ))}
                        </div>
                        <Separator />
                      </>)}
                      {/* Positions */}
                      <div>
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Positions ({positions.length})</p>
                        {posQ.isLoading ? <LoadingSpinner text="..." /> : posQ.isError ? <p className="text-xs text-red-400">Erreur de chargement des positions.</p> : positions.length === 0 ? <p className="text-xs text-muted-foreground">Aucune position.</p>
                        : positions.map((pos: Pos) => (
                          <div key={pos.id} className="mb-2 p-2.5 rounded-lg border border-border/50 bg-card/50">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="text-[10px] font-mono text-muted-foreground">{pos.date}</span>
                              <Badge variant="secondary">{pos.position_type}</Badge>
                              {pos.stance && <span className={`text-[10px] font-semibold ${STANCE_COLOR[pos.stance] || ''}`}>{pos.stance.toUpperCase()}</span>}
                            </div>
                            <p className="text-xs font-medium text-foreground">{pos.subject}</p>
                            <p className="text-xs text-muted-foreground line-clamp-2 mt-0.5">{pos.position_text}</p>
                            {pos.source_url && <a href={pos.source_url} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-1 text-[10px] text-cyan-400 hover:underline mt-1"><ExternalLink size={9} />Source</a>}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* ── TAB: Network ── */}
        <TabsContent value="network" className="flex-1 mt-3">
          <NetworkTab
            chamberFilter={chamberFilter}
            selectedId={selectedId}
            onSelectPolitician={setSelectedId}
            onSwitchTab={setActiveTab}
          />
        </TabsContent>

        {/* ── TAB: Contradictions ── */}
        <TabsContent value="contradictions" className="flex-1 mt-3">
          <Card className="h-[calc(100vh-380px)] flex flex-col">
            <CardHeader className="border-b">
              <CardTitle>Toutes les contradictions</CardTitle>
            </CardHeader>
            <CardContent className="flex-1 p-0">
              <ScrollArea className="h-full">
                {allContraQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
                : allContraQ.isError ? <div className="p-8"><ErrorBanner message={(allContraQ.error as Error)?.message || 'Impossible de charger les contradictions'} /></div>
                : allContras.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16 text-center">
                    <AlertTriangle size={36} className="text-muted-foreground mb-3" />
                    <p className="text-sm text-muted-foreground">Aucune contradiction detectee.</p>
                  </div>
                ) : allContras.map((c: Contra) => (
                  <div key={c.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30 transition-colors">
                    <Badge variant={c.severity === 'high' ? 'destructive' : c.severity === 'medium' ? 'secondary' : 'outline'}>
                      {c.severity}
                    </Badge>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-foreground">{c.subject}</p>
                      <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{c.description}</p>
                    </div>
                    <Button variant="ghost" size="xs" onClick={() => { setSelectedId(c.politician_id); setActiveTab('politicians'); }}>
                      Voir
                    </Button>
                  </div>
                ))}
              </ScrollArea>
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── TAB: Presse ── */}
        <TabsContent value="press" className="flex-1 mt-3">
          <PressTab />
        </TabsContent>

        {/* ── TAB: Social ── */}
        <TabsContent value="social" className="flex-1 mt-3">
          <SocialTab />
        </TabsContent>

        {/* ── TAB: Videos ── */}
        <TabsContent value="videos" className="flex-1 mt-3">
          <VideosTab />
        </TabsContent>

        {/* ── TAB: Alertes ── */}
        <TabsContent value="alerts" className="flex-1 mt-3">
          <AlertsTab />
        </TabsContent>

        {/* ── TAB: Pipeline ── */}
        <TabsContent value="pipeline" className="flex-1 mt-3">
          <PipelineTab />
        </TabsContent>
      </Tabs>
    </div>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   NETWORK TAB (Reagraph)
   ═══════════════════════════════════════════════════════════════════ */

function NetworkTab({ chamberFilter, selectedId, onSelectPolitician, onSwitchTab }: {
  chamberFilter: string; selectedId: string | null;
  onSelectPolitician: (id: string) => void; onSwitchTab: (tab: string) => void;
}) {
  const graphRef = useRef<GraphCanvasRef | null>(null);
  const [layout, setLayout] = useState<LayoutTypes>('forceDirected2d');
  const [relationFilter, setRelationFilter] = useState('all');
  const [clustering, setClustering] = useState(false);
  const [gSelectedId, setGSelectedId] = useState<string | null>(null);
  const [gSelectedType, setGSelectedType] = useState<'node' | 'edge' | null>(null);
  const [gSelectedData, setGSelectedData] = useState<ApiNode | ApiEdge | null>(null);

  const graphQ = useGovGraph(chamberFilter !== 'all' ? { chamber: chamberFilter } : undefined);
  const apiData = graphQ.data || { nodes: [], edges: [] };

  const nodes = useMemo(() => (apiData.nodes || []).map((n: ApiNode) => ({
    id: n.id, label: n.label || n.id,
    subLabel: `${n.party || 'SE'} — ${n.chamber === 'assemblee' ? 'AN' : 'Senat'}`,
    fill: PARTY_COLORS[n.party || ''] || DEFAULT_COLOR,
    size: Math.max(3, Math.min(20, (n.position_count || 0) + 3)),
    cluster: n.party || 'Autres', data: n,
  })), [apiData.nodes]);

  const allEdges = useMemo(() => (apiData.edges || []).map((e: ApiEdge) => ({
    id: e.id, source: e.source, target: e.target,
    label: e.type === 'party' ? '' : (e.label || ''),
    fill: EDGE_COLORS[e.type || ''] || '#374151',
    size: e.type === 'opposition' ? 2 : 1, data: e,
  })), [apiData.edges]);

  const edges = useMemo(() =>
    relationFilter === 'all' ? allEdges : allEdges.filter(e => e.data?.type === relationFilter),
  [allEdges, relationFilter]);

  const handleNodeClick = useCallback((node: InternalGraphNode) => {
    setGSelectedId(node.id); setGSelectedType('node'); setGSelectedData(node.data as ApiNode);
    onSelectPolitician(node.id);
  }, [onSelectPolitician]);
  const handleEdgeClick = useCallback((edge: InternalGraphEdge) => {
    setGSelectedId(edge.id); setGSelectedType('edge'); setGSelectedData(edge.data as ApiEdge);
  }, []);
  const handleCanvasClick = useCallback(() => { setGSelectedId(null); setGSelectedType(null); setGSelectedData(null); }, []);
  const handleExport = useCallback(() => {
    const url = graphRef.current?.exportCanvas();
    if (url) { const a = document.createElement('a'); a.download = 'reseau-politique.png'; a.href = url; a.click(); }
  }, []);

  return (
    <div className="flex flex-col gap-2 h-[calc(100vh-380px)]">
      {/* Controls */}
      <div className="flex items-center gap-2 flex-wrap">
        <Select value={layout} onValueChange={v => setLayout(v as LayoutTypes)}>
          <SelectTrigger size="sm" className="w-36"><SelectValue /></SelectTrigger>
          <SelectContent>
            {LAYOUTS.map(l => <SelectItem key={l.value} value={l.value}>{l.label}</SelectItem>)}
          </SelectContent>
        </Select>
        <Separator orientation="vertical" className="h-5" />
        {['all', 'opposition', 'agreement', 'party'].map(v => (
          <Button key={v} variant={relationFilter === v ? 'default' : 'outline'} size="xs"
            onClick={() => setRelationFilter(v)}>
            {v === 'all' ? 'Tous' : v === 'opposition' ? 'Oppositions' : v === 'agreement' ? 'Accords' : 'Parti'}
          </Button>
        ))}
        <Separator orientation="vertical" className="h-5" />
        <Button variant={clustering ? 'default' : 'outline'} size="xs" onClick={() => setClustering(c => !c)}>
          <Group className="size-3" /> Grouper
        </Button>
        <div className="ml-auto flex gap-1">
          <Button variant="outline" size="icon-xs" onClick={() => graphRef.current?.centerGraph()} aria-label="Centrer le graphe"><Maximize2 className="size-3" /></Button>
          <Button variant="outline" size="icon-xs" onClick={handleExport} aria-label="Exporter le graphe"><Download className="size-3" /></Button>
        </div>
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
                  <p className="text-muted-foreground">{nodes.length} noeuds</p>
                  <p className="text-muted-foreground">{edges.length} liens</p>
                </div>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* Graph */}
        <Card className="flex-1 overflow-hidden p-0">
          {graphQ.isLoading ? <div className="flex items-center justify-center h-full"><LoadingSpinner text="Chargement du graphe..." /></div>
          : graphQ.isError ? <div className="flex items-center justify-center h-full"><ErrorBanner message={(graphQ.error as Error)?.message || 'Impossible de charger le graphe'} /></div>
          : nodes.length === 0 ? <div className="flex flex-col items-center justify-center h-full"><Network size={40} className="text-muted-foreground mb-3" /><p className="text-sm text-muted-foreground">Aucune donnee. Lancez un scan.</p></div>
          : <GraphCanvas ref={graphRef} nodes={nodes} edges={edges} layoutType={layout} theme={REAGRAPH_THEME}
              cameraMode="pan" draggable clusterAttribute={clustering ? 'cluster' : undefined}
              sizingType="attribute" sizingAttribute="size" labelType="all" edgeArrowPosition="none"
              selections={gSelectedId ? [gSelectedId] : []}
              onNodeClick={handleNodeClick} onEdgeClick={handleEdgeClick} onCanvasClick={handleCanvasClick} />
          }
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

/* ═══════════════════════════════════════════════════════════════════
   PRESS TAB
   ═══════════════════════════════════════════════════════════════════ */

function PressTab() {
  const [sentiment, setSentiment] = useState('');
  const pressQ = useGovPress(sentiment || undefined);
  const articles: PressArticle[] = Array.isArray(pressQ.data) ? pressQ.data : [];

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Revue de presse</CardTitle>
        <CardAction>
          <div className="flex gap-1">
            {['', 'positive', 'neutral', 'negative'].map(s => (
              <Button key={s} variant={sentiment === s ? 'default' : 'outline'} size="xs"
                onClick={() => setSentiment(s)}>
                {s || 'Tous'}
              </Button>
            ))}
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {pressQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : pressQ.isError ? <div className="p-8"><ErrorBanner message={(pressQ.error as Error)?.message || 'Impossible de charger la presse'} /></div>
          : articles.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Newspaper size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucun article.</p>
            </div>
          ) : articles.map((a: PressArticle) => (
            <div key={a.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <Badge variant={a.sentiment === 'positive' ? 'default' : a.sentiment === 'negative' ? 'destructive' : 'outline'}>
                {a.sentiment || '?'}
              </Badge>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground truncate">{a.title}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.source_name} — {a.published_at ? new Date(a.published_at).toLocaleDateString('fr-FR') : ''}</p>
                {a.summary && <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{a.summary}</p>}
              </div>
              {a.url && (
                <a href={a.url} target="_blank" rel="noopener noreferrer">
                  <Button variant="ghost" size="icon-xs" aria-label="Ouvrir l'article"><ExternalLink className="size-3" /></Button>
                </a>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   SOCIAL TAB
   ═══════════════════════════════════════════════════════════════════ */

function SocialTab() {
  const [platform, setPlatform] = useState('');
  const socialQ = useGovAllSocial(platform || undefined);
  const posts: SocialPost[] = Array.isArray(socialQ.data) ? socialQ.data : [];

  const platformIcon = (p: string) => {
    if (p === 'twitter') return <Hash className="size-3" />;
    if (p === 'facebook') return <Globe className="size-3" />;
    if (p === 'instagram') return <Camera className="size-3" />;
    if (p === 'youtube') return <Tv className="size-3" />;
    return <MessageSquare className="size-3" />;
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Reseaux sociaux</CardTitle>
        <CardAction>
          <div className="flex gap-1">
            {['', 'twitter', 'facebook', 'instagram'].map(p => (
              <Button key={p} variant={platform === p ? 'default' : 'outline'} size="xs"
                onClick={() => setPlatform(p)}>
                {p ? platformIcon(p) : null} {p || 'Tous'}
              </Button>
            ))}
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {socialQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : socialQ.isError ? <div className="p-8"><ErrorBanner message={(socialQ.error as Error)?.message || 'Impossible de charger les posts'} /></div>
          : posts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <MessageSquare size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucun post.</p>
            </div>
          ) : posts.map((p: SocialPost) => (
            <div key={p.id} className="flex items-start gap-3 px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <div className="pt-0.5">{platformIcon(p.platform)}</div>
              <div className="flex-1 min-w-0">
                <p className="text-xs text-muted-foreground">{p.platform} — {p.posted_at ? new Date(p.posted_at).toLocaleDateString('fr-FR') : ''}</p>
                <p className="text-sm text-foreground mt-0.5 line-clamp-3">{p.content}</p>
              </div>
              {p.url && (
                <a href={p.url} target="_blank" rel="noopener noreferrer">
                  <Button variant="ghost" size="icon-xs" aria-label="Ouvrir le post"><ExternalLink className="size-3" /></Button>
                </a>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   VIDEOS TAB
   ═══════════════════════════════════════════════════════════════════ */

function VideosTab() {
  const [searchQ, setSearchQ] = useState('');
  const transcriptionsQ = useGovAllTranscriptions();
  const transcriptions: Transcription[] = Array.isArray(transcriptionsQ.data) ? transcriptionsQ.data : [];
  const filtered = searchQ
    ? transcriptions.filter((t: Transcription) =>
        (t.title || '').toLowerCase().includes(searchQ.toLowerCase()) ||
        (t.transcription || '').toLowerCase().includes(searchQ.toLowerCase())
      )
    : transcriptions;

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Transcriptions video</CardTitle>
        <CardAction>
          <div className="relative">
            <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
            <Input placeholder="Rechercher dans les transcriptions..." value={searchQ}
              onChange={e => setSearchQ(e.target.value)} className="pl-8 h-7 w-64 text-xs" />
          </div>
        </CardAction>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {transcriptionsQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : transcriptionsQ.isError ? <div className="p-8"><ErrorBanner message={(transcriptionsQ.error as Error)?.message || 'Impossible de charger les transcriptions'} /></div>
          : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Video size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucune transcription.</p>
            </div>
          ) : filtered.map((t: Transcription) => (
            <div key={t.id} className="px-4 py-3 border-b border-border/50 hover:bg-muted/30">
              <div className="flex items-center gap-2 mb-1">
                <Video className="size-3.5 text-cyan-400 shrink-0" />
                <p className="text-sm font-medium text-foreground truncate">{t.title || 'Sans titre'}</p>
                {t.duration_seconds && (
                  <span className="text-xs text-muted-foreground shrink-0">{Math.floor(t.duration_seconds / 60)}min</span>
                )}
              </div>
              <p className="text-xs text-muted-foreground line-clamp-3 mt-1">{(t.transcription || '').slice(0, 300)}...</p>
              <div className="flex gap-2 mt-1.5">
                {t.source_url && (
                  <a href={t.source_url} target="_blank" rel="noopener noreferrer"
                    className="text-xs text-cyan-400 hover:underline flex items-center gap-1">
                    <ExternalLink className="size-2.5" /> Source
                  </a>
                )}
                <span className="text-xs text-muted-foreground">{t.model_used}</span>
              </div>
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   ALERTS TAB
   ═══════════════════════════════════════════════════════════════════ */

function AlertsTab() {
  const alertsQ = useGovAlerts();
  const markRead = useMarkAlertRead();
  const alerts: GovAlert[] = Array.isArray(alertsQ.data) ? alertsQ.data : [];
  const unread = alerts.filter((a: GovAlert) => !a.is_read);
  const read = alerts.filter((a: GovAlert) => a.is_read);

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Alertes ({unread.length} non lues)</CardTitle>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {alertsQ.isLoading ? <div className="p-8"><LoadingSpinner text="Chargement..." /></div>
          : alertsQ.isError ? <div className="p-8"><ErrorBanner message={(alertsQ.error as Error)?.message || 'Impossible de charger les alertes'} /></div>
          : alerts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Bell size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Aucune alerte.</p>
            </div>
          ) : [...unread, ...read].map((a: GovAlert) => (
            <div key={a.id} className={`flex items-start gap-3 px-4 py-3 border-b border-border/50 transition-colors ${a.is_read ? 'opacity-50' : 'hover:bg-muted/30'}`}>
              <Badge variant={a.severity === 'high' ? 'destructive' : a.severity === 'medium' ? 'secondary' : 'outline'}>
                {a.alert_type}
              </Badge>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground">{a.title}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.description}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{a.created_at ? new Date(a.created_at).toLocaleString('fr-FR') : ''}</p>
              </div>
              {!a.is_read && (
                <Button variant="ghost" size="xs" onClick={() => markRead.mutate(a.id)}>
                  <CheckCircle className="size-3" /> Lu
                </Button>
              )}
            </div>
          ))}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

/* ═══════════════════════════════════════════════════════════════════
   PIPELINE TAB
   ═══════════════════════════════════════════════════════════════════ */

function PipelineTab() {
  const workersQ = useGovWorkers();
  const data = workersQ.data || { running: false, workers: 0, worker_status: [] };
  const workers: GovWorkerStatus[] = Array.isArray(data.worker_status) ? data.worker_status : [];

  const statusColor = (s: string) => {
    if (s === 'processing') return 'text-green-400';
    if (s === 'error' || s === 'circuit_open') return 'text-red-400';
    if (s === 'idle') return 'text-muted-foreground';
    return 'text-yellow-400';
  };

  return (
    <Card className="h-[calc(100vh-380px)] flex flex-col">
      <CardHeader className="border-b">
        <CardTitle>Pipeline ({workers.length} workers)</CardTitle>
        <CardAction>
          <Badge variant={data.running ? 'default' : 'destructive'}>
            {data.running ? 'Actif' : 'Arrete'}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[calc(100vh-460px)]">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
            {workersQ.isError ? <div className="col-span-full p-8"><ErrorBanner message={(workersQ.error as Error)?.message || 'Impossible de charger le pipeline'} /></div>
            : workersQ.isLoading ? <div className="col-span-full p-8"><LoadingSpinner text="Chargement..." /></div>
            : null}
            {workers.map((w: GovWorkerStatus) => (
              <div key={w.name} className="rounded-lg border border-border/50 bg-muted/20 px-3 py-2.5">
                <div className="flex items-center gap-2 mb-1">
                  <div className={`size-1.5 rounded-full ${w.status === 'processing' ? 'bg-green-400' : w.status === 'error' ? 'bg-red-400' : 'bg-zinc-500'}`} />
                  <p className="text-xs font-medium text-foreground truncate">{w.name?.replace('gov_', '')}</p>
                </div>
                <p className={`text-xs ${statusColor(w.status || 'idle')}`}>{w.status || 'idle'}</p>
                {w.events_processed !== undefined && (
                  <p className="text-xs text-muted-foreground mt-0.5">{w.events_processed} traites</p>
                )}
                {w.events_errored > 0 && (
                  <p className="text-xs text-red-400">{w.events_errored} erreurs</p>
                )}
              </div>
            ))}
          </div>
          {workers.length === 0 && !workersQ.isLoading && !workersQ.isError && (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Activity size={36} className="text-muted-foreground mb-3" />
              <p className="text-sm text-muted-foreground">Pipeline non demarre.</p>
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
