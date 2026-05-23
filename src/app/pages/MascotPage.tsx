import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type { AppStateStatus, VoiceSessionEvent } from '../../types';
import { hideMascotWindow, openSettingsWindow, setOutputMode, startVoiceInput, stopVoiceInput } from '../../lib/tauri';
import '../styles/mascot.css';

const ACTIVE_STATUSES: AppStateStatus[] = ['Listening', 'Uploading', 'Recognizing'];

export function MascotPage() {
  const [status, setStatus] = useState<AppStateStatus>('Idle');
  const [partial, setPartial] = useState('');
  const [menuOpen, setMenuOpen] = useState(false);
  const dragMoved = useRef(false);

  useEffect(() => {
    document.documentElement.style.setProperty('background', 'transparent', 'important');
    document.body.style.setProperty('background', 'transparent', 'important');
    document.getElementById('root')?.style.setProperty('background', 'transparent', 'important');

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
    setMenuOpen((open) => !open);
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
      {menuOpen && (
        <div className="mascot-menu" role="menu">
          <button type="button" role="menuitem" onClick={openSettingsWindow}>
            Settings
          </button>
          <button type="button" role="menuitem" onClick={() => setOutputMode('raw')}>
            Raw transcript
          </button>
          <button type="button" role="menuitem" onClick={() => setOutputMode('clean')}>
            Clean text
          </button>
          <button type="button" role="menuitem" onClick={() => setOutputMode('formal')}>
            Formal writing
          </button>
          <button type="button" role="menuitem" onClick={() => hideMascotWindow()}>
            Hide floating window
          </button>
        </div>
      )}
    </div>
  );
}
