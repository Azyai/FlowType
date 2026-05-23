import type { LocaleCode, LocalePreference } from '../../types';

export function detectLocale(language = navigator.language): LocaleCode {
  return language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

export function resolveLocale(preference: LocalePreference, language = navigator.language): LocaleCode {
  if (preference === 'auto') {
    return detectLocale(language);
  }

  return preference;
}
