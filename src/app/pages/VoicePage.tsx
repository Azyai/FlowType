import type { AppSettings } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

export function VoicePage({
  settings,
  setSettings
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
}) {
  const { t } = useI18n();

  return (
    <section className="panel">
      <label className="field">
        <span>{t('voice.recognitionMode')}</span>
        <select
          value={settings.asr_mode}
          onChange={(event) =>
            setSettings({ ...settings, asr_mode: event.target.value as AppSettings['asr_mode'] })
          }
        >
          <option value="local_first">{t('voice.localFirst')}</option>
          <option value="cloud_first">{t('voice.cloudFirst')}</option>
          <option value="cloud_only">{t('voice.cloudOnly')}</option>
        </select>
      </label>
      <label className="field">
        <span>{t('voice.defaultModel')}</span>
        <input
          value={settings.default_model}
          onChange={(event) => setSettings({ ...settings, default_model: event.target.value })}
        />
      </label>
      <p className="muted">{t('voice.futureModelWork')}</p>
    </section>
  );
}
