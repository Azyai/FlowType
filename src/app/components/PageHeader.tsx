import { useI18n } from '../../lib/i18n/I18nContext';

export function PageHeader({ title, version }: { title: string; version: string }) {
  const { t } = useI18n();

  return (
    <header className="content-header">
      <div>
        <p className="eyebrow">{t('phase')}</p>
        <h1>{title}</h1>
      </div>
      <div className="version-pill">{t('label.version', { version })}</div>
    </header>
  );
}
