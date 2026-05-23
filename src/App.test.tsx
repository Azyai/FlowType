import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import App from './App';
import * as bridge from './lib/tauri';
import type { AppSettings } from './types';

const settings: AppSettings = {
  hotkey: 'Alt',
  input_mode: 'hold_to_talk',
  asr_mode: 'local_first',
  default_model: 'whisper-small-q8',
  output_style: 'clean',
  clipboard_restore: 'always',
  floating_window_position: 'bottom_right',
  save_history: true,
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
    vi.spyOn(bridge, 'getSettings').mockResolvedValue(settings);
    vi.spyOn(bridge, 'saveSettings').mockResolvedValue(settings);
    vi.spyOn(bridge, 'getAppStatus').mockResolvedValue({
      app_version: '0.1.0',
      paused: false,
      current_mode: 'clean',
      tray_available: true
    });
    vi.spyOn(bridge, 'getDatabaseHealth').mockResolvedValue({
      ok: true,
      path: 'app.db',
      applied_migrations: 1,
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
  });

  afterEach(() => {
    vi.useRealTimers();
    Object.defineProperty(navigator, 'language', {
      value: originalLanguage,
      configurable: true
    });
    vi.restoreAllMocks();
  });

  test('renders the status dashboard after loading native state', async () => {
    render(<App />);

    expect(await screen.findByRole('heading', { name: 'Status' })).toBeInTheDocument();
    expect(screen.getByText('Version 0.1.0')).toBeInTheDocument();
    expect(screen.getByText('SQLite healthy')).toBeInTheDocument();
    expect(screen.getByText('Mode: clean')).toBeInTheDocument();
  });

  test('navigates between all Phase 0 settings pages', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Status' });
    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    expect(screen.getByRole('heading', { name: 'Hotkey' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Voice Model' }));
    expect(screen.getByRole('heading', { name: 'Voice Model' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Permissions' }));
    expect(screen.getByRole('heading', { name: 'Permissions' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Text Output' }));
    expect(screen.getByRole('heading', { name: 'Text Output' })).toBeInTheDocument();

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

    await screen.findByRole('heading', { name: 'Status' });
    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    await user.clear(screen.getByLabelText('Hold-to-talk hotkey'));
    await user.type(screen.getByLabelText('Hold-to-talk hotkey'), 'Ctrl+Space');
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => {
      expect(bridge.saveSettings).toHaveBeenCalledWith(
        expect.objectContaining({ hotkey: 'Ctrl+Space' })
      );
    });
  });

  test('checks updates with a mock manifest and renders the result', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Status' });
    await user.click(screen.getByRole('button', { name: 'Advanced' }));
    await user.click(screen.getByRole('button', { name: 'Check update' }));

    expect(await screen.findByText('New version 0.1.1 available')).toBeInTheDocument();
  });

  test('toggles autostart through the native bridge', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Status' });
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

    expect(await screen.findByRole('heading', { name: '状态' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '高级设置' })).toBeInTheDocument();
  });

  test('allows manual language selection and persists the preference', async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole('heading', { name: 'Status' });
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
    vi.useFakeTimers();
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(<App />);

    await screen.findByRole('heading', { name: 'Status' });
    await user.click(screen.getByRole('button', { name: 'Hotkey' }));
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByRole('status')).toHaveTextContent('Settings saved');

    vi.advanceTimersByTime(3000);

    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });
  });
});
