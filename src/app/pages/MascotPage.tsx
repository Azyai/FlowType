import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { AppStateStatus } from '../../types';
import '../styles/mascot.css';

export function MascotPage() {
  const [status, setStatus] = useState<AppStateStatus>('Idle');
  const dragMoved = useRef(false);

  useEffect(() => {
    document.documentElement.style.setProperty('background', 'transparent', 'important');
    document.body.style.setProperty('background', 'transparent', 'important');
    const root = document.getElementById('root');
    if (root) {
      root.style.setProperty('background', 'transparent', 'important');
    }

    const unlisten = listen<AppStateStatus>('status_changed', (event) => {
      setStatus(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleMouseDown = async (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    dragMoved.current = false;
    const startX = e.screenX;
    const startY = e.screenY;

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
  };

  const handleDoubleClick = async () => {
    if (dragMoved.current) return;
    try {
      await invoke('toggle_recording');
    } catch (err) {
      console.error('Failed to toggle recording:', err);
    }
  };

  return (
    <div className="mascot-container">
      <div
        className={`mascot-avatar ${status.toLowerCase()}`}
        onMouseDown={handleMouseDown}
        onDoubleClick={handleDoubleClick}
      >
        <div className="mascot-eyes blink"></div>
      </div>
    </div>
  );
}