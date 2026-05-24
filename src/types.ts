export type InputMode = 'hold_to_talk' | 'toggle';
export type RtasrLanguage = 'zh_cn' | 'en_us' | 'zh_en';
export type HistoryRetentionDays = 7 | 14 | 30;
export type OutputStyle = 'raw' | 'clean' | 'formal';
export type ClipboardRestore = 'always' | 'delayed' | 'never' | 'text_only';
export type FloatingWindowPosition = 'bottom_right' | 'cursor_nearby';
export type UpdateChannel = 'stable' | 'beta' | 'dev';
export type LocaleCode = 'zh-CN' | 'en-US';
export type LocalePreference = 'auto' | LocaleCode;
export type AppStateStatus = 'Idle' | 'Listening' | 'Uploading' | 'Recognizing' | 'Injecting' | 'Success' | 'Failed';
export type VoiceTrigger = 'hotkey' | 'mascot' | 'tray';

export interface AppSettings {
  hotkey: string;
  input_mode: InputMode;
  rtasr_app_id: string;
  rtasr_api_key: string;
  rtasr_language: RtasrLanguage;
  rtasr_timeout_ms: number;
  output_style: OutputStyle;
  clipboard_restore: ClipboardRestore;
  floating_window_position: FloatingWindowPosition;
  show_floating_window: boolean;
  floating_window_always_on_top: boolean;
  floating_window_animation_enabled: boolean;
  save_history: boolean;
  history_retention_days: HistoryRetentionDays;
  vad_enabled: boolean;
  hotwords_enabled: boolean;
  min_recording_ms: number;
  max_recording_ms: number;
  auto_start: boolean;
  update_channel: UpdateChannel;
  update_manifest_url: string;
  auto_check_update: boolean;
  locale_preference: LocalePreference;
}

export interface AppStatus {
  app_version: string;
  paused: boolean;
  current_mode: string;
  tray_available: boolean;
}

export interface DatabaseHealth {
  ok: boolean;
  path: string;
  applied_migrations: number;
  last_error: string | null;
}

export type UpdateStatus = 'latest' | 'available' | 'failed' | 'channel_unavailable';

export interface UpdateCheckResult {
  status: UpdateStatus;
  current_version: string;
  latest_version: string | null;
  channel: UpdateChannel;
  notes: string | null;
  manifest_url: string;
}

export type AsrServiceStatus = 'ready' | 'missing_config';

export interface AsrServiceConfig {
  provider: 'xfyun_rtasr';
  rtasr_app_id_masked: string;
  rtasr_api_key_masked: string;
  language: RtasrLanguage;
  timeout_ms: number;
}

export interface AsrServiceCheckResult {
  status: AsrServiceStatus;
  provider: 'xfyun_rtasr';
  message: string;
  missing_fields: string[];
  checked_at: string;
}

export interface ClearHistoryResult {
  deleted_count: number;
}

export interface TranscriptHistoryItem {
  id: number;
  raw_text: string;
  final_text: string;
  output_style: OutputStyle;
  recognition_started_at: number;
  recognition_duration_ms: number;
  injected: boolean;
  error_code: string | null;
  error_summary: string | null;
  created_at: number;
}

export interface TranscriptHistoryPage {
  items: TranscriptHistoryItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface VoiceSessionEvent {
  status: AppStateStatus;
  transcript_partial: string | null;
  transcript_final: string | null;
  error_code: string | null;
  message: string | null;
}

export interface NativeErrorShape {
  code: string;
  message: string;
  details?: string | null;
}
