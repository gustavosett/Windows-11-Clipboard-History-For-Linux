/**
 * Symbol Picker Hook
 * Manages symbol state, search, and recently used symbols
 */
import { useState, useMemo, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getSymbols, getSymbolCategories, type SymbolItem } from '../services/symbolService'

const RECENT_SYMBOLS_KEY = 'win11_clipboard_recent_symbols'
const MAX_RECENT_SYMBOLS = 24

export function useSymbolPicker() {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)

  const [recentSymbols, setRecentSymbols] = useState<SymbolItem[]>(() => {
    try {
      const stored = localStorage.getItem(RECENT_SYMBOLS_KEY)
      if (stored) {
        return JSON.parse(stored) as SymbolItem[]
      }
    } catch (e) {
      console.error('Failed to load recent symbols', e)
    }
    return []
  })

  const categories = useMemo(() => getSymbolCategories(), [])

  // Filtered symbols
  const filteredSymbols = useMemo(() => {
    return getSymbols(selectedCategory, searchQuery)
  }, [selectedCategory, searchQuery])

  // Paste symbol
  const pasteSymbol = useCallback(async (symbol: SymbolItem) => {
    try {
      // Use the generic paste_text command
      await invoke('paste_text', { text: symbol.char, itemType: 'symbol' })

      // Update recent
      setRecentSymbols((prev) => {
        const filtered = prev.filter((s) => s.char !== symbol.char)
        const newRecent = [symbol, ...filtered].slice(0, MAX_RECENT_SYMBOLS)
        localStorage.setItem(RECENT_SYMBOLS_KEY, JSON.stringify(newRecent))
        return newRecent
      })
    } catch (err) {
      console.error('Failed to paste symbol:', err)
      // Fallback: write to clipboard and maybe warn user?
      // For now assume paste_emoji works or fails silently.
    }
  }, [])

  return {
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories,
    filteredSymbols,
    recentSymbols,
    pasteSymbol,
  }
}
