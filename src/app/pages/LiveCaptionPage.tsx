import { useEffect, useMemo, useRef, useState, type MutableRefObject } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type { AppStateStatus, VoiceSessionEvent } from '../../types';
import '../styles/live-caption.css';

const ACTIVE_STATUSES: AppStateStatus[] = ['Listening', 'Uploading', 'Recognizing', 'Injecting'];
const HIDE_DELAY_MS = 900;
const TYPE_INTERVAL_MS = 18;

export function LiveCaptionPage() {
  const [status, setStatus] = useState<AppStateStatus>('Idle');
  const [targetText, setTargetText] = useState('');
  const [displayText, setDisplayText] = useState('');
  const [visible, setVisible] = useState(false);
  const hideTimerRef = useRef<number | null>(null);
  const latestTextRef = useRef('');

  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty('background', 'transparent', 'important');
    document.body.style.setProperty('background', 'transparent', 'important');
    document.getElementById('root')?.style.setProperty('background', 'transparent', 'important');

    const window = getCurrentWindow();
    swallowAsync(window.setIgnoreCursorEvents(true));
    swallowAsync(window.setAlwaysOnTop(true));
    swallowAsync(window.hide());

    const unlistenVoice = listen<VoiceSessionEvent>('voice_status_changed', (event) => {
      const nextStatus = event.payload.status;
      const partial = event.payload.transcript_partial?.trim() ?? '';
      const finalText = event.payload.transcript_final?.trim() ?? '';

      setStatus(nextStatus);

      if (partial) {
        clearHideTimer(hideTimerRef);
        swallowAsync(window.show());
        latestTextRef.current = partial;
        setTargetText(partial);
        setVisible(true);
        return;
      }

      if (finalText) {
        clearHideTimer(hideTimerRef);
        swallowAsync(window.show());
        latestTextRef.current = finalText;
        setTargetText(finalText);
        setVisible(true);
        scheduleHide(hideTimerRef, () => {
          setVisible(false);
          setTargetText('');
          setDisplayText('');
          latestTextRef.current = '';
          swallowAsync(window.hide());
        });
        return;
      }

      if (ACTIVE_STATUSES.includes(nextStatus)) {
        clearHideTimer(hideTimerRef);
        swallowAsync(window.show());
        setVisible(true);
        return;
      }

      if (nextStatus === 'Idle' || nextStatus === 'Success' || nextStatus === 'Failed') {
        scheduleHide(hideTimerRef, () => {
          setVisible(false);
          setTargetText('');
          setDisplayText('');
          latestTextRef.current = '';
          swallowAsync(window.hide());
        });
      }
    });

    return () => {
      clearHideTimer(hideTimerRef);
      unlistenVoice.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (!targetText) {
      if (displayText) {
        setDisplayText('');
      }
      return;
    }

    if (targetText === displayText) {
      return;
    }

    if (targetText.length < displayText.length) {
      setDisplayText(targetText);
      return;
    }

    if (!targetText.startsWith(displayText)) {
      const prefixLength = sharedPrefixLength(displayText, targetText);
      setDisplayText(targetText.slice(0, prefixLength));
      return;
    }

    const timer = window.setTimeout(() => {
      setDisplayText(targetText.slice(0, displayText.length + 1));
    }, TYPE_INTERVAL_MS);

    return () => window.clearTimeout(timer);
  }, [displayText, targetText]);

  const statusLabel = useMemo(() => {
    if (status === 'Listening') return '实时识别中';
    if (status === 'Recognizing' || status === 'Injecting') return '识别收尾中';
    if (status === 'Uploading') return '上传音频中';
    return '实时识别中';
  }, [status]);

  const shouldShowCard = visible && (Boolean(displayText) || ACTIVE_STATUSES.includes(status));
  const captionText = useMemo(() => {
    if (displayText) {
      return displayText;
    }
    if (ACTIVE_STATUSES.includes(status)) {
      return '正在聆听...';
    }
    return '\u00A0';
  }, [displayText, status]);

  return (
    <main className={`live-caption-shell${shouldShowCard ? ' visible' : ''}`}>
      <section className="live-caption-card" aria-live="polite" aria-atomic="true">
        <span className={`live-caption-status ${status.toLowerCase()}`}>{statusLabel}</span>
        <p className={`live-caption-text${displayText ? '' : ' placeholder'}`}>{captionText}</p>
      </section>
    </main>
  );
}

function clearHideTimer(timerRef: MutableRefObject<number | null>) {
  if (timerRef.current !== null) {
    window.clearTimeout(timerRef.current);
    timerRef.current = null;
  }
}

function scheduleHide(
  timerRef: MutableRefObject<number | null>,
  onHide: () => void
) {
  clearHideTimer(timerRef);
  timerRef.current = window.setTimeout(() => {
    timerRef.current = null;
    onHide();
  }, HIDE_DELAY_MS);
}

function sharedPrefixLength(left: string, right: string) {
  const max = Math.min(left.length, right.length);
  let index = 0;
  while (index < max && left[index] === right[index]) {
    index += 1;
  }
  return index;
}

function swallowAsync(result: Promise<unknown> | unknown) {
  if (result && typeof (result as Promise<unknown>).catch === 'function') {
    void (result as Promise<unknown>).catch(() => {});
  }
}
