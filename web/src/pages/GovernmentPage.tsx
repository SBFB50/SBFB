import { useState, lazy, Suspense } from 'react';
import {
  Landmark, Users, FileText, AlertTriangle, Clock,
  Play, Square, ExternalLink, Search, Loader2,
  Network, Vote, BarChart3,
  Newspaper, MessageSquare, Video, Bell, Activity,
  Scale, FileCheck, Scroll, MapPin, GitCompare,
} from 'lucide-react';

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
import { Hemicycle } from '../components/Hemicycle';

// Heavy components — lazy loaded (graph libs, map, charts)
const NetworkTab = lazy(() => import('../components/gov/NetworkTab').then(m => ({ default: m.NetworkTab })));
const CarteTab = lazy(() => import('../components/gov/CarteTab').then(m => ({ default: m.CarteTab })));
const ComparateurTab = lazy(() => import('../components/gov/ComparateurTab').then(m => ({ default: m.ComparateurTab })));
const StatsTab = lazy(() => import('../components/gov/StatsTab').then(m => ({ default: m.StatsTab })));
const SearchTab = lazy(() => import('../components/gov/SearchTab').then(m => ({ default: m.SearchTab })));
const TimelineTab = lazy(() => import('../components/gov/TimelineTab').then(m => ({ default: m.TimelineTab })));

// Light components — keep static
import { PressTab } from '../components/gov/PressTab';
import { SocialTab } from '../components/gov/SocialTab';
import { VideosTab } from '../components/gov/VideosTab';
import { AlertsTab } from '../components/gov/AlertsTab';
import { PipelineTab } from '../components/gov/PipelineTab';
import { AffairesTab } from '../components/gov/AffairesTab';
import { DeclarationsTab } from '../components/gov/DeclarationsTab';
import { LegislationTab } from '../components/gov/LegislationTab';
import { RecapTab } from '../components/gov/RecapTab';

import {
  useGovStats, useGovPoliticians, useGovPositions,
  useGovPoliticianContradictions, useGovAllContradictions,
  useTriggerGovScan, useStopGovScan, useGovScanStatus,
  useDetectGovContradictions, useStopGovDetection, useGovDetectionStatus,
  useGovAffairsByPolitician, useGovDeclarations, useGovPressByPolitician,
} from '../hooks/useGovernment';

import {
  PARTY_COLORS, DEFAULT_COLOR, STANCE_COLOR,
} from '../components/gov/types';
import type {
  Pol, Pos, Contra, Affair, Declaration, PressArticle,
} from '../components/gov/types';

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

  // Enriched detail data
  const affairsQ = useGovAffairsByPolitician(selectedId);
  const declarationsQ = useGovDeclarations(selectedId);
  const pressPolQ = useGovPressByPolitician(selectedId);

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
  const polAffairs: Affair[] = Array.isArray(affairsQ.data) ? affairsQ.data : [];
  const polDeclarations: Declaration[] = Array.isArray(declarationsQ.data) ? declarationsQ.data : [];
  const polPress: PressArticle[] = Array.isArray(pressPolQ.data) ? pressPolQ.data : [];
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
          <TabsTrigger value="hemicycle"><Vote className="size-3.5" /> Hemicycle</TabsTrigger>
          <TabsTrigger value="carte"><MapPin className="size-3.5" /> Carte</TabsTrigger>
          <TabsTrigger value="comparateur"><GitCompare className="size-3.5" /> Comparateur</TabsTrigger>
          <TabsTrigger value="network"><Network className="size-3.5" /> Reseau</TabsTrigger>
          <TabsTrigger value="contradictions">Contradictions ({stats.contradictions})</TabsTrigger>
          <TabsTrigger value="press"><Newspaper className="size-3.5" /> Presse</TabsTrigger>
          <TabsTrigger value="social"><MessageSquare className="size-3.5" /> Social</TabsTrigger>
          <TabsTrigger value="videos"><Video className="size-3.5" /> Videos</TabsTrigger>
          <TabsTrigger value="stats"><BarChart3 className="size-3.5" /> Statistiques</TabsTrigger>
          <TabsTrigger value="alerts"><Bell className="size-3.5" /> Alertes</TabsTrigger>
          <TabsTrigger value="affaires"><Scale className="size-3.5" /> Affaires</TabsTrigger>
          <TabsTrigger value="declarations"><FileCheck className="size-3.5" /> Declarations</TabsTrigger>
          <TabsTrigger value="legislation"><Scroll className="size-3.5" /> Legislation</TabsTrigger>
          <TabsTrigger value="timeline"><Clock className="size-3.5" /> Chronologie</TabsTrigger>
          <TabsTrigger value="search"><Search className="size-3.5" /> Recherche</TabsTrigger>
          <TabsTrigger value="recap"><FileText className="size-3.5" /> Recap</TabsTrigger>
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

                      {/* Affaires */}
                      {polAffairs.length > 0 && (<>
                        <Separator />
                        <div>
                          <p className="text-xs font-semibold text-orange-400 uppercase tracking-wider mb-2">
                            <Scale className="size-3 inline mr-1" />
                            Affaires ({polAffairs.length})
                          </p>
                          {polAffairs.map((a: Affair) => (
                            <div key={a.id} className="mb-2 p-2.5 rounded-lg bg-orange-500/5 border border-orange-500/20">
                              <div className="flex items-center gap-2 mb-1">
                                <Badge variant={a.status === 'condamne' ? 'destructive' : a.status === 'mis_en_examen' ? 'secondary' : 'outline'}>
                                  {a.status || 'inconnu'}
                                </Badge>
                                <span className="text-xs font-medium text-foreground truncate">{a.title}</span>
                              </div>
                              {a.description && <p className="text-xs text-muted-foreground line-clamp-2">{a.description}</p>}
                              {a.start_date && <p className="text-[10px] text-muted-foreground mt-1">{new Date(a.start_date).toLocaleDateString('fr-FR')}</p>}
                            </div>
                          ))}
                        </div>
                      </>)}

                      {/* Declarations HATVP */}
                      {polDeclarations.length > 0 && (<>
                        <Separator />
                        <div>
                          <p className="text-xs font-semibold text-purple-400 uppercase tracking-wider mb-2">
                            <FileCheck className="size-3 inline mr-1" />
                            Declarations HATVP ({polDeclarations.length})
                          </p>
                          {polDeclarations.map((d: Declaration) => (
                            <div key={d.id} className="mb-2 p-2.5 rounded-lg border border-purple-500/20 bg-purple-500/5">
                              <div className="flex items-center gap-2 mb-1">
                                <Badge variant="outline">{d.type || 'declaration'}</Badge>
                                {d.year && <span className="text-[10px] font-mono text-muted-foreground">{d.year}</span>}
                              </div>
                              {d.patrimony_total != null && (
                                <p className="text-xs text-foreground">Patrimoine: <span className="font-medium">{Number(d.patrimony_total).toLocaleString('fr-FR')} EUR</span></p>
                              )}
                              {d.interests && <p className="text-xs text-muted-foreground line-clamp-2 mt-0.5">{d.interests}</p>}
                              {d.url && (
                                <a href={d.url} target="_blank" rel="noopener noreferrer"
                                  className="inline-flex items-center gap-1 text-[10px] text-cyan-400 hover:underline mt-1">
                                  <ExternalLink size={9} /> Voir sur HATVP
                                </a>
                              )}
                            </div>
                          ))}
                        </div>
                      </>)}

                      {/* Presse */}
                      {polPress.length > 0 && (<>
                        <Separator />
                        <div>
                          <p className="text-xs font-semibold text-blue-400 uppercase tracking-wider mb-2">
                            <Newspaper className="size-3 inline mr-1" />
                            Presse ({polPress.length})
                          </p>
                          {polPress.slice(0, 8).map((a: PressArticle) => (
                            <div key={a.id} className="mb-2 p-2.5 rounded-lg border border-border/50 bg-card/50">
                              <div className="flex items-center gap-2 mb-0.5">
                                <Badge variant={a.sentiment === 'positive' ? 'default' : a.sentiment === 'negative' ? 'destructive' : 'outline'} className="text-[9px]">
                                  {a.sentiment || '?'}
                                </Badge>
                                <span className="text-[10px] text-muted-foreground">{a.source_name}</span>
                                {a.published_at && <span className="text-[10px] text-muted-foreground">{new Date(a.published_at).toLocaleDateString('fr-FR')}</span>}
                              </div>
                              <p className="text-xs font-medium text-foreground truncate">{a.title}</p>
                              {a.summary && <p className="text-[10px] text-muted-foreground line-clamp-2 mt-0.5">{a.summary}</p>}
                              {a.url && (
                                <a href={a.url} target="_blank" rel="noopener noreferrer"
                                  className="inline-flex items-center gap-1 text-[10px] text-cyan-400 hover:underline mt-1">
                                  <ExternalLink size={9} /> Lire
                                </a>
                              )}
                            </div>
                          ))}
                          {polPress.length > 8 && (
                            <p className="text-[10px] text-muted-foreground italic">+{polPress.length - 8} autres articles</p>
                          )}
                        </div>
                      </>)}

                      {/* Coherence score */}
                      {selectedPol.metadata && typeof (selectedPol as any).metadata === 'object' && (selectedPol as any).metadata.coherence_score != null && (<>
                        <Separator />
                        <div className="flex items-center gap-3 p-2.5 rounded-lg bg-cyan-500/5 border border-cyan-500/20">
                          <div className="text-center">
                            <p className="text-lg font-bold text-cyan-400 tabular-nums">
                              {Math.round(Number((selectedPol as any).metadata.coherence_score) * 100)}%
                            </p>
                            <p className="text-[10px] text-muted-foreground">Coherence</p>
                          </div>
                          <div className="flex-1">
                            <div className="h-2 bg-muted rounded-full overflow-hidden">
                              <div className="h-full bg-cyan-400 rounded-full transition-all"
                                style={{ width: `${Math.round(Number((selectedPol as any).metadata.coherence_score) * 100)}%` }} />
                            </div>
                          </div>
                        </div>
                      </>)}
                    </div>
                  )}
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* ── TAB: Hemicycle ── */}
        <TabsContent value="hemicycle" className="flex-1 mt-3">
          <div className="flex gap-3 h-[calc(100vh-380px)]">
            <Card className="flex-[3] flex flex-col">
              <CardHeader className="border-b">
                <CardTitle>Hemicycle</CardTitle>
                <CardAction>
                  <Select value={chamberFilter} onValueChange={setChamberFilter}>
                    <SelectTrigger size="sm" className="w-40"><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">Toutes chambres</SelectItem>
                      <SelectItem value="assemblee">Assemblee Nationale</SelectItem>
                      <SelectItem value="senat">Senat</SelectItem>
                    </SelectContent>
                  </Select>
                </CardAction>
              </CardHeader>
              <CardContent className="flex-1 p-4 overflow-auto">
                {polsQ.isLoading ? <LoadingSpinner text="Chargement des sieges..." /> :
                 polsQ.isError ? <ErrorBanner message="Impossible de charger les politiciens" /> :
                 <Hemicycle
                   politicians={pols.map(p => ({
                     id: p.id,
                     name: p.name,
                     party: p.party,
                     chamber: p.chamber,
                     role: p.role,
                   }))}
                   chamber={chamberFilter === 'assemblee' ? 'an' : chamberFilter === 'senat' ? 'senat' : 'all'}
                   onSeatClick={(pol) => { setSelectedId(pol.id); setActiveTab('politicians'); }}
                 />}
              </CardContent>
            </Card>

            {/* Party breakdown sidebar */}
            <Card className="flex-1 flex flex-col">
              <CardHeader className="border-b">
                <CardTitle>Groupes parlementaires</CardTitle>
              </CardHeader>
              <CardContent className="flex-1 p-0">
                <ScrollArea className="h-full">
                  {(() => {
                    const groups: Record<string, { count: number; color: string }> = {};
                    for (const p of pols) {
                      const key = p.party || 'Sans etiquette';
                      if (!groups[key]) groups[key] = { count: 0, color: PARTY_COLORS[key] || DEFAULT_COLOR };
                      groups[key].count++;
                    }
                    return Object.entries(groups)
                      .sort((a, b) => b[1].count - a[1].count)
                      .map(([name, { count, color }]) => (
                        <div key={name} className="flex items-center gap-3 px-4 py-2 border-b border-border/30 hover:bg-muted/30">
                          <div className="w-3 h-3 rounded-full shrink-0" style={{ backgroundColor: color }} />
                          <span className="flex-1 text-sm text-foreground">{name}</span>
                          <span className="text-sm font-mono text-muted-foreground tabular-nums">{count}</span>
                          <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                            <div className="h-full rounded-full" style={{ width: `${(count / pols.length) * 100}%`, backgroundColor: color }} />
                          </div>
                        </div>
                      ));
                  })()}
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* ── TAB: Carte ── */}
        <TabsContent value="carte" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement de la carte..." />}>
            <CarteTab />
          </Suspense>
        </TabsContent>

        {/* ── TAB: Comparateur ── */}
        <TabsContent value="comparateur" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement du comparateur..." />}>
            <ComparateurTab />
          </Suspense>
        </TabsContent>

        {/* ── TAB: Network ── */}
        <TabsContent value="network" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement du reseau..." />}>
            <NetworkTab chamberFilter={chamberFilter} selectedId={selectedId} onSelectPolitician={setSelectedId} onSwitchTab={setActiveTab} />
          </Suspense>
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

        {/* ── Extracted Tabs ── */}
        <TabsContent value="press" className="flex-1 mt-3"><PressTab /></TabsContent>
        <TabsContent value="social" className="flex-1 mt-3"><SocialTab /></TabsContent>
        <TabsContent value="videos" className="flex-1 mt-3"><VideosTab /></TabsContent>
        <TabsContent value="stats" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement des statistiques..." />}>
            <StatsTab politicians={pols} />
          </Suspense>
        </TabsContent>
        <TabsContent value="alerts" className="flex-1 mt-3"><AlertsTab /></TabsContent>
        <TabsContent value="affaires" className="flex-1 mt-3"><AffairesTab /></TabsContent>
        <TabsContent value="declarations" className="flex-1 mt-3"><DeclarationsTab /></TabsContent>
        <TabsContent value="legislation" className="flex-1 mt-3"><LegislationTab /></TabsContent>
        <TabsContent value="timeline" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement de la chronologie..." />}>
            <TimelineTab />
          </Suspense>
        </TabsContent>
        <TabsContent value="search" className="flex-1 mt-3">
          <Suspense fallback={<LoadingSpinner text="Chargement de la recherche..." />}>
            <SearchTab />
          </Suspense>
        </TabsContent>
        <TabsContent value="recap" className="flex-1 mt-3"><RecapTab /></TabsContent>
        <TabsContent value="pipeline" className="flex-1 mt-3"><PipelineTab /></TabsContent>
      </Tabs>
    </div>
  );
}
