import { Copy, Trash2 } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { readableError } from '../../lib/formatters/errors';
import { useI18n } from '../../lib/i18n/I18nContext';
import { deleteHistoryItem, getHistory } from '../../lib/tauri';
import type {
  AppSettings,
  TranscriptHistoryItem,
  TranscriptHistoryPage as TranscriptHistoryPageResult
} from '../../types';

const PAGE_SIZE = 20;
const HISTORY_PREVIEW_LENGTH = 20;

function confirmAction(message: string) {
  if (typeof window.confirm !== 'function') {
    return true;
  }

  return window.confirm(message);
}

export function HistoryPage({
  settings,
  onClearHistory,
  onToast,
  onSummaryChange
}: {
  settings: AppSettings;
  onClearHistory: () => Promise<{ deleted_count: number } | null> | { deleted_count: number } | null;
  onToast: (kind: 'success' | 'error', message: string) => void;
  onSummaryChange: (summary: { total: number; enabled: boolean; retentionDays: number }) => void;
}) {
  const { locale, t } = useI18n();
  const [historyPage, setHistoryPage] = useState<TranscriptHistoryPageResult>({
    items: [],
    total: 0,
    limit: PAGE_SIZE,
    offset: 0
  });
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);
  const [deletingId, setDeletingId] = useState<number | null>(null);
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

  useEffect(() => {
    onSummaryChange({
      total: historyPage.total,
      enabled: settings.save_history,
      retentionDays: settings.history_retention_days
    });
  }, [historyPage.total, onSummaryChange, settings.history_retention_days, settings.save_history]);

  async function handleClearHistory() {
    setClearing(true);
    try {
      const result = await onClearHistory();
      if (result) {
        await loadHistory();
      }
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

  async function handleCopyHistoryItem(item: TranscriptHistoryItem) {
    try {
      await copyTextToClipboard(historyItemText(item));
      onToast('success', t('notice.historyItemCopied'));
    } catch (copyError) {
      onToast('error', readableError(copyError));
    }
  }

  async function handleDeleteHistoryItem(id: number) {
    if (!confirmAction(t('history.confirmDelete'))) {
      return;
    }

    setDeletingId(id);

    try {
      const result = await deleteHistoryItem(id);
      if (result.deleted_count > 0) {
        const nextOffset =
          historyPage.offset > 0 && historyPage.items.length === result.deleted_count
            ? Math.max(0, historyPage.offset - PAGE_SIZE)
            : historyPage.offset;
        await loadHistory(nextOffset);
        onToast('success', t('notice.historyItemDeleted'));
      }
    } catch (deleteError) {
      onToast('error', readableError(deleteError));
    } finally {
      setDeletingId(null);
    }
  }

  const canLoadMore = historyPage.items.length < historyPage.total;

  return (
    <section className="panel history-panel">
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
            const itemText = historyItemText(item);
            const previewText = truncateHistoryText(itemText);
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
                    <span>{t('history.createdAt')}: {formatDate(item.created_at)}</span>
                    <span>{t('history.duration')}: {formatDuration(item.recognition_duration_ms)}</span>
                  </div>
                  <span className={`history-chip${isFailed ? ' danger' : ''}`}>
                    {item.error_code ?? item.output_style}
                  </span>
                </div>

                <div className="history-item-body compact">
                  <p className="history-preview">{previewText}</p>
                  <div className="history-item-actions">
                    <button
                      type="button"
                      className="secondary-button history-action-button"
                      onClick={() => void handleCopyHistoryItem(item)}
                      disabled={loading || clearing || deletingId === item.id}
                    >
                      <Copy aria-hidden="true" />
                      {t('history.copy')}
                    </button>
                    <button
                      type="button"
                      className="secondary-button history-action-button danger"
                      onClick={() => void handleDeleteHistoryItem(item.id)}
                      disabled={loading || clearing || deletingId === item.id}
                    >
                      <Trash2 aria-hidden="true" />
                      {t('history.delete')}
                    </button>
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

function historyItemText(item: TranscriptHistoryItem) {
  return item.final_text.trim() || item.raw_text.trim() || item.error_summary?.trim() || '-';
}

function truncateHistoryText(text: string) {
  const chars = Array.from(text.trim());
  if (chars.length <= HISTORY_PREVIEW_LENGTH) {
    return chars.join('');
  }
  return `${chars.slice(0, HISTORY_PREVIEW_LENGTH).join('')}...`;
}

async function copyTextToClipboard(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', 'true');
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();

  try {
    if (!document.execCommand('copy')) {
      throw new Error('Copy command was rejected by the browser.');
    }
  } finally {
    document.body.removeChild(textarea);
  }
}
