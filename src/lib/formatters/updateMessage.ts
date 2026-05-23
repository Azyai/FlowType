import type { UpdateCheckResult } from '../../types';
import type { I18nContextValue } from '../i18n/types';

export function updateMessage(result: UpdateCheckResult, t?: I18nContextValue['t']) {
  if (result.status === 'available') {
    return t
      ? t('update.available', { version: result.latest_version ?? '' })
      : `New version ${result.latest_version} available`;
  }
  if (result.status === 'failed') {
    return t ? t('update.failed') : 'Update check failed';
  }
  if (result.status === 'channel_unavailable') {
    return t ? t('update.channelUnavailable') : 'Current channel unavailable';
  }
  return t ? t('update.latest') : 'Already on latest version';
}
