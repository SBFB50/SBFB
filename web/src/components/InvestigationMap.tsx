import { useState, useEffect } from 'react';
import { MapContainer, TileLayer, Marker, Popup, Polyline } from 'react-leaflet';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { api } from '../api/client';

// Fix default marker icons in Leaflet + Vite
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png',
  iconUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png',
  shadowUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png',
});

const MARKER_COLORS: Record<string, string> = {
  crime_scene: '#ef4444',
  witness: '#3b82f6',
  suspect: '#f97316',
  location: '#22c55e',
  default: '#a855f7',
};

function createIcon(color: string) {
  return L.divIcon({
    className: '',
    html: `<div style="
      width:12px;height:12px;border-radius:50%;
      background:${color};border:2px solid white;
      box-shadow:0 0 6px ${color}88;
    "></div>`,
    iconSize: [12, 12],
    iconAnchor: [6, 6],
  });
}

interface MapLocation {
  name: string;
  lat: number;
  lon: number;
  type?: string;
  entity_id?: string;
  description?: string;
}

export default function InvestigationMap({ caseId }: { caseId: string }) {
  const [locations, setLocations] = useState<MapLocation[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!caseId) return;
    let active = true;

    const fetchMap = async () => {
      try {
        const resp = await api.get(`/cases/${caseId}/map`);
        const data = resp.data;

        // Extract locations from map data
        const locs: MapLocation[] = [];
        if (data?.locations) {
          for (const loc of data.locations) {
            if (loc.lat && loc.lon) {
              locs.push({
                name: loc.name || 'Lieu',
                lat: loc.lat,
                lon: loc.lon,
                type: loc.type || 'location',
                entity_id: loc.entity_id,
                description: loc.description,
              });
            }
          }
        }
        if (active) {
          setLocations(locs);
          setLoading(false);
        }
      } catch {
        if (active) setLoading(false);
      }
    };

    fetchMap();
    const interval = setInterval(fetchMap, 15000);
    return () => { active = false; clearInterval(interval); };
  }, [caseId]);

  if (loading && locations.length === 0) {
    return (
      <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3">Carte de l'enquete</h3>
        <div className="h-64 flex items-center justify-center text-[var(--text-muted)] text-xs">
          Chargement de la carte...
        </div>
      </div>
    );
  }

  if (locations.length === 0) {
    return (
      <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3">Carte de l'enquete</h3>
        <div className="h-32 flex items-center justify-center text-[var(--text-muted)] text-xs">
          Aucun lieu geocode — les lieux apparaitront apres le geocodage
        </div>
      </div>
    );
  }

  // Calculate bounds
  const center: [number, number] = [
    locations.reduce((s, l) => s + l.lat, 0) / locations.length,
    locations.reduce((s, l) => s + l.lon, 0) / locations.length,
  ];

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-[var(--text-primary)]">
          Carte de l'enquete
        </h3>
        <span className="text-[10px] text-[var(--text-muted)]">
          {locations.length} lieu{locations.length > 1 ? 'x' : ''} geocode{locations.length > 1 ? 's' : ''}
        </span>
      </div>
      <div className="h-72 rounded-lg overflow-hidden border border-[var(--border)]">
        <MapContainer
          center={center}
          zoom={10}
          style={{ height: '100%', width: '100%' }}
          scrollWheelZoom={true}
        >
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a>'
            url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
          />
          {locations.map((loc, i) => {
            const color = MARKER_COLORS[loc.type || 'default'] || MARKER_COLORS.default;
            return (
              <Marker key={i} position={[loc.lat, loc.lon]} icon={createIcon(color)}>
                <Popup>
                  <div className="text-xs">
                    <p className="font-bold">{loc.name}</p>
                    {loc.description && <p className="text-gray-600 mt-1">{loc.description}</p>}
                    <p className="text-gray-400 mt-1">{loc.lat.toFixed(4)}, {loc.lon.toFixed(4)}</p>
                  </div>
                </Popup>
              </Marker>
            );
          })}
          {/* Connect locations with lines if more than 1 */}
          {locations.length > 1 && (
            <Polyline
              positions={locations.map(l => [l.lat, l.lon] as [number, number])}
              pathOptions={{ color: '#6366f1', weight: 1, opacity: 0.3, dashArray: '5 10' }}
            />
          )}
        </MapContainer>
      </div>
      {/* Legend */}
      <div className="flex gap-3 mt-2 text-[9px] text-[var(--text-muted)]">
        {Object.entries(MARKER_COLORS).filter(([k]) => k !== 'default').map(([type, color]) => (
          <span key={type} className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full" style={{ backgroundColor: color }} />
            {type.replace('_', ' ')}
          </span>
        ))}
      </div>
    </div>
  );
}
