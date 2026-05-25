import { ChevronLeft, ChevronRight, Copy, Search, Trash2 } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';

import { readableError } from '../../lib/formatters/errors';
import { useI18n } from '../../lib/i18n/I18nContext';
import { deleteHistoryItem, getHistory } from '../../lib/tauri';
import type {
  AppSettings,
  TranscriptHistoryItem,
  TranscriptHistoryPage as TranscriptHistoryPageResult
} from '../../types';

const PAGE_SIZE = 10;
const HISTORY_PREVIEW_LENGTH = 30;

export function HistoryPage({
  settings,
  onClearHistory,
  onToast,
  onSummaryChange,
  onRequestConfirm
}: {
  settings: AppSettings;
  onClearHistory: () => Promise<{ deleted_count: number } | null> | { deleted_count: number } | null;
  onToast: (kind: 'success' | 'error', message: string) => void;
  onSummaryChange: (summary: { total: number; enabled: boolean; retentionDays: number }) => void;
  onRequestConfirm: (options: { title: string; message: string; tone?: 'danger' | 'default' }) => Promise<boolean>;
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
  const [currentPage, setCurrentPage] = useState(1);
  const [jumpPageValue, setJumpPageValue] = useState('1');
  const [searchTerm, setSearchTerm] = useState('');
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<TranscriptHistoryItem[] | null>(null);
  const [searchSource, setSearchSource] = useState<TranscriptHistoryItem[] | null>(null);

  const loadHistory = useCallback(async (pageNumber = 1) => {
    setLoading(true);
    setError(null);

    try {
      const requestedPage = Math.max(1, pageNumber);
      const initialOffset = (requestedPage - 1) * PAGE_SIZE;
      let page = await getHistory(PAGE_SIZE, initialOffset);
      const resolvedTotalPages = Math.max(1, Math.ceil(page.total / PAGE_SIZE));
      const resolvedPage = page.total === 0 ? 1 : Math.min(requestedPage, resolvedTotalPages);

      if (resolvedPage !== requestedPage) {
        const resolvedOffset = (resolvedPage - 1) * PAGE_SIZE;
        page = await getHistory(PAGE_SIZE, resolvedOffset);
      }

      setHistoryPage(page);
      setSearchSource(null);
      setCurrentPage(resolvedPage);
      setJumpPageValue(String(resolvedPage));
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
    const confirmed = await onRequestConfirm({
      title: t('history.confirmClearTitle'),
      message: t('history.confirmClear'),
      tone: 'danger'
    });
    if (!confirmed) {
      return;
    }

    setClearing(true);
    try {
      const result = await onClearHistory();
      if (result) {
        await loadHistory(1);
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
    const confirmed = await onRequestConfirm({
      title: t('history.confirmDeleteTitle'),
      message: t('history.confirmDelete'),
      tone: 'danger'
    });
    if (!confirmed) {
      return;
    }

    setDeletingId(id);

    try {
      const result = await deleteHistoryItem(id);
      if (result.deleted_count > 0) {
        const remainingTotal = Math.max(0, historyPage.total - result.deleted_count);
        const nextPage = remainingTotal === 0 ? 1 : Math.min(currentPage, Math.ceil(remainingTotal / PAGE_SIZE));
        await loadHistory(nextPage);
        onToast('success', t('notice.historyItemDeleted'));
      }
    } catch (deleteError) {
      onToast('error', readableError(deleteError));
    } finally {
      setDeletingId(null);
    }
  }

  const normalizedSearchTerm = searchTerm.trim().toLocaleLowerCase();
  const isSearching = normalizedSearchTerm.length > 0;
  const displayTotal = isSearching ? searchResults?.length ?? 0 : historyPage.total;
  const totalPages = Math.max(1, Math.ceil(displayTotal / PAGE_SIZE));
  const resolvedCurrentPage = Math.min(currentPage, totalPages);
  const displayItems = useMemo(() => {
    if (!isSearching) {
      return historyPage.items;
    }

    const source = searchResults ?? [];
    const offset = (resolvedCurrentPage - 1) * PAGE_SIZE;
    return source.slice(offset, offset + PAGE_SIZE);
  }, [historyPage.items, isSearching, resolvedCurrentPage, searchResults]);
  const paginationItems = buildPaginationItems(resolvedCurrentPage, totalPages);
  const parsedJumpPage = Number.parseInt(jumpPageValue, 10);
  const canJumpToPage =
    Number.isInteger(parsedJumpPage) &&
    parsedJumpPage >= 1 &&
    parsedJumpPage <= totalPages &&
    parsedJumpPage !== resolvedCurrentPage;
  const actionsDisabled = loading || clearing || searchLoading;

  useEffect(() => {
    let cancelled = false;

    async function runSearch() {
      if (!isSearching) {
        setSearchLoading(false);
        setSearchResults(null);
        return;
      }

      setSearchLoading(true);
      setError(null);

      try {
        let source = searchSource;
        if (!source) {
          const page = await getHistory(Math.max(historyPage.total, PAGE_SIZE), 0);
          if (cancelled) {
            return;
          }
          source = page.items;
          setSearchSource(source);
        }

        const filtered = source.filter((item) => matchesHistorySearch(item, normalizedSearchTerm));
        if (cancelled) {
          return;
        }

        setSearchResults(filtered);
      } catch (searchError) {
        if (cancelled) {
          return;
        }
        setError(readableError(searchError));
        setSearchResults([]);
      } finally {
        if (!cancelled) {
          setSearchLoading(false);
        }
      }
    }

    void runSearch();

    return () => {
      cancelled = true;
    };
  }, [historyPage.total, isSearching, normalizedSearchTerm, searchSource]);

  useEffect(() => {
    if (currentPage !== resolvedCurrentPage) {
      setCurrentPage(resolvedCurrentPage);
      setJumpPageValue(String(resolvedCurrentPage));
    }
  }, [currentPage, resolvedCurrentPage]);

  useEffect(() => {
    setCurrentPage(1);
    setJumpPageValue('1');
  }, [normalizedSearchTerm]);

  function handlePageChange(pageNumber: number) {
    const nextPage = Math.min(Math.max(1, pageNumber), totalPages);
    if (isSearching) {
      setCurrentPage(nextPage);
      setJumpPageValue(String(nextPage));
      return;
    }

    void loadHistory(nextPage);
  }

  function handleJumpToPage() {
    if (!canJumpToPage) {
      return;
    }

    handlePageChange(parsedJumpPage);
  }

  return (
    <section className="panel history-panel">
      <div className="history-actions">
        <button
          type="button"
          className="secondary-button"
          onClick={() => void loadHistory(resolvedCurrentPage)}
          disabled={actionsDisabled}
        >
          {t('history.refresh')}
        </button>
        <button
          type="button"
          className="secondary-button"
          onClick={() => void handleClearHistory()}
          disabled={actionsDisabled || historyPage.total === 0}
        >
          {t('advanced.clearHistory')}
        </button>
        <label className="history-search-field">
          <Search aria-hidden="true" />
          <input
            type="search"
            value={searchTerm}
            onChange={(event) => setSearchTerm(event.target.value)}
            placeholder={t('history.searchPlaceholder')}
            aria-label={t('history.search')}
            disabled={loading || clearing}
          />
        </label>
      </div>

      {error && (
        <div className="inline-result danger" role="alert">
          {error}
        </div>
      )}

      <div className="history-list-shell">
        <div className="history-list-scroll">
          {actionsDisabled && displayItems.length === 0 ? (
            <div className="history-state-shell">
              <p className="muted">{t('history.loading')}</p>
            </div>
          ) : displayItems.length === 0 ? (
            <div className="history-empty">
              <strong>{isSearching ? t('history.searchEmptyTitle') : t('history.emptyTitle')}</strong>
              <p className="muted">{isSearching ? t('history.searchEmptyBody') : t('history.emptyBody')}</p>
            </div>
          ) : (
            <div className="history-list">
              {displayItems.map((item) => {
                const itemText = historyItemText(item);
                const previewText = truncateHistoryText(itemText);
                const isFailed = Boolean(item.error_code);
                const chipLabel = historyItemChipLabel(item, t);
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
                      {chipLabel && (
                        <span className={`history-chip${isFailed ? ' danger' : ''}`}>
                          {chipLabel}
                        </span>
                      )}
                    </div>

                    <div className="history-item-body compact">
                      <p className="history-preview">{previewText}</p>
                      <div className="history-item-actions">
                        <button
                          type="button"
                          className="secondary-button history-action-button"
                          onClick={() => void handleCopyHistoryItem(item)}
                          disabled={actionsDisabled || deletingId === item.id}
                        >
                          <Copy aria-hidden="true" />
                          {t('history.copy')}
                        </button>
                        <button
                          type="button"
                          className="secondary-button history-action-button danger"
                          onClick={() => void handleDeleteHistoryItem(item.id)}
                          disabled={actionsDisabled || deletingId === item.id}
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
        </div>

        {displayItems.length > 0 && (
          <div className="history-pagination">
            <span className="history-page-summary">
              {t('history.pageLabel', { current: resolvedCurrentPage, total: totalPages })}
            </span>

            <div className="history-page-buttons">
              <button
                type="button"
                className="secondary-button history-page-button"
                onClick={() => handlePageChange(resolvedCurrentPage - 1)}
                disabled={actionsDisabled || resolvedCurrentPage === 1}
              >
                <ChevronLeft aria-hidden="true" />
                {t('history.previousPage')}
              </button>

              {paginationItems.map((item, index) =>
                item === 'ellipsis' ? (
                  <span key={`ellipsis-${index}`} className="history-page-ellipsis" aria-hidden="true">
                    ...
                  </span>
                ) : (
                  <button
                    key={item}
                    type="button"
                    className={`secondary-button history-page-button${item === resolvedCurrentPage ? ' active' : ''}`}
                    onClick={() => handlePageChange(item)}
                    disabled={actionsDisabled || item === resolvedCurrentPage}
                    aria-current={item === resolvedCurrentPage ? 'page' : undefined}
                  >
                    {item}
                  </button>
                )
              )}

              <button
                type="button"
                className="secondary-button history-page-button"
                onClick={() => handlePageChange(resolvedCurrentPage + 1)}
                disabled={actionsDisabled || resolvedCurrentPage === totalPages}
              >
                {t('history.nextPage')}
                <ChevronRight aria-hidden="true" />
              </button>
            </div>

            <div className="history-page-jump">
              <label htmlFor="history-page-jump-input">{t('history.jumpToPage')}</label>
              <input
                id="history-page-jump-input"
                type="number"
                min={1}
                max={totalPages}
                inputMode="numeric"
                value={jumpPageValue}
                onChange={(event) => setJumpPageValue(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    handleJumpToPage();
                  }
                }}
                disabled={actionsDisabled}
              />
              <button
                type="button"
                className="secondary-button history-jump-button"
                onClick={handleJumpToPage}
                disabled={actionsDisabled || !canJumpToPage}
              >
                {t('history.goToPage')}
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}

function historyItemText(item: TranscriptHistoryItem) {
  return item.final_text.trim() || item.raw_text.trim() || item.error_summary?.trim() || '-';
}

function historyItemChipLabel(
  item: TranscriptHistoryItem,
  t: ReturnType<typeof useI18n>['t']
) {
  if (item.error_code) {
    return item.error_code;
  }

  if (item.output_style === 'raw') {
    return null;
  }

  return t(`output.${item.output_style}`);
}

function matchesHistorySearch(item: TranscriptHistoryItem, query: string) {
  const haystack = [
    item.final_text,
    item.raw_text,
    item.error_summary ?? '',
    item.error_code ?? ''
  ]
    .join(' ')
    .toLocaleLowerCase();

  return haystack.includes(query);
}

function buildPaginationItems(currentPage: number, totalPages: number) {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, index) => index + 1);
  }

  const start = Math.max(2, Math.min(currentPage - 1, totalPages - 4));
  const end = Math.min(totalPages - 1, Math.max(currentPage + 1, 5));
  const items: Array<number | 'ellipsis'> = [1];

  if (start > 2) {
    items.push('ellipsis');
  }

  for (let page = start; page <= end; page += 1) {
    items.push(page);
  }

  if (end < totalPages - 1) {
    items.push('ellipsis');
  }

  items.push(totalPages);

  return items;
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
