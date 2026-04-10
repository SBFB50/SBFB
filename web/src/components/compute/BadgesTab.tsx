import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Award, Star, Flame, Crown, Clock, Rocket, Zap } from 'lucide-react';

interface Badge {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  requirement: string;
}

const BADGES: Badge[] = [
  {
    id: 'first_task',
    name: 'Premiere tache',
    description: 'Completez votre premiere tache de calcul',
    icon: 'star',
    color: '#22c55e',
    requirement: '1 tache completee',
  },
  {
    id: 'centurion',
    name: 'Centurion',
    description: '100 taches completees — contribution significative',
    icon: 'flame',
    color: '#f59e0b',
    requirement: '100 taches',
  },
  {
    id: 'millionnaire',
    name: 'Millionnaire',
    description: '1 000 taches — pilier du reseau',
    icon: 'crown',
    color: '#a855f7',
    requirement: '1 000 taches',
  },
  {
    id: 'pilier',
    name: 'Pilier',
    description: '10 000 taches — vous etes indispensable',
    icon: 'award',
    color: '#ec4899',
    requirement: '10 000 taches',
  },
  {
    id: 'always_on',
    name: '24/7',
    description: 'Uptime continu de plus de 7 jours',
    icon: 'clock',
    color: '#06b6d4',
    requirement: '7 jours d\'uptime continu',
  },
  {
    id: 'early_adopter',
    name: 'Early Adopter',
    description: 'Parmi les 10 premiers contributeurs du reseau',
    icon: 'rocket',
    color: '#3b82f6',
    requirement: 'Top 10 premiers inscrits',
  },
  {
    id: 'power_node',
    name: 'Power Node',
    description: 'GPU avec plus de 24 GB de VRAM',
    icon: 'zap',
    color: '#eab308',
    requirement: 'VRAM > 24 GB',
  },
];

const ICON_MAP: Record<string, typeof Award> = {
  star: Star,
  flame: Flame,
  crown: Crown,
  award: Award,
  clock: Clock,
  rocket: Rocket,
  zap: Zap,
};

export function BadgesTab() {
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="border-b border-[var(--border)] py-2 px-4">
          <CardTitle className="text-sm flex items-center gap-2">
            <Award size={16} className="text-yellow-400" />
            Badges contributeur
          </CardTitle>
        </CardHeader>
        <CardContent className="p-4">
          <p className="text-xs text-[var(--text-muted)] mb-4">
            Gagnez des badges en contribuant au reseau. Chaque contribution compte !
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
            {BADGES.map(badge => {
              const Icon = ICON_MAP[badge.icon] || Award;
              return (
                <div
                  key={badge.id}
                  className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 flex items-start gap-3 hover:border-[color:var(--badge-color)] transition-colors"
                  style={{ '--badge-color': badge.color + '40' } as React.CSSProperties}
                >
                  <div
                    className="p-2 rounded-lg shrink-0"
                    style={{ backgroundColor: badge.color + '15' }}
                  >
                    <Icon size={20} style={{ color: badge.color }} />
                  </div>
                  <div className="min-w-0">
                    <h3 className="text-sm font-medium text-[var(--text-primary)]">{badge.name}</h3>
                    <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">{badge.description}</p>
                    <p className="text-[10px] text-[var(--text-muted)] mt-1 font-mono">{badge.requirement}</p>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
