import { invoke } from '@tauri-apps/api/core';

import {
  defaultSettings,
  fallbackAsrServiceCheck,
  fallbackAsrServiceConfig,
  fallbackDatabaseHealth,
  fallbackStatus
} from './defaults';
import type {
  AppSettings,
  AppStatus,
  AsrServiceCheckResult,
  AsrServiceConfig,
  ClearHistoryResult,
  DatabaseHealth,
  UpdateCheckResult
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
  | 'asr_service_mode'
  | 'iflytek_app_id'
  | 'iflytek_api_key'
  | 'iflytek_api_secret'
  | 'iflytek_language'
  | 'iflytek_mixed_language'
  | 'iflytek_timeout_ms'
  | 'iflytek_retry_count'
>): Promise<AppSettings> {
  return nativeInvoke('save_asr_service_config', { config }, { ...defaultSettings, ...config });
}

export function checkAsrService(): Promise<AsrServiceCheckResult> {
  return nativeInvoke('check_asr_service', undefined, fallbackAsrServiceCheck);
}

export function clearHistory(): Promise<ClearHistoryResult> {
  return nativeInvoke('clear_history', undefined, { deleted_count: 0 });
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
