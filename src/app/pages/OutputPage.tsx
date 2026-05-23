import type { AppSettings } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

export function OutputPage({
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
        <span>{t('output.style')}</span>
        <select
          value={settings.output_style}
          onChange={(event) =>
            setSettings({ ...settings, output_style: event.target.value as AppSettings['output_style'] })
          }
        >
          <option value="raw">{t('output.raw')}</option>
          <option value="clean">{t('output.clean')}</option>
          <option value="formal">{t('output.formal')}</option>
        </select>
      </label>
      <label className="field">
        <span>{t('output.clipboardRestore')}</span>
        <select
          value={settings.clipboard_restore}
          onChange={(event) =>
            setSettings({
              ...settings,
              clipboard_restore: event.target.value as AppSettings['clipboard_restore']
            })
          }
        >
          <option value="always">{t('output.restoreAlways')}</option>
          <option value="delayed">{t('output.restoreDelayed')}</option>
          <option value="never">{t('output.restoreNever')}</option>
          <option value="text_only">{t('output.restoreTextOnly')}</option>
        </select>
      </label>
    </section>
  );
}
