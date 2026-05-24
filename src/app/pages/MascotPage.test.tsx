import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, test, vi } from 'vitest';

import { MascotPage } from './MascotPage';
import * as bridge from '../../lib/tauri';
import type { VoiceSessionEvent } from '../../types';

let voiceListener: ((event: { payload: VoiceSessionEvent }) => void) | null = null;
let voiceLevelListener: ((event: { payload: number }) => void) | null = null;

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((eventName: string, handler: (event: { payload: unknown }) => void) => {
    if (eventName === 'voice_status_changed') {
      voiceListener = handler as (event: { payload: VoiceSessionEvent }) => void;
    }
    if (eventName === 'voice_level_changed') {
      voiceLevelListener = handler as (event: { payload: number }) => void;
    }
    return Promise.resolve(() => {});
  })
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    startDragging: vi.fn().mockResolvedValue(undefined)
  })
}));

vi.mock('@tauri-apps/api/menu', () => ({
  Menu: {
    new: vi.fn(async ({ items }: { items: Array<{ text: string }> }) => ({
      popup: vi.fn(async () => {
        const menu = document.createElement('div');
        for (const item of items) {
          const element = document.createElement('button');
          element.setAttribute('role', 'menuitem');
          element.textContent = item.text.replace(/\s*✔$/, '');
          menu.appendChild(element);
        }
        document.body.appendChild(menu);
      })
    }))
  }
}));

describe('MascotPage', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    voiceListener = null;
    voiceLevelListener = null;
  });

  test('starts and stops voice input from single click using the shared state machine commands', async () => {
    const user = userEvent.setup();
    const toggleRecording = vi.spyOn(bridge, 'toggleRecording');
    toggleRecording.mockResolvedValueOnce({
      status: 'Listening',
      transcript_partial: null,
      transcript_final: null,
      error_code: null,
      message: null
    });
    toggleRecording.mockResolvedValueOnce({
      status: 'Recognizing',
      transcript_partial: null,
      transcript_final: null,
      error_code: null,
      message: null
    });

    render(<MascotPage />);
    const mascot = screen.getByRole('button', { name: 'FlowType voice mascot' });
    await user.click(mascot);
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
    await new Promise((resolve) => setTimeout(resolve, 260));
    await user.click(mascot);

    expect(toggleRecording).toHaveBeenCalledTimes(2);
  });

  test('ignores duplicate single clicks that arrive within the guard window', async () => {
    const user = userEvent.setup();
    vi.spyOn(Date, 'now').mockReturnValueOnce(2_000).mockReturnValueOnce(2_080);
    const toggleRecording = vi.spyOn(bridge, 'toggleRecording').mockResolvedValue({
      status: 'Listening',
      transcript_partial: null,
      transcript_final: null,
      error_code: null,
      message: null
    });

    render(<MascotPage />);
    const mascot = screen.getByRole('button', { name: 'FlowType voice mascot' });

    await user.click(mascot);
    await user.click(mascot);

    expect(toggleRecording).toHaveBeenCalledTimes(1);
  });

  test('keeps listening style while showing live partial transcript and opens right click menu', async () => {
    const user = userEvent.setup();
    vi.spyOn(bridge, 'openSettingsWindow').mockResolvedValue(undefined);
    vi.spyOn(bridge, 'hideMascotWindow').mockResolvedValue(undefined);

    const { container } = render(<MascotPage />);
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
      voiceLevelListener?.({ payload: 0.72 });
      voiceListener?.({
        payload: {
          status: 'Listening',
          transcript_partial: 'hello world',
          transcript_final: null,
          error_code: null,
          message: null
        }
      });
    });

    expect(await screen.findByText('hello world')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'FlowType voice mascot' })).toHaveClass('listening');
    expect(container.querySelector('.mascot-ripple')).toBeInTheDocument();
    await user.pointer({ keys: '[MouseRight]', target: screen.getByRole('button', { name: 'FlowType voice mascot' }) });

    expect(screen.getByRole('menuitem', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: 'Hide floating window' })).toBeInTheDocument();
  });

  test('shows voice ripple while listening and reacts to live level events', async () => {
    const { container } = render(<MascotPage />);

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
      voiceLevelListener?.({ payload: 0.64 });
    });

    const ripple = container.querySelector('.mascot-ripple') as HTMLElement | null;
    expect(ripple).toBeInTheDocument();
    expect(ripple?.style.getPropertyValue('--voice-level')).toBe('0.640');
  });
});
