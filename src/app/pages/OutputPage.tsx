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
        <span>{t('output.deliveryMode')}</span>
        <input value={t('output.deliveryCombined')} readOnly />
      </label>
    </section>
  );
}
