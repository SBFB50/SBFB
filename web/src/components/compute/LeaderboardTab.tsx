import { Trophy, Cpu, Zap, Shield } from 'lucide-react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';

interface LeaderboardEntry {
  rank: number;
  name: string;
  gpu_model: string;
  vram_mb: number;
  tasks_completed: number;
  avg_tokens_per_sec: number;
  trust_score: number;
  status: string;
}

interface Props {
  entries: LeaderboardEntry[];
  totalContributors: number;
}

const RANK_COLORS: Record<number, string> = {
  1: '#fbbf24', // gold
  2: '#94a3b8', // silver
  3: '#cd7c2f', // bronze
};

export function LeaderboardTab({ entries, totalContributors }: Props) {
  return (
    <Card>
      <CardHeader className="border-b border-[var(--border)] py-2 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm flex items-center gap-2">
            <Trophy size={16} className="text-yellow-400" />
            Classement des contributeurs
          </CardTitle>
          <span className="text-[10px] text-[var(--text-muted)]">
            {totalContributors} contributeurs au total
          </span>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        {entries.length === 0 ? (
          <p className="text-sm text-[var(--text-muted)] text-center py-8">
            Aucun contributeur pour le moment. Soyez le premier !
          </p>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--border)] text-[10px] uppercase tracking-wider text-[var(--text-muted)]">
                <th className="text-left px-4 py-2 w-12">#</th>
                <th className="text-left px-4 py-2">Pseudo</th>
                <th className="text-left px-4 py-2 hidden md:table-cell">GPU</th>
                <th className="text-right px-4 py-2">Tasks</th>
                <th className="text-right px-4 py-2 hidden sm:table-cell">Vitesse</th>
                <th className="text-right px-4 py-2 hidden lg:table-cell">Confiance</th>
                <th className="text-center px-4 py-2 w-16">Status</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => (
                <tr
                  key={e.rank}
                  className="border-b border-[var(--border)] hover:bg-[var(--bg-hover)] transition-colors"
                >
                  <td className="px-4 py-2.5">
                    <span
                      className="text-sm font-bold"
                      style={{ color: RANK_COLORS[e.rank] || 'var(--text-muted)' }}
                    >
                      {e.rank <= 3 ? (
                        <span className="flex items-center gap-1">
                          <Trophy size={12} style={{ color: RANK_COLORS[e.rank] }} />
                          {e.rank}
                        </span>
                      ) : e.rank}
                    </span>
                  </td>
                  <td className="px-4 py-2.5">
                    <span className="text-sm font-medium text-[var(--text-primary)]">{e.name}</span>
                  </td>
                  <td className="px-4 py-2.5 hidden md:table-cell">
                    <div className="flex items-center gap-1.5 text-xs text-[var(--text-secondary)]">
                      <Cpu size={12} />
                      <span>{e.gpu_model}</span>
                      <span className="text-[var(--text-muted)]">({(e.vram_mb / 1024).toFixed(0)} GB)</span>
                    </div>
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    <span className="text-sm font-mono text-emerald-400">{e.tasks_completed.toLocaleString()}</span>
                  </td>
                  <td className="px-4 py-2.5 text-right hidden sm:table-cell">
                    <span className="text-xs font-mono text-[var(--text-secondary)]">
                      {e.avg_tokens_per_sec > 0 ? `${e.avg_tokens_per_sec.toFixed(1)} t/s` : '-'}
                    </span>
                  </td>
                  <td className="px-4 py-2.5 text-right hidden lg:table-cell">
                    <div className="flex items-center justify-end gap-1">
                      <Shield size={10} className={e.trust_score >= 80 ? 'text-emerald-400' : 'text-[var(--text-muted)]'} />
                      <span className={`text-xs font-mono ${
                        e.trust_score >= 80 ? 'text-emerald-400' :
                        e.trust_score >= 50 ? 'text-[var(--text-secondary)]' :
                        'text-red-400'
                      }`}>
                        {e.trust_score}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-2.5 text-center">
                    <span className={`inline-block w-2 h-2 rounded-full ${
                      e.status === 'idle' ? 'bg-emerald-500' :
                      e.status === 'busy' ? 'bg-blue-500 animate-pulse' :
                      'bg-gray-500'
                    }`} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </CardContent>
    </Card>
  );
}
