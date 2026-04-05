import type { ReactNode } from 'react';

const colorMap: Record<string, string> = {
  person: 'bg-blue-500/20 text-blue-400',
  location: 'bg-green-500/20 text-green-400',
  organization: 'bg-purple-500/20 text-purple-400',
  event: 'bg-yellow-500/20 text-yellow-400',
  vehicle: 'bg-orange-500/20 text-orange-400',
  weapon: 'bg-red-500/20 text-red-400',
  phone: 'bg-cyan-500/20 text-cyan-400',
  email: 'bg-indigo-500/20 text-indigo-400',
  document: 'bg-amber-500/20 text-amber-400',
  date: 'bg-teal-500/20 text-teal-400',
  other: 'bg-zinc-500/20 text-zinc-400',

  // Status
  active: 'bg-green-500/20 text-green-400',
  running: 'bg-green-500/20 text-green-400',
  stopped: 'bg-red-500/20 text-red-400',
  idle: 'bg-zinc-500/20 text-zinc-400',
  pending: 'bg-yellow-500/20 text-yellow-400',
  processing: 'bg-yellow-500/20 text-yellow-400',
  completed: 'bg-blue-500/20 text-blue-400',
  processed: 'bg-green-500/20 text-green-400',
  error: 'bg-red-500/20 text-red-400',
  failed: 'bg-red-500/20 text-red-400',
  refuted: 'bg-red-500/20 text-red-400',
  confirmed: 'bg-green-500/20 text-green-400',

  // Severity
  critical: 'bg-red-600/30 text-red-300',
  warning: 'bg-yellow-500/20 text-yellow-400',
  info: 'bg-blue-500/20 text-blue-400',
  high: 'bg-red-500/20 text-red-400',
  medium: 'bg-yellow-500/20 text-yellow-400',
  low: 'bg-green-500/20 text-green-400',

  // Named variants
  blue: 'bg-blue-500/20 text-blue-400',
  green: 'bg-green-500/20 text-green-400',
  red: 'bg-red-500/20 text-red-400',
  yellow: 'bg-yellow-500/20 text-yellow-400',
  purple: 'bg-purple-500/20 text-purple-400',
  gray: 'bg-zinc-500/20 text-zinc-400',
};

interface BadgeProps {
  type?: string;
  variant?: string;
  children?: ReactNode;
  className?: string;
}

export default function Badge({ type, variant, children, className = '' }: BadgeProps) {
  const key = (variant || type || 'gray').toLowerCase();
  const colors = colorMap[key] || 'bg-zinc-500/20 text-zinc-400';
  const label = children || type || variant || '';

  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${colors} ${className}`}>
      {label}
    </span>
  );
}
