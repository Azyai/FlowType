import type { AppSettings, UpdateCheckResult } from '../../types';
import { updateMessage } from '../../lib/formatters/updateMessage';
import { useI18n } from '../../lib/i18n/I18nContext';

export function AdvancedPage({
  settings,
  setSettings,
  updateResult,
  onCheckUpdate,
  onAutostart
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
  updateResult: UpdateCheckResult | null;
  onCheckUpdate: () => void;
  onAutostart: (enabled: boolean) => void;
}) {
  const { t } = useI18n();

  return (
    <section className="panel">
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.auto_start}
          onChange={(event) => onAutostart(event.target.checked)}
          aria-label={t('advanced.launchAtStartup')}
        />
        <span>{t('advanced.launchAtStartup')}</span>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.save_history}
          onChange={(event) => setSettings({ ...settings, save_history: event.target.checked })}
        />
        <span>{t('advanced.keepHistory')}</span>
      </label>
      <label className="field">
        <span>{t('advanced.displayLanguage')}</span>
        <select
          value={settings.locale_preference}
          onChange={(event) =>
            setSettings({
              ...settings,
              locale_preference: event.target.value as AppSettings['locale_preference']
            })
          }
          aria-label={t('advanced.displayLanguage')}
        >
          <option value="auto">{t('advanced.languageAuto')}</option>
          <option value="zh-CN">{t('advanced.languageZhCN')}</option>
          <option value="en-US">{t('advanced.languageEnUS')}</option>
        </select>
      </label>
      <label className="field">
        <span>{t('advanced.floatingPosition')}</span>
        <select
          value={settings.floating_window_position}
          onChange={(event) =>
            setSettings({
              ...settings,
              floating_window_position: event.target.value as AppSettings['floating_window_position']
            })
          }
        >
          <option value="bottom_right">{t('advanced.bottomRight')}</option>
          <option value="cursor_nearby">{t('advanced.nearCursor')}</option>
        </select>
      </label>
      <label className="field">
        <span>{t('advanced.updateChannel')}</span>
        <select
          value={settings.update_channel}
          onChange={(event) =>
            setSettings({ ...settings, update_channel: event.target.value as AppSettings['update_channel'] })
          }
        >
          <option value="stable">{t('advanced.stable')}</option>
          <option value="beta">{t('advanced.beta')}</option>
          <option value="dev">{t('advanced.dev')}</option>
        </select>
      </label>
      <label className="field">
        <span>{t('advanced.mockManifestUrl')}</span>
        <input
          value={settings.update_manifest_url}
          onChange={(event) => setSettings({ ...settings, update_manifest_url: event.target.value })}
        />
      </label>
      <button type="button" className="secondary-button" onClick={onCheckUpdate}>
        {t('advanced.checkUpdate')}
      </button>
      {updateResult && <p className="inline-result">{updateMessage(updateResult, t)}</p>}
    </section>
  );
}
