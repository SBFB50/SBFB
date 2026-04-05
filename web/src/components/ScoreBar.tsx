interface ScoreBarProps {
  label: string;
  score: number; // 0-100
  sublabel?: string;
  className?: string;
}

function getScoreColor(score: number): string {
  if (score >= 70) return 'var(--accent-green)';
  if (score >= 40) return 'var(--accent-yellow)';
  return 'var(--accent-red)';
}

export default function ScoreBar({ label, score, sublabel, className = '' }: ScoreBarProps) {
  const color = getScoreColor(score);
  const clampedScore = Math.max(0, Math.min(100, score));

  return (
    <div className={`${className}`}>
      <div className="flex items-center justify-between mb-1.5">
        <span className="text-sm text-[var(--text-primary)] font-medium truncate pr-3">{label}</span>
        <span className="text-sm font-bold shrink-0" style={{ color }}>
          {Math.round(clampedScore)}%
        </span>
      </div>
      <div className="w-full h-2 bg-[var(--bg-primary)] rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-500"
          style={{ width: `${clampedScore}%`, backgroundColor: color }}
        />
      </div>
      {sublabel && <p className="text-xs text-[var(--text-muted)] mt-1">{sublabel}</p>}
    </div>
  );
}
