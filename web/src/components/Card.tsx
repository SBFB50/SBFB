import type { ReactNode } from 'react';

interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  action?: ReactNode;
}

export default function Card({ children, className = '', title, action }: CardProps) {
  return (
    <div className={`bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-5 ${className}`}>
      {(title || action) && (
        <div className="flex items-center justify-between mb-4">
          {title && <h3 className="text-sm font-semibold text-[var(--text-primary)] uppercase tracking-wider">{title}</h3>}
          {action}
        </div>
      )}
      {children}
    </div>
  );
}
