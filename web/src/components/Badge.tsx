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

  // Status badges
  active: 'bg-green-500/20 text-green-400',
  running: 'bg-green-500/20 text-green-400',
  stopped: 'bg-red-500/20 text-red-400',
  idle: 'bg-zinc-500/20 text-zinc-400',
  pending: 'bg-yellow-500/20 text-yellow-400',
  completed: 'bg-blue-500/20 text-blue-400',
  error: 'bg-red-500/20 text-red-400',
  failed: 'bg-red-500/20 text-red-400',

  // Priority / severity
  high: 'bg-red-500/20 text-red-400',
  medium: 'bg-yellow-500/20 text-yellow-400',
  low: 'bg-green-500/20 text-green-400',
  critical: 'bg-red-600/30 text-red-300',
};

interface BadgeProps {
  type: string;
  className?: string;
}

export default function Badge({ type, className = '' }: BadgeProps) {
  const colors = colorMap[type.toLowerCase()] || 'bg-zinc-500/20 text-zinc-400';
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${colors} ${className}`}>
      {type}
    </span>
  );
}
