import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

interface MetricCardProps {
  label: string;
  value: string | number;
  icon?: LucideIcon;
  color?: string;
  subtitle?: ReactNode;
}

export default function MetricCard({ label, value, icon: Icon, color = 'var(--accent)', subtitle }: MetricCardProps) {
  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4 flex items-start gap-3">
      {Icon && (
        <div
          className="p-2.5 rounded-lg shrink-0"
          style={{ backgroundColor: `color-mix(in srgb, ${color} 15%, transparent)` }}
        >
          <Icon size={20} style={{ color }} />
        </div>
      )}
      <div className="min-w-0">
        <p className="text-xs font-medium text-[var(--text-muted)] uppercase tracking-wider mb-0.5">{label}</p>
        <p className="text-xl font-bold text-[var(--text-primary)]">{value}</p>
        {subtitle && <p className="text-xs text-[var(--text-secondary)] mt-1">{subtitle}</p>}
      </div>
    </div>
  );
}
