import { useState, useMemo } from 'react';
import { MapContainer, TileLayer, CircleMarker, Popup, useMap } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';

import { Card, CardHeader, CardTitle, CardContent, CardAction } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { MapPin, Users } from 'lucide-react';

import { useGovPoliticians } from '../../hooks/useGovernment';
import LoadingSpinner from '../LoadingSpinner';

/* ═══════════════════════════════════════════════════════════════════
   PARTY COLORS (same as GovernmentPage)
   ═══════════════════════════════════════════════════════════════════ */

const PARTY_COLORS: Record<string, string> = {
  'LFI': '#cc2443', 'FI': '#cc2443', 'PCF': '#dd0000', 'GDR': '#dd0000',
  'PS': '#ff8080', 'SOC': '#ff8080', 'EELV': '#00c000', 'ECO': '#00c000',
  'RE': '#ffcc00', 'REN': '#ffcc00', 'DEM': '#ff9900', 'MODEM': '#ff9900',
  'HOR': '#00bfff', 'LR': '#0066cc', 'UDI': '#00cccc', 'LIOT': '#87ceeb',
  'RN': '#0d2244', 'SE': '#64748b',
};
const DEFAULT_COLOR = '#64748b';

/* ═══════════════════════════════════════════════════════════════════
   FRENCH CITY COORDS LOOKUP (major cities / departement capitals)
   ═══════════════════════════════════════════════════════════════════ */

const CITY_COORDS: Record<string, [number, number]> = {
  'paris': [48.8566, 2.3522],
  'marseille': [43.2965, 5.3698],
  'lyon': [45.7640, 4.8357],
  'toulouse': [43.6047, 1.4442],
  'nice': [43.7102, 7.2620],
  'nantes': [47.2184, -1.5536],
  'strasbourg': [48.5734, 7.7521],
  'montpellier': [43.6108, 3.8767],
  'bordeaux': [44.8378, -0.5792],
  'lille': [50.6292, 3.0573],
  'rennes': [48.1173, -1.6778],
  'reims': [49.2583, 3.2794],
  'saint-etienne': [45.4397, 4.3872],
  'toulon': [43.1242, 5.9280],
  'le havre': [49.4944, 0.1079],
  'grenoble': [45.1885, 5.7245],
  'dijon': [47.3220, 5.0415],
  'angers': [47.4784, -0.5632],
  'nimes': [43.8367, 4.3601],
  'clermont-ferrand': [45.7772, 3.0870],
  'aix-en-provence': [43.5297, 5.4474],
  'brest': [48.3904, -4.4861],
  'tours': [47.3941, 0.6848],
  'amiens': [49.8941, 2.2958],
  'limoges': [45.8315, 1.2578],
  'perpignan': [42.6887, 2.8948],
  'metz': [49.1193, 6.1757],
  'besancon': [47.2378, 6.0241],
  'orleans': [47.9029, 1.9039],
  'rouen': [49.4432, 1.0999],
  'mulhouse': [47.7508, 7.3359],
  'caen': [49.1829, -0.3707],
  'nancy': [48.6921, 6.1844],
  'argenteuil': [48.9472, 2.2467],
  'montreuil': [48.8638, 2.4484],
  'saint-denis': [48.9362, 2.3574],
  'pau': [43.2951, -0.3708],
  'calais': [50.9513, 1.8587],
  'ajaccio': [41.9192, 8.7386],
  'bastia': [42.6977, 9.4509],
  'poitiers': [46.5802, 0.3404],
  'troyes': [48.2973, 4.0744],
  'la rochelle': [46.1591, -1.1520],
  'cherbourg': [49.6337, -1.6222],
  'bourges': [47.0810, 2.3988],
  'valence': [44.9334, 4.8924],
  'charleville-mezieres': [49.7718, 4.7200],
  'quimper': [48.0000, -4.0999],
  'colmar': [48.0794, 7.3585],
  'vannes': [47.6584, -2.7609],
  'dunkerque': [51.0343, 2.3768],
  'bayonne': [43.4929, -1.4748],
  'avignon': [43.9493, 4.8055],
  'la reunion': [-21.1151, 55.5364],
  'guadeloupe': [16.2650, -61.5510],
  'martinique': [14.6415, -61.0242],
  'guyane': [3.9339, -53.1258],
  'mayotte': [-12.8275, 45.1662],
  'nouvelle-caledonie': [-22.2558, 166.4505],
  'polynesie': [-17.6797, -149.4068],
  // Departement numbers to approximate locations
  'ain': [46.2, 5.2], 'aisne': [49.5, 3.6], 'allier': [46.3, 3.2],
  'alpes-de-haute-provence': [44.1, 6.2], 'hautes-alpes': [44.6, 6.2],
  'alpes-maritimes': [43.8, 7.2], 'ardeche': [44.7, 4.4],
  'ardennes': [49.8, 4.7], 'ariege': [42.9, 1.5], 'aube': [48.3, 4.1],
  'aude': [43.1, 2.4], 'aveyron': [44.3, 2.6], 'bouches-du-rhone': [43.5, 5.1],
  'calvados': [49.1, -0.4], 'cantal': [45.0, 2.7], 'charente': [45.7, 0.2],
  'charente-maritime': [45.9, -0.8], 'cher': [47.1, 2.4], 'correze': [45.4, 1.9],
  'corse-du-sud': [41.9, 9.0], 'haute-corse': [42.4, 9.2],
  'cote-d\'or': [47.3, 4.8], 'cotes-d\'armor': [48.5, -3.0],
  'creuse': [46.1, 2.1], 'dordogne': [45.1, 0.7], 'doubs': [47.2, 6.4],
  'drome': [44.7, 5.2], 'eure': [49.1, 1.2], 'eure-et-loir': [48.3, 1.5],
  'finistere': [48.3, -4.1], 'gard': [43.9, 4.2], 'haute-garonne': [43.4, 1.2],
  'gers': [43.7, 0.6], 'gironde': [44.8, -0.6], 'herault': [43.6, 3.4],
  'ille-et-vilaine': [48.1, -1.7], 'indre': [46.8, 1.6],
  'indre-et-loire': [47.3, 0.7], 'isere': [45.3, 5.6], 'jura': [46.7, 5.8],
  'landes': [44.0, -0.8], 'loir-et-cher': [47.6, 1.3], 'loire': [45.7, 4.2],
  'haute-loire': [45.1, 3.7], 'loire-atlantique': [47.3, -1.7],
  'loiret': [47.9, 2.2], 'lot': [44.6, 1.7], 'lot-et-garonne': [44.4, 0.5],
  'lozere': [44.5, 3.5], 'maine-et-loire': [47.4, -0.5],
  'manche': [49.0, -1.3], 'marne': [48.9, 3.9], 'haute-marne': [48.1, 5.3],
  'mayenne': [48.1, -0.8], 'meurthe-et-moselle': [48.7, 6.2],
  'meuse': [49.0, 5.4], 'morbihan': [47.7, -2.8], 'moselle': [49.0, 6.6],
  'nievre': [47.1, 3.5], 'nord': [50.3, 3.2], 'oise': [49.4, 2.4],
  'orne': [48.6, 0.1], 'pas-de-calais': [50.5, 2.3],
  'puy-de-dome': [45.7, 3.1], 'pyrenees-atlantiques': [43.3, -0.8],
  'hautes-pyrenees': [43.1, 0.1], 'pyrenees-orientales': [42.6, 2.5],
  'bas-rhin': [48.6, 7.5], 'haut-rhin': [47.9, 7.2], 'rhone': [45.8, 4.7],
  'haute-saone': [47.6, 6.2], 'saone-et-loire': [46.6, 4.5],
  'sarthe': [47.9, 0.2], 'savoie': [45.5, 6.4], 'haute-savoie': [46.0, 6.4],
  'seine-maritime': [49.6, 1.1], 'seine-et-marne': [48.6, 2.9],
  'yvelines': [48.8, 1.9], 'deux-sevres': [46.5, -0.3],
  'somme': [49.9, 2.3], 'tarn': [43.8, 2.2], 'tarn-et-garonne': [44.0, 1.3],
  'var': [43.5, 6.3], 'vaucluse': [44.1, 5.1], 'vendee': [46.7, -1.3],
  'vienne': [46.6, 0.5], 'haute-vienne': [45.8, 1.3], 'vosges': [48.2, 6.4],
  'yonne': [47.8, 3.6], 'territoire-de-belfort': [47.6, 6.9],
  'essonne': [48.5, 2.2], 'hauts-de-seine': [48.8, 2.2],
  'seine-saint-denis': [48.9, 2.5], 'val-de-marne': [48.8, 2.5],
  'val-d\'oise': [49.1, 2.2],
};

/** Try to resolve constituency text to coordinates */
function resolveConstituency(constituency: string | undefined): [number, number] | null {
  if (!constituency) return null;
  const lower = constituency.toLowerCase()
    .normalize('NFD').replace(/[\u0300-\u036f]/g, '') // strip accents
    .replace(/[()]/g, '');

  // Direct lookup
  for (const [key, coords] of Object.entries(CITY_COORDS)) {
    if (lower.includes(key)) return coords;
  }

  // Try extracting city name after common patterns like "3eme circ. de Paris"
  const circMatch = lower.match(/(?:circ(?:\.|onscription)?)\s+(?:de\s+)?(.+)/);
  if (circMatch) {
    const city = circMatch[1].trim();
    for (const [key, coords] of Object.entries(CITY_COORDS)) {
      if (city.includes(key)) return coords;
    }
  }

  // Try extracting departement name
  const deptMatch = lower.match(/(\d+)\s*(?:ere?|eme|e)\s+circ/);
  if (deptMatch) {
    // Try the rest of the string for departement name
    const rest = lower.replace(deptMatch[0], '').trim();
    for (const [key, coords] of Object.entries(CITY_COORDS)) {
      if (rest.includes(key)) return coords;
    }
  }

  return null;
}

/* ═══════════════════════════════════════════════════════════════════
   TYPES
   ═══════════════════════════════════════════════════════════════════ */

interface Pol {
  id: string;
  name: string;
  party?: string;
  chamber?: string;
  role?: string;
  constituency?: string;
  [k: string]: unknown;
}

interface ConstituencyGroup {
  key: string;
  label: string;
  coords: [number, number];
  politicians: Pol[];
  parties: Record<string, number>;
  dominantParty: string;
  dominantColor: string;
}

/* ═══════════════════════════════════════════════════════════════════
   FLY-TO COMPONENT (Leaflet needs imperative API)
   ═══════════════════════════════════════════════════════════════════ */

function FlyTo({ center, zoom }: { center: [number, number]; zoom: number }) {
  const map = useMap();
  map.flyTo(center, zoom, { duration: 0.8 });
  return null;
}

/* ═══════════════════════════════════════════════════════════════════
   MAIN COMPONENT
   ═══════════════════════════════════════════════════════════════════ */

export function CarteTab() {
  const [chamberFilter, setChamberFilter] = useState('all');
  const [partyFilter, setPartyFilter] = useState('all');
  const [flyTarget, setFlyTarget] = useState<{ center: [number, number]; zoom: number } | null>(null);

  const polsQ = useGovPoliticians(
    chamberFilter !== 'all' ? { chamber: chamberFilter } : undefined
  );
  const pols: Pol[] = Array.isArray(polsQ.data) ? polsQ.data : [];

  // Group politicians by constituency
  const groups = useMemo(() => {
    const map = new Map<string, ConstituencyGroup>();
    const filteredPols = partyFilter !== 'all'
      ? pols.filter(p => p.party === partyFilter)
      : pols;

    for (const p of filteredPols) {
      const coords = resolveConstituency(p.constituency);
      if (!coords) continue;

      // Add small jitter based on constituency string to separate overlapping markers
      const key = `${coords[0].toFixed(2)}_${coords[1].toFixed(2)}`;
      if (!map.has(key)) {
        map.set(key, {
          key,
          label: p.constituency || 'Inconnu',
          coords,
          politicians: [],
          parties: {},
          dominantParty: '',
          dominantColor: DEFAULT_COLOR,
        });
      }
      const group = map.get(key)!;
      group.politicians.push(p);
      const party = p.party || 'SE';
      group.parties[party] = (group.parties[party] || 0) + 1;
    }

    // Compute dominant party per group
    for (const group of map.values()) {
      let maxCount = 0;
      for (const [party, count] of Object.entries(group.parties)) {
        if (count > maxCount) {
          maxCount = count;
          group.dominantParty = party;
          group.dominantColor = PARTY_COLORS[party] || DEFAULT_COLOR;
        }
      }
    }

    return Array.from(map.values());
  }, [pols, partyFilter]);

  // Stats
  const geoCount = groups.reduce((s, g) => s + g.politicians.length, 0);
  const ungeoCount = pols.length - geoCount;

  // Unique parties for filter
  const uniqueParties = useMemo(() => {
    const set = new Set<string>();
    for (const p of pols) if (p.party) set.add(p.party);
    return Array.from(set).sort();
  }, [pols]);

  if (polsQ.isLoading) {
    return (
      <Card className="h-[calc(100vh-380px)] flex items-center justify-center">
        <LoadingSpinner text="Chargement des politiciens..." />
      </Card>
    );
  }

  return (
    <div className="flex gap-3 h-[calc(100vh-380px)]">
      {/* Map */}
      <Card className="flex-[3] flex flex-col overflow-hidden">
        <CardHeader className="border-b">
          <CardTitle className="flex items-center gap-2">
            <MapPin className="size-4 text-cyan-400" />
            Carte des circonscriptions
          </CardTitle>
          <CardAction>
            <div className="flex items-center gap-2">
              <Select value={chamberFilter} onValueChange={setChamberFilter}>
                <SelectTrigger size="sm" className="w-36"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Toutes chambres</SelectItem>
                  <SelectItem value="assemblee">Assemblee</SelectItem>
                  <SelectItem value="senat">Senat</SelectItem>
                </SelectContent>
              </Select>
              <Select value={partyFilter} onValueChange={setPartyFilter}>
                <SelectTrigger size="sm" className="w-28"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Tous partis</SelectItem>
                  {uniqueParties.map(p => (
                    <SelectItem key={p} value={p}>{p}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </CardAction>
        </CardHeader>
        <CardContent className="flex-1 p-0">
          {groups.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center gap-3">
              <MapPin size={36} className="text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                {pols.length === 0
                  ? 'Aucun politicien. Lancez un scan.'
                  : 'Aucune circonscription geocodee.'}
              </p>
            </div>
          ) : (
            <MapContainer
              center={[46.6, 2.3]}
              zoom={6}
              style={{ height: '100%', width: '100%' }}
              scrollWheelZoom={true}
            >
              <TileLayer
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a>'
                url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
              />
              {flyTarget && <FlyTo center={flyTarget.center} zoom={flyTarget.zoom} />}
              {groups.map(g => (
                <CircleMarker
                  key={g.key}
                  center={g.coords}
                  radius={Math.max(6, Math.min(22, 4 + g.politicians.length * 2))}
                  pathOptions={{
                    color: g.dominantColor,
                    fillColor: g.dominantColor,
                    fillOpacity: 0.6,
                    weight: 2,
                    opacity: 0.9,
                  }}
                >
                  <Popup>
                    <div className="text-xs min-w-[180px]">
                      <p className="font-bold text-sm mb-1">{g.label}</p>
                      <p className="text-gray-400 mb-2">{g.politicians.length} politicien{g.politicians.length > 1 ? 's' : ''}</p>
                      {Object.entries(g.parties)
                        .sort(([, a], [, b]) => b - a)
                        .map(([party, count]) => (
                          <div key={party} className="flex items-center gap-2 mb-0.5">
                            <span className="w-2 h-2 rounded-full inline-block" style={{ backgroundColor: PARTY_COLORS[party] || DEFAULT_COLOR }} />
                            <span>{party}: {count}</span>
                          </div>
                        ))
                      }
                      <hr className="my-1.5 border-gray-600" />
                      {g.politicians.slice(0, 8).map(p => (
                        <p key={p.id} className="text-gray-300 truncate">{p.name} ({p.party || 'SE'})</p>
                      ))}
                      {g.politicians.length > 8 && (
                        <p className="text-gray-500 italic">+{g.politicians.length - 8} autres</p>
                      )}
                    </div>
                  </Popup>
                </CircleMarker>
              ))}
            </MapContainer>
          )}
        </CardContent>
      </Card>

      {/* Sidebar: grouped list */}
      <Card className="flex-1 flex flex-col">
        <CardHeader className="border-b">
          <CardTitle>Repartition</CardTitle>
          <CardAction>
            <span className="text-[10px] text-muted-foreground">
              {geoCount} geocodes / {ungeoCount} non-resolus
            </span>
          </CardAction>
        </CardHeader>
        <CardContent className="flex-1 p-0">
          <ScrollArea className="h-full">
            {groups
              .sort((a, b) => b.politicians.length - a.politicians.length)
              .map(g => (
                <button
                  key={g.key}
                  onClick={() => setFlyTarget({ center: g.coords, zoom: 10 })}
                  className="w-full flex items-center gap-3 px-4 py-2 text-left border-b border-border/30 hover:bg-muted/30 transition-colors"
                >
                  <div
                    className="w-3 h-3 rounded-full shrink-0"
                    style={{ backgroundColor: g.dominantColor }}
                  />
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium text-foreground truncate">{g.label}</p>
                    <div className="flex gap-1 mt-0.5">
                      {Object.entries(g.parties).slice(0, 3).map(([party, count]) => (
                        <Badge key={party} variant="outline" className="text-[9px] px-1 py-0">
                          <span className="w-1.5 h-1.5 rounded-full mr-1 inline-block" style={{ backgroundColor: PARTY_COLORS[party] || DEFAULT_COLOR }} />
                          {party} {count}
                        </Badge>
                      ))}
                    </div>
                  </div>
                  <span className="text-xs font-mono text-muted-foreground tabular-nums shrink-0">
                    {g.politicians.length}
                  </span>
                </button>
              ))}

            {/* Ungeolocated politicians */}
            {ungeoCount > 0 && (
              <>
                <Separator />
                <div className="px-4 py-2">
                  <p className="text-[10px] text-muted-foreground uppercase tracking-wider mb-1.5">
                    Non geocodes ({ungeoCount})
                  </p>
                  {pols
                    .filter(p => {
                      const filtered = partyFilter !== 'all' ? p.party === partyFilter : true;
                      return filtered && !resolveConstituency(p.constituency);
                    })
                    .slice(0, 20)
                    .map(p => (
                      <div key={p.id} className="flex items-center gap-2 py-0.5">
                        <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: PARTY_COLORS[p.party || ''] || DEFAULT_COLOR }} />
                        <span className="text-[10px] text-muted-foreground truncate">{p.name}</span>
                        <span className="text-[9px] text-muted-foreground/50 ml-auto shrink-0">{p.constituency || '?'}</span>
                      </div>
                    ))
                  }
                </div>
              </>
            )}
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  );
}
