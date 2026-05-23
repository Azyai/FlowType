import { Activity } from 'lucide-react';

import { DesktopTitlebar } from './app/components/DesktopTitlebar';
import { FormActions } from './app/components/FormActions';
import { MascotPage } from './app/pages/MascotPage';
import { PageHeader } from './app/components/PageHeader';
import { Sidebar } from './app/components/Sidebar';
import { Toast } from './app/components/Toast';
import { useSettingsShell } from './app/hooks/useSettingsShell';
import { AboutPage } from './app/pages/AboutPage';
import { AdvancedPage } from './app/pages/AdvancedPage';
import { HistoryPage } from './app/pages/HistoryPage';
import { HotkeyPage } from './app/pages/HotkeyPage';
import { OutputPage } from './app/pages/OutputPage';
import { PermissionsPage } from './app/pages/PermissionsPage';
import { StatusPage } from './app/pages/StatusPage';
import { VoicePage } from './app/pages/VoicePage';
import { I18nContext, translate } from './lib/i18n/I18nContext';
import { resolveLocale } from './lib/i18n/locale';

export default function App() {
  const isMascot = new URLSearchParams(window.location.search).get('window') === 'mascot';

  if (isMascot) {
    return <MascotPage />;
  }

  const shell = useSettingsShell();
  const { activePage, activeTitle, databaseHealth, settings, status } = shell;

  if (!settings || !status || !databaseHealth) {
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

        <section className="workspace">
          <DesktopTitlebar />

          <section className="content">
            <PageHeader title={activeTitle} version={status.app_version} />

            <form onSubmit={shell.handleSave}>
              {activePage === 'status' && (
                <StatusPage status={status} databaseHealth={databaseHealth} settings={settings} />
              )}
              {activePage === 'hotkey' && (
                <HotkeyPage settings={settings} setSettings={shell.setSettings} />
              )}
              {activePage === 'voice' && (
                <VoicePage settings={settings} setSettings={shell.setSettings} />
              )}
              {activePage === 'permissions' && <PermissionsPage />}
              {activePage === 'output' && (
                <OutputPage settings={settings} setSettings={shell.setSettings} />
              )}
              {activePage === 'advanced' && (
                <AdvancedPage
                  settings={settings}
                  setSettings={shell.setSettings}
                  updateResult={shell.updateResult}
                  onCheckUpdate={shell.handleCheckUpdate}
                  onAutostart={shell.handleAutostart}
                />
              )}
              {activePage === 'history' && <HistoryPage settings={settings} />}
              {activePage === 'about' && <AboutPage />}

              {activePage !== 'status' && activePage !== 'about' && (
                <FormActions onReset={shell.handleReset} />
              )}
            </form>
          </section>
        </section>
      </main>
    </I18nContext.Provider>
  );
}
