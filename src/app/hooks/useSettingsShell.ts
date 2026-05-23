import { FormEvent, useEffect, useMemo, useState } from 'react';

import { PageId, pageTitleKey } from '../config/navigation';
import {
  checkUpdate,
  getAppStatus,
  getDatabaseHealth,
  getSettings,
  resetSettings,
  saveSettings,
  setAutostart
} from '../../lib/tauri';
import { readableError } from '../../lib/formatters/errors';
import { resolveLocale } from '../../lib/i18n/locale';
import { translate } from '../../lib/i18n/I18nContext';
import type { AppSettings, AppStatus, DatabaseHealth, UpdateCheckResult } from '../../types';

export function useSettingsShell() {
  const [activePage, setActivePage] = useState<PageId>('status');
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [databaseHealth, setDatabaseHealth] = useState<DatabaseHealth | null>(null);
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;

    async function load() {
      try {
        const [loadedSettings, loadedStatus, loadedDatabaseHealth] = await Promise.all([
          getSettings(),
          getAppStatus(),
          getDatabaseHealth()
        ]);

        if (!alive) return;
        setSettings(loadedSettings);
        setStatus(loadedStatus);
        setDatabaseHealth(loadedDatabaseHealth);
      } catch (loadError) {
        if (!alive) return;
        setError(readableError(loadError));
      }
    }

    load();
    return () => {
      alive = false;
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

  async function handleSave(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!settings) return;

    try {
      const saved = await saveSettings(settings);
      setSettings(saved);
      setNotice(t('notice.settingsSaved'));
      setError(null);
    } catch (saveError) {
      setError(readableError(saveError));
    }
  }

  async function handleReset() {
    try {
      const next = await resetSettings();
      setSettings(next);
      setNotice(t('notice.settingsReset'));
      setError(null);
    } catch (resetError) {
      setError(readableError(resetError));
    }
  }

  async function handleAutostart(enabled: boolean) {
    if (!settings) return;
    setSettings({ ...settings, auto_start: enabled });

    try {
      const saved = await setAutostart(enabled);
      setSettings(saved);
      setNotice(enabled ? t('notice.startupEnabled') : t('notice.startupDisabled'));
      setError(null);
    } catch (autostartError) {
      setSettings({ ...settings, auto_start: !enabled });
      setError(readableError(autostartError));
    }
  }

  async function handleCheckUpdate() {
    try {
      const result = await checkUpdate();
      setUpdateResult(result);
      setError(null);
    } catch (updateError) {
      setError(readableError(updateError));
    }
  }

  return {
    activePage,
    activeTitle,
    databaseHealth,
    error,
    handleAutostart,
    handleCheckUpdate,
    handleReset,
    handleSave,
    locale,
    localePreference: settings?.locale_preference ?? 'auto',
    notice,
    setActivePage,
    setSettings,
    settings,
    status,
    t,
    updateResult
  };
}
