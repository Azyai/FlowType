import { RotateCcw } from 'lucide-react';

import { useI18n } from '../../lib/i18n/I18nContext';

export function FormActions({ onReset }: { onReset: () => void }) {
  const { t } = useI18n();

  return (
    <div className="form-actions">
      <button type="button" className="secondary-button" onClick={onReset}>
        <RotateCcw aria-hidden="true" />
        {t('actions.reset')}
      </button>
      <button type="submit" className="primary-button">
        {t('actions.save')}
      </button>
    </div>
  );
}
