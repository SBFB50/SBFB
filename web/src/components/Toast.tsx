import { useState, useEffect, useCallback } from 'react';
import { X, AlertCircle, CheckCircle, Info } from 'lucide-react';

export interface ToastMessage {
  id: string;
  type: 'error' | 'success' | 'info';
  message: string;
}

let toastListener: ((msg: ToastMessage) => void) | null = null;

/** Show a toast from anywhere (no React context needed). */
export function showToast(type: ToastMessage['type'], message: string) {
  const id = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
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

  const addToast = useCallback((msg: ToastMessage) => {
    setToasts(prev => [...prev.slice(-4), msg]);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  // Register global listener
  useEffect(() => {
    toastListener = addToast;
    return () => { toastListener = null; };
  }, [addToast]);

  // Auto-dismiss after 5s
  useEffect(() => {
    if (toasts.length === 0) return;
    const oldest = toasts[0];
    const timer = setTimeout(() => removeToast(oldest.id), 5000);
    return () => clearTimeout(timer);
  }, [toasts, removeToast]);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      {toasts.map(toast => {
        const Icon = ICON_MAP[toast.type];
        return (
          <div
            key={toast.id}
            className={`flex items-start gap-2 px-3 py-2.5 rounded-lg border text-sm animate-[slideIn_0.2s_ease-out] ${COLOR_MAP[toast.type]}`}
          >
            <Icon size={14} className="shrink-0 mt-0.5" />
            <span className="flex-1 break-words">{toast.message}</span>
            <button onClick={() => removeToast(toast.id)} className="shrink-0 opacity-60 hover:opacity-100">
              <X size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
