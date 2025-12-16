/**
 * Emoji Picker Hook
 * Manages emoji state, search, and recently used emojis
 */
import { useState, useEffect, useMemo, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { loadEmojis, getCategories, type Emoji } from '../services/emojiService'
import { searchEmojis, preloadSearchIndex } from '../services/emojiSearch'

/** Recently used emoji from backend */
interface RecentEmoji {
  char: string
  use_count: number
}

export function useEmojiPicker() {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [recentEmojis, setRecentEmojis] = useState<Emoji[]>([])
  const [isLoading, setIsLoading] = useState(true)

  // Load all emojis once
  const allEmojis = useMemo(() => loadEmojis(), [])
  const categories = useMemo(() => getCategories(), [])

  // Preload search index on mount
  useEffect(() => {
    const t = setTimeout(() => {
      preloadSearchIndex()
      setIsLoading(false)
    }, 0)
    return () => clearTimeout(t)
  }, [])

  // Load recent emojis from backend
  useEffect(() => {
    async function loadRecent() {
      try {
        const recent: RecentEmoji[] = await invoke('get_recent_emojis')
        // Map recent chars back to full Emoji objects
        const emojiMap = new Map(allEmojis.map((e) => [e.char, e]))
        const recentWithData = recent
          .map((r) => emojiMap.get(r.char))
          .filter((e): e is Emoji => e !== undefined)
        setRecentEmojis(recentWithData)
      } catch (err) {
        console.error('Failed to load recent emojis:', err)
        setRecentEmojis([])
      }
    }
    loadRecent()
  }, [allEmojis])

  // Filtered emojis based on search or category
  const filteredEmojis = useMemo(() => {
    if (searchQuery.trim()) {
      return searchEmojis(searchQuery, 100)
    }

    if (selectedCategory) {
      return allEmojis.filter((e) => e.category === selectedCategory)
    }

    // Default: show recent first, then all
    if (recentEmojis.length > 0) {
      const recentChars = new Set(recentEmojis.map((e) => e.char))
      const nonRecent = allEmojis.filter((e) => !recentChars.has(e.char))
      return [...recentEmojis, ...nonRecent]
    }

    return allEmojis
  }, [searchQuery, selectedCategory, allEmojis, recentEmojis])

  // Paste an emoji
  const pasteEmoji = useCallback(async (emoji: Emoji) => {
    try {
      await invoke('paste_emoji', { char: emoji.char })
      // Refresh recent emojis after paste
      const recent: RecentEmoji[] = await invoke('get_recent_emojis')
      const emojiMap = new Map(loadEmojis().map((e) => [e.char, e]))
      const recentWithData = recent
        .map((r) => emojiMap.get(r.char))
        .filter((e): e is Emoji => e !== undefined)
      setRecentEmojis(recentWithData)
    } catch (err) {
      console.error('Failed to paste emoji:', err)
    }
  }, [])

  return {
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories,
    filteredEmojis,
    recentEmojis,
    isLoading,
    pasteEmoji,
  }
}
