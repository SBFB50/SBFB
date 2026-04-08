import { useState, useEffect, useCallback, useRef } from 'react';
import { X, AlertCircle, CheckCircle, Info } from 'lucide-react';

export interface ToastMessage {
  id: string;
  type: 'error' | 'success' | 'info';
  message: string;
}

let toastListener: ((msg: ToastMessage) => void) | null = null;

/** Dedup window: suppress identical messages within this period (ms). */
const DEDUP_WINDOW_MS = 2000;
const recentMessages = new Map<string, number>();

/** Show a toast from anywhere (no React context needed). */
export function showToast(type: ToastMessage['type'], message: string) {
  // Dedup: skip if same type+message was shown recently
  const key = `${type}:${message}`;
  const now = Date.now();
  const lastShown = recentMessages.get(key);
  if (lastShown && now - lastShown < DEDUP_WINDOW_MS) return;
  recentMessages.set(key, now);
  // Prune old entries periodically
  if (recentMessages.size > 50) {
    for (const [k, ts] of recentMessages) {
      if (now - ts > DEDUP_WINDOW_MS) recentMessages.delete(k);
    }
  }

  const id = `${now}-${Math.random().toString(36).slice(2, 8)}`;
  toastListener?.({ id, type, message });
}

const ICON_MAP = {
  error: AlertCircle,
  success: CheckCircle,
  info: Info,
} as const;

const COLOR_MAP = {
  error: 'border-red-500/50 bg-red-500/10 text-red-400',
  success: 'border-green-500/50 bg-green-500/10 text-green-400',
  info: 'border-blue-500/50 bg-blue-500/10 text-blue-400',
} as const;

export default function ToastContainer() {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: string) => {
    const timer = timersRef.current.get(id);
    if (timer) { clearTimeout(timer); timersRef.current.delete(id); }
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  const addToast = useCallback((msg: ToastMessage) => {
    setToasts(prev => [...prev.slice(-4), msg]);
    // Each toast gets its own independent 5s auto-dismiss timer
    const timer = setTimeout(() => removeToast(msg.id), 5000);
    timersRef.current.set(msg.id, timer);
  }, [removeToast]);

  // Register global listener
  useEffect(() => {
    toastListener = addToast;
    return () => { toastListener = null; };
  }, [addToast]);

  // Cleanup all timers on unmount
  useEffect(() => {
    const timers = timersRef.current;
    return () => { timers.forEach(t => clearTimeout(t)); timers.clear(); };
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm" role="log" aria-live="assertive" aria-atomic="false">
      {toasts.map(toast => {
        const Icon = ICON_MAP[toast.type];
        return (
          <div
            key={toast.id}
            role="alert"
            className={`flex items-start gap-2 px-3 py-2.5 rounded-lg border text-sm animate-[slideIn_0.2s_ease-out] ${COLOR_MAP[toast.type]}`}
          >
            <Icon size={14} className="shrink-0 mt-0.5" aria-hidden="true" />
            <span className="flex-1 break-words">{toast.message}</span>
            <button onClick={() => removeToast(toast.id)} className="shrink-0 opacity-60 hover:opacity-100" aria-label="Fermer la notification">
              <X size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
