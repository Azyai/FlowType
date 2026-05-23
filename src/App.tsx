import { Activity } from 'lucide-react';

import { FormActions } from './app/components/FormActions';
import { PageHeader } from './app/components/PageHeader';
import { Sidebar } from './app/components/Sidebar';
import { useSettingsShell } from './app/hooks/useSettingsShell';
import { AboutPage } from './app/pages/AboutPage';
import { AdvancedPage } from './app/pages/AdvancedPage';
import { HistoryPage } from './app/pages/HistoryPage';
import { HotkeyPage } from './app/pages/HotkeyPage';
import { OutputPage } from './app/pages/OutputPage';
import { PermissionsPage } from './app/pages/PermissionsPage';
import { StatusPage } from './app/pages/StatusPage';
import { VoicePage } from './app/pages/VoicePage';
import { I18nContext } from './lib/i18n/I18nContext';

export default function App() {
  const shell = useSettingsShell();
  const { activePage, activeTitle, databaseHealth, error, notice, settings, status } = shell;

  if (!settings || !status || !databaseHealth) {
    return (
      <main className="loading-shell">
        <Activity aria-hidden="true" />
        <span>Loading FlowType</span>
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
      <main className="app-shell">
        <Sidebar activePage={activePage} status={status} onSelectPage={shell.setActivePage} />

        <section className="content">
          <PageHeader title={activeTitle} version={status.app_version} />

          {notice && <p className="notice">{notice}</p>}
          {error && <p className="error">{error}</p>}

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
      </main>
    </I18nContext.Provider>
  );
}
