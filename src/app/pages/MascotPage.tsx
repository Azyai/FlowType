import { useEffect, useRef, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type { AppStateStatus, VoiceSessionEvent } from '../../types';
import { getSettings, saveSettings, openSettingsWindow, setOutputMode, startVoiceInput, stopVoiceInput } from '../../lib/tauri';
import '../styles/mascot.css';

import { Menu } from '@tauri-apps/api/menu';
import { translate } from '../../lib/i18n/I18nContext';
import { resolveLocale } from '../../lib/i18n/locale';

const ACTIVE_STATUSES: AppStateStatus[] = ['Listening', 'Uploading', 'Recognizing'];

export function MascotPage() {
  const [status, setStatus] = useState<AppStateStatus>('Idle');
  const [partial, setPartial] = useState('');
  const [locale, setLocale] = useState(resolveLocale('auto'));
  const dragMoved = useRef(false);

  const t = useCallback((key: any) => translate(locale, key), [locale]);

  useEffect(() => {
    document.documentElement.style.setProperty('background', 'transparent', 'important');
    document.body.style.setProperty('background', 'transparent', 'important');
    document.getElementById('root')?.style.setProperty('background', 'transparent', 'important');

    getSettings()
      .then((settings) => setLocale(resolveLocale(settings.locale_preference)))
      .catch(() => {});

    const unlistenVoice = listen<VoiceSessionEvent>('voice_status_changed', (event) => {
      setStatus(event.payload.status);
      if (event.payload.transcript_partial) {
        setPartial(event.payload.transcript_partial);
      }
      if (event.payload.transcript_final) {
        setPartial(event.payload.transcript_final);
      }
      if (event.payload.status === 'Idle') {
        setPartial('');
      }
    });
    const unlistenLegacy = listen<AppStateStatus>('status_changed', (event) => {
      setStatus(event.payload);
    });

    return () => {
      unlistenVoice.then((fn) => fn());
      unlistenLegacy.then((fn) => fn());
    };
  }, []);

  async function handleMouseDown(event: React.MouseEvent) {
    if (event.button !== 0) return;
    dragMoved.current = false;
    const startX = event.screenX;
    const startY = event.screenY;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const dx = Math.abs(moveEvent.screenX - startX);
      const dy = Math.abs(moveEvent.screenY - startY);
      if (dx > 5 || dy > 5) {
        dragMoved.current = true;
        window.removeEventListener('mousemove', onMouseMove);
        window.removeEventListener('mouseup', onMouseUp);
        getCurrentWindow().startDragging().catch(() => {});
      }
    };

    const onMouseUp = () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  async function handleDoubleClick() {
    if (dragMoved.current) return;
    if (ACTIVE_STATUSES.includes(status)) {
      await stopVoiceInput('mascot');
    } else {
      setPartial('');
      await startVoiceInput('mascot');
    }
  }

  async function handleContextMenu(event: React.MouseEvent) {
    event.preventDefault();
    try {
      const currentSettings = await getSettings();
      const currentMode = currentSettings.output_style || 'raw';
      
      const menu = await Menu.new({
        items: [
          {
            id: 'settings',
            text: t('label.settings'),
            action: openSettingsWindow
          },
          {
            id: 'raw',
            text: t('output.raw') + (currentMode === 'raw' ? ' ✔' : ''),
            action: () => setOutputMode('raw')
          },
          {
            id: 'clean',
            text: t('output.clean') + (currentMode === 'clean' ? ' ✔' : ''),
            action: () => setOutputMode('clean')
          },
          {
            id: 'formal',
            text: t('output.formal') + (currentMode === 'formal' ? ' ✔' : ''),
            action: () => setOutputMode('formal')
          },
          {
            id: 'hide',
            text: t('label.hideFloatingWindow'),
            action: async () => {
              const latest = await getSettings();
              await saveSettings({ ...latest, show_floating_window: false });
            }
          }
        ]
      });
      await menu.popup();
    } catch (e) {
      console.warn("Native menu not available, falling back to basic");
    }
  }

  return (
    <div className="mascot-container" onContextMenu={handleContextMenu}>
      <button
        type="button"
        aria-label="FlowType voice mascot"
        className={`mascot-avatar ${status.toLowerCase()}`}
        onMouseDown={handleMouseDown}
        onDoubleClick={handleDoubleClick}
      >
        <span className="mascot-face" aria-hidden="true">
          <span className="mascot-eyes blink" />
        </span>
      </button>
      <div className="mascot-tooltip">
        <strong>{status}</strong>
        {partial && <span>{partial}</span>}
      </div>
    </div>
  );
}
