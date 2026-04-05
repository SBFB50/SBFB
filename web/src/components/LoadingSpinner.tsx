interface LoadingSpinnerProps {
  size?: number;
  className?: string;
  text?: string;
}

export default function LoadingSpinner({ size = 32, className = '', text }: LoadingSpinnerProps) {
  return (
    <div className={`flex flex-col items-center justify-center gap-3 ${className}`}>
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className="animate-spin">
        <circle cx="12" cy="12" r="10" stroke="var(--border)" strokeWidth="3" />
        <path
          d="M12 2a10 10 0 0 1 10 10"
          stroke="var(--accent)"
          strokeWidth="3"
          strokeLinecap="round"
        />
      </svg>
      {text && <p className="text-sm text-[var(--text-muted)]">{text}</p>}
    </div>
  );
}
