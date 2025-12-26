import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import type { ClipboardItem } from '../types/clipboard'

/**
 * Hook for managing clipboard history
 */
export function useClipboardHistory() {
  const [history, setHistory] = useState<ClipboardItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Fetch initial history
  const fetchHistory = useCallback(async () => {
    try {
      setIsLoading(true)
      const items = await invoke<ClipboardItem[]>('get_history')
      setHistory(items)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch history')
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Clear all history
  const clearHistory = useCallback(async () => {
    try {
      await invoke('clear_history')
      setHistory((prev) => prev.filter((item) => item.pinned))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear history')
    }
  }, [])

  // Delete a specific item
  const deleteItem = useCallback(async (id: string) => {
    try {
      await invoke('delete_item', { id })
      setHistory((prev) => prev.filter((item) => item.id !== id))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete item')
    }
  }, [])

  // Toggle pin status
  const togglePin = useCallback(
    async (id: string) => {
      try {
        const updatedItem = await invoke<ClipboardItem>('toggle_pin', { id })
        if (updatedItem) {
          setHistory((prev) => {
            // Remove the item from its current position
            const otherItems = prev.filter((item) => item.id !== id)
            const pinnedItems = otherItems.filter((item) => item.pinned)
            const unpinnedItems = otherItems.filter((item) => !item.pinned)

            if (updatedItem.pinned) {
              // Item was pinned - add to the end of pinned items (top of list)
              return [...pinnedItems, updatedItem, ...unpinnedItems]
            } else {
              // Item was unpinned - insert in correct position by timestamp
              const allUnpinned = [updatedItem, ...unpinnedItems]
              allUnpinned.sort(
                (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
              )
              return [...pinnedItems, ...allUnpinned]
            }
          })
        } else {
          // Item not found - refresh history
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

  // Paste an item
  const pasteItem = useCallback(
    async (id: string) => {
      try {
        await invoke('paste_item', { id })
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err)
        console.warn('[useClipboardHistory] Paste failed, refreshing history:', errorMessage)
        // If paste failed due to item not found, refresh history
        // The backend already emits history-sync event, but we fetch as backup
        await fetchHistory()
        setError(errorMessage)
      }
    },
    [fetchHistory]
  )

  // Listen for clipboard changes
  useEffect(() => {
    fetchHistory()

    let unlistenChanged: UnlistenFn | undefined
    let unlistenCleared: UnlistenFn | undefined
    let unlistenSync: UnlistenFn | undefined

    const setupListeners = async () => {
      unlistenChanged = await listen<ClipboardItem>('clipboard-changed', async () => {
        // Backend emits the event and already enforces trimming. Fetch full history
        // to keep frontend in sync with backend limits and ordering.
        try {
          await fetchHistory()
        } catch (e) {
          console.warn('[useClipboardHistory] Failed to refresh history on clipboard-changed', e)
        }
      })

      unlistenCleared = await listen('history-cleared', async () => {
        try {
          await fetchHistory()
        } catch (e) {
          console.warn('[useClipboardHistory] Failed to refresh history on history-cleared', e)
        }
      })

      // Listen for history sync events (triggered when backend detects desync)
      unlistenSync = await listen<ClipboardItem[]>('history-sync', (event) => {
        console.log('[useClipboardHistory] Received history-sync event, refreshing history')
        setHistory(event.payload)
      })
    }

    setupListeners()

    return () => {
      unlistenChanged?.()
      unlistenCleared?.()
      unlistenSync?.()
    }
  }, [fetchHistory])

  return {
    history,
    isLoading,
    error,
    fetchHistory,
    clearHistory,
    deleteItem,
    togglePin,
    pasteItem,
  }
}
