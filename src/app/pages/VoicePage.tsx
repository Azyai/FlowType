import type { AppSettings, AsrServiceCheckResult } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

export function VoicePage({
  settings,
  setSettings,
  asrResult,
  onCheckAsrService
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
  asrResult: AsrServiceCheckResult | null;
  onCheckAsrService: () => void;
}) {
  const { t } = useI18n();

  return (
    <section className="panel asr-panel">
      <div className="service-summary">
        <strong>{t('voice.provider')}</strong>
        <span>{t('voice.realtimeOnly')}</span>
      </div>

      <p className="muted">{t('voice.privacy')}</p>
      <p className="muted">{t('voice.realtimeNote')}</p>

      <div className="grid two-column-grid">
        <label className="field">
          <span>{t('voice.appId')}</span>
          <input
            value={settings.rtasr_app_id}
            aria-label={t('voice.appId')}
            onChange={(event) => setSettings({ ...settings, rtasr_app_id: event.target.value })}
          />
        </label>
        <label className="field">
          <span>{t('voice.apiKey')}</span>
          <input
            type="password"
            value={settings.rtasr_api_key}
            aria-label={t('voice.apiKey')}
            onChange={(event) => setSettings({ ...settings, rtasr_api_key: event.target.value })}
          />
        </label>
        <label className="field">
          <span>{t('voice.language')}</span>
          <select
            value={settings.rtasr_language}
            onChange={(event) =>
              setSettings({ ...settings, rtasr_language: event.target.value as AppSettings['rtasr_language'] })
            }
          >
            <option value="zh_cn">{t('voice.languageZhCn')}</option>
            <option value="en_us">{t('voice.languageEnUs')}</option>
            <option value="zh_en">{t('voice.languageZhEn')}</option>
          </select>
        </label>
        <label className="field">
          <span>{t('voice.timeout')}</span>
          <input
            type="number"
            min={1000}
            step={500}
            value={settings.rtasr_timeout_ms}
            onChange={(event) =>
              setSettings({ ...settings, rtasr_timeout_ms: Number(event.target.value) || 10000 })
            }
          />
        </label>
      </div>

      <button type="button" className="secondary-button" onClick={onCheckAsrService}>
        {t('voice.checkService')}
      </button>

      {asrResult && (
        <div className="inline-result">
          <p>{asrResult.message}</p>
          {asrResult.missing_fields.length > 0 && (
            <p>{t('voice.missingFields', { fields: asrResult.missing_fields.join(', ') })}</p>
          )}
        </div>
      )}
    </section>
  );
}
