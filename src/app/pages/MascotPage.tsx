import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { AppStateStatus } from '../../types';
import '../styles/mascot.css';

export function MascotPage() {
  const [status, setStatus] = useState<AppStateStatus>('Idle');

  useEffect(() => {
    // Force transparent background for the whole window
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

  return (
    <div 
      className="mascot-container" 
      data-tauri-drag-region 
    >
      <div className={`mascot-avatar ${status.toLowerCase()}`} data-tauri-drag-region>
        {status === 'Idle' && <div className="mascot-eyes blink" data-tauri-drag-region></div>}
        {status === 'Listening' && <div className="mascot-ears listening" data-tauri-drag-region></div>}
        {status === 'Processing' && <div className="mascot-spinner" data-tauri-drag-region></div>}
        {status === 'Injecting' && <div className="mascot-sparkles" data-tauri-drag-region></div>}
      </div>
    </div>
  );
}