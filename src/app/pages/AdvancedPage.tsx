import type { AppSettings, HistoryRetentionDays, UpdateCheckResult } from '../../types';
import { updateMessage } from '../../lib/formatters/updateMessage';
import { useI18n } from '../../lib/i18n/I18nContext';
import type { TranslationKey } from '../../lib/i18n/types';

const formalSceneOptions: Array<{
  value: AppSettings['formal_scene'];
  labelKey: TranslationKey;
  descriptionKey: TranslationKey;
}> = [
  {
    value: 'general',
    labelKey: 'advanced.formalSkillGeneral',
    descriptionKey: 'advanced.formalSkillGeneralDesc'
  },
  {
    value: 'email',
    labelKey: 'advanced.formalSkillEmail',
    descriptionKey: 'advanced.formalSkillEmailDesc'
  },
  {
    value: 'greeting',
    labelKey: 'advanced.formalSkillGreeting',
    descriptionKey: 'advanced.formalSkillGreetingDesc'
  },
  {
    value: 'professional_reply',
    labelKey: 'advanced.formalSkillReply',
    descriptionKey: 'advanced.formalSkillReplyDesc'
  }
];

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
  const formalModeEnabled = settings.output_style === 'formal';
  const recordingRangeInvalid = settings.max_recording_ms < settings.min_recording_ms;

  const updateSettings = (patch: Partial<AppSettings>) => {
    setSettings({ ...settings, ...patch });
  };

  return (
    <section className="panel">
      <section className="settings-section">
        <div className="settings-section-header">
          <div>
            <h3>{t('advanced.sectionOutput')}</h3>
            <p>{t('advanced.sectionOutputDesc')}</p>
          </div>
        </div>

        <label className="field">
          <span>{t('output.style')}</span>
          <select
            value={settings.output_style}
            aria-label={t('output.style')}
            onChange={(event) => updateSettings({ output_style: event.target.value as AppSettings['output_style'] })}
          >
            <option value="raw">{t('output.raw')}</option>
            <option value="clean">{t('output.clean')}</option>
            <option value="formal">{t('output.formal')}</option>
          </select>
        </label>

        {formalModeEnabled && (
          <fieldset className="skill-fieldset">
            <legend>{t('advanced.formalSkill')}</legend>
            <div className="skill-fieldset-header">
              <span className="skill-state-badge is-active">{t('advanced.skillEnabled')}</span>
            </div>

            <div className="skill-grid" role="radiogroup" aria-label={t('advanced.formalSkill')}>
              {formalSceneOptions.map((option) => {
                const checked = settings.formal_scene === option.value;
                return (
                  <label key={option.value} className={`skill-card${checked ? ' is-selected' : ''}`}>
                    <input
                      type="radio"
                      name="formal-scene"
                      value={option.value}
                      checked={checked}
                      aria-label={t(option.labelKey)}
                      onChange={(event) =>
                        updateSettings({ formal_scene: event.target.value as AppSettings['formal_scene'] })
                      }
                    />
                    <div className="skill-card-copy">
                      <strong>{t(option.labelKey)}</strong>
                      <span>{t(option.descriptionKey)}</span>
                    </div>
                  </label>
                );
              })}
            </div>
          </fieldset>
        )}
      </section>

      <section className="settings-section">
        <div className="settings-section-header">
          <div>
            <h3>{t('advanced.sectionExperience')}</h3>
            <p>{t('advanced.sectionExperienceDesc')}</p>
          </div>
        </div>

        <label className="field">
          <span>{t('advanced.displayLanguage')}</span>
          <select
            value={settings.locale_preference}
            onChange={(event) =>
              updateSettings({ locale_preference: event.target.value as AppSettings['locale_preference'] })
            }
            aria-label={t('advanced.displayLanguage')}
          >
            <option value="auto">{t('advanced.languageAuto')}</option>
            <option value="zh-CN">{t('advanced.languageZhCN')}</option>
            <option value="en-US">{t('advanced.languageEnUS')}</option>
          </select>
        </label>

        <label className="switch-row">
          <input
            type="checkbox"
            checked={settings.show_floating_window}
            onChange={(event) => updateSettings({ show_floating_window: event.target.checked })}
            aria-label={t('advanced.showFloatingWindow')}
          />
          <span>{t('advanced.showFloatingWindow')}</span>
        </label>

        {settings.show_floating_window && (
          <div className="settings-subgrid">
            <label className="field">
              <span>{t('advanced.floatingPosition')}</span>
              <select
                value={settings.floating_window_position}
                onChange={(event) =>
                  updateSettings({
                    floating_window_position: event.target.value as AppSettings['floating_window_position']
                  })
                }
                aria-label={t('advanced.floatingPosition')}
              >
                <option value="bottom_right">{t('advanced.bottomRight')}</option>
                <option value="cursor_nearby">{t('advanced.nearCursor')}</option>
              </select>
            </label>
          </div>
        )}
      </section>

      <section className="settings-section">
        <div className="settings-section-header">
          <div>
            <h3>{t('advanced.sectionRecognition')}</h3>
            <p>{t('advanced.sectionRecognitionDesc')}</p>
          </div>
        </div>

        <div className="settings-subgrid">
          <label className="field">
            <span>{t('advanced.minRecording')}</span>
            <input
              type="number"
              min={0}
              step={100}
              value={settings.min_recording_ms}
              aria-label={t('advanced.minRecording')}
              onChange={(event) => updateSettings({ min_recording_ms: Number(event.target.value) || 0 })}
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
              onChange={(event) => updateSettings({ max_recording_ms: Number(event.target.value) || 60000 })}
            />
          </label>
        </div>

        <p className={`helper-text${recordingRangeInvalid ? ' danger' : ''}`}>
          {recordingRangeInvalid ? t('advanced.recordingRangeInvalid') : t('advanced.recordingRangeHint')}
        </p>

        <div className="switch-stack">
          <label className="switch-row">
            <input
              type="checkbox"
              checked={settings.hotwords_enabled}
              onChange={(event) => updateSettings({ hotwords_enabled: event.target.checked })}
              aria-label={t('advanced.hotwords')}
            />
            <span>{t('advanced.hotwords')}</span>
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-header">
          <div>
            <h3>{t('advanced.sectionData')}</h3>
            <p>{t('advanced.sectionDataDesc')}</p>
          </div>
        </div>

        <label className="switch-row">
          <input
            type="checkbox"
            checked={settings.save_history}
            onChange={(event) => updateSettings({ save_history: event.target.checked })}
            aria-label={t('advanced.keepHistory')}
          />
          <span>{t('advanced.keepHistory')}</span>
        </label>

        <label className="field">
          <span>{t('advanced.historyRetention')}</span>
          <select
            value={settings.history_retention_days}
            aria-label={t('advanced.historyRetention')}
            onChange={(event) =>
              updateSettings({ history_retention_days: Number(event.target.value) as HistoryRetentionDays })
            }
          >
            {[7, 14, 30].map((days) => (
              <option key={days} value={days}>
                {t('advanced.retentionDays', { days })}
              </option>
            ))}
          </select>
        </label>
      </section>

      <section className="settings-section">
        <div className="settings-section-header">
          <div>
            <h3>{t('advanced.sectionSystem')}</h3>
            <p>{t('advanced.sectionSystemDesc')}</p>
          </div>
        </div>

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
            checked={settings.auto_check_update}
            onChange={(event) => updateSettings({ auto_check_update: event.target.checked })}
            aria-label={t('advanced.autoCheckUpdate')}
          />
          <span>{t('advanced.autoCheckUpdate')}</span>
        </label>

        <div className="settings-subgrid">
          <label className="field">
            <span>{t('advanced.updateChannel')}</span>
            <select
              value={settings.update_channel}
              onChange={(event) =>
                updateSettings({ update_channel: event.target.value as AppSettings['update_channel'] })
              }
              aria-label={t('advanced.updateChannel')}
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
              aria-label={t('advanced.mockManifestUrl')}
              onChange={(event) => updateSettings({ update_manifest_url: event.target.value })}
            />
          </label>
        </div>

        <div className="inline-actions">
          <button type="button" className="secondary-button" onClick={onCheckUpdate}>
            {t('advanced.checkUpdate')}
          </button>
          {updateResult && <p className="inline-result">{updateMessage(updateResult, t)}</p>}
        </div>
      </section>
    </section>
  );
}
