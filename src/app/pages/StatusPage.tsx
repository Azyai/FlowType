import { Clipboard, Cpu, Database, Keyboard, Mic, Power } from 'lucide-react';

import type { AppSettings, AppStatus, DatabaseHealth } from '../../types';
import { InfoCard } from '../components/InfoCard';
import { useI18n } from '../../lib/i18n/I18nContext';

export function StatusPage({
  status,
  databaseHealth,
  settings
}: {
  status: AppStatus;
  databaseHealth: DatabaseHealth;
  settings: AppSettings;
}) {
  const { t } = useI18n();
  const modeLabel = t('status.mode', { mode: '' }).replace(/[:：]\s*$/, '');

  return (
    <div className="grid">
      <InfoCard
        icon={Power}
        label={t('status.background')}
        value={status.paused ? t('status.paused') : t('status.ready')}
      />
      <InfoCard icon={Cpu} label={modeLabel} value={t('status.mode', { mode: status.current_mode })} />
      <InfoCard
        icon={Database}
        label={t('status.database')}
        value={databaseHealth.ok ? t('status.sqliteHealthy') : t('status.sqliteUnavailable')}
        detail={t('status.migrationsApplied', { count: databaseHealth.applied_migrations })}
      />
      <InfoCard icon={Mic} label={t('status.voiceInput')} value={t('status.notImplementedPhase0')} />
      <InfoCard icon={Keyboard} label={t('status.hotkey')} value={settings.hotkey} />
      <InfoCard icon={Clipboard} label={t('status.clipboard')} value={settings.clipboard_restore} />
    </div>
  );
}
