import type { ToastState } from '../hooks/useSettingsShell';

export function Toast({ toast }: { toast: ToastState | null }) {
  if (!toast) return null;

  return (
    <div className={`toast toast-${toast.kind}`} role="status" aria-live="polite">
      {toast.message}
    </div>
  );
}
