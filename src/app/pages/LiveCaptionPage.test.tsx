import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { LiveCaptionPage } from './LiveCaptionPage';
import type { VoiceSessionEvent } from '../../types';

let voiceListener: ((event: { payload: VoiceSessionEvent }) => void) | null = null;
const setIgnoreCursorEvents = vi.fn().mockResolvedValue(undefined);
const setAlwaysOnTop = vi.fn().mockResolvedValue(undefined);
const showWindow = vi.fn().mockResolvedValue(undefined);
const hideWindow = vi.fn().mockResolvedValue(undefined);

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((eventName: string, handler: (event: { payload: unknown }) => void) => {
    if (eventName === 'voice_status_changed') {
      voiceListener = handler as (event: { payload: VoiceSessionEvent }) => void;
    }
    return Promise.resolve(() => {});
  })
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    setIgnoreCursorEvents,
    setAlwaysOnTop,
    show: showWindow,
    hide: hideWindow
  })
}));

describe('LiveCaptionPage', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    voiceListener = null;
    vi.useRealTimers();
    vi.restoreAllMocks();
    setIgnoreCursorEvents.mockClear();
    setAlwaysOnTop.mockClear();
    showWindow.mockClear();
    hideWindow.mockClear();
  });

  test('reveals partial transcript progressively and keeps the overlay visible', async () => {
    render(<LiveCaptionPage />);

    await act(async () => {
      voiceListener?.({
        payload: {
          status: 'Listening',
          transcript_partial: '你好世界',
          transcript_final: null,
          error_code: null,
          message: null
        }
      });
    });

    await revealAllText('你好世界');

    expect(screen.getByText('实时识别中')).toBeInTheDocument();
    expect(screen.getByText('你好世界')).toBeInTheDocument();
    expect(document.querySelector('.live-caption-shell')?.className).toContain('visible');
    expect(setIgnoreCursorEvents).toHaveBeenCalledWith(true);
    expect(showWindow).toHaveBeenCalled();
  });

  test('shows the overlay immediately when listening starts even before partial text arrives', async () => {
    render(<LiveCaptionPage />);

    await act(async () => {
      voiceListener?.({
        payload: {
          status: 'Listening',
          transcript_partial: null,
          transcript_final: null,
          error_code: null,
          message: null
        }
      });
    });

    expect(screen.getByText('实时识别中')).toBeInTheDocument();
    expect(screen.getByText('正在聆听...')).toBeInTheDocument();
    expect(document.querySelector('.live-caption-shell')?.className).toContain('visible');
    expect(showWindow).toHaveBeenCalled();
  });

  test('keeps the final transcript briefly and hides after success', async () => {
    render(<LiveCaptionPage />);

    await act(async () => {
      voiceListener?.({
        payload: {
          status: 'Listening',
          transcript_partial: 'flow',
          transcript_final: null,
          error_code: null,
          message: null
        }
      });
      vi.advanceTimersByTime(120);
      voiceListener?.({
        payload: {
          status: 'Success',
          transcript_partial: null,
          transcript_final: 'flow type',
          error_code: null,
          message: null
        }
      });
    });

    await revealAllText('flow type');
    expect(screen.getByText('flow type')).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(950);
    });

    expect(document.querySelector('.live-caption-shell')?.className).not.toContain('visible');
    expect(hideWindow).toHaveBeenCalled();
  });

  test('shows only the latest 20 characters for long transcripts', async () => {
    const longText = '这是一个很长很长的实时字幕内容用于测试最新二十字展示策略';
    const expected = `...${Array.from(longText).slice(-20).join('')}`;

    render(<LiveCaptionPage />);

    await act(async () => {
      voiceListener?.({
        payload: {
          status: 'Listening',
          transcript_partial: longText,
          transcript_final: null,
          error_code: null,
          message: null
        }
      });
    });

    await revealAllText(expected);

    expect(screen.getByText(expected)).toBeInTheDocument();
  });
});

async function revealAllText(text: string) {
  for (let index = 0; index < text.length; index += 1) {
    await act(async () => {
      vi.advanceTimersByTime(20);
    });
  }
}
