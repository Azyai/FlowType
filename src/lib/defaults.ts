import type { AppSettings, AppStatus, DatabaseHealth } from '../types';

export const defaultSettings: AppSettings = {
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

export const fallbackStatus: AppStatus = {
  app_version: '0.1.0',
  paused: false,
  current_mode: defaultSettings.output_style,
  tray_available: false
};

export const fallbackDatabaseHealth: DatabaseHealth = {
  ok: false,
  path: 'app.db',
  applied_migrations: 0,
  last_error: 'Native database is only available inside the desktop app.'
};
