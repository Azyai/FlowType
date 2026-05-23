import { Minus, Square, X } from 'lucide-react';

import { hideWindow, minimizeWindow, toggleMaximizeWindow } from '../../lib/windowControls';
import { useI18n } from '../../lib/i18n/I18nContext';

export function DesktopTitlebar() {
  const { t } = useI18n();

  return (
    <header className="desktop-titlebar" data-tauri-drag-region>
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
