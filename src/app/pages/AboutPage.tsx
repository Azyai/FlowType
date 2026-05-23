import { useI18n } from '../../lib/i18n/I18nContext';

export function AboutPage() {
  const { t } = useI18n();

  return (
    <section className="panel">
      <p className="muted">{t('about.description')}</p>
    </section>
  );
}
