import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Blocks, Server, Zap, CheckCircle, XCircle, Wifi } from 'lucide-react';

interface Props {
  swarm: Record<string, any>;
}

const HEALTH_COLORS: Record<string, { color: string; label: string }> = {
  healthy: { color: '#22c55e', label: 'Operationnel' },
  degraded: { color: '#f59e0b', label: 'Degrade' },
  offline: { color: '#ef4444', label: 'Hors ligne' },
  unknown: { color: '#6b7280', label: 'Inconnu' },
};

export function SwarmTab({ swarm }: Props) {
  const health = HEALTH_COLORS[swarm.health || 'unknown'] || HEALTH_COLORS.unknown;
  const blocksCovered = swarm.blocks_covered || 0;
  const blocksTotal = swarm.blocks_total || 80;
  const coveragePct = swarm.coverage_pct || 0;
  const nodesOnline = swarm.nodes_online || 0;
  const model = swarm.model || '';
  const throughput = swarm.throughput_tok_s || 0;
  const isReady = swarm.is_ready || false;

  return (
    <div className="space-y-4">
      {/* Health banner */}
      <div
        className="border rounded-lg px-4 py-3 flex items-center justify-between"
        style={{ borderColor: health.color + '40', backgroundColor: health.color + '08' }}
      >
        <div className="flex items-center gap-3">
          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: health.color }} />
          <div>
            <p className="text-sm font-medium" style={{ color: health.color }}>{health.label}</p>
            <p className="text-[10px] text-[var(--text-muted)]">
              Petals Swarm — {model ? model.split('/').pop() : 'Non configure'}
            </p>
          </div>
        </div>
        {isReady && (
          <span className="flex items-center gap-1 text-xs text-emerald-400">
            <CheckCircle size={14} /> Pret a servir
          </span>
        )}
      </div>

      {/* Block coverage */}
      <Card>
        <CardHeader className="border-b border-[var(--border)] py-2 px-4">
          <CardTitle className="text-sm flex items-center gap-2">
            <Blocks size={16} className="text-purple-400" />
            Couverture des blocs transformer
          </CardTitle>
        </CardHeader>
        <CardContent className="p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs text-[var(--text-muted)]">
              {blocksCovered} / {blocksTotal} blocs couverts
            </span>
            <span className="text-sm font-bold" style={{ color: coveragePct >= 100 ? '#22c55e' : '#f59e0b' }}>
              {coveragePct.toFixed(0)}%
            </span>
          </div>
          <div className="h-3 bg-[var(--bg-primary)] rounded-full overflow-hidden">
            <div
              className="h-full rounded-full transition-all duration-500"
              style={{
                width: `${Math.min(coveragePct, 100)}%`,
                backgroundColor: coveragePct >= 100 ? '#22c55e' : coveragePct >= 50 ? '#f59e0b' : '#ef4444',
              }}
            />
          </div>

          {/* Block visualization grid */}
          <div className="mt-4 flex flex-wrap gap-0.5">
            {Array.from({ length: blocksTotal }, (_, i) => (
              <div
                key={i}
                className="w-2 h-2 rounded-sm"
                style={{
                  backgroundColor: i < blocksCovered ? '#22c55e' : 'var(--bg-hover)',
                }}
                title={`Bloc ${i + 1}: ${i < blocksCovered ? 'couvert' : 'manquant'}`}
              />
            ))}
          </div>
          <p className="text-[10px] text-[var(--text-muted)] mt-2">
            Chaque carre = 1 bloc transformer du modele. Vert = heberge par un contributeur.
          </p>
        </CardContent>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <Card>
          <CardContent className="p-4 text-center">
            <Server size={20} className="mx-auto mb-1 text-blue-400" />
            <p className="text-2xl font-bold text-[var(--text-primary)]">{nodesOnline}</p>
            <p className="text-[10px] text-[var(--text-muted)] uppercase tracking-wider">Noeuds Petals</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4 text-center">
            <Zap size={20} className="mx-auto mb-1 text-yellow-400" />
            <p className="text-2xl font-bold text-[var(--text-primary)]">
              {throughput > 0 ? `${throughput.toFixed(0)}` : '-'}
            </p>
            <p className="text-[10px] text-[var(--text-muted)] uppercase tracking-wider">Tokens/s (batch)</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4 text-center">
            <Wifi size={20} className="mx-auto mb-1 text-cyan-400" />
            <p className="text-2xl font-bold text-[var(--text-primary)]">
              {model ? model.split('/').pop()?.split('-').pop() || '' : '-'}
            </p>
            <p className="text-[10px] text-[var(--text-muted)] uppercase tracking-wider">Modele distribue</p>
          </CardContent>
        </Card>
      </div>

      {/* How it works */}
      <Card>
        <CardHeader className="border-b border-[var(--border)] py-2 px-4">
          <CardTitle className="text-sm">Comment ca fonctionne</CardTitle>
        </CardHeader>
        <CardContent className="p-4 text-xs text-[var(--text-secondary)] space-y-2">
          <p>
            Petals decoupe un modele 405B en blocs transformer et les distribue sur les GPUs des contributeurs.
            Chaque GPU heberge 1 a 5 blocs selon sa VRAM.
          </p>
          <p>
            Quand NEXUS envoie un prompt, il traverse tous les blocs en pipeline —
            chaque GPU traite sa partie et passe le resultat au suivant.
          </p>
          <p>
            Avec 50 contributeurs fibre, le 405B tourne a ~2 tok/s (single) ou ~80 tok/s (batch) —
            qualite comparable a GPT-4 / Claude 3 Opus, heberge par les citoyens.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
