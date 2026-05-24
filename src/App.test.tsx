import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import App from './App';
import * as bridge from './lib/tauri';
import type { AppSettings } from './types';

const writeText = vi.fn().mockResolvedValue(undefined);

const settings: AppSettings = {
  hotkey: 'Ctrl+Alt+V',
  input_mode: 'hold_to_talk',
  toggle_hotkey: 'Ctrl+Alt+M',
  rtasr_app_id: '',
  rtasr_api_key: '',
  rtasr_language: 'zh_cn',
  rtasr_timeout_ms: 10000,
  output_style: 'raw',
  clipboard_restore: 'never',
  floating_window_position: 'bottom_right',
  show_floating_window: true,
  floating_window_always_on_top: true,
  floating_window_animation_enabled: true,
  save_history: true,
  history_retention_days: 14,
  vad_enabled: false,
  hotwords_enabled: false,
  min_recording_ms: 500,
  max_recording_ms: 60000,
  auto_start: false,
  update_channel: 'stable',
  update_manifest_url: 'mock://updates/stable.json',
  auto_check_update: false,
  locale_preference: 'auto'
};

describe('FlowType settings shell', () => {
  const originalLanguage = navigator.language;

  beforeEach(() => {
    vi.useRealTimers();
    Object.defineProperty(navigator, 'language', {
      value: 'en-US',
      configurable: true
    });
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true
    });
    vi.spyOn(bridge, 'getSettings').mockResolvedValue(settings);
    vi.spyOn(bridge, 'saveSettings').mockResolvedValue(settings);
    vi.spyOn(bridge, 'getAppStatus').mockResolvedValue({
      app_version: '0.1.0',
      paused: false,
      current_mode: 'raw',
      tray_available: true
    });
    vi.spyOn(bridge, 'getDatabaseHealth').mockResolvedValue({
      ok: true,
      path: 'app.db',
      applied_migrations: 2,
      last_error: null
    });
    vi.spyOn(bridge, 'checkUpdate').mockResolvedValue({
      status: 'available',
      current_version: '0.1.0',
      latest_version: '0.1.1',
      channel: 'stable',
      notes: 'Mock release',
      manifest_url: 'mock://updates/stable.json'
    });
    vi.spyOn(bridge, 'setAutostart').mockResolvedValue(settings);
    vi.spyOn(bridge, 'checkAsrService').mockResolvedValue({
      status: 'ready',
      provider: 'xfyun_rtasr',
      message: 'RTASR credentials are configured.',
      missing_fields: [],
      checked_at: '0'
    });
    vi.spyOn(bridge, 'clearHistory').mockResolvedValue({ deleted_count: 0 });
    vi.spyOn(bridge, 'deleteHistoryItem').mockResolvedValue({ deleted_count: 1 });
    vi.spyOn(bridge, 'getHistory').mockResolvedValue({
      items: [
        {
          id: 1,
          raw_text: 'raw transcript',
          final_text: 'this is a long final transcript that should be truncated in history view',
          output_style: 'raw',
          recognition_started_at: 1700000000,
          recognition_duration_ms: 820,
          injected: true,
          error_code: null,
          error_summary: null,
          created_at: 1700000000
        }
      ],
      total: 1,
      limit: 20,
      offset: 0
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    Object.defineProperty(navigator, 'language', {
      value: originalLanguage,
      configurable: true
    });
    vi.restoreAllMocks();
    writeText.mockClear();
  });

  test('renders the hotkey page after loading native state', async () => {
    render(<App />);

    expect(await screen.findByRole('heading', { name: 'Hotkey' })).toBeInTheDocument();
    expect(screen.getByText('Version 0.1.0')).toBeInTheDocument();
    expect(screen.getByLabelText('Hold-to-talk hotkey')).toBeInTheDocument();
    expect(screen.getByLabelText('Toggle recording hotkey')).toBeInTheDocument();
  });

  test('shows only user-facing settings pages in navigation', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    expect(screen.queryByRole('button', { name: 'Status' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'ASR Service' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Permissions' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Text Output' })).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    expect(screen.getByRole('heading', { name: 'Hotkey' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    expect(screen.getByRole('heading', { name: 'Advanced' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'History' }));
    expect(screen.getByRole('heading', { name: 'History' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'About' }));
    expect(screen.getByRole('heading', { name: 'About FlowType' })).toBeInTheDocument();
  });

  test('saves changed configuration values through the native bridge', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    await user.click(screen.getByLabelText('Hold-to-talk hotkey'));
    await user.keyboard('{Control>}{Space}{/Control}');
    await user.click(screen.getByLabelText('Toggle recording hotkey'));
    await user.keyboard('{Control>}{Shift>}{KeyT}{/Shift}{/Control}');
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => {
      expect(bridge.saveSettings).toHaveBeenCalledWith(
        expect.objectContaining({ hotkey: 'Ctrl+Space', toggle_hotkey: 'Ctrl+Shift+T' })
      );
    });
  });

  test('checks updates with a mock manifest and renders the result', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    await user.click(screen.getByRole('button', { name: 'Check update' }));

    expect(await screen.findByText('New version 0.1.1 available')).toBeInTheDocument();
  });

  test('advanced settings include output mode and history controls', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    await user.selectOptions(screen.getByLabelText('Output style'), 'formal');
    await user.selectOptions(screen.getByLabelText('History retention'), '30');
    await user.click(screen.getByRole('checkbox', { name: 'Show floating pet window' }));
    await user.click(screen.getByRole('button', { name: 'Clear history' }));
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => {
      expect(bridge.clearHistory).toHaveBeenCalled();
      expect(bridge.saveSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          output_style: 'formal',
          history_retention_days: 30,
          show_floating_window: false
        })
      );
    });
  });

  test('renders compact transcript history items and supports copy/delete actions', async () => {
    const user = userEvent.setup();
    vi.mocked(bridge.getHistory)
      .mockResolvedValueOnce({
        items: [
          {
            id: 1,
            raw_text: 'raw transcript',
            final_text: 'this is a long final transcript that should be truncated in history view',
            output_style: 'raw',
            recognition_started_at: 1700000000,
            recognition_duration_ms: 820,
            injected: true,
            error_code: null,
            error_summary: null,
            created_at: 1700000000
          }
        ],
        total: 1,
        limit: 20,
        offset: 0
      })
      .mockResolvedValueOnce({
        items: [],
        total: 0,
        limit: 20,
        offset: 0
      });
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'History' }));

    expect(
      await screen.findByText('this is a long final transcript...')
    ).toBeInTheDocument();
    expect(screen.queryByText('raw transcript')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Copy' }));
    expect(writeText).toHaveBeenCalledWith(
      'this is a long final transcript that should be truncated in history view'
    );

    await user.click(screen.getByRole('button', { name: 'Delete' }));

    await waitFor(() => {
      expect(bridge.deleteHistoryItem).toHaveBeenCalledWith(1);
      expect(bridge.getHistory).toHaveBeenCalledTimes(2);
    });
  });

  test('toggles autostart through the native bridge', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    await user.click(screen.getByRole('checkbox', { name: 'Launch FlowType at startup' }));

    await waitFor(() => {
      expect(bridge.setAutostart).toHaveBeenCalledWith(true);
    });
  });

  test('uses simplified Chinese automatically for Chinese regional locales', async () => {
    Object.defineProperty(navigator, 'language', {
      value: 'zh-CN',
      configurable: true
    });
    render(<App />);

    expect(await screen.findByRole('heading', { name: '快捷键' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '高级设置' })).toBeInTheDocument();
  });

  test('allows manual language selection and persists the preference', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    await user.selectOptions(screen.getByLabelText('Display language'), 'zh-CN');
    expect(await screen.findByRole('heading', { name: '高级设置' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: '保存设置' }));

    await waitFor(() => {
      expect(bridge.saveSettings).toHaveBeenCalledWith(
        expect.objectContaining({ locale_preference: 'zh-CN' })
      );
    });
  });

  test('shows global feedback as a top toast and hides it after three seconds', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Hotkey' });
    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByRole('status')).toHaveTextContent('Settings saved');

    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    }, { timeout: 4000 });
  });
});
