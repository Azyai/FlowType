import type { AppStatus } from '../../types';
import { PageId, pages } from '../config/navigation';
import { useI18n } from '../../lib/i18n/I18nContext';

export function Sidebar({
  activePage,
  status,
  onSelectPage
}: {
  activePage: PageId;
  status: AppStatus;
  onSelectPage: (page: PageId) => void;
}) {
  const { t } = useI18n();

  return (
    <aside className="sidebar" aria-label={t('app.settingsSections')}>
      <div className="brand">
        <div className="brand-mark">F</div>
        <div>
          <strong>FlowType</strong>
          <span>{t('app.subtitle')}</span>
        </div>
      </div>

      <nav>
        {pages.map((page) => {
          const Icon = page.icon;
          return (
            <button
              key={page.id}
              type="button"
              className={activePage === page.id ? 'nav-button active' : 'nav-button'}
              onClick={() => onSelectPage(page.id)}
            >
              <Icon aria-hidden="true" />
              <span>{t(page.labelKey)}</span>
            </button>
          );
        })}
      </nav>

      <div className="sidebar-footer">
        <span className={status.paused ? 'status-dot paused' : 'status-dot'} />
        {status.paused ? t('status.paused') : t('status.ready')}
      </div>
    </aside>
  );
}
