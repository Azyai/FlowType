export type InputMode = 'hold_to_talk' | 'toggle';
export type AsrMode = 'local_first' | 'cloud_first' | 'cloud_only';
export type OutputStyle = 'raw' | 'clean' | 'formal';
export type ClipboardRestore = 'always' | 'delayed' | 'never' | 'text_only';
export type FloatingWindowPosition = 'bottom_right' | 'cursor_nearby';
export type UpdateChannel = 'stable' | 'beta' | 'dev';
export type LocaleCode = 'zh-CN' | 'en-US';
export type LocalePreference = 'auto' | LocaleCode;

export interface AppSettings {
  hotkey: string;
  input_mode: InputMode;
  asr_mode: AsrMode;
  default_model: string;
  output_style: OutputStyle;
  clipboard_restore: ClipboardRestore;
  floating_window_position: FloatingWindowPosition;
  save_history: boolean;
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

export interface NativeErrorShape {
  code: string;
  message: string;
  details?: string | null;
}
