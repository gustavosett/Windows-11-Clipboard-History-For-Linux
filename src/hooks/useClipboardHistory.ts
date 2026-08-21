import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import type { ClipboardItem, HistoryPage } from '../types/clipboard'
import {
  clampOffset,
  clampPageSize,
  DEFAULT_PAGE_SIZE,
  mergePageById,
} from '../utils/pagination'

/** Options for windowed history loading (ADR-0007).
 *  گزینه‌های بارگذاری پنجره‌ای تاریخچه (ADR-0007). */
export interface UseClipboardHistoryOptions {
  /**
   * Page size for `get_history_page`. Defaults to DEFAULT_PAGE_SIZE (100).
   * اندازهٔ صفحه برای `get_history_page`. پیش‌فرض ۱۰۰.
   */
  pageSize?: number
}

function isHistoryPage(payload: unknown): payload is HistoryPage {
  return (
    typeof payload === 'object' &&
    payload !== null &&
    Array.isArray((payload as HistoryPage).items) &&
    typeof (payload as HistoryPage).total === 'number'
  )
}

/**
 * Hook for managing clipboard history with bounded IPC pages.
 * هوک مدیریت تاریخچهٔ کلیپ‌بورد با صفحات محدود IPC.
 */
export function useClipboardHistory(options: UseClipboardHistoryOptions = {}) {
  const [history, setHistory] = useState<ClipboardItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [total, setTotal] = useState<number | null>(null)

  const pageSize = clampPageSize(options.pageSize ?? DEFAULT_PAGE_SIZE)
  const nextOffsetRef = useRef(0)

  const fetchHistory = useCallback(async () => {
    try {
      setIsLoading(true)
      const page = await invoke<HistoryPage>('get_history_page', {
        limit: pageSize,
        offset: 0,
      })
      nextOffsetRef.current = page.items.length
      setTotal(page.total)
      setHistory(page.items)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch history')
    } finally {
      setIsLoading(false)
    }
  }, [pageSize])

  /** Fetch the next bounded window and merge it into the list.
   *  دریافت پنجرهٔ بعدی و ادغام آن در فهرست. */
  const loadMore = useCallback(async () => {
    const limit = pageSize
    const offset = clampOffset(nextOffsetRef.current)
    const currentTotal = total
    if (currentTotal != null && offset >= currentTotal) return
    try {
      setIsLoadingMore(true)
      const page = await invoke<HistoryPage>('get_history_page', { limit, offset })
      nextOffsetRef.current = offset + page.items.length
      setTotal(page.total)
      setHistory((prev) => mergePageById(prev, page.items))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more history')
    } finally {
      setIsLoadingMore(false)
    }
  }, [pageSize, total])

  const clearHistory = useCallback(async () => {
    try {
      await invoke('clear_history')
      setHistory((prev) => {
        const pinned = prev.filter((item) => item.pinned)
        setTotal(pinned.length)
        nextOffsetRef.current = pinned.length
        return pinned
      })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear history')
    }
  }, [])

  const deleteItem = useCallback(async (id: string) => {
    try {
      await invoke('delete_item', { id })
      setHistory((prev) => prev.filter((item) => item.id !== id))
      setTotal((prev) => (prev == null ? prev : Math.max(0, prev - 1)))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete item')
    }
  }, [])

  const togglePin = useCallback(
    async (id: string) => {
      try {
        const updatedItem = await invoke<ClipboardItem>('toggle_pin', { id })
        if (updatedItem) {
          setHistory((prev) => {
            const otherItems = prev.filter((item) => item.id !== id)
            const pinnedItems = otherItems.filter((item) => item.pinned)
            const unpinnedItems = otherItems.filter((item) => !item.pinned)

            if (updatedItem.pinned) {
              return [...pinnedItems, updatedItem, ...unpinnedItems]
            }
            const allUnpinned = [updatedItem, ...unpinnedItems]
            allUnpinned.sort(
              (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
            )
            return [...pinnedItems, ...allUnpinned]
          })
        } else {
          console.warn('[useClipboardHistory] Toggle pin returned null, refreshing history')
          await fetchHistory()
        }
      } catch (err) {
        console.warn('[useClipboardHistory] Toggle pin failed, refreshing history')
        await fetchHistory()
        setError(err instanceof Error ? err.message : 'Failed to toggle pin')
      }
    },
    [fetchHistory]
  )

  const pasteItem = useCallback(
    async (id: string) => {
      try {
        await invoke('paste_item', { id })
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err)
        console.warn('[useClipboardHistory] Paste failed, refreshing history:', errorMessage)
        await fetchHistory()
        setError(errorMessage)
      }
    },
    [fetchHistory]
  )

  useEffect(() => {
    const initialFetchTimer = globalThis.setTimeout(() => {
      void fetchHistory()
    }, 0)

    let isMounted = true
    let unlistenChanged: UnlistenFn | undefined
    let unlistenCleared: UnlistenFn | undefined
    let unlistenSync: UnlistenFn | undefined

    const setupListeners = async () => {
      const uChanged = await listen<ClipboardItem>('clipboard-changed', (event) => {
        const incoming = event.payload
        setHistory((prev) => {
          const without = prev.filter((item) => item.id !== incoming.id)
          const pinned = without.filter((item) => item.pinned)
          const unpinned = without.filter((item) => !item.pinned)
          if (incoming.pinned) {
            return [incoming, ...pinned, ...unpinned]
          }
          return [...pinned, incoming, ...unpinned]
        })
        setTotal((prev) => (prev == null ? prev : prev + 1))
      })
      if (!isMounted) {
        uChanged()
      } else {
        unlistenChanged = uChanged
      }

      const uCleared = await listen('history-cleared', () => {
        // The listener callback is sync; the refresh is fire-and-forget.
        // کال‌بک شنونده همگام است؛ به‌روزرسانی به‌صورت fire-and-forget اجرا می‌شود.
        fetchHistory().catch((e) => {
          console.warn('[useClipboardHistory] Failed to refresh history on history-cleared', e)
        })
      })
      if (!isMounted) {
        uCleared()
      } else {
        unlistenCleared = uCleared
      }

      const uSync = await listen<HistoryPage | ClipboardItem[]>('history-sync', (event) => {
        const payload = event.payload
        if (isHistoryPage(payload)) {
          nextOffsetRef.current = payload.items.length
          setTotal(payload.total)
          setHistory(payload.items)
          return
        }
        if (Array.isArray(payload)) {
          setHistory(payload)
          setTotal(payload.length)
          nextOffsetRef.current = payload.length
        }
      })
      if (!isMounted) {
        uSync()
      } else {
        unlistenSync = uSync
      }
    }

    void setupListeners()

    return () => {
      globalThis.clearTimeout(initialFetchTimer)
      isMounted = false
      unlistenChanged?.()
      unlistenCleared?.()
      unlistenSync?.()
    }
  }, [fetchHistory])

  const hasMore = total != null && history.length < total

  return {
    history,
    isLoading,
    isLoadingMore,
    error,
    total,
    hasMore,
    loadMore,
    fetchHistory,
    clearHistory,
    deleteItem,
    togglePin,
    pasteItem,
  }
}
