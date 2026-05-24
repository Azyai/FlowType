import {
  History,
  Info,
  Keyboard,
  Settings
} from 'lucide-react';

import type { TranslationKey } from '../../lib/i18n/types';

export type PageId =
  | 'hotkey'
  | 'advanced'
  | 'history'
  | 'about';

export const pages: Array<{ id: PageId; labelKey: TranslationKey; icon: typeof Keyboard }> = [
  { id: 'hotkey', labelKey: 'nav.hotkey', icon: Keyboard },
  { id: 'advanced', labelKey: 'nav.advanced', icon: Settings },
  { id: 'history', labelKey: 'nav.history', icon: History },
  { id: 'about', labelKey: 'nav.about', icon: Info }
] as const;

export function pageTitleKey(pageId: PageId): TranslationKey {
  if (pageId === 'about') {
    return 'heading.about';
  }

  return pages.find((page) => page.id === pageId)?.labelKey ?? 'nav.hotkey';
}
