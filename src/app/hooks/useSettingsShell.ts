import { FormEvent, useEffect, useMemo, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

import { PageId, pageTitleKey } from '../config/navigation';
import {
  checkUpdate,
  clearHistory,
  getAppStatus,
  getSettings,
  resetSettings,
  saveSettings,
  setAutostart
} from '../../lib/tauri';
import { readableError } from '../../lib/formatters/errors';
import { resolveLocale } from '../../lib/i18n/locale';
import { translate } from '../../lib/i18n/I18nContext';
import type { AppSettings, AppStatus, ClearHistoryResult, UpdateCheckResult } from '../../types';

export interface ToastState {
  kind: 'success' | 'error';
  message: string;
}

export function useSettingsShell() {
  const [activePage, setActivePage] = useState<PageId>('hotkey');
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);

  useEffect(() => {
    let alive = true;

    async function load() {
      try {
        const [loadedSettings, loadedStatus] = await Promise.all([
          getSettings(),
          getAppStatus()
        ]);

        if (!alive) return;
        setSettings(loadedSettings);
        setStatus(loadedStatus);
      } catch (loadError) {
        if (!alive) return;
        showToast('error', readableError(loadError));
      }
    }

    load();

    const unlistenSettings =
      '__TAURI_INTERNALS__' in window
        ? listen<AppSettings>('settings_updated', (event) => {
            if (alive) {
              setSettings(event.payload);
            }
          })
        : Promise.resolve(() => {});

    return () => {
      alive = false;
      unlistenSettings.then((fn) => fn());
    };
  }, []);

  const locale = useMemo(
    () => resolveLocale(settings?.locale_preference ?? 'auto'),
    [settings?.locale_preference]
  );
  const t = useMemo(
    () => (key: Parameters<typeof translate>[1], params?: Record<string, string | number>) =>
      translate(locale, key, params),
    [locale]
  );
  const activeTitle = useMemo(() => t(pageTitleKey(activePage)), [activePage, t]);

  useEffect(() => {
    if (!toast) return;

    const timeout = window.setTimeout(() => {
      setToast(null);
    }, 3000);

    return () => window.clearTimeout(timeout);
  }, [toast]);

  function showToast(kind: ToastState['kind'], message: string) {
    setToast({ kind, message });
  }

  async function handleSave(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!settings) return;

    try {
      const saved = await saveSettings(settings);
      setSettings(saved);
      showToast('success', t('notice.settingsSaved'));
    } catch (saveError) {
      showToast('error', readableError(saveError));
    }
  }

  async function handleReset() {
    try {
      const next = await resetSettings();
      setSettings(next);
      showToast('success', t('notice.settingsReset'));
    } catch (resetError) {
      showToast('error', readableError(resetError));
    }
  }

  async function handleAutostart(enabled: boolean) {
    if (!settings) return;
    setSettings({ ...settings, auto_start: enabled });

    try {
      const saved = await setAutostart(enabled);
      setSettings(saved);
      showToast('success', enabled ? t('notice.startupEnabled') : t('notice.startupDisabled'));
    } catch (autostartError) {
      setSettings({ ...settings, auto_start: !enabled });
      showToast('error', readableError(autostartError));
    }
  }

  async function handleCheckUpdate() {
    try {
      const result = await checkUpdate();
      setUpdateResult(result);
    } catch (updateError) {
      showToast('error', readableError(updateError));
    }
  }

  async function handleClearHistory(): Promise<ClearHistoryResult | null> {
    try {
      const result = await clearHistory();
      showToast('success', t('notice.historyCleared', { count: result.deleted_count }));
      return result;
    } catch (historyError) {
      showToast('error', readableError(historyError));
      throw historyError;
    }
  }

  return {
    activePage,
    activeTitle,
    handleAutostart,
    handleCheckUpdate,
    handleClearHistory,
    handleReset,
    handleSave,
    locale,
    localePreference: settings?.locale_preference ?? 'auto',
    setActivePage,
    setSettings,
    settings,
    showToast,
    status,
    t,
    toast,
    updateResult
  };
}
