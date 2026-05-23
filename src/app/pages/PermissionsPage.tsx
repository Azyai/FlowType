import { useI18n } from '../../lib/i18n/I18nContext';

export function PermissionsPage() {
  const { t } = useI18n();

  return (
    <section className="panel">
      <div className="check-list">
        <span>{t('permissions.microphone')}</span>
        <span>{t('permissions.inputMonitoring')}</span>
        <span>{t('permissions.clipboard')}</span>
        <span>{t('permissions.autostart')}</span>
      </div>
    </section>
  );
}
