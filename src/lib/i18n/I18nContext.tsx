import { createContext, useContext } from 'react';

import { dictionaries } from './dictionaries';
import type { I18nContextValue, TranslationKey } from './types';

export const I18nContext = createContext<I18nContextValue | null>(null);

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within I18nContext.Provider');
  }

  return context;
}

export function translate(
  locale: I18nContextValue['locale'],
  key: TranslationKey,
  params: Record<string, string | number> = {}
) {
  const template = dictionaries[locale][key] ?? dictionaries['en-US'][key] ?? key;
  return Object.entries(params).reduce(
    (text, [name, value]) => text.split(`{${name}}`).join(String(value)),
    template
  );
}
