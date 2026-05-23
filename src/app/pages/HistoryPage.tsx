import type { AppSettings } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

export function HistoryPage({ settings }: { settings: AppSettings }) {
  const { t } = useI18n();
  const status = settings.save_history ? t('history.enabled') : t('history.disabled');

  return (
    <section className="panel">
      <p className="muted">
        {t('history.storage', { status })} {t('history.retention', { days: settings.history_retention_days })}
      </p>
      <p className="muted">{t('history.textOnly')}</p>
    </section>
  );
}
