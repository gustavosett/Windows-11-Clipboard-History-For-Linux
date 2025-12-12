/**
 * GIF Picker Hook
 * Manages GIF state, search, and debouncing
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { fetchTrendingGifs, searchGifs } from '../services/gifService'
import type { Gif } from '../types/gif'

/** Debounce delay for search input (ms) */
const SEARCH_DEBOUNCE_MS = 300

/** Number of GIFs to fetch */
const GIF_LIMIT = 30

export function useGifPicker() {
  const [searchQuery, setSearchQuery] = useState('')
  const [gifs, setGifs] = useState<Gif[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isPasting, setIsPasting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Debounce timer ref
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  // Track if component is mounted
  const isMountedRef = useRef(true)

  // Fetch GIFs (trending or search)
  const fetchGifs = useCallback(async (query: string) => {
    setIsLoading(true)
    setError(null)

    try {
      const results = query.trim()
        ? await searchGifs(query, GIF_LIMIT)
        : await fetchTrendingGifs(GIF_LIMIT)

      if (isMountedRef.current) {
        setGifs(results)
      }
    } catch (err) {
      console.error('Failed to fetch GIFs:', err)
      if (isMountedRef.current) {
        setError(err instanceof Error ? err.message : 'Failed to load GIFs')
        setGifs([])
      }
    } finally {
      if (isMountedRef.current) {
        setIsLoading(false)
      }
    }
  }, [])

  // Load trending GIFs on mount
  useEffect(() => {
    isMountedRef.current = true
    fetchGifs('')

    return () => {
      isMountedRef.current = false
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
      }
    }
  }, [fetchGifs])

  // Debounced search handler
  const handleSearchChange = useCallback(
    (query: string) => {
      setSearchQuery(query)

      // Clear existing timer
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
      }

      // Set new debounced search
      debounceTimerRef.current = setTimeout(() => {
        fetchGifs(query)
      }, SEARCH_DEBOUNCE_MS)
    },
    [fetchGifs]
  )

  // Reset pasting state on window focus/blur to ensure clean state
  useEffect(() => {
    const resetState = () => setIsPasting(false)

    window.addEventListener('focus', resetState)
    window.addEventListener('blur', resetState)

    return () => {
      window.removeEventListener('focus', resetState)
      window.removeEventListener('blur', resetState)
    }
  }, [])

  // Paste a GIF
  const pasteGif = useCallback(async (gif: Gif) => {
    setIsPasting(true)
    try {
      // 1. Download and copy to clipboard
      await invoke('paste_gif_from_url', { url: gif.fullUrl })

      // 2. Reset loading state BEFORE hiding window
      setIsPasting(false)

      // 3. Finish paste sequence (hide window, simulate Ctrl+V)
      // We use a small timeout to ensure the UI update has painted
      setTimeout(async () => {
        try {
          await invoke('finish_paste')
        } catch (err) {
          console.error('Failed to finish paste:', err)
        }
      }, 100)
    } catch (err) {
      console.error('Failed to paste GIF:', err)
      setIsPasting(false)
    }
  }, [])

  // Refresh trending GIFs
  const refreshTrending = useCallback(() => {
    // Clear any pending search to prevent race conditions
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current)
    }
    setSearchQuery('')
    fetchGifs('')
  }, [fetchGifs])

  return {
    searchQuery,
    setSearchQuery: handleSearchChange,
    gifs,
    isLoading,
    isPasting,
    error,
    pasteGif,
    refreshTrending,
  }
}
