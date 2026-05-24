import type { AppSettings, HistoryRetentionDays, UpdateCheckResult } from '../../types';
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
      <label className="field">
        <span>{t('output.style')}</span>
        <select
          value={settings.output_style}
          aria-label={t('output.style')}
          onChange={(event) =>
            setSettings({ ...settings, output_style: event.target.value as AppSettings['output_style'] })
          }
        >
          <option value="raw">{t('output.raw')}</option>
          <option value="clean">{t('output.clean')}</option>
          <option value="formal">{t('output.formal')}</option>
        </select>
      </label>

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
        <span>{t('advanced.historyRetention')}</span>
        <select
          value={settings.history_retention_days}
          aria-label={t('advanced.historyRetention')}
          onChange={(event) =>
            setSettings({
              ...settings,
              history_retention_days: Number(event.target.value) as HistoryRetentionDays
            })
          }
        >
          {[7, 14, 30].map((days) => (
            <option key={days} value={days}>
              {t('advanced.retentionDays', { days })}
            </option>
          ))}
        </select>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.show_floating_window}
          onChange={(event) => setSettings({ ...settings, show_floating_window: event.target.checked })}
        />
        <span>{t('advanced.showFloatingWindow')}</span>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.floating_window_always_on_top}
          onChange={(event) =>
            setSettings({ ...settings, floating_window_always_on_top: event.target.checked })
          }
        />
        <span>{t('advanced.floatingAlwaysOnTop')}</span>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.floating_window_animation_enabled}
          onChange={(event) =>
            setSettings({ ...settings, floating_window_animation_enabled: event.target.checked })
          }
        />
        <span>{t('advanced.floatingAnimation')}</span>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.vad_enabled}
          onChange={(event) => setSettings({ ...settings, vad_enabled: event.target.checked })}
        />
        <span>{t('advanced.vad')}</span>
      </label>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.hotwords_enabled}
          onChange={(event) => setSettings({ ...settings, hotwords_enabled: event.target.checked })}
        />
        <span>{t('advanced.hotwords')}</span>
      </label>
      <label className="field">
        <span>{t('advanced.minRecording')}</span>
        <input
          type="number"
          min={0}
          step={100}
          value={settings.min_recording_ms}
          aria-label={t('advanced.minRecording')}
          onChange={(event) =>
            setSettings({ ...settings, min_recording_ms: Number(event.target.value) || 0 })
          }
        />
      </label>
      <label className="field">
        <span>{t('advanced.maxRecording')}</span>
        <input
          type="number"
          min={1000}
          step={1000}
          value={settings.max_recording_ms}
          aria-label={t('advanced.maxRecording')}
          onChange={(event) =>
            setSettings({ ...settings, max_recording_ms: Number(event.target.value) || 60000 })
          }
        />
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
