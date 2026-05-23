import {
  Activity,
  AudioWaveform,
  Clipboard,
  History,
  Info,
  Keyboard,
  Settings,
  ShieldCheck
} from 'lucide-react';

import type { TranslationKey } from '../../lib/i18n/types';

export type PageId =
  | 'status'
  | 'hotkey'
  | 'voice'
  | 'permissions'
  | 'output'
  | 'advanced'
  | 'history'
  | 'about';

export const pages: Array<{ id: PageId; labelKey: TranslationKey; icon: typeof Activity }> = [
  { id: 'status', labelKey: 'nav.status', icon: Activity },
  { id: 'hotkey', labelKey: 'nav.hotkey', icon: Keyboard },
  { id: 'voice', labelKey: 'nav.voice', icon: AudioWaveform },
  { id: 'permissions', labelKey: 'nav.permissions', icon: ShieldCheck },
  { id: 'output', labelKey: 'nav.output', icon: Clipboard },
  { id: 'advanced', labelKey: 'nav.advanced', icon: Settings },
  { id: 'history', labelKey: 'nav.history', icon: History },
  { id: 'about', labelKey: 'nav.about', icon: Info }
] as const;

export function pageTitleKey(pageId: PageId): TranslationKey {
  if (pageId === 'about') {
    return 'heading.about';
  }

  return pages.find((page) => page.id === pageId)?.labelKey ?? 'nav.status';
}
