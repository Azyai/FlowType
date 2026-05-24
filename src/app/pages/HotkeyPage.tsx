import { useState } from 'react';

import type { AppSettings } from '../../types';
import { useI18n } from '../../lib/i18n/I18nContext';

const RESERVED_COMBOS = new Set(['Meta+L', 'Ctrl+Alt+Delete']);

export function HotkeyPage({
  settings,
  setSettings
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
}) {
  const { t } = useI18n();
  const [errorMsg, setErrorMsg] = useState('');

  function handleKeyDown(field: 'hotkey' | 'toggle_hotkey') {
    return (event: React.KeyboardEvent<HTMLInputElement>) => {
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
      const rawMainKey = keyMap[event.key] || (event.key.length === 1 ? event.key.toUpperCase() : event.key);
      const mainKey = /^Key[A-Z]$/.test(rawMainKey)
        ? rawMainKey.slice(3)
        : /^Digit\d$/.test(rawMainKey)
          ? rawMainKey.slice(5)
          : rawMainKey;

      if (!['Ctrl', 'Alt', 'Shift', 'Meta'].includes(mainKey)) {
        keys.push(mainKey);
      }

      const combo = keys.join('+');
      if (RESERVED_COMBOS.has(combo)) {
        setErrorMsg(t('hotkey.reserved'));
        return;
      }

      if (keys.length === 0) {
        return;
      }

      const otherField = field === 'hotkey' ? 'toggle_hotkey' : 'hotkey';
      if (combo === settings[otherField]) {
        setErrorMsg(t('hotkey.conflict'));
        return;
      }

      setSettings({ ...settings, [field]: combo });
      setErrorMsg('');
    };
  }

  return (
    <section className="panel">
      <label className="field">
        <span>{t('hotkey.holdToTalk')}</span>
        <input
          value={settings.hotkey}
          onKeyDown={handleKeyDown('hotkey')}
          readOnly
          placeholder={t('hotkey.placeholder')}
          aria-label={t('hotkey.holdToTalk')}
          style={{ cursor: 'pointer' }}
        />
        <span className="muted">{t('hotkey.holdToTalkHint')}</span>
      </label>
      <label className="field">
        <span>{t('hotkey.toggleRecording')}</span>
        <input
          value={settings.toggle_hotkey}
          onKeyDown={handleKeyDown('toggle_hotkey')}
          readOnly
          placeholder={t('hotkey.placeholder')}
          aria-label={t('hotkey.toggleRecording')}
          style={{ cursor: 'pointer' }}
        />
        <span className="muted">{t('hotkey.toggleHint')}</span>
      </label>
      {errorMsg && <p className="inline-result danger">{errorMsg}</p>}
    </section>
  );
}
