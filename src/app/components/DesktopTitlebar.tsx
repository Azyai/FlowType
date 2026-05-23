import { Minus, Square, X } from 'lucide-react';
import type { MouseEvent } from 'react';

import { hideWindow, minimizeWindow, startWindowDrag, toggleMaximizeWindow } from '../../lib/windowControls';
import { useI18n } from '../../lib/i18n/I18nContext';

export function DesktopTitlebar() {
  const { t } = useI18n();

  function handleDragStart(event: MouseEvent<HTMLElement>) {
    if (event.button !== 0) return;
    if ((event.target as HTMLElement).closest('button')) return;
    void startWindowDrag();
  }

  return (
    <header className="desktop-titlebar" data-tauri-drag-region onMouseDown={handleDragStart}>
      <div className="titlebar-drag-zone" data-tauri-drag-region />

      <div className="window-controls">
        <button type="button" className="window-control" aria-label={t('window.minimize')} onClick={minimizeWindow}>
          <Minus aria-hidden="true" />
        </button>
        <button
          type="button"
          className="window-control"
          aria-label={t('window.maximize')}
          onClick={toggleMaximizeWindow}
        >
          <Square aria-hidden="true" />
        </button>
        <button type="button" className="window-control close" aria-label={t('window.hide')} onClick={hideWindow}>
          <X aria-hidden="true" />
        </button>
      </div>
    </header>
  );
}
