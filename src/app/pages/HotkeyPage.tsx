import { useState } from 'react';

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
  const [errorMsg, setErrorMsg] = useState('');

  function handleKeyDown(event: React.KeyboardEvent<HTMLInputElement>) {
    event.preventDefault();
    event.stopPropagation();

    const keys = [];
    if (event.ctrlKey) keys.push('Ctrl');
    if (event.altKey) keys.push('Alt');
    if (event.shiftKey) keys.push('Shift');
    if (event.metaKey) keys.push('Meta');

    const keyMap: Record<string, string> = {
      ' ': 'Space'
    };
    const mainKey = keyMap[event.key] || (event.key.length === 1 ? event.key.toUpperCase() : event.key);

    if (!['Ctrl', 'Alt', 'Shift', 'Meta'].includes(mainKey)) {
      keys.push(mainKey);
    }

    const combo = keys.join('+');
    if (combo === 'Meta+L' || combo === 'Ctrl+Alt+Delete') {
      setErrorMsg(t('hotkey.reserved'));
      return;
    }

    if (keys.length > 0) {
      setSettings({ ...settings, hotkey: combo });
      setErrorMsg('');
    }
  }

  return (
    <section className="panel">
      <label className="field">
        <span>{t('hotkey.holdToTalk')}</span>
        <input
          value={settings.hotkey}
          onKeyDown={handleKeyDown}
          readOnly
          placeholder={t('hotkey.placeholder')}
          aria-label={t('hotkey.holdToTalk')}
          style={{ cursor: 'pointer' }}
        />
      </label>
      {errorMsg && <p className="inline-result danger">{errorMsg}</p>}
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
