import { AlertTriangle } from 'lucide-react';
import { useEffect } from 'react';

export interface ConfirmDialogState {
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  tone?: 'danger' | 'default';
}

export function ConfirmDialog({
  dialog,
  onCancel,
  onConfirm
}: {
  dialog: ConfirmDialogState | null;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  useEffect(() => {
    if (!dialog) {
      return undefined;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onCancel();
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [dialog, onCancel]);

  if (!dialog) {
    return null;
  }

  return (
    <div className="confirm-dialog-backdrop" onClick={onCancel}>
      <div
        className="confirm-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-message"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="confirm-dialog-icon">
          <AlertTriangle aria-hidden="true" />
        </div>
        <div className="confirm-dialog-content">
          <h2 id="confirm-dialog-title">{dialog.title}</h2>
          <p id="confirm-dialog-message">{dialog.message}</p>
        </div>
        <div className="confirm-dialog-actions">
          <button type="button" className="secondary-button" onClick={onCancel}>
            {dialog.cancelLabel}
          </button>
          <button
            type="button"
            className={`primary-button${dialog.tone === 'danger' ? ' danger' : ''}`}
            onClick={onConfirm}
            autoFocus
          >
            {dialog.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
