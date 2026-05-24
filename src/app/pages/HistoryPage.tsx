import { useCallback, useEffect, useState } from 'react';

import { readableError } from '../../lib/formatters/errors';
import { useI18n } from '../../lib/i18n/I18nContext';
import { getHistory } from '../../lib/tauri';
import type { AppSettings, TranscriptHistoryPage as TranscriptHistoryPageResult } from '../../types';

const PAGE_SIZE = 20;

export function HistoryPage({
  settings,
  onClearHistory
}: {
  settings: AppSettings;
  onClearHistory: () => Promise<void> | void;
}) {
  const { locale, t } = useI18n();
  const status = settings.save_history ? t('history.enabled') : t('history.disabled');
  const [historyPage, setHistoryPage] = useState<TranscriptHistoryPageResult>({
    items: [],
    total: 0,
    limit: PAGE_SIZE,
    offset: 0
  });
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadHistory = useCallback(async (offset = 0, append = false) => {
    setLoading(true);
    setError(null);

    try {
      const page = await getHistory(PAGE_SIZE, offset);
      setHistoryPage((current) =>
        append
          ? {
              ...page,
              items: [...current.items, ...page.items]
            }
          : page
      );
    } catch (loadError) {
      setError(readableError(loadError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadHistory();
  }, [loadHistory]);

  async function handleClearHistory() {
    setClearing(true);
    try {
      await onClearHistory();
      await loadHistory();
    } finally {
      setClearing(false);
    }
  }

  function formatDate(timestampSeconds: number) {
    return new Intl.DateTimeFormat(locale, {
      dateStyle: 'medium',
      timeStyle: 'short'
    }).format(new Date(timestampSeconds * 1000));
  }

  function formatDuration(durationMs: number) {
    if (durationMs < 1000) {
      return `${durationMs} ms`;
    }

    return `${(durationMs / 1000).toFixed(durationMs >= 10_000 ? 0 : 1)} s`;
  }

  const canLoadMore = historyPage.items.length < historyPage.total;

  return (
    <section className="panel history-panel">
      <div className="service-summary">
        <strong>{t('history.total', { count: historyPage.total })}</strong>
        <span>{status}</span>
      </div>
      <p className="muted">
        {t('history.storage', { status })} {t('history.retention', { days: settings.history_retention_days })}
      </p>
      <p className="muted">{t('history.textOnly')}</p>

      <div className="history-actions">
        <button
          type="button"
          className="secondary-button"
          onClick={() => void loadHistory()}
          disabled={loading || clearing}
        >
          {t('history.refresh')}
        </button>
        <button
          type="button"
          className="secondary-button"
          onClick={() => void handleClearHistory()}
          disabled={loading || clearing || historyPage.total === 0}
        >
          {t('advanced.clearHistory')}
        </button>
      </div>

      {error && (
        <div className="inline-result danger" role="alert">
          {error}
        </div>
      )}

      {loading && historyPage.items.length === 0 ? (
        <p className="muted">{t('history.loading')}</p>
      ) : historyPage.items.length === 0 ? (
        <div className="history-empty">
          <strong>{t('history.emptyTitle')}</strong>
          <p className="muted">{t('history.emptyBody')}</p>
        </div>
      ) : (
        <div className="history-list">
          {historyPage.items.map((item) => {
            const finalText = item.final_text.trim();
            const rawText = item.raw_text.trim();
            const isFailed = Boolean(item.error_code);
            const deliveryLabel = isFailed
              ? t('history.failed')
              : item.injected
                ? t('history.injected')
                : t('history.copied');

            return (
              <article key={item.id} className="history-item">
                <div className="history-item-header">
                  <div className="history-item-meta">
                    <strong>{deliveryLabel}</strong>
                    <span>{t('history.duration')}: {formatDuration(item.recognition_duration_ms)}</span>
                    <span>{t('history.createdAt')}: {formatDate(item.created_at)}</span>
                  </div>
                  <span className={`history-chip${isFailed ? ' danger' : ''}`}>
                    {item.error_code ?? item.output_style}
                  </span>
                </div>

                <div className="history-item-body">
                  <div className="history-copy-block">
                    <strong>{t('history.finalText')}</strong>
                    <p>{finalText || rawText || '-'}</p>
                  </div>

                  <div className="history-copy-block">
                    <strong>{t('history.rawText')}</strong>
                    <p>{rawText || '-'}</p>
                  </div>
                </div>

                {item.error_summary && (
                  <div className="inline-result danger">
                    {item.error_code ? `${item.error_code}: ` : ''}
                    {item.error_summary}
                  </div>
                )}
              </article>
            );
          })}
        </div>
      )}

      {canLoadMore && (
        <button
          type="button"
          className="secondary-button"
          onClick={() => void loadHistory(historyPage.items.length, true)}
          disabled={loading || clearing}
        >
          {t('history.loadMore')}
        </button>
      )}
    </section>
  );
}
