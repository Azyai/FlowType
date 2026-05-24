import { invoke } from '@tauri-apps/api/core';

import {
  defaultSettings,
  fallbackAsrServiceCheck,
  fallbackAsrServiceConfig,
  fallbackDatabaseHealth,
  fallbackTranscriptHistoryPage,
  fallbackStatus
} from './defaults';
import type {
  AppSettings,
  AppStatus,
  AsrServiceCheckResult,
  AsrServiceConfig,
  ClearHistoryResult,
  DatabaseHealth,
  TranscriptHistoryPage,
  UpdateCheckResult,
  VoiceSessionEvent,
  VoiceTrigger,
  OutputStyle
} from '../types';

const isTauriRuntime = () => Boolean('__TAURI_INTERNALS__' in window);

async function nativeInvoke<T>(command: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (!isTauriRuntime()) {
    if (fallback === undefined) {
      throw new Error(`Command ${command} is only available in the desktop runtime.`);
    }
    return fallback;
  }

  return invoke<T>(command, args);
}

export function getSettings(): Promise<AppSettings> {
  return nativeInvoke('get_settings', undefined, defaultSettings);
}

export function saveSettings(settings: AppSettings): Promise<AppSettings> {
  return nativeInvoke('save_settings', { settings }, settings);
}

export function resetSettings(): Promise<AppSettings> {
  return nativeInvoke('reset_settings', undefined, defaultSettings);
}

export function getAppStatus(): Promise<AppStatus> {
  return nativeInvoke('get_app_status', undefined, fallbackStatus);
}

export function setAutostart(enabled: boolean): Promise<AppSettings> {
  return nativeInvoke('set_autostart', { enabled }, { ...defaultSettings, auto_start: enabled });
}

export function getDatabaseHealth(): Promise<DatabaseHealth> {
  return nativeInvoke('get_database_health', undefined, fallbackDatabaseHealth);
}

export function checkUpdate(): Promise<UpdateCheckResult> {
  return nativeInvoke('check_update', undefined, {
    status: 'latest',
    current_version: '0.1.0',
    latest_version: '0.1.0',
    channel: defaultSettings.update_channel,
    notes: null,
    manifest_url: defaultSettings.update_manifest_url
  });
}

export function getAsrServiceConfig(): Promise<AsrServiceConfig> {
  return nativeInvoke('get_asr_service_config', undefined, fallbackAsrServiceConfig);
}

export function saveAsrServiceConfig(config: Pick<
  AppSettings,
  | 'rtasr_app_id'
  | 'rtasr_api_key'
  | 'rtasr_language'
  | 'rtasr_timeout_ms'
>): Promise<AppSettings> {
  return nativeInvoke('save_asr_service_config', { config }, { ...defaultSettings, ...config });
}

export function checkAsrService(): Promise<AsrServiceCheckResult> {
  return nativeInvoke('check_asr_service', undefined, fallbackAsrServiceCheck);
}

export function clearHistory(): Promise<ClearHistoryResult> {
  return nativeInvoke('clear_history', undefined, { deleted_count: 0 });
}

export function getHistory(limit = 20, offset = 0): Promise<TranscriptHistoryPage> {
  return nativeInvoke('get_history', { limit, offset }, { ...fallbackTranscriptHistoryPage, limit, offset });
}

export function startVoiceInput(trigger: VoiceTrigger): Promise<VoiceSessionEvent> {
  return nativeInvoke('start_voice_input', { trigger }, {
    status: 'Listening',
    transcript_partial: null,
    transcript_final: null,
    error_code: null,
    message: null
  });
}

export function stopVoiceInput(trigger: VoiceTrigger): Promise<VoiceSessionEvent> {
  return nativeInvoke('stop_voice_input', { trigger }, {
    status: 'Recognizing',
    transcript_partial: null,
    transcript_final: null,
    error_code: null,
    message: null
  });
}

export function toggleRecording(): Promise<VoiceSessionEvent> {
  return nativeInvoke('toggle_recording', undefined, {
    status: 'Listening',
    transcript_partial: null,
    transcript_final: null,
    error_code: null,
    message: null
  });
}

export function cancelVoiceInput(): Promise<VoiceSessionEvent> {
  return nativeInvoke('cancel_voice_input', undefined, {
    status: 'Idle',
    transcript_partial: null,
    transcript_final: null,
    error_code: null,
    message: null
  });
}

export function getVoiceStatus(): Promise<VoiceSessionEvent> {
  return nativeInvoke('get_voice_status', undefined, {
    status: 'Idle',
    transcript_partial: null,
    transcript_final: null,
    error_code: null,
    message: null
  });
}

export function showMascotWindow(): Promise<void> {
  return nativeInvoke('show_mascot_window', undefined, undefined);
}

export function hideMascotWindow(): Promise<void> {
  return nativeInvoke('hide_mascot_window', undefined, undefined);
}

export function setOutputMode(output_style: OutputStyle): Promise<AppSettings> {
  return nativeInvoke('set_output_mode', { outputStyle: output_style, output_style }, {
    ...defaultSettings,
    output_style
  });
}

export function openSettingsWindow(): Promise<void> {
  return nativeInvoke('open_settings_window', undefined, undefined);
}

export function openAboutWindow(): Promise<void> {
  return nativeInvoke('open_about_window', undefined, undefined);
}

export function quitApp(): Promise<void> {
  return nativeInvoke('quit_app', undefined, undefined);
}
