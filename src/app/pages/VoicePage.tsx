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
        <span>
          {t('voice.serviceMode')}: {settings.asr_service_mode === 'built_in' ? t('voice.builtIn') : t('voice.customDev')}
        </span>
      </div>

      <p className="muted">{t('voice.privacy')}</p>
      <p className="muted">{t('voice.devSecretNote')}</p>

      <label className="field">
        <span>{t('voice.serviceMode')}</span>
        <select
          value={settings.asr_service_mode}
          aria-label={t('voice.serviceMode')}
          onChange={(event) =>
            setSettings({
              ...settings,
              asr_service_mode: event.target.value as AppSettings['asr_service_mode']
            })
          }
        >
          <option value="built_in">{t('voice.builtIn')}</option>
          <option value="custom_dev">{t('voice.customDev')}</option>
        </select>
      </label>

      <div className="grid two-column-grid">
        <label className="field">
          <span>{t('voice.appId')}</span>
          <input
            value={settings.iflytek_app_id}
            aria-label={t('voice.appId')}
            onChange={(event) => setSettings({ ...settings, iflytek_app_id: event.target.value })}
          />
        </label>
        <label className="field">
          <span>{t('voice.apiKey')}</span>
          <input
            type="password"
            value={settings.iflytek_api_key}
            aria-label={t('voice.apiKey')}
            onChange={(event) => setSettings({ ...settings, iflytek_api_key: event.target.value })}
          />
        </label>
        <label className="field">
          <span>{t('voice.apiSecret')}</span>
          <input
            type="password"
            value={settings.iflytek_api_secret}
            aria-label={t('voice.apiSecret')}
            onChange={(event) => setSettings({ ...settings, iflytek_api_secret: event.target.value })}
          />
        </label>
        <label className="field">
          <span>{t('voice.language')}</span>
          <select
            value={settings.iflytek_language}
            onChange={(event) =>
              setSettings({ ...settings, iflytek_language: event.target.value as AppSettings['iflytek_language'] })
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
            value={settings.iflytek_timeout_ms}
            onChange={(event) =>
              setSettings({ ...settings, iflytek_timeout_ms: Number(event.target.value) || 10000 })
            }
          />
        </label>
        <label className="field">
          <span>{t('voice.retryCount')}</span>
          <input
            type="number"
            min={0}
            max={5}
            value={settings.iflytek_retry_count}
            onChange={(event) =>
              setSettings({ ...settings, iflytek_retry_count: Number(event.target.value) || 0 })
            }
          />
        </label>
      </div>

      <label className="switch-row">
        <input
          type="checkbox"
          checked={settings.iflytek_mixed_language}
          onChange={(event) => setSettings({ ...settings, iflytek_mixed_language: event.target.checked })}
        />
        <span>{t('voice.mixedLanguage')}</span>
      </label>

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
