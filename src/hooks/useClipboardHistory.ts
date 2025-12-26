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
          setHistory((prev) => prev.map((item) => (item.id === id ? updatedItem : item)))
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
      unlistenChanged = await listen<ClipboardItem>('clipboard-changed', (event) => {
        setHistory((prev) => {
          const newItem = event.payload

          // Check if item already exists by id
          if (prev.some((i) => i.id === newItem.id)) {
            return prev
          }

          // Helper to get plain text from any text-based content
          const getPlainText = (content: ClipboardItem['content']): string | null => {
            if (content.type === 'Text') return content.data
            if (content.type === 'RichText') return content.data.plain
            return null
          }

          // Also check for content duplicates in the first few unpinned items
          // This handles race conditions between fetchHistory and events
          const unpinnedItems = prev.filter((i) => !i.pinned)
          const newPlainText = getPlainText(newItem.content)
          const isDuplicate =
            newPlainText !== null &&
            unpinnedItems.slice(0, 5).some((i) => {
              const existingPlainText = getPlainText(i.content)
              return existingPlainText === newPlainText
            })

          if (isDuplicate) {
            return prev
          }

          // Add new item at the top (after pinned items)
          const pinnedItems = prev.filter((i) => i.pinned)
          return [...pinnedItems, newItem, ...unpinnedItems.slice(0, 49)]
        })
      })

      unlistenCleared = await listen('history-cleared', () => {
        setHistory((prev) => prev.filter((item) => item.pinned))
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
