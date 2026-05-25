import type {
  AppSettings,
  AppStatus,
  AsrServiceCheckResult,
  AsrServiceConfig,
  DatabaseHealth,
  TranscriptHistoryPage
} from '../types';

export const defaultSettings: AppSettings = {
  hotkey: 'Ctrl+Alt+V',
  input_mode: 'hold_to_talk',
  toggle_hotkey: 'Ctrl+Alt+M',
  rtasr_app_id: '',
  rtasr_api_key: '',
  rtasr_language: 'zh_en',
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
  locale_preference: 'auto',
  formal_scene: 'general'
};

export const fallbackStatus: AppStatus = {
  app_version: '0.1.1',
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
  provider: 'xfyun_rtasr',
  rtasr_app_id_masked: '',
  rtasr_api_key_masked: '',
  language: defaultSettings.rtasr_language,
  timeout_ms: defaultSettings.rtasr_timeout_ms
};

export const fallbackAsrServiceCheck: AsrServiceCheckResult = {
  status: 'missing_config',
  provider: 'xfyun_rtasr',
  message: 'ASR service status is only available inside the desktop app.',
  missing_fields: [],
  checked_at: new Date(0).toISOString()
};

export const fallbackTranscriptHistoryPage: TranscriptHistoryPage = {
  items: [],
  total: 0,
  limit: 20,
  offset: 0
};
