import { Activity } from 'lucide-react';
import { useCallback, useRef, useState } from 'react';

import { ConfirmDialog, type ConfirmDialogState } from './app/components/ConfirmDialog';
import { DesktopTitlebar } from './app/components/DesktopTitlebar';
import { FormActions } from './app/components/FormActions';
import { LiveCaptionPage } from './app/pages/LiveCaptionPage';
import { MascotPage } from './app/pages/MascotPage';
import { PageHeader } from './app/components/PageHeader';
import { Sidebar } from './app/components/Sidebar';
import { Toast } from './app/components/Toast';
import { useSettingsShell } from './app/hooks/useSettingsShell';
import { AboutPage } from './app/pages/AboutPage';
import { AdvancedPage } from './app/pages/AdvancedPage';
import { HistoryPage } from './app/pages/HistoryPage';
import { HotkeyPage } from './app/pages/HotkeyPage';
import { I18nContext, translate } from './lib/i18n/I18nContext';
import { resolveLocale } from './lib/i18n/locale';

export default function App() {
  const windowKind = new URLSearchParams(window.location.search).get('window');
  const isMascot = windowKind === 'mascot';
  const isLiveCaption = windowKind === 'live-caption';
  const [historySummary, setHistorySummary] = useState<{
    total: number;
    enabled: boolean;
    retentionDays: number;
  } | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const confirmResolverRef = useRef<((confirmed: boolean) => void) | null>(null);

  if (isMascot) {
    return <MascotPage />;
  }

  if (isLiveCaption) {
    return <LiveCaptionPage />;
  }

  const shell = useSettingsShell();
  const { activePage, activeTitle, settings, status } = shell;

  const handleRequestConfirm = useCallback(
    (options: Omit<ConfirmDialogState, 'confirmLabel' | 'cancelLabel'>) =>
      new Promise<boolean>((resolve) => {
        confirmResolverRef.current = resolve;
        setConfirmDialog({
          ...options,
          confirmLabel: shell.t('actions.confirm'),
          cancelLabel: shell.t('actions.cancel')
        });
      }),
    [shell]
  );

  const closeConfirmDialog = useCallback((confirmed: boolean) => {
    confirmResolverRef.current?.(confirmed);
    confirmResolverRef.current = null;
    setConfirmDialog(null);
  }, []);

  const handleClearHistory = useCallback(async () => {
    const confirmed = await handleRequestConfirm({
      title: shell.t('history.confirmClearTitle'),
      message: shell.t('history.confirmClear'),
      tone: 'danger'
    });
    if (!confirmed) {
      return null;
    }

    return shell.handleClearHistory();
  }, [handleRequestConfirm, shell]);

  if (!settings || !status) {
    const loadingLocale = resolveLocale('auto');

    return (
      <main className="loading-shell">
        <Activity aria-hidden="true" />
        <span>{translate(loadingLocale, 'loading')}</span>
      </main>
    );
  }

  return (
    <I18nContext.Provider
      value={{
        locale: shell.locale,
        localePreference: shell.localePreference,
        t: shell.t
      }}
    >
      <main className="desktop-frame">
        <Sidebar activePage={activePage} status={status} onSelectPage={shell.setActivePage} />
        <Toast toast={shell.toast} />
        <ConfirmDialog
          dialog={confirmDialog}
          onCancel={() => closeConfirmDialog(false)}
          onConfirm={() => closeConfirmDialog(true)}
        />

        <section className="workspace">
          <DesktopTitlebar />

          <section className="content">
            <PageHeader
              title={activeTitle}
              version={status.app_version}
              meta={
                activePage === 'history' && historySummary ? (
                  <div className="content-title-meta muted">
                    <span>{shell.t('history.total', { count: historySummary.total })}</span>
                    <span>
                      {shell.t('history.storageShort', {
                        status: historySummary.enabled
                          ? shell.t('history.enabled')
                          : shell.t('history.disabled')
                      })}
                    </span>
                    <span>{shell.t('history.retentionShort', { days: historySummary.retentionDays })}</span>
                  </div>
                ) : null
              }
            />

            <form onSubmit={shell.handleSave}>
              {activePage === 'hotkey' && <HotkeyPage settings={settings} setSettings={shell.setSettings} />}
              {activePage === 'advanced' && (
                <AdvancedPage
                  settings={settings}
                  setSettings={shell.setSettings}
                  updateResult={shell.updateResult}
                  onCheckUpdate={shell.handleCheckUpdate}
                  onAutostart={shell.handleAutostart}
                />
              )}
              {activePage === 'history' && (
                <HistoryPage
                  settings={settings}
                  onClearHistory={handleClearHistory}
                  onToast={shell.showToast}
                  onSummaryChange={setHistorySummary}
                  onRequestConfirm={handleRequestConfirm}
                />
              )}
              {activePage === 'about' && <AboutPage />}

              {(activePage === 'hotkey' || activePage === 'advanced') && (
                <FormActions onReset={shell.handleReset} />
              )}
            </form>
          </section>
        </section>
      </main>
    </I18nContext.Provider>
  );
}
