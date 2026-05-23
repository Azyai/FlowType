import type { AppSettings, AppStatus, AsrServiceCheckResult, AsrServiceConfig, DatabaseHealth } from '../types';

export const defaultSettings: AppSettings = {
  hotkey: 'Alt',
  input_mode: 'hold_to_talk',
  asr_service_mode: 'built_in',
  iflytek_app_id: '',
  iflytek_api_key: '',
  iflytek_api_secret: '',
  iflytek_language: 'zh_cn',
  iflytek_mixed_language: true,
  iflytek_timeout_ms: 10000,
  iflytek_retry_count: 1,
  output_style: 'raw',
  clipboard_restore: 'always',
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

export const fallbackAsrServiceConfig: AsrServiceConfig = {
  provider: 'iflytek',
  service_mode: defaultSettings.asr_service_mode,
  iflytek_app_id_masked: '',
  iflytek_api_key_masked: '',
  iflytek_api_secret_configured: false,
  language: defaultSettings.iflytek_language,
  mixed_language: defaultSettings.iflytek_mixed_language,
  timeout_ms: defaultSettings.iflytek_timeout_ms,
  retry_count: defaultSettings.iflytek_retry_count
};

export const fallbackAsrServiceCheck: AsrServiceCheckResult = {
  status: 'missing_config',
  provider: 'iflytek',
  service_mode: 'built_in',
  message: 'ASR service status is only available inside the desktop app.',
  missing_fields: [],
  checked_at: new Date(0).toISOString()
};
