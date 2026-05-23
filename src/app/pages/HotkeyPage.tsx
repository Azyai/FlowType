import type { AppSettings } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

export function HotkeyPage({
  settings,
  setSettings
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
}) {
  const { t } = useI18n();

  return (
    <section className="panel">
      <label className="field">
        <span>{t('hotkey.holdToTalk')}</span>
        <input
          value={settings.hotkey}
          onChange={(event) => setSettings({ ...settings, hotkey: event.target.value })}
          aria-label={t('hotkey.holdToTalk')}
        />
      </label>
      <label className="field">
        <span>{t('hotkey.inputMode')}</span>
        <select
          value={settings.input_mode}
          onChange={(event) =>
            setSettings({ ...settings, input_mode: event.target.value as AppSettings['input_mode'] })
          }
        >
          <option value="hold_to_talk">{t('hotkey.holdToTalkOption')}</option>
          <option value="toggle">{t('hotkey.toggleOption')}</option>
        </select>
      </label>
    </section>
  );
}
