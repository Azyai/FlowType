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

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const keys = [];
    if (e.ctrlKey) keys.push('Ctrl');
    if (e.altKey) keys.push('Alt');
    if (e.shiftKey) keys.push('Shift');
    if (e.metaKey) keys.push('Meta');
    
    const keyMap: Record<string, string> = {
      ' ': 'Space',
    };
    
    const mainKey = keyMap[e.key] || (e.key.length === 1 ? e.key.toUpperCase() : e.key);
    
    if (!['Ctrl', 'Alt', 'Shift', 'Meta'].includes(mainKey)) {
      keys.push(mainKey);
    }

    const combo = keys.join('+');
    
    // Basic reserved key check
    if (combo === 'Meta+L' || combo === 'Ctrl+Alt+Delete') {
      setErrorMsg('此快捷键被系统保留，请重新设置 (System reserved)');
      return;
    }

    if (keys.length > 0) {
      setSettings({ ...settings, hotkey: combo });
      setErrorMsg('');
    }
  };

  return (
    <section className="panel">
      <label className="field">
        <span>{t('hotkey.holdToTalk')}</span>
        <input
          value={settings.hotkey}
          onKeyDown={handleKeyDown}
          readOnly
          placeholder="点击此处修改，按下你需要设置的组合键"
          aria-label={t('hotkey.holdToTalk')}
          style={{ cursor: 'pointer' }}
        />
      </label>
      {errorMsg && (
        <div style={{ color: 'var(--color-danger, #ff4d4f)', fontSize: '0.85em', marginTop: '-8px', marginBottom: '8px' }}>
          {errorMsg}
        </div>
      )}
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
