import type { ReactNode } from 'react';

import { useI18n } from '../../lib/i18n/I18nContext';

export function PageHeader({
  title,
  version,
  meta
}: {
  title: string;
  version: string;
  meta?: ReactNode;
}) {
  const { t } = useI18n();

  return (
    <header className="content-header">
      <div>
        <div className="content-title-row">
          <h1>{title}</h1>
          {meta}
        </div>
      </div>
      <div className="version-pill">{t('label.version', { version })}</div>
    </header>
  );
}
