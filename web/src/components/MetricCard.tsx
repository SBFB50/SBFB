import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

interface MetricCardProps {
  label: string;
  value: string | number;
  icon: LucideIcon;
  color?: string;
  subtitle?: ReactNode;
}

export default function MetricCard({ label, value, icon: Icon, color = 'var(--accent)', subtitle }: MetricCardProps) {
  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-5 flex items-start gap-4">
      <div
        className="p-3 rounded-lg shrink-0"
        style={{ backgroundColor: `color-mix(in srgb, ${color} 15%, transparent)` }}
      >
        <Icon size={22} style={{ color }} />
      </div>
      <div className="min-w-0">
        <p className="text-xs font-medium text-[var(--text-muted)] uppercase tracking-wider mb-1">{label}</p>
        <p className="text-2xl font-bold text-[var(--text-primary)]">{value}</p>
        {subtitle && <p className="text-xs text-[var(--text-secondary)] mt-1">{subtitle}</p>}
      </div>
    </div>
  );
}
